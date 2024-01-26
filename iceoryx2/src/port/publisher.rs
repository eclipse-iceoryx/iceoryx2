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

//! # Examples
//!
//! ```
//! use iceoryx2::{prelude::*, service::port_factory::publisher::UnableToDeliverStrategy};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let service_name = ServiceName::new("My/Funk/ServiceName")?;
//! let service = zero_copy::Service::new(&service_name)
//!     .publish_subscribe()
//!     .open_or_create::<u64>()?;
//!
//! let publisher = service
//!     .publisher()
//!     // defines how many samples can be loaned in parallel
//!     .max_loaned_samples(5)
//!     // defines behavior when subscriber queue is full in an non-overflowing service
//!     .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample)
//!     .create()?;
//!
//! // loan some initialized memory and send it
//! // the message type must implement the [`core::default::Default`] trait in order to be able to use this API
//! let mut sample = publisher.loan()?;
//! *sample.payload_mut() = 1337;
//! sample.send()?;
//!
//! // loan some uninitialized memory and send it
//! let sample = publisher.loan_uninit()?;
//! let sample = sample.write_payload(1337);
//! sample.send()?;
//!
//! // loan some uninitialized memory and send it (with direct access of [`core::mem::MaybeUninit<MessageType>`])
//! let mut sample = publisher.loan_uninit()?;
//! sample.payload_mut().write(1337);
//! let sample = unsafe { sample.assume_init() };
//! sample.send()?;
//!
//! // send a copy of the value
//! publisher.send_copy(313)?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! See also, [`crate::port::publisher::Publisher`]

use std::cell::UnsafeCell;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::{alloc::Layout, marker::PhantomData, mem::MaybeUninit};

use super::port_identifiers::{UniquePublisherId, UniqueSubscriberId};
use super::publish::internal::PublishMgmt;
use super::publish::{
    DefaultLoan, Publish, PublisherCreateError, PublisherLoanError, PublisherSendError, SendCopy,
    UninitLoan,
};
use crate::message::Message;
use crate::payload_mut::{internal::PayloadMgmt, PayloadMut, UninitPayloadMut};
use crate::port::details::subscriber_connections::*;
use crate::port::update_connections::{ConnectionFailure, UpdateConnections};
use crate::port::{DegrationAction, DegrationCallback};
use crate::raw_sample::RawSampleMut;
use crate::service;
use crate::service::config_scheme::data_segment_config;
use crate::service::header::publish_subscribe::Header;
use crate::service::naming_scheme::data_segment_name;
use crate::service::port_factory::publisher::{LocalPublisherConfig, UnableToDeliverStrategy};
use crate::service::static_config::publish_subscribe;
use crate::{config, sample_mut::SampleMut};
use iceoryx2_bb_container::queue::Queue;
use iceoryx2_bb_elementary::allocator::AllocationError;
use iceoryx2_bb_lock_free::mpmc::container::ContainerState;
use iceoryx2_bb_lock_free::mpmc::unique_index_set::UniqueIndex;
use iceoryx2_bb_log::{fail, fatal_panic, warn};
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::named_concept::NamedConceptBuilder;
use iceoryx2_cal::shared_memory::{SharedMemory, SharedMemoryBuilder, SharedMemoryCreateError};
use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;
use iceoryx2_cal::shm_allocator::{self, PointerOffset, ShmAllocationError};
use iceoryx2_cal::zero_copy_connection::{
    ZeroCopyConnection, ZeroCopyCreationError, ZeroCopySendError, ZeroCopySender,
};

/// Sending endpoint of a publish-subscriber based communication.
#[derive(Debug)]
pub struct Publisher<'a, 'config: 'a, Service: service::Details<'config>, MessageType: Debug> {
    port_id: UniquePublisherId,
    pub(crate) sample_reference_counter: Vec<AtomicU64>,
    pub(crate) data_segment: Service::SharedMemory,
    config: LocalPublisherConfig,

