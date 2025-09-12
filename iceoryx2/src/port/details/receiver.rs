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

use super::channel_management::ChannelManagement;
use super::channel_management::INVALID_CHANNEL_STATE;
use super::chunk::Chunk;
use super::chunk_details::ChunkDetails;
use super::data_segment::{DataSegmentType, DataSegmentView};
use crate::port::update_connections::ConnectionFailure;
use crate::port::{DegradationAction, DegradationCallback, ReceiveError};
use crate::service::naming_scheme::data_segment_name;
use crate::service::static_config::message_type_details::MessageTypeDetails;
use crate::service::NoResource;
use crate::service::ServiceState;
use crate::service::{self, config_scheme::connection_config, naming_scheme::connection_name};
use alloc::sync::Arc;
use iceoryx2_bb_container::slotmap::SlotMap;
use iceoryx2_bb_container::slotmap::SlotMapKey;
use iceoryx2_bb_container::vec::Vec;
use iceoryx2_bb_elementary::cyclic_tagger::*;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_log::{error, fail, warn};
use iceoryx2_cal::named_concept::NamedConceptBuilder;
use iceoryx2_cal::zero_copy_connection::*;

#[derive(Clone, Copy)]
pub(crate) struct SenderDetails {
    pub(crate) port_id: u128,
    pub(crate) number_of_samples: usize,
    pub(crate) max_number_of_segments: u8,
    pub(crate) data_segment_type: DataSegmentType,
}

#[derive(Debug)]
pub(crate) struct Connection<Service: service::Service> {
    pub(crate) receiver: <Service::Connection as ZeroCopyConnection>::Receiver,
    pub(crate) data_segment: DataSegmentView<Service>,
    pub(crate) sender_port_id: u128,
    tag: Tag,
}

impl<Service: service::Service> Taggable for Connection<Service> {
    fn tag(&self) -> &Tag {
        &self.tag
    }
}

impl<Service: service::Service> Connection<Service> {
    fn new(
        this: &Receiver<Service>,
        data_segment_type: DataSegmentType,
        sender_port_id: u128,
        number_of_samples: usize,
        max_number_of_segments: u8,
        cyclic_tagger: &CyclicTagger,
    ) -> Result<Self, ConnectionFailure> {
        let msg = format!(
            "Unable to establish connection to sender port {:?} from receiver port {:?}.",
            sender_port_id, this.receiver_port_id
        );

        let global_config = this.service_state.shared_node.config();
        let receiver = fail!(from this,
                        when <Service::Connection as ZeroCopyConnection>::
                            Builder::new( &connection_name(sender_port_id, this.receiver_port_id))
                                    .config(&connection_config::<Service>(global_config))
                                    .buffer_size(this.buffer_size)
                                    .receiver_max_borrowed_samples_per_channel(this.receiver_max_borrowed_samples)
                                    .enable_safe_overflow(this.enable_safe_overflow)
                                    .number_of_samples_per_segment(number_of_samples)
                                    .number_of_channels(this.number_of_channels)
                                    .initial_channel_state(INVALID_CHANNEL_STATE)
                                    .max_supported_shared_memory_segments(max_number_of_segments)
                                    .timeout(global_config.global.service.creation_timeout)
                                    .create_receiver(),
                        "{} since the zero copy connection could not be established.", msg);

        let segment_name = data_segment_name(sender_port_id);
        let data_segment = match data_segment_type {
            DataSegmentType::Static => {
                DataSegmentView::open_static_segment(&segment_name, global_config)
            }
            DataSegmentType::Dynamic => {
                DataSegmentView::open_dynamic_segment(&segment_name, global_config)
            }
        };

        let data_segment = fail!(from this,
                                 when data_segment,
                                "{} since the sender data segment could not be opened.", msg);

        Ok(Self {
            receiver,
            data_segment,
            sender_port_id,
            tag: cyclic_tagger.create_tag(),
        })
    }
}

