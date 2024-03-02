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
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::{alloc::Layout, marker::PhantomData, mem::MaybeUninit};

use super::port_identifiers::{UniquePublisherId, UniqueSubscriberId};
use crate::message::Message;
use crate::port::details::subscriber_connections::*;
use crate::port::update_connections::{ConnectionFailure, UpdateConnections};
use crate::port::DegrationAction;
use crate::raw_sample::RawSampleMut;
use crate::service;
use crate::service::config_scheme::data_segment_config;
use crate::service::dynamic_config::publish_subscribe::PublisherDetails;
use crate::service::header::publish_subscribe::Header;
use crate::service::naming_scheme::data_segment_name;
use crate::service::port_factory::publisher::{LocalPublisherConfig, UnableToDeliverStrategy};
use crate::service::static_config::publish_subscribe;
use crate::{config, sample_mut::SampleMut};
use iceoryx2_bb_container::queue::Queue;
use iceoryx2_bb_elementary::allocator::AllocationError;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_lock_free::mpmc::container::{ContainerHandle, ContainerState};
use iceoryx2_bb_log::{error, fail, fatal_panic, warn};
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::named_concept::NamedConceptBuilder;
use iceoryx2_cal::shared_memory::{
    SharedMemory, SharedMemoryBuilder, SharedMemoryCreateError, ShmPointer,
};
use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;
use iceoryx2_cal::shm_allocator::{self, PointerOffset, ShmAllocationError};
use iceoryx2_cal::zero_copy_connection::{
    ZeroCopyConnection, ZeroCopyCreationError, ZeroCopySendError, ZeroCopySender,
};

/// Defines a failure that can occur when a [`Publisher`] is created with
/// [`crate::service::port_factory::publisher::PortFactoryPublisher`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum PublisherCreateError {
    ExceedsMaxSupportedPublishers,
    UnableToCreateDataSegment,
}

impl std::fmt::Display for PublisherCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for PublisherCreateError {}

/// Defines a failure that can occur in [`Publisher::loan()`] and [`Publisher::loan_uninit()`]
/// or is part of [`PublisherSendError`] emitted in [`Publisher::send_copy()`].
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum PublisherLoanError {
    OutOfMemory,
    ExceedsMaxLoanedChunks,
    InternalFailure,
}

impl std::fmt::Display for PublisherLoanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for PublisherLoanError {}

enum_gen! {
    /// Failure that can be emitted when a [`crate::sample::Sample`] is sent via [`Publisher::send()`].
    PublisherSendError
  entry:
    ConnectionBrokenSincePublisherNoLongerExists,
    ConnectionCorrupted
  mapping:
    PublisherLoanError to LoanError,
    ConnectionFailure to ConnectionError
}

impl std::fmt::Display for PublisherSendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for PublisherSendError {}

#[derive(Debug)]
pub(crate) struct DataSegment<Service: service::Service> {
    sample_reference_counter: Vec<AtomicU64>,
    memory: Service::SharedMemory,
    message_size: usize,
    message_type_layout: Layout,
    port_id: UniquePublisherId,
    config: LocalPublisherConfig,
    dynamic_storage: Rc<Service::DynamicStorage>,

    subscriber_connections: SubscriberConnections<Service>,
    subscriber_list_state: UnsafeCell<ContainerState<UniqueSubscriberId>>,
    history: Option<UnsafeCell<Queue<usize>>>,
    static_config: crate::service::static_config::StaticConfig,
    loan_counter: AtomicUsize,
    is_active: AtomicBool,
}

impl<Service: service::Service> DataSegment<Service> {
    fn sample_index(&self, distance_to_chunk: usize) -> usize {
        distance_to_chunk / self.message_size
    }

    fn allocate(&self, layout: Layout) -> Result<ShmPointer, ShmAllocationError> {
        self.retrieve_returned_samples();

        let msg = "Unable to allocate Sample";
        let ptr = self.memory.allocate(layout)?;
        if self.sample_reference_counter[self.sample_index(ptr.offset.value())]
            .fetch_add(1, Ordering::Relaxed)
            != 0
        {
            fatal_panic!(from self,
                "{} since the allocated sample is already in use! This should never happen!", msg);
        }

        Ok(ptr)
    }

