// Copyright (c) 2023 - 2024 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Example
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .open_or_create()?;
//!
//! let subscriber = service.subscriber_builder().create()?;
//!
//! while let Some(sample) = subscriber.receive()? {
//!     println!("received: {:?}", *sample);
//! }
//!
//! # Ok(())
//! # }
//! ```

use core::any::TypeId;
use core::cell::UnsafeCell;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::sync::atomic::Ordering;

extern crate alloc;

use iceoryx2_bb_container::vec::Vec;
use iceoryx2_bb_elementary::cyclic_tagger::CyclicTagger;
use iceoryx2_bb_elementary::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_lock_free::mpmc::container::{ContainerHandle, ContainerState};
use iceoryx2_bb_log::{fail, warn};
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::zero_copy_connection::ChannelId;

use crate::service::builder::CustomPayloadMarker;
use crate::service::dynamic_config::publish_subscribe::{PublisherDetails, SubscriberDetails};
use crate::service::header::publish_subscribe::Header;
use crate::service::port_factory::subscriber::SubscriberConfig;
use crate::service::static_config::publish_subscribe::StaticConfig;
use crate::{raw_sample::RawSample, sample::Sample, service};

use super::details::chunk::Chunk;
use super::details::chunk_details::ChunkDetails;
use super::details::receiver::*;
use super::port_identifiers::UniqueSubscriberId;
use super::update_connections::ConnectionFailure;
use super::ReceiveError;

/// Describes the failures when a new [`Subscriber`] is created via the
/// [`crate::service::port_factory::subscriber::PortFactorySubscriber`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SubscriberCreateError {
    /// The maximum amount of [`Subscriber`]s that can connect to a
    /// [`Service`](crate::service::Service) is
    /// defined in [`crate::config::Config`]. When this is exceeded no more [`Subscriber`]s
    /// can be created for a specific [`Service`](crate::service::Service).
    ExceedsMaxSupportedSubscribers,
    /// When the [`Subscriber`] requires a larger buffer size than the
    /// [`Service`](crate::service::Service) offers the creation will fail.
    BufferSizeExceedsMaxSupportedBufferSizeOfService,
}

impl core::fmt::Display for SubscriberCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "SubscriberCreateError::{:?}", self)
    }
}

impl core::error::Error for SubscriberCreateError {}

/// The receiving endpoint of a publish-subscribe communication.
#[derive(Debug)]
pub struct Subscriber<
    Service: service::Service,
    Payload: Debug + ZeroCopySend + ?Sized + 'static,
    UserHeader: Debug + ZeroCopySend,
> {
    dynamic_subscriber_handle: Option<ContainerHandle>,
    receiver: Receiver<Service>,

    publisher_list_state: UnsafeCell<ContainerState<PublisherDetails>>,
    _payload: PhantomData<Payload>,
    _user_header: PhantomData<UserHeader>,
}

impl<
        Service: service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: Debug + ZeroCopySend,
    > Drop for Subscriber<Service, Payload, UserHeader>
{
    fn drop(&mut self) {
        if let Some(handle) = self.dynamic_subscriber_handle {
            self.receiver
                .service_state
                .dynamic_storage
                .get()
                .publish_subscribe()
                .release_subscriber_handle(handle)
        }
    }
}

