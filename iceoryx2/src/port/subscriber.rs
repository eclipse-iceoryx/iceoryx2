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

use iceoryx2_bb_lock_free::mpmc::container::ContainerState;
use iceoryx2_bb_lock_free::mpmc::unique_index_set::UniqueIndex;
use iceoryx2_bb_log::{fail, fatal_panic, warn};
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::{shared_memory::*, zero_copy_connection::*};

use crate::port::DegrationAction;
use crate::service::static_config::publish_subscribe::StaticConfig;
use crate::{
    message::Message, raw_sample::RawSample, sample::Sample, service,
    service::header::publish_subscribe::Header,
};

use super::details::publisher_connections::{Connection, PublisherConnections};
use super::port_identifiers::{UniquePublisherId, UniqueSubscriberId};
use super::subscribe::internal::SubscribeMgmt;
use super::subscribe::{Subscribe, SubscriberCreateError, SubscriberReceiveError};
use super::update_connections::ConnectionFailure;
use super::DegrationCallback;

/// The receiving endpoint of a publish-subscribe communication.
#[derive(Debug)]
pub struct Subscriber<'a, Service: service::Service, MessageType: Debug> {
    dynamic_config_guard: Option<UniqueIndex<'a>>,
    publisher_connections: PublisherConnections<Service>,
    service: &'a Service,
    degration_callback: Option<DegrationCallback<'a>>,

    publisher_list_state: UnsafeCell<ContainerState<'a, UniquePublisherId>>,
    _phantom_message_type: PhantomData<MessageType>,
}

impl<'a, Service: service::Service, MessageType: Debug> Subscriber<'a, Service, MessageType> {
    pub(crate) fn new(
        service: &'a Service,
        static_config: &StaticConfig,
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

        let mut new_self = Self {
            publisher_connections: PublisherConnections::new(
                publisher_list.capacity(),
                port_id,
                &service.state().global_config,
                static_config,
            ),
            publisher_list_state: UnsafeCell::new(unsafe { publisher_list.get_state() }),
            dynamic_config_guard: None,
            service,
            degration_callback: None,
            _phantom_message_type: PhantomData,
        };

        if let Err(e) = new_self.populate_publisher_channels() {
            warn!(from new_self, "The new subscriber is unable to connect to every publisher, caused by {:?}.", e);
        }

        // !MUST! be the last task otherwise a subscriber is added to the dynamic config without
        // the creation of all required channels
        new_self.dynamic_config_guard = Some(
            match service
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
            },
        );

        Ok(new_self)
    }

    fn populate_publisher_channels(&self) -> Result<(), ConnectionFailure> {
        let mut visited_indices = vec![];
        visited_indices.resize(self.publisher_connections.capacity(), None);

        unsafe {
            (*self.publisher_list_state.get()).for_each(|index, publisher_id| {
                visited_indices[index as usize] = Some(*publisher_id);
            })
        };

        // update all connections
        for (i, index) in visited_indices.iter().enumerate() {
            match index {
                Some(publisher_id) => match self.publisher_connections.create(i, *publisher_id) {
                    Ok(()) => (),
                    Err(e) => match &self.degration_callback {
                        None => {
                            warn!(from self, "Unable to establish connection to new publisher {:?}.", publisher_id)
                        }
                        Some(c) => {
                            match c.call(
                                self.service.state().static_config.clone(),
                                *publisher_id,
                                self.publisher_connections.subscriber_id(),
                            ) {
                                DegrationAction::Ignore => (),
                                DegrationAction::Warn => {
                                    warn!(from self, "Unable to establish connection to new publisher {:?}.", publisher_id)
                                }
                                DegrationAction::Fail => {
                                    fail!(from self, with e, "Unable to establish connection to new publisher {:?}.", publisher_id);
                                }
                            }
                        }
                    },
                },
                None => self.publisher_connections.remove(i),
            }
        }

        Ok(())
    }

    fn receive_from_connection<'subscriber>(
        &'subscriber self,
        channel_id: usize,
        connection: &mut Connection<Service>,
    ) -> Result<Option<Sample<'subscriber, MessageType>>, SubscriberReceiveError> {
        let msg = "Unable to receive another sample";
        match connection.receiver.receive() {
            Ok(data) => match data {
                None => Ok(None),
                Some(relative_addr) => {
                    let absolute_address = relative_addr.value()
                        + connection.data_segment.allocator_data_start_address();
                    Ok(Some(Sample {
                        subscriber: self,
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

    /// Sets the [`DegrationCallback`] of the [`Subscriber`]. Whenever a connection to a
    /// [`crate::port::publisher::Publisher`] is corrupted or a seems to be dead, this callback
    /// is called and depending on the returned [`DegrationAction`] measures will be taken.
    pub fn set_degration_callback<
        F: Fn(
                service::static_config::StaticConfig,
                UniquePublisherId,
                UniqueSubscriberId,
            ) -> DegrationAction
            + 'a,
    >(
        &mut self,
        callback: Option<F>,
    ) {
        match callback {
            Some(c) => self.degration_callback = Some(DegrationCallback::new(c)),
            None => self.degration_callback = None,
        }
    }
}

impl<'a, Service: service::Service, MessageType: Debug> Subscribe<MessageType>
    for Subscriber<'a, Service, MessageType>
{
    fn receive(&self) -> Result<Option<Sample<MessageType>>, SubscriberReceiveError> {
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

    fn update_connections(&self) -> Result<(), ConnectionFailure> {
        if unsafe { (*self.publisher_list_state.get()).update() } {
            fail!(from self, when self.populate_publisher_channels(),
                "Connections were updated only partially since at least one connection to a publisher failed.");
        }

        Ok(())
    }
}

impl<'a, Service: service::Service, MessageType: Debug> SubscribeMgmt
    for Subscriber<'a, Service, MessageType>
{
    fn release_sample(&self, channel_id: usize, sample: usize) {
        match self.publisher_connections.get(channel_id) {
            Some(c) => {
                let distance = sample - c.data_segment.allocator_data_start_address();
                match c.receiver.release(PointerOffset::new(distance)) {
                    Ok(()) => (),
                    Err(ZeroCopyReleaseError::RetrieveBufferFull) => {
                        fatal_panic!(from self, when c.receiver.release(PointerOffset::new(distance)),
                                    "This should never happen! The publishers retrieve channel is full and the sample cannot be returned.");
                    }
                }
            }
            None => {
                warn!(from self, "Unable to release sample since the connection is broken. The sample will be discarded and has to be reclaimed manually by the publisher.");
            }
        }
    }
}
