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

use core::cell::UnsafeCell;

extern crate alloc;
use alloc::sync::Arc;

use iceoryx2_bb_log::fail;
use iceoryx2_cal::named_concept::NamedConceptBuilder;
use iceoryx2_cal::zero_copy_connection::{
    ZeroCopyConnection, ZeroCopyConnectionBuilder, ZeroCopyCreationError,
};

use crate::node::SharedNode;
use crate::service::config_scheme::connection_config;
use crate::{service, service::naming_scheme::connection_name};

#[derive(Debug)]
pub(crate) struct Connection<Service: service::Service> {
    pub(crate) sender: <Service::Connection as ZeroCopyConnection>::Sender,
    pub(crate) receiver_port_id: u128,
}

impl<Service: service::Service> Connection<Service> {
    fn new(
        this: &OutgoingConnections<Service>,
        receiver_port_id: u128,
        buffer_size: usize,
        number_of_samples: usize,
    ) -> Result<Self, ZeroCopyCreationError> {
        let msg = format!(
            "Unable to establish connection to receiver {:?} from sender {:?}",
            receiver_port_id, this.sender_port_id
        );
        if this.receiver_max_buffer_size < buffer_size {
            fail!(from this, with ZeroCopyCreationError::IncompatibleBufferSize,
                "{} since the subscribers buffer size {} exceeds the services max subscriber buffer size of {}.",
                msg, buffer_size, this.receiver_max_buffer_size);
        }

        let sender = fail!(from this, when <Service::Connection as ZeroCopyConnection>::
                        Builder::new( &connection_name(this.sender_port_id, receiver_port_id))
                                .config(&connection_config::<Service>(this.shared_node.config()))
                                .buffer_size(buffer_size)
                                .receiver_max_borrowed_samples(this.receiver_max_borrowed_samples)
                                .enable_safe_overflow(this.enable_safe_overflow)
                                .number_of_samples_per_segment(number_of_samples)
                                .max_supported_shared_memory_segments(this.max_number_of_segments)
                                .timeout(this.shared_node.config().global.service.creation_timeout)
                                .create_sender(),
                        "{}.", msg);

        Ok(Self {
            sender,
            receiver_port_id,
        })
    }
}

#[derive(Debug)]
pub(crate) struct OutgoingConnections<Service: service::Service> {
    connections: Vec<UnsafeCell<Option<Connection<Service>>>>,
    sender_port_id: u128,
    shared_node: Arc<SharedNode<Service>>,
    receiver_max_buffer_size: usize,
    receiver_max_borrowed_samples: usize,
    enable_safe_overflow: bool,
    number_of_samples: usize,
    max_number_of_segments: u8,
}

impl<Service: service::Service> OutgoingConnections<Service> {
    pub(crate) fn new(
        capacity: usize,
        shared_node: Arc<SharedNode<Service>>,
        sender_port_id: u128,
        receiver_max_buffer_size: usize,
        receiver_max_borrowed_samples: usize,
        enable_safe_overflow: bool,
        number_of_samples: usize,
        max_number_of_segments: u8,
    ) -> Self {
        Self {
            connections: (0..capacity).map(|_| UnsafeCell::new(None)).collect(),
            shared_node,
            sender_port_id,
            receiver_max_buffer_size,
            receiver_max_borrowed_samples,
            enable_safe_overflow,
            number_of_samples,
            max_number_of_segments,
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
        receiver_port_id: u128,
        buffer_size: usize,
    ) -> Result<(), ZeroCopyCreationError> {
        *self.get_mut(index) = Some(Connection::new(
            self,
            receiver_port_id,
            buffer_size,
            self.number_of_samples,
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