#[derive(Debug)]
pub(crate) struct Receiver<Service: service::Service> {
    pub(crate) connections: Vec<UnsafeCell<Option<SlotMapKey>>>,
    pub(crate) receiver_port_id: u128,
    pub(crate) service_state: Arc<ServiceState<Service, NoResource>>,
    pub(crate) buffer_size: usize,
    pub(crate) tagger: CyclicTagger,
    pub(crate) to_be_removed_connections: Option<UnsafeCell<Vec<SlotMapKey>>>,
    pub(crate) degradation_callback: Option<DegradationCallback<'static>>,
    pub(crate) message_type_details: MessageTypeDetails,
    pub(crate) receiver_max_borrowed_samples: usize,
    pub(crate) enable_safe_overflow: bool,
    pub(crate) number_of_channels: usize,
    pub(crate) connection_storage: UnsafeCell<SlotMap<Connection<Service>>>,
}

impl<Service: service::Service> Receiver<Service> {
    pub(crate) fn release_offset(&self, chunk: &ChunkDetails, channel_id: ChannelId) {
        let connection_storage = unsafe { &mut *self.connection_storage.get() };
        if let Some(connection) = connection_storage.get(chunk.connection_key) {
            if connection.sender_port_id != chunk.origin {
                return;
            }

            unsafe { connection.data_segment.unregister_offset(chunk.offset) };
            match connection.receiver.release(chunk.offset, channel_id) {
                Ok(()) => (),
                Err(ZeroCopyReleaseError::RetrieveBufferFull) => {
                    error!(from self, "This should never happen! The publishers retrieve channel is full and the sample cannot be returned.");
                }
            }
        }
    }

    pub(crate) fn set_channel_state(&self, channel_id: ChannelId, state: u64) -> bool {
        let mut ret_val = true;
        let connection_storage = unsafe { &mut *self.connection_storage.get() };
        for (_, connection) in connection_storage.iter() {
            ret_val &= connection.receiver.set_channel_state(channel_id, state)
        }

        ret_val
    }

    pub(crate) fn at_least_one_channel_has_state(&self, channel_id: ChannelId, state: u64) -> bool {
        let mut ret_val = false;
        let connection_storage = unsafe { &mut *self.connection_storage.get() };
        for (_, connection) in connection_storage.iter() {
            ret_val |= connection.receiver.has_channel_state(channel_id, state);
            if ret_val {
                break;
            }
        }

        ret_val
    }

    pub(crate) fn set_disconnect_hint(&self, channel_id: ChannelId, expected_state: u64) {
        let connection_storage = unsafe { &mut *self.connection_storage.get() };
        for (_, connection) in connection_storage.iter() {
            connection
                .receiver
                .set_disconnect_hint(channel_id, expected_state);
        }
    }

    pub(crate) fn invalidate_channel_state(&self, channel_id: ChannelId, expected_state: u64) {
        let connection_storage = unsafe { &mut *self.connection_storage.get() };
        for (_, connection) in connection_storage.iter() {
            connection
                .receiver
                .invalidate_channel_state(channel_id, expected_state);
        }
    }

    pub(crate) fn receiver_port_id(&self) -> u128 {
        self.receiver_port_id
    }

    pub(crate) fn create(
        &self,
        index: usize,
        sender_details: &SenderDetails,
    ) -> Result<(), ConnectionFailure> {
        let connection_storage = unsafe { &mut *self.connection_storage.get() };
        let key = connection_storage.insert(Connection::new(
            self,
            sender_details.data_segment_type,
            sender_details.port_id,
            sender_details.number_of_samples,
            sender_details.max_number_of_segments,
            &self.tagger,
        )?);
        let key = match key {
            Some(v) => v,
            None => {
                fatal_panic!(from self, "This should never happen! Internal connection storage capacity exceeded.")
            }
        };

        unsafe { *self.connections[index].get() = Some(key) };

        Ok(())
    }