    subscriber_connections: SubscriberConnections<'config, Service>,
    subscriber_list_state: UnsafeCell<ContainerState<'a, UniqueSubscriberId>>,
    history: Option<UnsafeCell<Queue<usize>>>,
    service: &'a Service,
    degration_callback: Option<DegrationCallback<'a>>,
    loan_counter: AtomicUsize,
    _dynamic_config_guard: UniqueIndex<'a>,
    _phantom_message_type: PhantomData<MessageType>,
}

impl<'a, 'config: 'a, Service: service::Details<'config>, MessageType: Debug>
    Publisher<'a, 'config, Service, MessageType>
{
    pub(crate) fn new(
        service: &'a Service,
        static_config: &publish_subscribe::StaticConfig,
        config: &LocalPublisherConfig,
    ) -> Result<Self, PublisherCreateError> {
        let msg = "Unable to create Publisher port";
        let origin = "Publisher::new()";
        let port_id = UniquePublisherId::new();
        let subscriber_list = &service
            .state()
            .dynamic_storage
            .get()
            .publish_subscribe()
            .subscribers;

        let number_of_samples = service
            .state()
            .static_config
            .messaging_pattern
            .required_amount_of_samples_per_data_segment(config.max_loaned_samples);

        let data_segment = fail!(from origin, when Self::create_data_segment(port_id, service.state().global_config, number_of_samples),
                with PublisherCreateError::UnableToCreateDataSegment,
                "{} since the data segment could not be acquired.", msg);

        // !MUST! be the last task otherwise a publisher is added to the dynamic config without the
        // creation of all required resources
        let _dynamic_config_guard = match service
            .state()
            .dynamic_storage
            .get()
            .publish_subscribe()
            .add_publisher_id(port_id)
        {
            Some(unique_index) => unique_index,
            None => {
                fail!(from origin, with PublisherCreateError::ExceedsMaxSupportedPublishers,
                            "{} since it would exceed the maximum supported amount of publishers of {}.",
                            msg, service.state().static_config.publish_subscribe().max_publishers);
            }
        };

        let new_self = Self {
            port_id,
            subscriber_connections: SubscriberConnections::new(
                subscriber_list.capacity(),
                service.state().global_config,
                port_id,
                static_config,
            ),
            data_segment,
            config: *config,
            sample_reference_counter: {
                let mut v = Vec::with_capacity(number_of_samples);
                for _ in 0..number_of_samples {
                    v.push(AtomicU64::new(0));
                }
                v
            },
            subscriber_list_state: unsafe { UnsafeCell::new(subscriber_list.get_state()) },
            history: match static_config.history_size == 0 {
                true => None,
                false => Some(UnsafeCell::new(Queue::new(static_config.history_size))),
            },
            service,
            degration_callback: None,
            loan_counter: AtomicUsize::new(0),
            _dynamic_config_guard,
            _phantom_message_type: PhantomData,
        };

        if let Err(e) = new_self.populate_subscriber_channels() {
            warn!(from new_self, "The new Publisher port is unable to connect to every Subscriber port, caused by {:?}.", e);
        }

        Ok(new_self)
    }

    fn populate_subscriber_channels(&self) -> Result<(), ZeroCopyCreationError> {
        let mut visited_indices = vec![];
        visited_indices.resize(self.subscriber_connections.capacity(), None);

        unsafe {
            (*self.subscriber_list_state.get()).for_each(|index, subscriber_id| {
                visited_indices[index as usize] = Some(*subscriber_id);
            })
        };

        // retrieve samples before destroying channel
        self.retrieve_returned_samples();

        for (i, index) in visited_indices.iter().enumerate() {
            match index {
                Some(subscriber_id) => {
                    match self.subscriber_connections.create(i, *subscriber_id) {
                        Ok(false) => (),
                        Ok(true) => match &self.subscriber_connections.get(i) {
                            Some(connection) => self.deliver_history(connection),
                            None => {
                                fatal_panic!(from self, "This should never happen! Unable to acquire previously created subscriber connection.")
                            }
                        },
                        Err(e) => match &self.degration_callback {
                            Some(c) => match c.call(
                                self.service.state().static_config.clone(),
                                self.port_id,
                                *subscriber_id,
                            ) {
                                DegrationAction::Ignore => (),
                                DegrationAction::Warn => {
                                    warn!(from self, "Unable to establish connection to new subscriber {:?}.", subscriber_id )
                                }
                                DegrationAction::Fail => {
                                    fail!(from self, with e,
                                           "Unable to establish connection to new subscriber {:?}.", subscriber_id );
                                }
                            },
                            None => {
                                warn!(from self, "Unable to establish connection to new subscriber {:?}.", subscriber_id )
                            }
                        },
                    }
                }
                None => self.subscriber_connections.remove(i),
            }
        }

        Ok(())
    }

    fn deliver_history(&self, connection: &Connection<'config, Service>) {
        match &self.history {
            None => (),
            Some(history) => {
                let history = unsafe { &mut *history.get() };
                for i in 0..history.len() {
                    let ptr_distance = unsafe { history.get_unchecked(i) };

                    match connection.sender.try_send(PointerOffset::new(ptr_distance)) {
                        Ok(_) => {
                            self.sample_reference_counter[Self::sample_index(ptr_distance)]
                                .fetch_add(1, Ordering::Relaxed);
                        }
                        Err(e) => {
                            warn!(from self, "Failed to deliver history to new subscriber via {:?} due to {:?}", connection, e);
                        }
                    }
                }
            }
        }
    }

    fn sample_index(distance_to_chunk: usize) -> usize {
        distance_to_chunk / std::mem::size_of::<Message<Header, MessageType>>()
    }

    fn create_data_segment(
        port_id: UniquePublisherId,
        global_config: &'config config::Config,
        number_of_samples: usize,
    ) -> Result<Service::SharedMemory, SharedMemoryCreateError> {
        let allocator_config = shm_allocator::pool_allocator::Config {
            bucket_layout: Layout::new::<Message<Header, MessageType>>(),
        };
        let chunk_size = allocator_config.bucket_layout.size();
        let chunk_align = allocator_config.bucket_layout.align();

        Ok(fail!(from "Publisher::create_data_segment()",
            when <<Service::SharedMemory as SharedMemory<PoolAllocator>>::Builder as NamedConceptBuilder<
            Service::SharedMemory,
                >>::new(&data_segment_name(port_id))
                .config(&data_segment_config::<Service>(global_config))
                .size(chunk_size * number_of_samples + chunk_align - 1)
                .create(&allocator_config),
            "Unable to create the data segment."))
    }

    fn add_to_history(&self, address_to_chunk: usize) {
        match &self.history {
            None => (),
            Some(history) => {
                let history = unsafe { &mut *history.get() };
                self.sample_reference_counter[Self::sample_index(address_to_chunk)]
                    .fetch_add(1, Ordering::Relaxed);
                match unsafe { history.push_with_overflow(address_to_chunk) } {
                    None => (),
                    Some(old) => self.release_sample(PointerOffset::new(old)),
                }
            }
        }
    }

    fn deliver_sample(&self, address_to_chunk: usize) -> usize {
        let deliver_call = match self.config.unable_to_deliver_strategy {
            UnableToDeliverStrategy::Block => <<Service as service::Details<'config>>::Connection as ZeroCopyConnection>::Sender::blocking_send,
            UnableToDeliverStrategy::DiscardSample => <<Service as service::Details<'config>>::Connection as ZeroCopyConnection>::Sender::try_send,
        };

        let mut number_of_recipients = 0;
        for i in 0..self.subscriber_connections.len() {
            match self.subscriber_connections.get(i) {
                Some(ref connection) => {
                    match deliver_call(&connection.sender, PointerOffset::new(address_to_chunk)) {
                        Err(ZeroCopySendError::ReceiveBufferFull) => {
                            /* causes no problem
                             *   blocking_send => can never happen
                             *   try_send => we tried and expect that the buffer is full
                             * */
                        }
                        Err(ZeroCopySendError::ClearRetrieveChannelBeforeSend) => {
                            warn!(from self, "Unable to send sample via connection {:?} since the retrieve buffer is full. This can be caused by a corrupted retrieve channel.", connection);
                        }
                        Ok(overflow) => {
                            self.sample_reference_counter[Self::sample_index(address_to_chunk)]
                                .fetch_add(1, Ordering::Relaxed);
                            number_of_recipients += 1;

                            if let Some(old) = overflow {
                                self.release_sample(old)
                            }
                        }
                    }
                }
                None => (),
            }
        }
        number_of_recipients
    }

    fn release_sample(&self, distance_to_chunk: PointerOffset) {
        if self.sample_reference_counter[Self::sample_index(distance_to_chunk.value())]
            .fetch_sub(1, Ordering::Relaxed)
            == 1
        {
            unsafe {
                fatal_panic!(from self, when self.data_segment
                .deallocate(
                    distance_to_chunk,
                    Layout::new::<MessageType>(),
                ), "Internal logic error. The sample should always contain a valid memory chunk from the provided allocator.");
            };
        }
    }

    fn retrieve_returned_samples(&self) {
        for i in 0..self.subscriber_connections.len() {
            match self.subscriber_connections.get(i) {
                Some(ref connection) => loop {
                    match connection.sender.reclaim() {
                        Ok(Some(ptr_dist)) => {
                            let sample_index = Self::sample_index(ptr_dist.value());

                            if self.sample_reference_counter[sample_index]
                                .fetch_sub(1, Ordering::Relaxed)
                                == 1
                            {
                                unsafe {
                                    fatal_panic!(from self, when self.data_segment
                                    .deallocate(
                                        ptr_dist,
                                        Layout::new::<Message<Header, MessageType>>(),
                                    ), "This should never happen! Failed to deallocate the reclaimed ptr. Either the data was corrupted or an invalid ptr was returned.")
                                };
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            warn!(from self, "Unable to reclaim samples from connection {:?} due to {:?}. This may lead to a situation where no more samples will be delivered to this connection.", connection, e)
                        }
                    }
                },
                None => (),
            }
        }
    }

    /// Sets the [`DegrationCallback`] of the [`Publisher`]. Whenever a connection to a
    /// [`crate::port::subscriber::Subscriber`] is corrupted or a seems to be dead, this callback
    /// is called and depending on the returned [`DegrationAction`] measures will be taken.
    pub fn set_degration_callback<
        'publisher,
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
impl<'a, 'config: 'a, Service: service::Details<'config>, MessageType: Debug + Default>
    Publish<MessageType> for Publisher<'a, 'config, Service, MessageType>
{
}

impl<'a, 'config: 'a, Service: service::Details<'config>, MessageType: Debug> UpdateConnections
    for Publisher<'a, 'config, Service, MessageType>
{
    fn update_connections(&self) -> Result<(), ConnectionFailure> {
        if unsafe { (*self.subscriber_list_state.get()).update() } {
            fail!(from self, when self.populate_subscriber_channels(),
                "Connections were updated only partially since at least one connection to a Subscriber port failed.");
        }

        Ok(())
    }
}

impl<'a, 'config: 'a, Service: service::Details<'config>, MessageType: Debug> SendCopy<MessageType>
    for Publisher<'a, 'config, Service, MessageType>
{
    fn send_copy(&self, value: MessageType) -> Result<usize, PublisherSendError> {
        let msg = "Unable to send copy of message";
        let mut sample = fail!(from self, when self.loan_uninit(),
                                    "{} since the loan of a sample failed.", msg);

        sample.payload_mut().write(value);
        Ok(
            fail!(from self, when self.send_impl(sample.offset_to_chunk().value()),
            "{} since the underlying send operation failed.", msg),
        )
    }
}

impl<'a, 'config: 'a, Service: service::Details<'config>, MessageType: Debug>
    UninitLoan<MessageType> for Publisher<'a, 'config, Service, MessageType>
{
    fn loan_uninit(&self) -> Result<SampleMut<MaybeUninit<MessageType>>, PublisherLoanError> {
        self.retrieve_returned_samples();
        let msg = "Unable to loan Sample";

        if self.loan_counter.load(Ordering::Relaxed) >= self.config.max_loaned_samples {
            fail!(from self, with PublisherLoanError::ExceedsMaxLoanedChunks,
                "{} since already {} samples were loaned and it would exceed the maximum of parallel loans of {}. Release or send a loaned sample to loan another sample.",
                msg, self.loan_counter.load(Ordering::Relaxed), self.config.max_loaned_samples);
        }

        match self
            .data_segment
            .allocate(Layout::new::<Message<Header, MessageType>>())
        {
            Ok(chunk) => {
                if self.sample_reference_counter[Self::sample_index(chunk.offset.value())]
                    .fetch_add(1, Ordering::Relaxed)
                    != 0
                {
                    fatal_panic!(from self,
                                "{} since the allocated sample is already in use! This should never happen!", msg);
                }

                let message =
                    chunk.data_ptr as *mut MaybeUninit<Message<Header, MaybeUninit<MessageType>>>;

                let sample = unsafe {
                    (*message).write(Message {
                        header: Header::new(self.port_id),
                        data: MaybeUninit::uninit(),
                    });
                    RawSampleMut::new_unchecked(
                        message as *mut Message<Header, MaybeUninit<MessageType>>,
                    )
                };

                self.loan_counter.fetch_add(1, Ordering::Relaxed);
                Ok(SampleMut::new(self, sample, chunk.offset))
            }
            Err(ShmAllocationError::AllocationError(AllocationError::OutOfMemory)) => {
                fail!(from self, with PublisherLoanError::OutOfMemory,
                    "{} since the underlying shared memory is out of memory.", msg);
            }
            Err(ShmAllocationError::AllocationError(AllocationError::SizeTooLarge))
            | Err(ShmAllocationError::AllocationError(AllocationError::AlignmentFailure)) => {
                fatal_panic!(from self, "{} since the system seems to be corrupted.", msg);
            }
            Err(v) => {
                fail!(from self, with PublisherLoanError::InternalFailure,
                    "{} since an internal failure occurred ({:?}).", msg, v);
            }
        }
    }
}

impl<'a, 'config: 'a, Service: service::Details<'config>, MessageType: Debug> PublishMgmt
    for Publisher<'a, 'config, Service, MessageType>
{
    fn return_loaned_sample(&self, distance_to_chunk: PointerOffset) {
        self.release_sample(distance_to_chunk);
        self.loan_counter.fetch_sub(1, Ordering::Relaxed);
    }

    fn send_impl(&self, address_to_chunk: usize) -> Result<usize, ConnectionFailure> {
        fail!(from self, when self.update_connections(),
            "Unable to send sample since the connections could not be updated.");

        self.add_to_history(address_to_chunk);
        Ok(self.deliver_sample(address_to_chunk))
    }
}

impl<'a, 'config: 'a, Service: service::Details<'config>, MessageType: Default + Debug>
    DefaultLoan<MessageType> for Publisher<'a, 'config, Service, MessageType>
{
    fn loan(&self) -> Result<SampleMut<MessageType>, PublisherLoanError> {
        Ok(self.loan_uninit()?.write_payload(MessageType::default()))
    }
}