    fn borrow_sample(&self, distance_to_chunk: usize) {
        self.sample_reference_counter[self.sample_index(distance_to_chunk)]
            .fetch_add(1, Ordering::Relaxed);
    }

    fn release_sample(&self, distance_to_chunk: PointerOffset) {
        if self.sample_reference_counter[self.sample_index(distance_to_chunk.value())]
            .fetch_sub(1, Ordering::Relaxed)
            == 1
        {
            unsafe {
                self.memory
                    .deallocate(distance_to_chunk, self.message_type_layout);
            }
        }
    }

    fn retrieve_returned_samples(&self) {
        for i in 0..self.subscriber_connections.len() {
            match self.subscriber_connections.get(i) {
                Some(ref connection) => loop {
                    match connection.sender.reclaim() {
                        Ok(Some(ptr_dist)) => {
                            self.release_sample(ptr_dist);
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

    fn remove_connection(&self, i: usize) {
        if let Some(connection) = self.subscriber_connections.get(i) {
            while let Some(offset) =
                // # SAFETY: the receiver no longer exist, therefore we can
                //           reacquire all delivered samples
                unsafe { connection.sender.acquire_used_offsets() }
            {
                self.release_sample(offset);
            }

            self.subscriber_connections.remove(i);
        }
    }

    pub(crate) fn return_loaned_sample(&self, distance_to_chunk: PointerOffset) {
        self.release_sample(distance_to_chunk);
        self.loan_counter.fetch_sub(1, Ordering::Relaxed);
    }

    fn add_sample_to_history(&self, address_to_chunk: usize) {
        match &self.history {
            None => (),
            Some(history) => {
                let history = unsafe { &mut *history.get() };
                self.borrow_sample(address_to_chunk);
                match unsafe { history.push_with_overflow(address_to_chunk) } {
                    None => (),
                    Some(old) => self.release_sample(PointerOffset::new(old)),
                }
            }
        }
    }

    fn deliver_sample(&self, address_to_chunk: usize) -> Result<usize, PublisherSendError> {
        self.retrieve_returned_samples();

        let deliver_call = match self.config.unable_to_deliver_strategy {
            UnableToDeliverStrategy::Block => {
                <Service::Connection as ZeroCopyConnection>::Sender::blocking_send
            }
            UnableToDeliverStrategy::DiscardSample => {
                <Service::Connection as ZeroCopyConnection>::Sender::try_send
            }
        };

        let mut number_of_recipients = 0;
        for i in 0..self.subscriber_connections.len() {
            match self.subscriber_connections.get(i) {
                Some(ref connection) => {
                    match deliver_call(&connection.sender, PointerOffset::new(address_to_chunk)) {
                        Err(ZeroCopySendError::ReceiveBufferFull)
                        | Err(ZeroCopySendError::UsedChunkListFull) => {
                            /* causes no problem
                             *   blocking_send => can never happen
                             *   try_send => we tried and expect that the buffer is full
                             * */
                        }
                        Err(ZeroCopySendError::ConnectionCorrupted) => {
                            match &self.config.degration_callback {
                                Some(c) => match c.call(
                                    self.static_config.clone(),
                                    self.port_id,
                                    connection.subscriber_id,
                                ) {
                                    DegrationAction::Ignore => (),
                                    DegrationAction::Warn => {
                                        error!(from self,
                                            "While delivering the sample: {:?} a corrupted connection was detected with subscriber {:?}.",
                                            address_to_chunk, connection.subscriber_id);
                                    }
                                    DegrationAction::Fail => {
                                        fail!(from self, with PublisherSendError::ConnectionCorrupted,
                                            "While delivering the sample: {:?} a corrupted connection was detected with subscriber {:?}.",
                                            address_to_chunk, connection.subscriber_id);
                                    }
                                },
                                None => {
                                    error!(from self,
                                        "While delivering the sample: {:?} a corrupted connection was detected with subscriber {:?}.",
                                        address_to_chunk, connection.subscriber_id);
                                }
                            }
                        }
                        Ok(overflow) => {
                            self.borrow_sample(address_to_chunk);
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
        Ok(number_of_recipients)
    }

    fn populate_subscriber_channels(&self) -> Result<(), ZeroCopyCreationError> {
        let mut visited_indices = vec![];
        visited_indices.resize(self.subscriber_connections.capacity(), None);

        unsafe {
            (*self.subscriber_list_state.get()).for_each(|index, subscriber_id| {
                visited_indices[index as usize] = Some(*subscriber_id);
            })
        };

        for (i, index) in visited_indices.iter().enumerate() {
            match index {
                Some(subscriber_id) => {
                    let create_connection = match self.subscriber_connections.get(i) {
                        None => true,
                        Some(connection) => {
                            let create_connection = connection.subscriber_id != *subscriber_id;
                            if create_connection {
                                self.remove_connection(i);
                            }
                            create_connection
                        }
                    };

                    if create_connection {
                        match self.subscriber_connections.create(i, *subscriber_id) {
                            Ok(()) => match &self.subscriber_connections.get(i) {
                                Some(connection) => self.deliver_sample_history(connection),
                                None => {
                                    fatal_panic!(from self, "This should never happen! Unable to acquire previously created subscriber connection.")
                                }
                            },
                            Err(e) => match &self.config.degration_callback {
                                Some(c) => match c.call(
                                    self.static_config.clone(),
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
                }
                None => self.remove_connection(i),
            }
        }

        Ok(())
    }

    fn update_connections(&self) -> Result<(), ConnectionFailure> {
        if unsafe {
            self.dynamic_storage
                .get()
                .publish_subscribe()
                .subscribers
                .update_state(&mut *self.subscriber_list_state.get())
        } {
            fail!(from self, when self.populate_subscriber_channels(),
                "Connections were updated only partially since at least one connection to a Subscriber port failed.");
        }

        Ok(())
    }

    fn deliver_sample_history(&self, connection: &Connection<Service>) {
        match &self.history {
            None => (),
            Some(history) => {
                let history = unsafe { &mut *history.get() };
                for i in 0..history.len() {
                    let ptr_distance = unsafe { history.get_unchecked(i) };

                    match connection.sender.try_send(PointerOffset::new(ptr_distance)) {
                        Ok(_) => self.borrow_sample(ptr_distance),
                        Err(e) => {
                            warn!(from self, "Failed to deliver history to new subscriber via {:?} due to {:?}", connection, e);
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn send_sample(&self, address_to_chunk: usize) -> Result<usize, PublisherSendError> {
        let msg = "Unable to send sample";
        if !self.is_active.load(Ordering::Relaxed) {
            fail!(from self, with PublisherSendError::ConnectionBrokenSincePublisherNoLongerExists,
                "{} since the connections could not be updated.", msg);
        }

        fail!(from self, when self.update_connections(),
            "{} since the connections could not be updated.", msg);

        self.add_sample_to_history(address_to_chunk);
        self.deliver_sample(address_to_chunk)
    }
}

/// Sending endpoint of a publish-subscriber based communication.
#[derive(Debug)]
pub struct Publisher<Service: service::Service, MessageType: Debug> {
    pub(crate) data_segment: Rc<DataSegment<Service>>,
    dynamic_publisher_handle: Option<ContainerHandle>,
    _phantom_message_type: PhantomData<MessageType>,
}

impl<Service: service::Service, MessageType: Debug> Drop for Publisher<Service, MessageType> {
    fn drop(&mut self) {
        if let Some(handle) = self.dynamic_publisher_handle {
            self.data_segment
                .dynamic_storage
                .get()
                .publish_subscribe()
                .release_publisher_handle(handle)
        }
    }
}

impl<Service: service::Service, MessageType: Debug> Publisher<Service, MessageType> {
    pub(crate) fn new(
        service: &Service,
        static_config: &publish_subscribe::StaticConfig,
        config: LocalPublisherConfig,
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

        let dynamic_storage = Rc::clone(&service.state().dynamic_storage);
        let number_of_samples = service
            .state()
            .static_config
            .messaging_pattern
            .required_amount_of_samples_per_data_segment(config.max_loaned_samples);

        let data_segment = fail!(from origin, when Self::create_data_segment(port_id, service.state().global_config.as_ref(), number_of_samples, static_config),
                with PublisherCreateError::UnableToCreateDataSegment,
                "{} since the data segment could not be acquired.", msg);

        let data_segment = Rc::new(DataSegment {
            is_active: AtomicBool::new(true),
            memory: data_segment,
            message_size: std::mem::size_of::<Message<Header, MessageType>>(),
            message_type_layout: Layout::new::<MessageType>(),
            sample_reference_counter: {
                let mut v = Vec::with_capacity(number_of_samples);
                for _ in 0..number_of_samples {
                    v.push(AtomicU64::new(0));
                }
                v
            },
            dynamic_storage,
            port_id,
            subscriber_connections: SubscriberConnections::new(
                subscriber_list.capacity(),
                &service.state().global_config,
                port_id,
                static_config,
                number_of_samples,
            ),
            config,
            subscriber_list_state: unsafe { UnsafeCell::new(subscriber_list.get_state()) },
            history: match static_config.history_size == 0 {
                true => None,
                false => Some(UnsafeCell::new(Queue::new(static_config.history_size))),
            },
            static_config: service.state().static_config.clone(),
            loan_counter: AtomicUsize::new(0),
        });

        let mut new_self = Self {
            data_segment,
            dynamic_publisher_handle: None,
            _phantom_message_type: PhantomData,
        };

        if let Err(e) = new_self.data_segment.populate_subscriber_channels() {
            warn!(from new_self, "The new Publisher port is unable to connect to every Subscriber port, caused by {:?}.", e);
        }

        std::sync::atomic::compiler_fence(Ordering::SeqCst);

        // !MUST! be the last task otherwise a publisher is added to the dynamic config without the
        // creation of all required resources
        let dynamic_publisher_handle = match service
            .state()
            .dynamic_storage
            .get()
            .publish_subscribe()
            .add_publisher_id(PublisherDetails {
                publisher_id: port_id,
                number_of_samples,
            }) {
            Some(unique_index) => unique_index,
            None => {
                fail!(from origin, with PublisherCreateError::ExceedsMaxSupportedPublishers,
                            "{} since it would exceed the maximum supported amount of publishers of {}.",
                            msg, service.state().static_config.publish_subscribe().max_publishers);
            }
        };

        new_self.dynamic_publisher_handle = Some(dynamic_publisher_handle);

        Ok(new_self)
    }

    fn create_data_segment(
        port_id: UniquePublisherId,
        global_config: &config::Config,
        number_of_samples: usize,
        static_config: &publish_subscribe::StaticConfig,
    ) -> Result<Service::SharedMemory, SharedMemoryCreateError> {
        let allocator_config = shm_allocator::pool_allocator::Config {
            bucket_layout:
                // # SAFETY: type_size and type_alignment are acquired via
                //           core::mem::{size_of|align_of}
                unsafe {
                Layout::from_size_align_unchecked(
                    static_config.type_size,
                    static_config.type_alignment,
                )
            },
        };

        Ok(fail!(from "Publisher::create_data_segment()",
            when <<Service::SharedMemory as SharedMemory<PoolAllocator>>::Builder as NamedConceptBuilder<
            Service::SharedMemory,
                >>::new(&data_segment_name(port_id))
                .config(&data_segment_config::<Service>(global_config))
                .size(static_config.type_size * number_of_samples + static_config.type_alignment - 1)
                .create(&allocator_config),
            "Unable to create the data segment."))
    }

    /// Copies the input `value` into a [`crate::sample_mut::SampleMut`] and delivers it.
    /// On success it returns the number of [`crate::port::subscriber::Subscriber`]s that received
    /// the data, otherwise a [`PublisherSendError`] describing the failure.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
    /// #
    /// # let service = zero_copy::Service::new(&service_name)
    /// #     .publish_subscribe()
    /// #     .open_or_create::<u64>()?;
    /// #
    /// # let publisher = service.publisher().create()?;
    ///
    /// publisher.send_copy(1234)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn send_copy(&self, value: MessageType) -> Result<usize, PublisherSendError> {
        let msg = "Unable to send copy of message";
        let mut sample = fail!(from self, when self.loan_uninit(),
                                    "{} since the loan of a sample failed.", msg);

        sample.payload_mut().write(value);
        Ok(
            fail!(from self, when self.data_segment.send_sample(sample.offset_to_chunk.value()),
            "{} since the underlying send operation failed.", msg),
        )
    }

    /// Loans/allocates a [`crate::sample_mut::SampleMut`] from the underlying data segment of the [`Publisher`].
    /// The user has to initialize the payload before it can be sent.
    ///
    /// On failure it returns [`PublisherLoanError`] describing the failure.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
    /// #
    /// # let service = zero_copy::Service::new(&service_name)
    /// #     .publish_subscribe()
    /// #     .open_or_create::<u64>()?;
    /// #
    /// # let publisher = service.publisher().create()?;
    ///
    /// let sample = publisher.loan_uninit()?;
    /// let sample = sample.write_payload(42); // alternatively `sample.payload_mut()` can be use to access the `MaybeUninit<MessageType>`
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn loan_uninit(
        &self,
    ) -> Result<SampleMut<MaybeUninit<MessageType>, Service>, PublisherLoanError> {
        let msg = "Unable to loan Sample";

        if self.data_segment.loan_counter.load(Ordering::Relaxed)
            >= self.data_segment.config.max_loaned_samples
        {
            fail!(from self, with PublisherLoanError::ExceedsMaxLoanedChunks,
                "{} since already {} samples were loaned and it would exceed the maximum of parallel loans of {}. Release or send a loaned sample to loan another sample.",
                msg, self.data_segment.loan_counter.load(Ordering::Relaxed), self.data_segment.config.max_loaned_samples);
        }

        match self
            .data_segment
            .allocate(Layout::new::<Message<Header, MessageType>>())
        {
            Ok(chunk) => {
                let message =
                    chunk.data_ptr as *mut MaybeUninit<Message<Header, MaybeUninit<MessageType>>>;

                let sample = unsafe {
                    (*message).write(Message {
                        header: Header::new(self.data_segment.port_id),
                        data: MaybeUninit::uninit(),
                    });
                    RawSampleMut::new_unchecked(
                        message as *mut Message<Header, MaybeUninit<MessageType>>,
                    )
                };

                self.data_segment
                    .loan_counter
                    .fetch_add(1, Ordering::Relaxed);
                Ok(SampleMut::new(&self.data_segment, sample, chunk.offset))
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

impl<Service: service::Service, MessageType: Default + Debug> Publisher<Service, MessageType> {
    /// Loans/allocates a [`crate::sample_mut::SampleMut`] from the underlying data segment of the [`Publisher`]
    /// and initialize it with the default value. This can be a performance hit and [`Publisher::loan_uninit`]
    /// can be used to loan a [`core::mem::MaybeUninit<MessageType>`].
    ///
    /// On failure it returns [`PublisherLoanError`] describing the failure.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
    /// #
    /// # let service = zero_copy::Service::new(&service_name)
    /// #     .publish_subscribe()
    /// #     .open_or_create::<u64>()?;
    /// #
    /// # let publisher = service.publisher().create()?;
    ///
    /// let mut sample = publisher.loan()?;
    /// *sample.payload_mut() = 42;
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn loan(&self) -> Result<SampleMut<MessageType, Service>, PublisherLoanError> {
        Ok(self.loan_uninit()?.write_payload(MessageType::default()))
    }
}

impl<Service: service::Service, MessageType: Debug> UpdateConnections
    for Publisher<Service, MessageType>
{
    fn update_connections(&self) -> Result<(), ConnectionFailure> {
        self.data_segment.update_connections()
    }
}