    pub(crate) fn prepare_connection_removal(&self, index: usize) {
        if let Some(to_be_removed_connections) = &self.to_be_removed_connections {
            let key = unsafe { *self.connections[index].get() };
            let key = match key {
                None => return,
                Some(key) => key,
            };

            let connection_storage = unsafe { &mut *self.connection_storage.get() };

            if let Some(connection) = connection_storage.get_mut(key) {
                let mut keep_connection = false;
                for id in 0..self.number_of_channels {
                    if connection.receiver.has_data(ChannelId::new(id)) {
                        keep_connection = true;
                        break;
                    }
                }

                if keep_connection {
                    if !unsafe { &mut *to_be_removed_connections.get() }.push(key) {
                        warn!(from self,
                            "Expired connection buffer exceeded. A sender disconnected with undelivered samples that will be discarded. Increase the expired connection buffer to mitigate the problem.");
                        connection_storage.remove(key);
                    }
                } else {
                    connection_storage.remove(key);
                }
            }
        }
    }

    pub(crate) fn remove_connection(&self, index: usize) {
        self.prepare_connection_removal(index);
        unsafe { *self.connections[index].get() = None };
    }

    pub(crate) fn has_samples(&self, channel_id: ChannelId) -> bool {
        let connection_storage = unsafe { &mut *self.connection_storage.get() };
        for (_, connection) in connection_storage.iter() {
            if connection.receiver.has_data(channel_id) {
                return true;
            }
        }

        false
    }

