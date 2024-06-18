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
//! let service_name = ServiceName::new("My/Funk/ServiceName")?;
//! let service = zero_copy::Service::new(&service_name)
//!     .publish_subscribe::<u64>()
//!     .open_or_create()?;
//!
//! let subscriber = service.subscriber().create()?;
//!
//! while let Some(sample) = subscriber.receive()? {
//!     println!("received: {:?}", *sample);
//! }
//!
//! # Ok(())
//! # }
//! ```

use std::cell::UnsafeCell;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use iceoryx2_bb_lock_free::mpmc::container::{ContainerHandle, ContainerState};
use iceoryx2_bb_log::{fail, warn};
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::{shared_memory::*, zero_copy_connection::*};

use crate::port::DegrationAction;
use crate::sample::SampleDetails;
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
    ExceedsMaxBorrowedSamples,
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
    ExceedsMaxSupportedSubscribers,
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
pub struct Subscriber<Service: service::Service, PayloadType: Debug + ?Sized> {
    dynamic_subscriber_handle: Option<ContainerHandle>,
    publisher_connections: Arc<PublisherConnections<Service>>,
    dynamic_storage: Arc<Service::DynamicStorage>,
    static_config: crate::service::static_config::StaticConfig,
    degration_callback: Option<DegrationCallback<'static>>,

    publisher_list_state: UnsafeCell<ContainerState<PublisherDetails>>,
    _phantom_payload_type: PhantomData<PayloadType>,
}

impl<Service: service::Service, PayloadType: Debug + ?Sized> Drop
    for Subscriber<Service, PayloadType>
{
    fn drop(&mut self) {
        if let Some(handle) = self.dynamic_subscriber_handle {
            self.dynamic_storage
                .get()
                .publish_subscribe()
                .release_subscriber_handle(handle)
        }
    }
}

impl<Service: service::Service, PayloadType: Debug + ?Sized> Subscriber<Service, PayloadType> {
    pub(crate) fn new(
        service: &Service,
        static_config: &StaticConfig,
        config: SubscriberConfig,
    ) -> Result<Self, SubscriberCreateError> {
        let msg = "Failed to create Subscriber port";
        let origin = "Subscriber::new()";
        let port_id = UniqueSubscriberId::new();

        let publisher_list = &service
            .state()
            .dynamic_storage
            .get()
            .publish_subscribe()
            .publishers;

        let dynamic_storage = Arc::clone(&service.state().dynamic_storage);

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

        let publisher_connections = Arc::new(PublisherConnections::new(
            publisher_list.capacity(),
            port_id,
            &service.state().global_config,
            static_config,
            buffer_size,
        ));

        let mut new_self = Self {
            degration_callback: config.degration_callback,
            publisher_connections,
            dynamic_storage,
            publisher_list_state: UnsafeCell::new(unsafe { publisher_list.get_state() }),
            dynamic_subscriber_handle: None,
            static_config: service.state().static_config.clone(),
            _phantom_payload_type: PhantomData,
        };

        if let Err(e) = new_self.populate_publisher_channels() {
            warn!(from new_self, "The new subscriber is unable to connect to every publisher, caused by {:?}.", e);
        }

        std::sync::atomic::compiler_fence(Ordering::SeqCst);

        // !MUST! be the last task otherwise a subscriber is added to the dynamic config without
        // the creation of all required channels
        let dynamic_subscriber_handle = match service
            .state()
            .dynamic_storage
            .get()
            .publish_subscribe()
            .add_subscriber_id(SubscriberDetails {
                port_id,
                buffer_size,
            }) {
            Some(unique_index) => unique_index,
            None => {
                fail!(from new_self, with SubscriberCreateError::ExceedsMaxSupportedSubscribers,
                                "{} since it would exceed the maximum supported amount of subscribers of {}.",
                                msg, service.state().static_config.publish_subscribe().max_subscribers);
            }
        };

        new_self.dynamic_subscriber_handle = Some(dynamic_subscriber_handle);

        Ok(new_self)
    }

