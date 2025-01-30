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

use iceoryx2_bb_log::{fail, fatal_panic, warn};
use iceoryx2_cal::named_concept::NamedConceptBuilder;
use iceoryx2_cal::shm_allocator::PointerOffset;
use iceoryx2_cal::zero_copy_connection::{
    ZeroCopyConnection, ZeroCopyConnectionBuilder, ZeroCopyCreationError, ZeroCopySender,
};

use crate::node::SharedNode;
use crate::port::{DegrationAction, DegrationCallback};
use crate::service::config_scheme::connection_config;
use crate::service::ServiceState;
use crate::{service, service::naming_scheme::connection_name};

#[derive(Clone, Copy)]
pub(crate) struct ReceiverDetails {
    pub(crate) port_id: u128,
    pub(crate) buffer_size: usize,
}

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
            "Unable to establish connection to receiver port {:?} from sender port {:?}",
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
    pub(crate) connections: Vec<UnsafeCell<Option<Connection<Service>>>>,
    pub(crate) sender_port_id: u128,
    pub(crate) shared_node: Arc<SharedNode<Service>>,
    pub(crate) receiver_max_buffer_size: usize,
    pub(crate) receiver_max_borrowed_samples: usize,
    pub(crate) enable_safe_overflow: bool,
    pub(crate) number_of_samples: usize,
    pub(crate) max_number_of_segments: u8,
    pub(crate) degration_callback: Option<DegrationCallback<'static>>,
    pub(crate) service_state: Arc<ServiceState<Service>>,
}

impl<Service: service::Service> OutgoingConnections<Service> {
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

    fn remove(&self, index: usize) {
        *self.get_mut(index) = None
    }

    fn create(
        &self,
        index: usize,
        receiver_details: ReceiverDetails,
    ) -> Result<(), ZeroCopyCreationError> {
        *self.get_mut(index) = Some(Connection::new(
            self,
            receiver_details.port_id,
            receiver_details.buffer_size,
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

    fn remove_connection<R: Fn(PointerOffset)>(&self, i: usize, release_pointer_offset_call: &R) {
        if let Some(connection) = self.get(i) {
            // # SAFETY: the receiver no longer exist, therefore we can
            //           reacquire all delivered samples
            unsafe {
                connection
                    .sender
                    .acquire_used_offsets(|offset| release_pointer_offset_call(offset))
            };

            self.remove(i);
        }
    }

    pub(crate) fn update_connections<R: Fn(PointerOffset), E: Fn(&Connection<Service>)>(
        &self,
        receiver_list: &[(usize, ReceiverDetails)],
        release_pointer_offset_call: R,
        establish_new_connection_call: E,
    ) -> Result<(), ZeroCopyCreationError> {
        let mut visited_indices = vec![];
        visited_indices.resize(self.connections.capacity(), None);

        for (index, details) in receiver_list {
            visited_indices[*index] = Some(details);
        }

        for (i, index) in visited_indices.iter().enumerate() {
            match index {
                Some(receiver_details) => {
                    let create_connection = match self.get(i) {
                        None => true,
                        Some(connection) => {
                            let is_connected =
                                connection.receiver_port_id != receiver_details.port_id;
                            if is_connected {
                                self.remove_connection(i, &release_pointer_offset_call);
                            }
                            is_connected
                        }
                    };

                    if create_connection {
                        match self.create(i, **receiver_details) {
                            Ok(()) => match &self.get(i) {
                                Some(connection) => establish_new_connection_call(connection),
                                None => {
                                    fatal_panic!(from self, "This should never happen! Unable to acquire previously created subscriber connection.")
                                }
                            },
                            Err(e) => match &self.degration_callback {
                                Some(c) => match c.call(
                                    &self.service_state.static_config,
                                    self.sender_port_id,
                                    receiver_details.port_id,
                                ) {
                                    DegrationAction::Ignore => (),
                                    DegrationAction::Warn => {
                                        warn!(from self,
                                            "Unable to establish connection to new receiver {:?}.",
                                            receiver_details.port_id )
                                    }
                                    DegrationAction::Fail => {
                                        fail!(from self, with e,
                                           "Unable to establish connection to new receiver {:?}.",
                                           receiver_details.port_id );
                                    }
                                },
                                None => {
                                    warn!(from self,
                                        "Unable to establish connection to new receiver {:?}.",
                                        receiver_details.port_id )
                                }
                            },
                        }
                    }
                }
                None => self.remove_connection(i, &release_pointer_offset_call),
            }
        }

        Ok(())
    }
}