    pub(crate) fn has_samples_in_active_connection(&self, channel_id: ChannelId) -> bool {
        let connection_storage = unsafe { &mut *self.connection_storage.get() };
        for connection_key in self.connections.iter() {
            if let Some(connection_key) = unsafe { &*connection_key.get() } {
                if let Some(connection) = connection_storage.get(*connection_key) {
                    if connection.receiver.has_data(channel_id) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn receive_from_connection(
        &self,
        connection: &Connection<Service>,
        connection_key: SlotMapKey,
        channel_id: ChannelId,
    ) -> Result<Option<(ChunkDetails, Chunk)>, ReceiveError> {
        let msg = "Unable to receive another sample";

        match connection.receiver.receive(channel_id) {
            Ok(data) => match data {
                None => Ok(None),
                Some(offset) => {
                    let details = ChunkDetails {
                        connection_key,
                        offset,
                        origin: connection.sender_port_id,
                    };

                    let offset = match connection
                        .data_segment
                        .register_and_translate_offset(offset)
                    {
                        Ok(offset) => offset,
                        Err(e) => {
                            if connection.data_segment.is_dynamic() {
                                warn!(from self, "Lost a sample. This only happens in the dynamic use case when a sender has reallocated its data segment and gone out of scope before the receiver has mapped the realloacted data segment. To circumvent this, you could either use static memory or increase the initial max slice len.");
                                return Ok(None);
                            }
                            fail!(from self, with ReceiveError::ConnectionFailure(ConnectionFailure::UnableToMapSendersDataSegment(e)),
                                "Unable to register and translate offset from sender {:?} since the received offset {:?} could not be registered and translated.",
                                connection.sender_port_id, offset);
                        }
                    };

                    Ok(Some((
                        details,
                        Chunk::new(&self.message_type_details, offset),
                    )))
                }
            },
            Err(ZeroCopyReceiveError::ReceiveWouldExceedMaxBorrowValue) => {
                fail!(from self, with ReceiveError::ExceedsMaxBorrows,
                    "{} since it would exceed the maximum {} of borrowed samples.",
                    msg, connection.receiver.max_borrowed_samples());
            }
        }
    }

    fn receive_from_to_be_removed_connections(
        &self,
        channel_id: ChannelId,
    ) -> Result<Option<(ChunkDetails, Chunk)>, ReceiveError> {
        let mut ret_val = None;
        if let Some(to_be_removed_connections) = &self.to_be_removed_connections {
            let to_be_removed_connections = unsafe { &mut *to_be_removed_connections.get() };

            if !to_be_removed_connections.is_empty() {
                let mut clean_connections = Vec::new(to_be_removed_connections.capacity());
                let connection_storage = unsafe { &mut *self.connection_storage.get() };

                for (n, connection_key) in to_be_removed_connections.iter().enumerate() {
                    let connection = match connection_storage.get(*connection_key) {
                        Some(connection) => connection,
                        None => {
                            clean_connections.push((n, *connection_key));
                            continue;
                        }
                    };

                    if connection.receiver.borrow_count(channel_id)
                        == connection.receiver.max_borrowed_samples()
                    {
                        continue;
                    }

                    if let Some((details, absolute_address)) =
                        self.receive_from_connection(connection, *connection_key, channel_id)?
                    {
                        ret_val = Some((details, absolute_address));
                        break;
                    } else {
                        clean_connections.push((n, *connection_key));
                    }
                }

                for idx in clean_connections.iter().rev() {
                    to_be_removed_connections.remove(idx.0);
                    connection_storage.remove(idx.1);
                }
            }
        }

        Ok(ret_val)
    }

    pub(crate) fn receive(
        &self,
        channel_id: ChannelId,
    ) -> Result<Option<(ChunkDetails, Chunk)>, ReceiveError> {
        if let Some(data) = self.receive_from_to_be_removed_connections(channel_id)? {
            return Ok(Some(data));
        }

        let msg = "Unable to receive data";
        let mut active_channel_count = 0;
        let mut all_channels_exceed_max_borrows = true;
        let connection_storage = unsafe { &*self.connection_storage.get() };
        for (connection_key, connection) in connection_storage.iter() {
            if !connection.receiver.has_data(channel_id) {
                continue;
            }

            active_channel_count += 1;
            if connection.receiver.borrow_count(channel_id)
                >= connection.receiver.max_borrowed_samples()
            {
                continue;
            } else {
                all_channels_exceed_max_borrows = false;
            }

            if let Some((details, absolute_address)) =
                self.receive_from_connection(connection, connection_key, channel_id)?
            {
                return Ok(Some((details, absolute_address)));
            }
        }

        if all_channels_exceed_max_borrows && active_channel_count != 0 {
            fail!(from self, with ReceiveError::ExceedsMaxBorrows,
                 "{msg} since every channel exceeds the max number of borrows.");
        }

        Ok(None)
    }

    pub(crate) fn start_update_connection_cycle(&self) {
        self.tagger.next_cycle();
    }

    pub(crate) fn update_connection(
        &self,
        index: usize,
        sender_details: SenderDetails,
    ) -> Result<(), ConnectionFailure> {
        let connection_storage = unsafe { &*self.connection_storage.get() };

        let is_connected = match unsafe { &*self.connections[index].get() } {
            None => true,
            Some(connection_key) => match connection_storage.get(*connection_key) {
                Some(connection) => {
                    let is_connected = connection.sender_port_id == sender_details.port_id;
                    if is_connected {
                        self.tagger.tag(connection);
                    }
                    !is_connected
                }
                None => true,
            },
        };

        if is_connected {
            self.prepare_connection_removal(index);

            match self.create(index, &sender_details) {
                Ok(()) => Ok(()),
                Err(e) => match &self.degradation_callback {
                    None => {
                        warn!(from self,
                                "Unable to establish connection to new sender {:?}.",
                                sender_details.port_id);
                        Ok(())
                    }
                    Some(c) => {
                        match c.call(
                            &self.service_state.static_config,
                            sender_details.port_id,
                            self.receiver_port_id(),
                        ) {
                            DegradationAction::Ignore => Ok(()),
                            DegradationAction::Warn => {
                                warn!(from self, "Unable to establish connection to new sender {:?}.",
                                        sender_details.port_id);
                                Ok(())
                            }
                            DegradationAction::Fail => {
                                fail!(from self, with e, "Unable to establish connection to new sender {:?}.",
                                        sender_details.port_id);
                            }
                        }
                    }
                },
            }
        } else {
            Ok(())
        }
    }

    pub(crate) fn finish_update_connection_cycle(&self) {
        let connection_storage = unsafe { &mut *self.connection_storage.get() };
        for (n, connection_key) in self.connections.iter().enumerate() {
            if let Some(connection_key) = unsafe { &*connection_key.get() } {
                if let Some(connection) = connection_storage.get(*connection_key) {
                    if !connection.was_tagged_by(&self.tagger) {
                        self.remove_connection(n);
                    }
                }
            }
        }
    }

    pub(crate) fn payload_size(&self) -> usize {
        self.message_type_details.payload.size
    }
}