    fn populate_publisher_channels(&self) -> Result<(), ConnectionFailure> {
        let mut visited_indices = vec![];
        visited_indices.resize(self.publisher_connections.capacity(), None);

        unsafe {
            (*self.publisher_list_state.get()).for_each(|index, details| {
                visited_indices[index as usize] = Some(*details);
            })
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
                None => self.publisher_connections.remove(i),
            }
        }

        Ok(())
    }

    fn receive_from_connection(
        &self,
        channel_id: usize,
        connection: &mut Connection<Service>,
    ) -> Result<Option<(SampleDetails<Service>, usize)>, SubscriberReceiveError> {
        let msg = "Unable to receive another sample";
        match connection.receiver.receive() {
            Ok(data) => match data {
                None => Ok(None),
                Some(offset) => {
                    let absolute_address =
                        offset.value() + connection.data_segment.payload_start_address();

                    let details = SampleDetails {
                        publisher_connections: Arc::clone(&self.publisher_connections),
                        channel_id,
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

    fn receive_impl(
        &self,
    ) -> Result<Option<(SampleDetails<Service>, usize)>, SubscriberReceiveError> {
        if let Err(e) = self.update_connections() {
            fail!(from self,
                with SubscriberReceiveError::ConnectionFailure(e),
                "Some samples are not being received since not all connections to publishers could be established.");
        }

        for id in 0..self.publisher_connections.len() {
            match &mut self.publisher_connections.get_mut(id) {
                Some(ref mut connection) => {
                    if let Some((details, absolute_address)) =
                        self.receive_from_connection(id, connection)?
                    {
                        return Ok(Some((details, absolute_address)));
                    }
                }
                None => (),
            }
        }

        Ok(None)
    }

    fn payload_ptr(&self, header: *const Header) -> *const u8 {
        self.publisher_connections
            .static_config
            .type_details
            .payload_ptr_from_header(header.cast())
            .cast()
    }
}

impl<Service: service::Service, PayloadType: Debug + ?Sized> UpdateConnections
    for Subscriber<Service, PayloadType>
{
    fn update_connections(&self) -> Result<(), ConnectionFailure> {
        if unsafe {
            self.dynamic_storage
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

impl<Service: service::Service, PayloadType: Debug> Subscriber<Service, PayloadType> {
    /// Receives a [`crate::sample::Sample`] from [`crate::port::publisher::Publisher`]. If no sample could be
    /// received [`None`] is returned. If a failure occurs [`SubscriberReceiveError`] is returned.
    pub fn receive(&self) -> Result<Option<Sample<PayloadType, Service>>, SubscriberReceiveError> {
        Ok(self.receive_impl()?.map(|(details, absolute_address)| {
            let header_ptr = absolute_address as *const Header;
            let payload_ptr = self.payload_ptr(header_ptr).cast();
            Sample {
                details,
                ptr: unsafe { RawSample::new_unchecked(header_ptr, payload_ptr) },
            }
        }))
    }
}

impl<Service: service::Service, PayloadType: Debug> Subscriber<Service, [PayloadType]> {
    /// Receives a [`crate::sample::Sample`] from [`crate::port::publisher::Publisher`]. If no sample could be
    /// received [`None`] is returned. If a failure occurs [`SubscriberReceiveError`] is returned.
    pub fn receive(
        &self,
    ) -> Result<Option<Sample<[PayloadType], Service>>, SubscriberReceiveError> {
        Ok(self.receive_impl()?.map(|(details, absolute_address)| {
            let header_ptr = absolute_address as *const Header;
            let payload_ptr = self.payload_ptr(header_ptr).cast();

            let payload_layout = unsafe { (*header_ptr).payload_type_layout() };
            let number_of_elements = payload_layout.size() / core::mem::size_of::<PayloadType>();

            Sample {
                details,
                ptr: unsafe {
                    RawSample::<Header, [PayloadType]>::new_slice_unchecked(
                        header_ptr,
                        core::slice::from_raw_parts(payload_ptr, number_of_elements),
                    )
                },
            }
        }))
    }
}
