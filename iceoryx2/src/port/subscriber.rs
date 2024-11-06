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
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
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

use std::any::TypeId;
use std::cell::UnsafeCell;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use iceoryx2_bb_container::queue::Queue;
use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_lock_free::mpmc::container::{ContainerHandle, ContainerState};
use iceoryx2_bb_log::{fail, warn};
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::{shared_memory::*, zero_copy_connection::*};

use crate::port::DegrationAction;
use crate::sample::SampleDetails;
use crate::service::builder::publish_subscribe::CustomPayloadMarker;
use crate::service::dynamic_config::publish_subscribe::{PublisherDetails, SubscriberDetails};
use crate::service::header::publish_subscribe::Header;
use crate::service::port_factory::subscriber::SubscriberConfig;
use crate::service::static_config::publish_subscribe::StaticConfig;
use crate::{raw_sample::RawSample, sample::Sample, service};

use super::details::publisher_connections::{Connection, PublisherConnections};
use super::port_identifiers::UniqueSubscriberId;
use super::update_connections::{ConnectionFailure, UpdateConnections};
use super::DegrationCallback;

/// Defines the failure that can occur when receiving data with [`Subscriber::receive()`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SubscriberReceiveError {
    /// The maximum amount of [`Sample`]s a user can borrow with [`Subscriber::receive()`] is
    /// defined in [`crate::config::Config`]. When this is exceeded [`Subscriber::receive()`]
    /// fails.
    ExceedsMaxBorrowedSamples,

    /// Occurs when a [`Subscriber`] is unable to connect to a corresponding
    /// [`Publisher`](crate::port::publisher::Publisher).
    ConnectionFailure(ConnectionFailure),
}

impl std::fmt::Display for SubscriberReceiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "SubscriberReceiveError::{:?}", self)
    }
}

impl std::error::Error for SubscriberReceiveError {}

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

impl std::fmt::Display for SubscriberCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "SubscriberCreateError::{:?}", self)
    }
}

impl std::error::Error for SubscriberCreateError {}

/// The receiving endpoint of a publish-subscribe communication.
#[derive(Debug)]
pub struct Subscriber<
    Service: service::Service,
    Payload: Debug + ?Sized + 'static,
    UserHeader: Debug,
> {
    dynamic_subscriber_handle: Option<ContainerHandle>,
    publisher_connections: PublisherConnections<Service>,
    to_be_removed_connections: UnsafeCell<Queue<Arc<Connection<Service>>>>,
    static_config: crate::service::static_config::StaticConfig,
    degration_callback: Option<DegrationCallback<'static>>,

    publisher_list_state: UnsafeCell<ContainerState<PublisherDetails>>,
    _payload: PhantomData<Payload>,
    _user_header: PhantomData<UserHeader>,
}

impl<Service: service::Service, Payload: Debug + ?Sized, UserHeader: Debug> Drop
    for Subscriber<Service, Payload, UserHeader>
{
    fn drop(&mut self) {
        if let Some(handle) = self.dynamic_subscriber_handle {
            self.publisher_connections
                .service_state
                .dynamic_storage
                .get()
                .publish_subscribe()
                .release_subscriber_handle(handle)
        }
    }
}