impl<
        Service: service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: Debug + ZeroCopySend,
    > Subscriber<Service, Payload, UserHeader>
{
    pub(crate) fn new(
        service: &Service,
        static_config: &StaticConfig,
        config: SubscriberConfig,
    ) -> Result<Self, SubscriberCreateError> {
        let msg = "Failed to create Subscriber port";
        let origin = "Subscriber::new()";
        let subscriber_id = UniqueSubscriberId::new();

        let publisher_list = &service
            .__internal_state()
            .dynamic_storage
            .get()
            .publish_subscribe()
            .publishers;

        let buffer_size = match config.buffer_size {
            Some(buffer_size) => {
                if static_config.subscriber_max_buffer_size < buffer_size {
                    fail!(from origin, with SubscriberCreateError::BufferSizeExceedsMaxSupportedBufferSizeOfService,
                        "{} since the requested buffer size {} exceeds the maximum supported buffer size {} of the service.",
                        msg, buffer_size, static_config.subscriber_max_buffer_size);
                }
                buffer_size
            }
            None => static_config.subscriber_max_buffer_size,
        };

        let receiver = Receiver {
            connections: Vec::from_fn(publisher_list.capacity(), |_| UnsafeCell::new(None)),
            receiver_port_id: subscriber_id.value(),
            service_state: service.__internal_state().clone(),
            message_type_details: static_config.message_type_details.clone(),
            receiver_max_borrowed_samples: static_config.subscriber_max_borrowed_samples,
            enable_safe_overflow: static_config.enable_safe_overflow,
            buffer_size,
            tagger: CyclicTagger::new(),
            to_be_removed_connections: Some(UnsafeCell::new(Vec::new(
                service
                    .__internal_state()
                    .shared_node
                    .config()
                    .defaults
                    .publish_subscribe
                    .subscriber_expired_connection_buffer,
            ))),
            degradation_callback: config.degradation_callback,
            number_of_channels: 1,
        };

        let mut new_self = Self {
            receiver,
            publisher_list_state: UnsafeCell::new(unsafe { publisher_list.get_state() }),
            dynamic_subscriber_handle: None,
            _payload: PhantomData,
            _user_header: PhantomData,
        };

        if let Err(e) = new_self.force_update_connections() {
            warn!(from new_self, "The new subscriber is unable to connect to every publisher, caused by {:?}.", e);
        }

        core::sync::atomic::compiler_fence(Ordering::SeqCst);

        // !MUST! be the last task otherwise a subscriber is added to the dynamic config without
        // the creation of all required channels
        let dynamic_subscriber_handle = match service
            .__internal_state()
            .dynamic_storage
            .get()
            .publish_subscribe()
            .add_subscriber_id(SubscriberDetails {
                subscriber_id,
                buffer_size,
                node_id: *service.__internal_state().shared_node.id(),
            }) {
            Some(unique_index) => unique_index,
            None => {
                fail!(from new_self, with SubscriberCreateError::ExceedsMaxSupportedSubscribers,
                                "{} since it would exceed the maximum supported amount of subscribers of {}.",
                                msg, service.__internal_state().static_config.publish_subscribe().max_subscribers);
            }
        };

        new_self.dynamic_subscriber_handle = Some(dynamic_subscriber_handle);

        Ok(new_self)
    }

    fn force_update_connections(&self) -> Result<(), ConnectionFailure> {
        self.receiver.start_update_connection_cycle();

        let mut result = Ok(());
        unsafe {
            (*self.publisher_list_state.get()).for_each(|h, details| {
                let inner_result = self.receiver.update_connection(
                    h.index() as usize,
                    SenderDetails {
                        port_id: details.publisher_id.value(),
                        number_of_samples: details.number_of_samples,
                        max_number_of_segments: details.max_number_of_segments,
                        data_segment_type: details.data_segment_type,
                    },
                );

                if result.is_ok() {
                    result = inner_result;
                }
                CallbackProgression::Continue
            })
        };

        self.receiver.finish_update_connection_cycle();

        result
    }

    /// Returns the [`UniqueSubscriberId`] of the [`Subscriber`]
    pub fn id(&self) -> UniqueSubscriberId {
        UniqueSubscriberId(UniqueSystemId::from(self.receiver.receiver_port_id()))
    }

    /// Returns the internal buffer size of the [`Subscriber`].
    pub fn buffer_size(&self) -> usize {
        self.receiver.buffer_size
    }

    /// Returns true if the [`Subscriber`] has samples in the buffer that can be received with [`Subscriber::receive`].
    pub fn has_samples(&self) -> Result<bool, ConnectionFailure> {
        fail!(from self, when self.update_connections(),
                "Some samples are not being received since not all connections to publishers could be established.");
        Ok(self.receiver.has_samples(ChannelId::new(0)))
    }

    fn receive_impl(&self) -> Result<Option<(ChunkDetails<Service>, Chunk)>, ReceiveError> {
        fail!(from self, when self.update_connections(),
                "Some samples are not being received since not all connections to publishers could be established.");

        self.receiver.receive(ChannelId::new(0))
    }

    fn update_connections(&self) -> Result<(), ConnectionFailure> {
        if unsafe {
            self.receiver
                .service_state
                .dynamic_storage
                .get()
                .publish_subscribe()
                .publishers
                .update_state(&mut *self.publisher_list_state.get())
        } {
            fail!(from self, when self.force_update_connections(),
                "Connections were updated only partially since at least one connection to a publisher failed.");
        }

        Ok(())
    }
}

