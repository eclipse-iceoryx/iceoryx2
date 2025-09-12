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

use alloc::sync::Arc;
use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::sync::atomic::Ordering;

use iceoryx2_bb_elementary::cyclic_tagger::*;
use iceoryx2_bb_log::{error, fail, fatal_panic, warn};
use iceoryx2_cal::named_concept::NamedConceptBuilder;
use iceoryx2_cal::shm_allocator::{AllocationError, PointerOffset, ShmAllocationError};
use iceoryx2_cal::zero_copy_connection::{
    ChannelId, ZeroCopyConnection, ZeroCopyConnectionBuilder, ZeroCopyCreationError,
    ZeroCopySendError, ZeroCopySender,
};
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicUsize;

use crate::node::SharedNode;
use crate::port::{DegradationAction, DegradationCallback, LoanError, SendError};
use crate::prelude::UnableToDeliverStrategy;
use crate::service::config_scheme::connection_config;
use crate::service::static_config::message_type_details::{MessageTypeDetails, TypeVariant};
use crate::service::{NoResource, ServiceState};
use crate::{service, service::naming_scheme::connection_name};

use super::channel_management::ChannelManagement;
use super::channel_management::INVALID_CHANNEL_STATE;
use super::chunk::ChunkMut;
use super::data_segment::DataSegment;
use super::segment_state::SegmentState;

#[derive(Clone, Copy)]
pub(crate) struct ReceiverDetails {
    pub(crate) port_id: u128,
    pub(crate) buffer_size: usize,
}

#[derive(Debug)]
pub(crate) struct Connection<Service: service::Service> {
    pub(crate) sender: <Service::Connection as ZeroCopyConnection>::Sender,
    pub(crate) receiver_port_id: u128,
    tag: Tag,
}

impl<Service: service::Service> Taggable for Connection<Service> {
    fn tag(&self) -> &Tag {
        &self.tag
    }
}

impl<Service: service::Service> Connection<Service> {
    fn new(
        this: &Sender<Service>,
        receiver_port_id: u128,
        buffer_size: usize,
        number_of_samples: usize,
        tag: Tag,
    ) -> Result<Self, ZeroCopyCreationError> {
        let msg = format!(
            "Unable to establish connection to receiver port {:?} from sender port {:?}",
            receiver_port_id, this.sender_port_id
        );
        if this.receiver_max_buffer_size < buffer_size {
            fail!(from this, with ZeroCopyCreationError::IncompatibleBufferSize,
                "{} since the receiver buffer size {} exceeds the max receiver buffer size of {}.",
                msg, buffer_size, this.receiver_max_buffer_size);
        }

        let sender = fail!(from this, when <Service::Connection as ZeroCopyConnection>::
                        Builder::new( &connection_name(this.sender_port_id, receiver_port_id))
                                .config(&connection_config::<Service>(this.shared_node.config()))
                                .buffer_size(buffer_size)
                                .receiver_max_borrowed_samples_per_channel(this.receiver_max_borrowed_samples)
                                .enable_safe_overflow(this.enable_safe_overflow)
                                .number_of_samples_per_segment(number_of_samples)
                                .max_supported_shared_memory_segments(this.max_number_of_segments)
                                .initial_channel_state(INVALID_CHANNEL_STATE)
                                .number_of_channels(this.number_of_channels)
                                .timeout(this.shared_node.config().global.service.creation_timeout)
                                .create_sender(),
                        "{}.", msg);

        Ok(Self {
            sender,
            receiver_port_id,
            tag,
        })
    }
}

#[derive(Debug)]
pub(crate) struct Sender<Service: service::Service> {
    pub(crate) segment_states: Vec<SegmentState>,
    pub(crate) data_segment: DataSegment<Service>,
    pub(crate) connections: Vec<UnsafeCell<Option<Connection<Service>>>>,
    pub(crate) sender_port_id: u128,
    pub(crate) shared_node: Arc<SharedNode<Service>>,
    pub(crate) receiver_max_buffer_size: usize,
    pub(crate) receiver_max_borrowed_samples: usize,
    pub(crate) sender_max_borrowed_samples: usize,
    pub(crate) enable_safe_overflow: bool,
    pub(crate) number_of_samples: usize,
    pub(crate) max_number_of_segments: u8,
    pub(crate) degradation_callback: Option<DegradationCallback<'static>>,
    pub(crate) service_state: Arc<ServiceState<Service, NoResource>>,
    pub(crate) tagger: CyclicTagger,
    pub(crate) loan_counter: IoxAtomicUsize,
    pub(crate) unable_to_deliver_strategy: UnableToDeliverStrategy,
    pub(crate) message_type_details: MessageTypeDetails,
    pub(crate) number_of_channels: usize,
}

