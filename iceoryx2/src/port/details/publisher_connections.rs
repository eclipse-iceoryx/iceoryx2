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

use crate::{
    config,
    port::port_identifiers::{UniquePublisherId, UniqueSubscriberId},
    service::{
        self,
        config_scheme::{connection_config, data_segment_config},
    },
    service::{
        naming_scheme::{connection_name, data_segment_name},
        static_config::publish_subscribe::StaticConfig,
    },
};

use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_log::fail;
use iceoryx2_cal::named_concept::NamedConceptBuilder;
use iceoryx2_cal::{
    shared_memory::SharedMemory,
    shared_memory::{SharedMemoryBuilder, SharedMemoryOpenError},
    shm_allocator::pool_allocator::PoolAllocator,
    zero_copy_connection::*,
};

enum_gen! { ConnectionFailure
  mapping:
    ZeroCopyCreationError to FailedToEstablishConnection,
    SharedMemoryOpenError to UnableToMapPublishersDataSegment
}

impl std::fmt::Display for ConnectionFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for ConnectionFailure {}

#[derive(Debug)]
pub(crate) struct Connection<'config, Service: service::Details<'config>> {
    pub(crate) receiver:
        <<Service as service::Details<'config>>::Connection as ZeroCopyConnection>::Receiver,
    pub(crate) data_segment: Service::SharedMemory,
}

impl<'config, Service: service::Details<'config>> Connection<'config, Service> {
    fn new(
        this: &PublisherConnections<'config, Service>,
        publisher_id: UniquePublisherId,
    ) -> Result<Self, ConnectionFailure> {
        let msg = format!(
            "Unable to establish connection to publisher {:?} from subscriber {:?}.",
            publisher_id, this.subscriber_id
        );

        let receiver = fail!(from this,
                        when <<Service as service::Details<'config>>::Connection as ZeroCopyConnection>::
                            Builder::new( &connection_name(publisher_id, this.subscriber_id))
                                    .config(&connection_config::<Service>(this.config))
                                    .buffer_size(this.static_config.subscriber_max_buffer_size)
                                    .receiver_max_borrowed_samples(this.static_config.subscriber_max_borrowed_samples)
                                    .enable_safe_overflow(this.static_config.enable_safe_overflow)
                                    .create_receiver(),
                        "{} since the zero copy connection could not be established.", msg);

        let data_segment = fail!(from this,
                            when <Service::SharedMemory as SharedMemory<PoolAllocator>>::
                                Builder::new(&data_segment_name(publisher_id))
                                .config(&data_segment_config::<Service>(this.config))
                                .open(),
                            "{} since the publishers data segment could not be mapped into the process.", msg);

        Ok(Self {
            receiver,
            data_segment,
        })
    }
}
#[derive(Debug)]
pub(crate) struct PublisherConnections<'config, Service: service::Details<'config>> {
    connections: Vec<UnsafeCell<Option<Connection<'config, Service>>>>,
    subscriber_id: UniqueSubscriberId,
    config: &'config config::Config,
    static_config: StaticConfig,
}

impl<'config, Service: service::Details<'config>> PublisherConnections<'config, Service> {
    pub(crate) fn new(
        capacity: usize,
        subscriber_id: UniqueSubscriberId,
        config: &'config config::Config,
        static_config: &StaticConfig,
    ) -> Self {
        Self {
            connections: (0..capacity).map(|_| UnsafeCell::new(None)).collect(),
            subscriber_id,
            config,
            static_config: static_config.clone(),
        }
    }

    pub(crate) fn subscriber_id(&self) -> UniqueSubscriberId {
        self.subscriber_id
    }

    pub(crate) fn get(&self, index: usize) -> &Option<Connection<'config, Service>> {
        unsafe { &*self.connections[index].get() }
    }

    // only used internally as convinience function
    #[allow(clippy::mut_from_ref)]
    pub(crate) fn get_mut(&self, index: usize) -> &mut Option<Connection<'config, Service>> {
        #[deny(clippy::mut_from_ref)]
        unsafe {
            &mut *self.connections[index].get()
        }
    }

    pub(crate) fn create(
        &self,
        index: usize,
        publisher_id: UniquePublisherId,
    ) -> Result<(), ConnectionFailure> {
        if self.get(index).is_none() {
            *self.get_mut(index) = Some(Connection::new(self, publisher_id)?);
        }

        Ok(())
    }

    pub(crate) fn remove(&self, index: usize) {
        *self.get_mut(index) = None;
    }

    pub(crate) fn len(&self) -> usize {
        self.connections.len()
    }

    pub(crate) fn capacity(&self) -> usize {
        self.connections.capacity()
    }
}
