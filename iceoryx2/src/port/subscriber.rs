// Copyright (c) 2023 Contributors to the Eclipse Foundation
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
//!     .publish_subscribe()
//!     .open_or_create::<u64>()?;
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
use std::rc::Rc;
use std::sync::atomic::Ordering;

use iceoryx2_bb_lock_free::mpmc::container::{ContainerHandle, ContainerState};
use iceoryx2_bb_log::{fail, warn};
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::{shared_memory::*, zero_copy_connection::*};

use crate::port::DegrationAction;
use crate::service::dynamic_config::publish_subscribe::PublisherDetails;
use crate::service::port_factory::subscriber::SubscriberConfig;
use crate::service::static_config::publish_subscribe::StaticConfig;
use crate::{
    message::Message, raw_sample::RawSample, sample::Sample, service,
    service::header::publish_subscribe::Header,
};

use super::details::publisher_connections::{Connection, PublisherConnections};
use super::port_identifiers::UniqueSubscriberId;
use super::update_connections::ConnectionFailure;

/// Defines the failure that can occur when receiving data with [`Subscriber::receive()`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SubscriberReceiveError {
    ExceedsMaxBorrowedSamples,
    ConnectionFailure(ConnectionFailure),
}

impl std::fmt::Display for SubscriberReceiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for SubscriberReceiveError {}

/// Describes the failures when a new [`Subscriber`] is created via the
/// [`crate::service::port_factory::subscriber::PortFactorySubscriber`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SubscriberCreateError {
    ExceedsMaxSupportedSubscribers,
}

impl std::fmt::Display for SubscriberCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for SubscriberCreateError {}

pub(crate) mod internal {
    use std::fmt::Debug;

    pub(crate) trait SubscribeMgmt: Debug {
        fn release_sample(&self, channel_id: usize, sample: usize);
    }
}

/// The receiving endpoint of a publish-subscribe communication.
#[derive(Debug)]
pub struct Subscriber<Service: service::Service, MessageType: Debug> {
    dynamic_subscriber_handle: Option<ContainerHandle>,
    publisher_connections: Rc<PublisherConnections<Service>>,
    dynamic_storage: Rc<Service::DynamicStorage>,
    static_config: crate::service::static_config::StaticConfig,
    config: SubscriberConfig,

    publisher_list_state: UnsafeCell<ContainerState<PublisherDetails>>,
    _phantom_message_type: PhantomData<MessageType>,
}

impl<Service: service::Service, MessageType: Debug> Drop for Subscriber<Service, MessageType> {
    fn drop(&mut self) {
        if let Some(handle) = self.dynamic_subscriber_handle {
            self.dynamic_storage
                .get()
                .publish_subscribe()
                .release_subscriber_handle(handle)
        }
    }
}

impl<Service: service::Service, MessageType: Debug> Subscriber<Service, MessageType> {
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

        let dynamic_storage = Rc::clone(&service.state().dynamic_storage);

        let publisher_connections = Rc::new(PublisherConnections::new(
            publisher_list.capacity(),
            port_id,
            &service.state().global_config,
            static_config,
        ));

        let mut new_self = Self {
            config,
            publisher_connections,
            dynamic_storage,
            publisher_list_state: UnsafeCell::new(unsafe { publisher_list.get_state() }),
            dynamic_subscriber_handle: None,
            static_config: service.state().static_config.clone(),
            _phantom_message_type: PhantomData,
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
            .add_subscriber_id(port_id)
        {
            Some(unique_index) => unique_index,
            None => {
                fail!(from origin, with SubscriberCreateError::ExceedsMaxSupportedSubscribers,
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
                        match self.publisher_connections.create(
                            i,
                            details.publisher_id,
                            details.number_of_samples,
                        ) {
                            Ok(()) => (),
                            Err(e) => match &self.config.degration_callback {
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
    ) -> Result<Option<Sample<MessageType, Service>>, SubscriberReceiveError> {
        let msg = "Unable to receive another sample";
        match connection.receiver.receive() {
            Ok(data) => match data {
                None => Ok(None),
                Some(relative_addr) => {
                    let absolute_address =
                        relative_addr.value() + connection.data_segment.payload_start_address();
                    Ok(Some(Sample {
                        publisher_connections: Rc::clone(&self.publisher_connections),
                        channel_id,
                        ptr: unsafe {
                            RawSample::new_unchecked(
                                absolute_address as *mut Message<Header, MessageType>,
                            )
                        },
                    }))
                }
            },
            Err(ZeroCopyReceiveError::ReceiveWouldExceedMaxBorrowValue) => {
                fail!(from self, with SubscriberReceiveError::ExceedsMaxBorrowedSamples,
                    "{} since it would exceed the maximum {} of borrowed samples.",
                    msg, connection.receiver.max_borrowed_samples());
            }
        }
    }

    /// Receives a [`crate::sample::Sample`] from [`crate::port::publisher::Publisher`]. If no sample could be
    /// received [`None`] is returned. If a failure occurs [`SubscriberReceiveError`] is returned.
    pub fn receive(&self) -> Result<Option<Sample<MessageType, Service>>, SubscriberReceiveError> {
        if let Err(e) = self.update_connections() {
            fail!(from self,
                with SubscriberReceiveError::ConnectionFailure(e),
                "Some samples are not being received since not all connections to publishers could be established.");
        }

        for id in 0..self.publisher_connections.len() {
            match &mut self.publisher_connections.get_mut(id) {
                Some(ref mut connection) => {
                    if let Some(sample) = self.receive_from_connection(id, connection)? {
                        return Ok(Some(sample));
                    }
                }
                None => (),
            }
        }

        Ok(None)
    }

    /// Explicitly updates all connections to the [`crate::port::publisher::Publisher`]s. This is
    /// required to be called whenever a new [`crate::port::publisher::Publisher`] connected to
    /// the service. It is done implicitly whenever [`Subscriber::receive()`]
    /// is called.
    pub fn update_connections(&self) -> Result<(), ConnectionFailure> {
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