impl<Service: service::Service> Sender<Service> {
    fn get(&self, index: usize) -> &Option<Connection<Service>> {
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

    pub(crate) fn get_connection_id_of(&self, receiver_port_id: u128) -> Option<usize> {
        for i in 0..self.len() {
            if let Some(connection) = self.get(i) {
                if connection.receiver_port_id == receiver_port_id {
                    return Some(i);
                }
            }
        }

        None
    }

    fn deliver_offset_to_connection_impl(
        &self,
        offset: PointerOffset,
        sample_size: usize,
        channel_id: ChannelId,
        connection_id: usize,
    ) -> Result<usize, SendError> {
        let deliver_call = match self.unable_to_deliver_strategy {
            UnableToDeliverStrategy::Block => {
                <Service::Connection as ZeroCopyConnection>::Sender::blocking_send
            }
            UnableToDeliverStrategy::DiscardSample => {
                <Service::Connection as ZeroCopyConnection>::Sender::try_send
            }
        };

        let mut number_of_recipients = 0;
        if let Some(ref connection) = self.get(connection_id) {
            match deliver_call(&connection.sender, offset, sample_size, channel_id) {
                Err(ZeroCopySendError::ReceiveBufferFull)
                | Err(ZeroCopySendError::UsedChunkListFull) => {
                    /* causes no problem
                     *   blocking_send => can never happen
                     *   try_send => we tried and expect that the buffer is full
                     * */
                }
                Err(ZeroCopySendError::ConnectionCorrupted) => match &self.degradation_callback {
                    Some(c) => match c.call(
                        &self.service_state.static_config,
                        self.sender_port_id,
                        connection.receiver_port_id,
                    ) {
                        DegradationAction::Ignore => (),
                        DegradationAction::Warn => {
                            error!(from self,
                                        "While delivering the sample: {:?} a corrupted connection was detected with receiver {:?}.",
                                        offset, connection.receiver_port_id);
                        }
                        DegradationAction::Fail => {
                            fail!(from self, with SendError::ConnectionCorrupted,
                                        "While delivering the sample: {:?} a corrupted connection was detected with receiver {:?}.",
                                        offset, connection.receiver_port_id);
                        }
                    },
                    None => {
                        error!(from self,
                                    "While delivering the sample: {:?} a corrupted connection was detected with receiver {:?}.",
                                    offset, connection.receiver_port_id);
                    }
                },
                Ok(overflow) => {
                    self.borrow_sample(offset);
                    number_of_recipients += 1;

                    if let Some(old) = overflow {
                        self.release_sample(old)
                    }
                }
            }
        }
        Ok(number_of_recipients)
    }

    pub(crate) fn has_disconnect_hint(
        &self,
        channel_id: ChannelId,
        connection_id: usize,
        state: u64,
    ) -> bool {
        if let Some(ref connection) = self.get(connection_id) {
            connection.sender.has_disconnect_hint(channel_id, state)
        } else {
            false
        }
    }

    pub(crate) fn has_channel_state(
        &self,
        channel_id: ChannelId,
        connection_id: usize,
        state: u64,
    ) -> bool {
        if let Some(ref connection) = self.get(connection_id) {
            connection.sender.has_channel_state(channel_id, state)
        } else {
            false
        }
    }

    pub(crate) fn invalidate_channel_state(
        &self,
        channel_id: ChannelId,
        connection_id: usize,
        expected_state: u64,
    ) {
        if let Some(ref connection) = self.get(connection_id) {
            connection
                .sender
                .invalidate_channel_state(channel_id, expected_state);
        }
    }

    pub(crate) fn deliver_offset_to_connection(
        &self,
        offset: PointerOffset,
        sample_size: usize,
        channel_id: ChannelId,
        connection_id: usize,
    ) -> Result<usize, SendError> {
        self.retrieve_returned_samples();
        self.deliver_offset_to_connection_impl(offset, sample_size, channel_id, connection_id)
    }

    pub(crate) fn deliver_offset(
        &self,
        offset: PointerOffset,
        sample_size: usize,
        channel_id: ChannelId,
    ) -> Result<usize, SendError> {
        self.retrieve_returned_samples();

        let mut number_of_recipients = 0;
        for i in 0..self.len() {
            number_of_recipients +=
                self.deliver_offset_to_connection_impl(offset, sample_size, channel_id, i)?;
        }
        Ok(number_of_recipients)
    }

    pub(crate) fn return_loaned_sample(&self, distance_to_chunk: PointerOffset) {
        self.release_sample(distance_to_chunk);
        self.loan_counter.fetch_sub(1, Ordering::Relaxed);
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
            self.tagger.create_tag(),
        )?);

