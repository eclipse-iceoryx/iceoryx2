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

use std::cell::UnsafeCell;
use std::sync::Arc;

use iceoryx2_bb_log::fail;
use iceoryx2_cal::named_concept::NamedConceptBuilder;
use iceoryx2_cal::zero_copy_connection::{
    ZeroCopyConnection, ZeroCopyConnectionBuilder, ZeroCopyCreationError,
};

use crate::service::config_scheme::connection_config;
use crate::service::dynamic_config::publish_subscribe::SubscriberDetails;
use crate::{
    config,
    port::port_identifiers::{UniquePublisherId, UniqueSubscriberId},
    service,
    service::{naming_scheme::connection_name, static_config::publish_subscribe::StaticConfig},
};

#[derive(Debug)]
pub(crate) struct Connection<Service: service::Service> {
    pub(crate) sender: <Service::Connection as ZeroCopyConnection>::Sender,
    pub(crate) subscriber_id: UniqueSubscriberId,
}

impl<Service: service::Service> Connection<Service> {
    fn new(
        this: &SubscriberConnections<Service>,
        subscriber_details: SubscriberDetails,
        number_of_samples: usize,
        max_slice_len: usize,
    ) -> Result<Self, ZeroCopyCreationError> {
        let msg = format!(
            "Unable to establish connection to subscriber {:?} from publisher {:?}",
            subscriber_details.port_id, this.port_id
        );
        if this.static_config.subscriber_max_buffer_size < subscriber_details.buffer_size {
            fail!(from this, with ZeroCopyCreationError::IncompatibleBufferSize,
                "{} since the subscribers buffer size {} exceeds the services max subscriber buffer size of {}.",
                msg, subscriber_details.buffer_size, this.static_config.subscriber_max_buffer_size);
        }

        let sender = fail!(from this, when <Service::Connection as ZeroCopyConnection>::
                        Builder::new( &connection_name(this.port_id, subscriber_details.port_id))
                                .config(&connection_config::<Service>(this.config.as_ref()))
                                .buffer_size(subscriber_details.buffer_size)
                                .receiver_max_borrowed_samples(this.static_config.subscriber_max_borrowed_samples)
                                .enable_safe_overflow(this.static_config.enable_safe_overflow)
                                .number_of_samples(number_of_samples)
                                .create_sender(this.static_config.type_details().sample_layout(max_slice_len).size()),
                        "{}.", msg);

        Ok(Self {
            sender,
            subscriber_id: subscriber_details.port_id,
        })
    }
}

#[derive(Debug)]
pub(crate) struct SubscriberConnections<Service: service::Service> {
    connections: Vec<UnsafeCell<Option<Connection<Service>>>>,
    port_id: UniquePublisherId,
    config: Arc<config::Config>,
    pub(crate) static_config: StaticConfig,
    number_of_samples: usize,
}

impl<Service: service::Service> SubscriberConnections<Service> {
    pub(crate) fn new(
        capacity: usize,
        config: &Arc<config::Config>,
        port_id: UniquePublisherId,
        static_config: &StaticConfig,
        number_of_samples: usize,
    ) -> Self {
        Self {
            connections: (0..capacity).map(|_| UnsafeCell::new(None)).collect(),
            config: Arc::clone(config),
            port_id,
            static_config: static_config.clone(),
            number_of_samples,
        }
    }

    pub(crate) fn get(&self, index: usize) -> &Option<Connection<Service>> {
        unsafe { &(*self.connections[index].get()) }
    }

    // only used internally as convinience function
    #[allow(clippy::mut_from_ref)]
    fn get_mut(&self, index: usize) -> &mut Option<Connection<Service>> {
        #[deny(clippy::mut_from_ref)]
        unsafe {
            &mut (*self.connections[index].get())
        }
    }

    pub(crate) fn remove(&self, index: usize) {
        *self.get_mut(index) = None
    }

    pub(crate) fn create(
        &self,
        index: usize,
        subscriber_details: SubscriberDetails,
        max_slice_len: usize,
    ) -> Result<(), ZeroCopyCreationError> {
        *self.get_mut(index) = Some(Connection::new(
            self,
            subscriber_details,
            self.number_of_samples,
            max_slice_len,
        )?);

        Ok(())
    }

    pub(crate) fn len(&self) -> usize {
        self.connections.len()
    }

    pub(crate) fn capacity(&self) -> usize {
        self.connections.capacity()
    }
}
