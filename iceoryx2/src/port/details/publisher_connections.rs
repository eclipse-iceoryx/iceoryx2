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

use std::{cell::UnsafeCell, sync::Arc};

use crate::{
    node::SharedNode,
    port::port_identifiers::{UniquePublisherId, UniqueSubscriberId},
    service::{
        self,
        config_scheme::{connection_config, data_segment_config},
        dynamic_config::publish_subscribe::PublisherDetails,
        naming_scheme::{connection_name, data_segment_name},
        static_config::publish_subscribe::StaticConfig,
    },
};

use crate::port::update_connections::ConnectionFailure;
use iceoryx2_bb_log::fail;
use iceoryx2_cal::named_concept::NamedConceptBuilder;
use iceoryx2_cal::{
    shared_memory::SharedMemory, shared_memory::SharedMemoryBuilder,
    shm_allocator::pool_allocator::PoolAllocator, zero_copy_connection::*,
};

#[derive(Debug)]
pub(crate) struct Connection<Service: service::Service> {
    pub(crate) receiver: <Service::Connection as ZeroCopyConnection>::Receiver,
    pub(crate) data_segment: Service::SharedMemory,
    pub(crate) publisher_id: UniquePublisherId,
}

impl<Service: service::Service> Connection<Service> {
    fn new(
        this: &PublisherConnections<Service>,
        details: &PublisherDetails,
    ) -> Result<Self, ConnectionFailure> {
        let msg = format!(
            "Unable to establish connection to publisher {:?} from subscriber {:?}.",
            details.publisher_id, this.subscriber_id
        );

        let receiver = fail!(from this,
                        when <Service::Connection as ZeroCopyConnection>::
                            Builder::new( &connection_name(details.publisher_id, this.subscriber_id))
                                    .config(&connection_config::<Service>(this.shared_node.config()))
                                    .buffer_size(this.buffer_size)
                                    .receiver_max_borrowed_samples(this.static_config.subscriber_max_borrowed_samples)
                                    .enable_safe_overflow(this.static_config.enable_safe_overflow)
                                    .number_of_samples(details.number_of_samples)
                                    .create_receiver(this.static_config.message_type_details().sample_layout(details.max_slice_len).size()),
                        "{} since the zero copy connection could not be established.", msg);

        let data_segment = fail!(from this,
                            when <Service::SharedMemory as SharedMemory<PoolAllocator>>::
                                Builder::new(&data_segment_name(&details.publisher_id))
                                .config(&data_segment_config::<Service>(this.shared_node.config()))
                                .open(),
                            "{} since the publishers data segment could not be mapped into the process.", msg);

        Ok(Self {
            receiver,
            data_segment,
            publisher_id: details.publisher_id,
        })
    }
}
#[derive(Debug)]
pub(crate) struct PublisherConnections<Service: service::Service> {
    connections: Vec<UnsafeCell<Option<Connection<Service>>>>,
    subscriber_id: UniqueSubscriberId,
    shared_node: Arc<SharedNode<Service>>,
    pub(crate) static_config: StaticConfig,
    pub(crate) buffer_size: usize,
}

impl<Service: service::Service> PublisherConnections<Service> {
    pub(crate) fn new(
        capacity: usize,
        subscriber_id: UniqueSubscriberId,
        shared_node: Arc<SharedNode<Service>>,
        static_config: &StaticConfig,
        buffer_size: usize,
    ) -> Self {
        Self {
            connections: (0..capacity).map(|_| UnsafeCell::new(None)).collect(),
            subscriber_id,
            shared_node,
            static_config: static_config.clone(),
            buffer_size,
        }
    }

    pub(crate) fn subscriber_id(&self) -> UniqueSubscriberId {
        self.subscriber_id
    }

    pub(crate) fn get(&self, index: usize) -> &Option<Connection<Service>> {
        unsafe { &*self.connections[index].get() }
    }

    // only used internally as convinience function
    #[allow(clippy::mut_from_ref)]
    pub(crate) fn get_mut(&self, index: usize) -> &mut Option<Connection<Service>> {
        #[deny(clippy::mut_from_ref)]
        unsafe {
            &mut *self.connections[index].get()
        }
    }

    pub(crate) fn create(
        &self,
        index: usize,
        details: &PublisherDetails,
    ) -> Result<(), ConnectionFailure> {
        *self.get_mut(index) = Some(Connection::new(self, details)?);

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