        Ok(())
    }

    fn len(&self) -> usize {
        self.connections.len()
    }

    pub(crate) fn allocate(&self, layout: Layout) -> Result<ChunkMut, LoanError> {
        self.retrieve_returned_samples();
        let msg = "Unable to allocate data";

        if self.loan_counter.load(Ordering::Relaxed) >= self.sender_max_borrowed_samples {
            fail!(from self, with LoanError::ExceedsMaxLoans,
                "{} {:?} since already {} samples were loaned and it would exceed the maximum of parallel loans of {}. Release or send a loaned sample to loan another sample.",
                msg, layout, self.loan_counter.load(Ordering::Relaxed), self.sender_max_borrowed_samples);
        }

        let shm_pointer = match self.data_segment.allocate(layout) {
            Ok(chunk) => chunk,
            Err(ShmAllocationError::AllocationError(AllocationError::OutOfMemory)) => {
                fail!(from self, with LoanError::OutOfMemory,
                    "{} {:?} since the underlying shared memory is out of memory.", msg, layout);
            }
            Err(ShmAllocationError::AllocationError(AllocationError::SizeTooLarge))
            | Err(ShmAllocationError::AllocationError(AllocationError::AlignmentFailure)) => {
                fatal_panic!(from self, "{} {:?} since the system seems to be corrupted.", msg, layout);
            }
            Err(v) => {
                fail!(from self, with LoanError::InternalFailure,
                    "{} {:?} since an internal failure occurred ({:?}).", msg, layout, v);
            }
        };

        let (ref_count, sample_size) = self.borrow_sample(shm_pointer.offset);
        if ref_count != 0 {
            fatal_panic!(from self,
                "{} since the allocated sample is already in use! This should never happen!", msg);
        }

        self.loan_counter.fetch_add(1, Ordering::Relaxed);
        Ok(ChunkMut::new(
            &self.message_type_details,
            shm_pointer,
            sample_size,
        ))
    }

    pub(crate) fn borrow_sample(&self, offset: PointerOffset) -> (u64, usize) {
        let segment_id = offset.segment_id();
        let segment_state = &self.segment_states[segment_id.value() as usize];
        let mut payload_size = segment_state.payload_size();
        if segment_state.payload_size() == 0 {
            payload_size = self.data_segment.bucket_size(segment_id);
            segment_state.set_payload_size(payload_size);
        }
        (segment_state.borrow_sample(offset.offset()), payload_size)
    }

    pub(crate) fn retrieve_returned_samples(&self) {
        for i in 0..self.len() {
            if let Some(ref connection) = self.get(i) {
                for channel_id in 0..self.number_of_channels {
                    let id = ChannelId::new(channel_id);
                    loop {
                        match connection.sender.reclaim(id) {
                            Ok(Some(ptr_dist)) => {
                                self.release_sample(ptr_dist);
                            }
                            Ok(None) => break,
                            Err(e) => {
                                warn!(from self, "Unable to reclaim samples from connection {:?} due to {:?}. This may lead to a situation where no more samples will be delivered to this connection.", connection, e)
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn release_sample(&self, offset: PointerOffset) {
        if self.segment_states[offset.segment_id().value() as usize].release_sample(offset.offset())
            == 1
        {
            unsafe {
                self.data_segment.deallocate_bucket(offset);
            }
        }
    }

    fn remove_connection(&self, i: usize) {
        if let Some(connection) = self.get(i) {
            // # SAFETY: the receiver no longer exist, therefore we can
            //           reacquire all delivered samples
            unsafe {
                connection
                    .sender
                    .acquire_used_offsets(|offset| self.release_sample(offset))
            };

            *self.get_mut(i) = None;
        }
    }

    pub(crate) fn start_update_connection_cycle(&self) {
        self.tagger.next_cycle();
    }

    pub(crate) fn update_connection<E: Fn(&Connection<Service>)>(
        &self,
        index: usize,
        receiver_details: ReceiverDetails,
        establish_new_connection_call: E,
    ) -> Result<(), ZeroCopyCreationError> {
        let create_connection = match self.get(index) {
            None => true,
            Some(connection) => {
                let is_connected = connection.receiver_port_id == receiver_details.port_id;
                if is_connected {
                    self.tagger.tag(connection);
                } else {
                    self.remove_connection(index);
                }
                !is_connected
            }
        };

        if create_connection {
            match self.create(index, receiver_details) {
                Ok(()) => match &self.get(index) {
                    Some(connection) => establish_new_connection_call(connection),
                    None => {
                        fatal_panic!(from self, "This should never happen! Unable to acquire previously created receiver connection.")
                    }
                },
                Err(e) => match &self.degradation_callback {
                    Some(c) => match c.call(
                        &self.service_state.static_config,
                        self.sender_port_id,
                        receiver_details.port_id,
                    ) {
                        DegradationAction::Ignore => (),
                        DegradationAction::Warn => {
                            warn!(from self,
                                            "Unable to establish connection to new receiver {:?}.",
                                            receiver_details.port_id )
                        }
                        DegradationAction::Fail => {
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

        Ok(())
    }

    pub(crate) fn finish_update_connection_cycle(&self) {
        for n in 0..self.len() {
            if let Some(connection) = self.get(n) {
                if !connection.was_tagged_by(&self.tagger) {
                    self.remove_connection(n);
                }
            }
        }
    }

    pub(crate) fn payload_size(&self) -> usize {
        self.message_type_details.payload.size
    }

    pub(crate) fn sample_layout(&self, number_of_elements: usize) -> Layout {
        self.message_type_details.sample_layout(number_of_elements)
    }

    pub(crate) fn payload_type_variant(&self) -> TypeVariant {
        self.message_type_details.payload.variant
    }
}
