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

use iceoryx2_bb_log::fail;
use iceoryx2_cal::named_concept::NamedConceptBuilder;
use iceoryx2_cal::zero_copy_connection::{
    ZeroCopyConnection, ZeroCopyConnectionBuilder, ZeroCopyCreationError,
};

use crate::service::config_scheme::connection_config;
use crate::{
    config,
    port::port_identifiers::{UniquePublisherId, UniqueSubscriberId},
    service,
    service::{naming_scheme::connection_name, static_config::publish_subscribe::StaticConfig},
};

#[derive(Debug)]
pub(crate) struct Connection<'config, Service: service::Details<'config>> {
    pub(crate) sender:
        <<Service as service::Details<'config>>::Connection as ZeroCopyConnection>::Sender,
}

impl<'config, Service: service::Details<'config>> Connection<'config, Service> {
    fn new(
        this: &SubscriberConnections<'config, Service>,
        subscriber_id: UniqueSubscriberId,
    ) -> Result<Self, ZeroCopyCreationError> {
        let sender = fail!(from this, when <<Service as service::Details<'config>>::Connection as ZeroCopyConnection>::
                        Builder::new( &connection_name(this.port_id, subscriber_id))
                                .config(&connection_config::<Service>(this.config))
                                .buffer_size(this.static_config.subscriber_max_buffer_size)
                                .receiver_max_borrowed_samples(this.static_config.subscriber_max_borrowed_samples)
                                .enable_safe_overflow(this.static_config.enable_safe_overflow)
                                .create_sender(),
                        "Unable to establish connection to subscriber {:?} from publisher {:?}.",
                        subscriber_id, this.port_id);

        Ok(Self { sender })
    }
}

#[derive(Debug)]
pub(crate) struct SubscriberConnections<'config, Service: service::Details<'config>> {
    connections: Vec<UnsafeCell<Option<Connection<'config, Service>>>>,
    port_id: UniquePublisherId,
    config: &'config config::Config,
    static_config: StaticConfig,
}

impl<'config, Service: service::Details<'config>> SubscriberConnections<'config, Service> {
    pub(crate) fn new(
        capacity: usize,
        config: &'config config::Config,
        port_id: UniquePublisherId,
        static_config: &StaticConfig,
    ) -> Self {
        Self {
            connections: (0..capacity).map(|_| UnsafeCell::new(None)).collect(),
            config,
            port_id,
            static_config: static_config.clone(),
        }
    }

    pub(crate) fn get(&self, index: usize) -> &Option<Connection<'config, Service>> {
        unsafe { &(*self.connections[index].get()) }
    }

    // only used internally as convinience function
    #[allow(clippy::mut_from_ref)]
    fn get_mut(&self, index: usize) -> &mut Option<Connection<'config, Service>> {
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
        subscriber_id: UniqueSubscriberId,
    ) -> Result<bool, ZeroCopyCreationError> {
        if self.get(index).is_none() {
            *self.get_mut(index) = Some(Connection::new(self, subscriber_id)?);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.connections.len()
    }

    pub(crate) fn capacity(&self) -> usize {
        self.connections.capacity()
    }
}