impl<Service: service::Service, Payload: Debug + ?Sized, UserHeader: Debug>
    Subscriber<Service, Payload, UserHeader>
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

        let publisher_connections = PublisherConnections::new(
            publisher_list.capacity(),
            subscriber_id,
            service.__internal_state().clone(),
            static_config,
            buffer_size,
        );

        let mut new_self = Self {
            to_be_removed_connections: UnsafeCell::new(Queue::new(
                service
                    .__internal_state()
                    .shared_node
                    .config()
                    .defaults
                    .publish_subscribe
                    .subscriber_expired_connection_buffer,
            )),
            degration_callback: config.degration_callback,
            publisher_connections,
            publisher_list_state: UnsafeCell::new(unsafe { publisher_list.get_state() }),
            dynamic_subscriber_handle: None,
            static_config: service.__internal_state().static_config.clone(),
            _payload: PhantomData,
            _user_header: PhantomData,
        };

        if let Err(e) = new_self.populate_publisher_channels() {
            warn!(from new_self, "The new subscriber is unable to connect to every publisher, caused by {:?}.", e);
        }

        std::sync::atomic::compiler_fence(Ordering::SeqCst);

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

    fn populate_publisher_channels(&self) -> Result<(), ConnectionFailure> {
        let mut visited_indices = vec![];
        visited_indices.resize(self.publisher_connections.capacity(), None);

        unsafe {
            (*self.publisher_list_state.get()).for_each(|h, details| {
                visited_indices[h.index() as usize] = Some(*details);
                CallbackProgression::Continue
            })
        };

        let prepare_connection_removal = |i| {
            if let Some(connection) = self.publisher_connections.get(i) {
                if connection.receiver.has_data()
                    && !unsafe { &mut *self.to_be_removed_connections.get() }
                        .push(connection.clone())
                {
                    warn!(from self, "Expired connection buffer exceeded. A publisher disconnected with undelivered samples that will be discarded. Increase the config entry `defaults.publish-subscribe.subscriber-expired-connection-buffer` to mitigate the problem.");
                }
            }
        };

        // update all connections
        for (i, index) in visited_indices.iter().enumerate() {
            match index {
                Some(details) => {
                    let create_connection = match self.publisher_connections.get(i) {
                        None => true,
                        Some(connection) => connection.publisher_id != details.publisher_id,
                    };

                    if create_connection {
                        prepare_connection_removal(i);

                        match self.publisher_connections.create(i, details) {
                            Ok(()) => (),
                            Err(e) => match &self.degration_callback {
                                None => {
                                    warn!(from self, "Unable to establish connection to new publisher {:?}.", details.publisher_id)
                                }
                                Some(c) => {
                                    match c.call(
                                        self.static_config.clone(),
                                        details.publisher_id,
                                        self.publisher_connections.subscriber_id(),
                                    ) {
                                        DegrationAction::Ignore => (),
                                        DegrationAction::Warn => {
                                            warn!(from self, "Unable to establish connection to new publisher {:?}.",
                                        details.publisher_id)
                                        }
                                        DegrationAction::Fail => {
                                            fail!(from self, with e, "Unable to establish connection to new publisher {:?}.",
                                        details.publisher_id);
                                        }
                                    }
                                }
                            },
                        }
                    }
                }
                None => {
                    prepare_connection_removal(i);

                    self.publisher_connections.remove(i)
                }
            }
        }

        Ok(())
    }

    fn receive_from_connection(
        &self,
        connection: &Arc<Connection<Service>>,
    ) -> Result<Option<(SampleDetails<Service>, usize)>, SubscriberReceiveError> {
        let msg = "Unable to receive another sample";
        match connection.receiver.receive() {
            Ok(data) => match data {
                None => Ok(None),
                Some(offset) => {
                    let absolute_address =
                        offset.value() + connection.data_segment.payload_start_address();

                    let details = SampleDetails {
                        publisher_connection: connection.clone(),
                        offset,
                        origin: connection.publisher_id,
                    };

                    Ok(Some((details, absolute_address)))
                }
            },
            Err(ZeroCopyReceiveError::ReceiveWouldExceedMaxBorrowValue) => {
                fail!(from self, with SubscriberReceiveError::ExceedsMaxBorrowedSamples,
                    "{} since it would exceed the maximum {} of borrowed samples.",
                    msg, connection.receiver.max_borrowed_samples());
            }
        }
    }

    /// Returns the [`UniqueSubscriberId`] of the [`Subscriber`]
    pub fn id(&self) -> UniqueSubscriberId {
        self.publisher_connections.subscriber_id()
    }

    /// Returns the internal buffer size of the [`Subscriber`].
    pub fn buffer_size(&self) -> usize {
        self.publisher_connections.buffer_size
    }

    /// Returns true if the [`Subscriber`] has samples in the buffer that can be received with [`Subscriber::receive`].
    pub fn has_samples(&self) -> Result<bool, ConnectionFailure> {
        fail!(from self, when self.update_connections(),
                "Some samples are not being received since not all connections to publishers could be established.");

        for id in 0..self.publisher_connections.len() {
            if let Some(ref connection) = &self.publisher_connections.get(id) {
                if connection.receiver.has_data() {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn receive_impl(
        &self,
    ) -> Result<Option<(SampleDetails<Service>, usize)>, SubscriberReceiveError> {
        if let Err(e) = self.update_connections() {
            fail!(from self,
                with SubscriberReceiveError::ConnectionFailure(e),
                "Some samples are not being received since not all connections to publishers could be established.");
        }

        let to_be_removed_connections = unsafe { &mut *self.to_be_removed_connections.get() };

        if let Some(connection) = to_be_removed_connections.peek() {
            if let Some((details, absolute_address)) = self.receive_from_connection(connection)? {
                return Ok(Some((details, absolute_address)));
            } else {
                to_be_removed_connections.pop();
            }
        }

        for id in 0..self.publisher_connections.len() {
            if let Some(ref mut connection) = &mut self.publisher_connections.get_mut(id) {
                if let Some((details, absolute_address)) =
                    self.receive_from_connection(connection)?
                {
                    return Ok(Some((details, absolute_address)));
                }
            }
        }

        Ok(None)
    }

    fn payload_ptr(&self, header: *const Header) -> *const u8 {
        self.publisher_connections
            .static_config
            .message_type_details
            .payload_ptr_from_header(header.cast())
            .cast()
    }

    fn user_header_ptr(&self, header: *const Header) -> *const u8 {
        self.publisher_connections
            .static_config
            .message_type_details
            .user_header_ptr_from_header(header.cast())
            .cast()
    }
}

impl<Service: service::Service, Payload: Debug + ?Sized, UserHeader: Debug> UpdateConnections
    for Subscriber<Service, Payload, UserHeader>
{
    fn update_connections(&self) -> Result<(), ConnectionFailure> {
        if unsafe {
            self.publisher_connections
                .service_state
                .dynamic_storage
                .get()
                .publish_subscribe()
                .publishers
                .update_state(&mut *self.publisher_list_state.get())
        } {
            fail!(from self, when self.populate_publisher_channels(),
                "Connections were updated only partially since at least one connection to a publisher failed.");
        }

        Ok(())
    }
}

impl<Service: service::Service, Payload: Debug, UserHeader: Debug>
    Subscriber<Service, Payload, UserHeader>
{
    /// Receives a [`crate::sample::Sample`] from [`crate::port::publisher::Publisher`]. If no sample could be
    /// received [`None`] is returned. If a failure occurs [`SubscriberReceiveError`] is returned.
    pub fn receive(
        &self,
    ) -> Result<Option<Sample<Service, Payload, UserHeader>>, SubscriberReceiveError> {
        Ok(self.receive_impl()?.map(|(details, absolute_address)| {
            let header_ptr = absolute_address as *const Header;
            let user_header_ptr = self.user_header_ptr(header_ptr).cast();
            let payload_ptr = self.payload_ptr(header_ptr).cast();
            Sample {
                details,
                ptr: unsafe { RawSample::new_unchecked(header_ptr, user_header_ptr, payload_ptr) },
            }
        }))
    }
}

impl<Service: service::Service, Payload: Debug, UserHeader: Debug>
    Subscriber<Service, [Payload], UserHeader>
{
    /// Receives a [`crate::sample::Sample`] from [`crate::port::publisher::Publisher`]. If no sample could be
    /// received [`None`] is returned. If a failure occurs [`SubscriberReceiveError`] is returned.
    pub fn receive(
        &self,
    ) -> Result<Option<Sample<Service, [Payload], UserHeader>>, SubscriberReceiveError> {
        debug_assert!(TypeId::of::<Payload>() != TypeId::of::<CustomPayloadMarker>());

        Ok(self.receive_impl()?.map(|(details, absolute_address)| {
            let header_ptr = absolute_address as *const Header;
            let user_header_ptr = self.user_header_ptr(header_ptr).cast();
            let payload_ptr = self.payload_ptr(header_ptr).cast();
            let number_of_elements = unsafe { (*header_ptr).number_of_elements() };

            Sample {
                details,
                ptr: unsafe {
                    RawSample::<Header, UserHeader, [Payload]>::new_slice_unchecked(
                        header_ptr,
                        user_header_ptr,
                        core::slice::from_raw_parts(payload_ptr, number_of_elements as _),
                    )
                },
            }
        }))
    }
}

impl<Service: service::Service, UserHeader: Debug>
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
    ) -> Result<Option<Sample<Service, [CustomPayloadMarker], UserHeader>>, SubscriberReceiveError>
    {
        Ok(self.receive_impl()?.map(|(details, absolute_address)| {
            let header_ptr = absolute_address as *const Header;
            let user_header_ptr = self.user_header_ptr(header_ptr).cast();
            let payload_ptr = self.payload_ptr(header_ptr).cast();
            let number_of_elements = unsafe { (*header_ptr).number_of_elements() };
            let number_of_bytes = number_of_elements as usize
                * self
                    .static_config
                    .publish_subscribe()
                    .message_type_details
                    .payload
                    .size;

            Sample {
                details,
                ptr: unsafe {
                    RawSample::<Header, UserHeader, [CustomPayloadMarker]>::new_slice_unchecked(
                        header_ptr,
                        user_header_ptr,
                        core::slice::from_raw_parts(payload_ptr, number_of_bytes),
                    )
                },
            }
        }))
    }
}