impl<
        Service: service::Service,
        Payload: Debug + ZeroCopySend,
        UserHeader: Debug + ZeroCopySend,
    > Subscriber<Service, Payload, UserHeader>
{
    /// Receives a [`crate::sample::Sample`] from [`crate::port::publisher::Publisher`]. If no sample could be
    /// received [`None`] is returned. If a failure occurs [`ReceiveError`] is returned.
    pub fn receive(&self) -> Result<Option<Sample<Service, Payload, UserHeader>>, ReceiveError> {
        Ok(self.receive_impl()?.map(|(details, chunk)| Sample {
            details,
            ptr: unsafe {
                RawSample::new_unchecked(
                    chunk.header.cast(),
                    chunk.user_header.cast(),
                    chunk.payload.cast(),
                )
            },
        }))
    }
}

impl<
        Service: service::Service,
        Payload: Debug + ZeroCopySend,
        UserHeader: Debug + ZeroCopySend,
    > Subscriber<Service, [Payload], UserHeader>
{
    /// Receives a [`crate::sample::Sample`] from [`crate::port::publisher::Publisher`]. If no sample could be
    /// received [`None`] is returned. If a failure occurs [`ReceiveError`] is returned.
    pub fn receive(&self) -> Result<Option<Sample<Service, [Payload], UserHeader>>, ReceiveError> {
        debug_assert!(TypeId::of::<Payload>() != TypeId::of::<CustomPayloadMarker>());

        Ok(self.receive_impl()?.map(|(details, chunk)| {
            let header_ptr = chunk.header as *const Header;
            let number_of_elements = unsafe { (*header_ptr).number_of_elements() };

            Sample {
                details,
                ptr: unsafe {
                    RawSample::<Header, UserHeader, [Payload]>::new_slice_unchecked(
                        header_ptr,
                        chunk.user_header.cast(),
                        core::slice::from_raw_parts(chunk.payload.cast(), number_of_elements as _),
                    )
                },
            }
        }))
    }
}

impl<Service: service::Service, UserHeader: Debug + ZeroCopySend>
    Subscriber<Service, [CustomPayloadMarker], UserHeader>
{
    /// # Safety
    ///
    ///  * The number_of_elements in the [`Header`](crate::service::header::publish_subscribe::Header)
    ///     corresponds to the payload type details that where overridden in
    ///     `MessageTypeDetails::payload.size`.
    ///     If the `payload.size == 8` a value for number_of_elements of 5 means that there are
    ///     5 elements of size 8 stored in the [`Sample`].
    ///  *  When the payload.size == 8 and the number of elements if 5, it means that the sample
    ///     will contain a slice of 8 * 5 = 40 [`CustomPayloadMarker`]s or 40 bytes.
    #[doc(hidden)]
    pub unsafe fn receive_custom_payload(
        &self,
    ) -> Result<Option<Sample<Service, [CustomPayloadMarker], UserHeader>>, ReceiveError> {
        Ok(self.receive_impl()?.map(|(details, chunk)| {
            let header_ptr = chunk.header as *const Header;
            let number_of_elements = unsafe { (*header_ptr).number_of_elements() };
            let number_of_bytes = number_of_elements as usize * self.receiver.payload_size();

            Sample {
                details,
                ptr: unsafe {
                    RawSample::<Header, UserHeader, [CustomPayloadMarker]>::new_slice_unchecked(
                        header_ptr,
                        chunk.user_header.cast(),
                        core::slice::from_raw_parts(chunk.payload.cast(), number_of_bytes),
                    )
                },
            }
        }))
    }
}
