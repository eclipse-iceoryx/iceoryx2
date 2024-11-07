// Copyright (c) 2023 - 2024 Contributors to the Eclipse Foundation
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
//! ## Typed API
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .open_or_create()?;
//!
//! let publisher = service
//!     .publisher_builder()
//!     // defines how many samples can be loaned in parallel
//!     .max_loaned_samples(5)
//!     // defines behavior when subscriber queue is full in an non-overflowing service
//!     .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample)
//!     .create()?;
//!
//! // loan some initialized memory and send it
//! // the payload type must implement the [`core::default::Default`] trait in order to be able to use this API
//! let mut sample = publisher.loan()?;
//! *sample.payload_mut() = 1337;
//! sample.send()?;
//!
//! // loan some uninitialized memory and send it
//! let sample = publisher.loan_uninit()?;
//! let sample = sample.write_payload(1337);
//! sample.send()?;
//!
//! // loan some uninitialized memory and send it (with direct access of [`core::mem::MaybeUninit<Payload>`])
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
//! ## Slice API
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<[usize]>()
//!     .open_or_create()?;
//!
//! let publisher = service
//!     .publisher_builder()
//!     // defines the maximum length of a slice
//!     .max_slice_len(128)
//!     // defines how many samples can be loaned in parallel
//!     .max_loaned_samples(5)
//!     // defines behavior when subscriber queue is full in an non-overflowing service
//!     .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample)
//!     .create()?;
//!
//! // loan some initialized memory and send it
//! // the payload type must implement the [`core::default::Default`] trait in order to be able to use this API
//! // we acquire a slice of length 12
//! let mut sample = publisher.loan_slice(12)?;
//! sample.payload_mut()[5] = 1337;
//! sample.send()?;
//!
//! // loan uninitialized slice of length 60 and send it
//! let sample = publisher.loan_slice_uninit(60)?;
//! // initialize the n element of the slice with the value n * 123
//! let sample = sample.write_from_fn(|n| n * 123 );
//! sample.send()?;
//!
//! // loan some uninitialized memory and send it (with direct access of [`core::mem::MaybeUninit<Payload>`])
//! let mut sample = publisher.loan_slice_uninit(42)?;
//! for element in sample.payload_mut() {
//!     element.write(1337);
//! }
//! let sample = unsafe { sample.assume_init() };
//! sample.send()?;
//!
//! # Ok(())
//! # }
//! ```

use super::port_identifiers::UniquePublisherId;
use super::UniqueSubscriberId;
use crate::port::details::subscriber_connections::*;
use crate::port::update_connections::{ConnectionFailure, UpdateConnections};
use crate::port::DegrationAction;
use crate::raw_sample::RawSampleMut;
use crate::sample_mut_uninit::SampleMutUninit;
use crate::service::builder::publish_subscribe::CustomPayloadMarker;
use crate::service::config_scheme::{connection_config, data_segment_config};
use crate::service::dynamic_config::publish_subscribe::{PublisherDetails, SubscriberDetails};
use crate::service::header::publish_subscribe::Header;
use crate::service::naming_scheme::{
    data_segment_name, extract_publisher_id_from_connection, extract_subscriber_id_from_connection,
};
use crate::service::port_factory::publisher::{LocalPublisherConfig, UnableToDeliverStrategy};
use crate::service::static_config::message_type_details::TypeVariant;
use crate::service::static_config::publish_subscribe::{self};
use crate::service::{self, ServiceState};
use crate::{config, sample_mut::SampleMut};
use iceoryx2_bb_container::queue::Queue;
use iceoryx2_bb_elementary::allocator::AllocationError;
use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_lock_free::mpmc::container::{ContainerHandle, ContainerState};
use iceoryx2_bb_log::{debug, error, fail, fatal_panic, warn};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::event::NamedConceptMgmt;
use iceoryx2_cal::named_concept::{
    NamedConceptBuilder, NamedConceptListError, NamedConceptRemoveError,
};
use iceoryx2_cal::shared_memory::{
    SharedMemory, SharedMemoryBuilder, SharedMemoryCreateError, ShmPointer,
};
use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;
use iceoryx2_cal::shm_allocator::{self, PointerOffset, ShmAllocationError};
use iceoryx2_cal::zero_copy_connection::{
    ZeroCopyConnection, ZeroCopyCreationError, ZeroCopySendError, ZeroCopySender,
};
use iceoryx2_pal_concurrency_sync::iox_atomic::{IoxAtomicBool, IoxAtomicU64, IoxAtomicUsize};
use std::any::TypeId;
use std::cell::UnsafeCell;
use std::fmt::Debug;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::{alloc::Layout, marker::PhantomData, mem::MaybeUninit};

/// Defines a failure that can occur when a [`Publisher`] is created with
/// [`crate::service::port_factory::publisher::PortFactoryPublisher`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum PublisherCreateError {
    /// The maximum amount of [`Publisher`]s that can connect to a
    /// [`Service`](crate::service::Service) is
    /// defined in [`crate::config::Config`]. When this is exceeded no more [`Publisher`]s
    /// can be created for a specific [`Service`](crate::service::Service).
    ExceedsMaxSupportedPublishers,
    /// The datasegment in which the payload of the [`Publisher`] is stored, could not be created.
    UnableToCreateDataSegment,
}

impl std::fmt::Display for PublisherCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "PublisherCreateError::{:?}", self)
    }
}

impl std::error::Error for PublisherCreateError {}

/// Defines a failure that can occur in [`Publisher::loan()`] and [`Publisher::loan_uninit()`]
/// or is part of [`PublisherSendError`] emitted in [`Publisher::send_copy()`].
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum PublisherLoanError {
    /// The [`Publisher`]s data segment does not have any more memory left
    OutOfMemory,
    /// The maximum amount of [`SampleMut`]s a user can borrow with [`Publisher::loan()`] or
    /// [`Publisher::loan_uninit()`] is
    /// defined in [`crate::config::Config`]. When this is exceeded those calls will fail.
    ExceedsMaxLoanedSamples,
    /// The provided slice size exceeds the configured max slice size of the [`Publisher`].
    /// To send a [`SampleMut`] with this size a new [`Publisher`] has to be created with
    /// a [`crate::service::port_factory::publisher::PortFactoryPublisher::max_slice_len()`]
    /// greater or equal to the required len.
    ExceedsMaxLoanSize,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalFailure,
}

impl std::fmt::Display for PublisherLoanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "PublisherLoanError::{:?}", self)
    }
}

impl std::error::Error for PublisherLoanError {}

/// Failure that can be emitted when a [`SampleMut`] is sent via [`SampleMut::send()`].
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum PublisherSendError {
    /// [`SampleMut::send()`] was called but the corresponding [`Publisher`] went already out of
    /// scope.
    ConnectionBrokenSincePublisherNoLongerExists,
    /// A connection between a [`Subscriber`](crate::port::subscriber::Subscriber) and a
    /// [`Publisher`] is corrupted.
    ConnectionCorrupted,
    /// A failure occurred while acquiring memory for the payload
    LoanError(PublisherLoanError),
    /// A failure occurred while establishing a connection to a
    /// [`Subscriber`](crate::port::subscriber::Subscriber)
    ConnectionError(ConnectionFailure),
}

impl From<PublisherLoanError> for PublisherSendError {
    fn from(value: PublisherLoanError) -> Self {
        PublisherSendError::LoanError(value)
    }
}

impl From<ConnectionFailure> for PublisherSendError {
    fn from(value: ConnectionFailure) -> Self {
        PublisherSendError::ConnectionError(value)
    }
}

impl std::fmt::Display for PublisherSendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "PublisherSendError::{:?}", self)
    }
}

impl std::error::Error for PublisherSendError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) enum RemovePubSubPortFromAllConnectionsError {
    InsufficientPermissions,
    InternalError,
}

#[derive(Debug)]
pub(crate) struct DataSegment<Service: service::Service> {
    sample_reference_counter: Vec<IoxAtomicU64>,
    memory: Service::SharedMemory,
    payload_size: usize,
    payload_type_layout: Layout,
    port_id: UniquePublisherId,
    config: LocalPublisherConfig,
    service_state: Arc<ServiceState<Service>>,

    subscriber_connections: SubscriberConnections<Service>,
    subscriber_list_state: UnsafeCell<ContainerState<SubscriberDetails>>,
    history: Option<UnsafeCell<Queue<usize>>>,
    static_config: crate::service::static_config::StaticConfig,
    loan_counter: IoxAtomicUsize,
    is_active: IoxAtomicBool,
}

impl<Service: service::Service> DataSegment<Service> {
    fn sample_index(&self, distance_to_chunk: usize) -> usize {
        distance_to_chunk / self.payload_size
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
                    .deallocate(distance_to_chunk, self.payload_type_layout);
            }
        }
    }

    fn retrieve_returned_samples(&self) {
        for i in 0..self.subscriber_connections.len() {
            if let Some(ref connection) = self.subscriber_connections.get(i) {
                loop {
                    match connection.sender.reclaim() {
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

    fn remove_connection(&self, i: usize) {
        if let Some(connection) = self.subscriber_connections.get(i) {
            // # SAFETY: the receiver no longer exist, therefore we can
            //           reacquire all delivered samples
            unsafe {
                connection
                    .sender
                    .acquire_used_offsets(|offset| self.release_sample(offset))
            };

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
                match history.push_with_overflow(address_to_chunk) {
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
            if let Some(ref connection) = self.subscriber_connections.get(i) {
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
        }
        Ok(number_of_recipients)
    }

    fn populate_subscriber_channels(&self) -> Result<(), ZeroCopyCreationError> {
        let mut visited_indices = vec![];
        visited_indices.resize(self.subscriber_connections.capacity(), None);

        unsafe {
            (*self.subscriber_list_state.get()).for_each(|h, subscriber_id| {
                visited_indices[h.index() as usize] = Some(*subscriber_id);
                CallbackProgression::Continue
            })
        };

        for (i, index) in visited_indices.iter().enumerate() {
            match index {
                Some(subscriber_details) => {
                    let create_connection = match self.subscriber_connections.get(i) {
                        None => true,
                        Some(connection) => {
                            let is_connected =
                                connection.subscriber_id != subscriber_details.subscriber_id;
                            if is_connected {
                                self.remove_connection(i);
                            }
                            is_connected
                        }
                    };

                    if create_connection {
                        match self.subscriber_connections.create(
                            i,
                            *subscriber_details,
                            self.config.max_slice_len,
                        ) {
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
                                    subscriber_details.subscriber_id,
                                ) {
                                    DegrationAction::Ignore => (),
                                    DegrationAction::Warn => {
                                        warn!(from self,
                                            "Unable to establish connection to new subscriber {:?}.",
                                            subscriber_details.subscriber_id )
                                    }
                                    DegrationAction::Fail => {
                                        fail!(from self, with e,
                                           "Unable to establish connection to new subscriber {:?}.",
                                           subscriber_details.subscriber_id );
                                    }
                                },
                                None => {
                                    warn!(from self,
                                        "Unable to establish connection to new subscriber {:?}.",
                                        subscriber_details.subscriber_id )
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
            self.service_state
                .dynamic_storage
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
pub struct Publisher<
    Service: service::Service,
    Payload: Debug + ?Sized + 'static,
    UserHeader: Debug,
> {
    pub(crate) data_segment: Arc<DataSegment<Service>>,
    dynamic_publisher_handle: Option<ContainerHandle>,
    payload_size: usize,
    _payload: PhantomData<Payload>,
    _user_header: PhantomData<UserHeader>,
}

impl<Service: service::Service, Payload: Debug + ?Sized, UserHeader: Debug> Drop
    for Publisher<Service, Payload, UserHeader>
{
    fn drop(&mut self) {
        if let Some(handle) = self.dynamic_publisher_handle {
            self.data_segment
                .service_state
                .dynamic_storage
                .get()
                .publish_subscribe()
                .release_publisher_handle(handle)
        }
    }
}

impl<Service: service::Service, Payload: Debug + ?Sized, UserHeader: Debug>
    Publisher<Service, Payload, UserHeader>
{
    pub(crate) fn new(
        service: &Service,
        static_config: &publish_subscribe::StaticConfig,
        config: LocalPublisherConfig,
    ) -> Result<Self, PublisherCreateError> {
        let msg = "Unable to create Publisher port";
        let origin = "Publisher::new()";
        let port_id = UniquePublisherId::new();
        let subscriber_list = &service
            .__internal_state()
            .dynamic_storage
            .get()
            .publish_subscribe()
            .subscribers;

        let number_of_samples = service
            .__internal_state()
            .static_config
            .messaging_pattern
            .required_amount_of_samples_per_data_segment(config.max_loaned_samples);

        let data_segment = fail!(from origin,
                when Self::create_data_segment(&port_id, service.__internal_state().shared_node.config(), number_of_samples, static_config, &config),
                with PublisherCreateError::UnableToCreateDataSegment,
                "{} since the data segment could not be acquired.", msg);

        let max_slice_len = config.max_slice_len;
        let data_segment = Arc::new(DataSegment {
            is_active: IoxAtomicBool::new(true),
            memory: data_segment,
            payload_size: static_config
                .message_type_details()
                .sample_layout(config.max_slice_len)
                .size(),
            payload_type_layout: static_config
                .message_type_details()
                .payload_layout(config.max_slice_len),
            sample_reference_counter: {
                let mut v = Vec::with_capacity(number_of_samples);
                for _ in 0..number_of_samples {
                    v.push(IoxAtomicU64::new(0));
                }
                v
            },
            service_state: service.__internal_state().clone(),
            port_id,
            subscriber_connections: SubscriberConnections::new(
                subscriber_list.capacity(),
                service.__internal_state().shared_node.clone(),
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
            static_config: service.__internal_state().static_config.clone(),
            loan_counter: IoxAtomicUsize::new(0),
        });

        let payload_size = data_segment
            .subscriber_connections
            .static_config
            .message_type_details
            .payload
            .size;

        let mut new_self = Self {
            data_segment,
            dynamic_publisher_handle: None,
            payload_size,
            _payload: PhantomData,
            _user_header: PhantomData,
        };

        if let Err(e) = new_self.data_segment.populate_subscriber_channels() {
            warn!(from new_self, "The new Publisher port is unable to connect to every Subscriber port, caused by {:?}.", e);
        }

        std::sync::atomic::compiler_fence(Ordering::SeqCst);

        // !MUST! be the last task otherwise a publisher is added to the dynamic config without the
        // creation of all required resources
        let dynamic_publisher_handle = match service
            .__internal_state()
            .dynamic_storage
            .get()
            .publish_subscribe()
            .add_publisher_id(PublisherDetails {
                publisher_id: port_id,
                number_of_samples,
                max_slice_len,
                node_id: *service.__internal_state().shared_node.id(),
            }) {
            Some(unique_index) => unique_index,
            None => {
                fail!(from origin, with PublisherCreateError::ExceedsMaxSupportedPublishers,
                            "{} since it would exceed the maximum supported amount of publishers of {}.",
                            msg, service.__internal_state().static_config.publish_subscribe().max_publishers);
            }
        };

        new_self.dynamic_publisher_handle = Some(dynamic_publisher_handle);

        Ok(new_self)
    }

    fn create_data_segment(
        port_id: &UniquePublisherId,
        global_config: &config::Config,
        number_of_samples: usize,
        static_config: &publish_subscribe::StaticConfig,
        config: &LocalPublisherConfig,
    ) -> Result<Service::SharedMemory, SharedMemoryCreateError> {
        let l = static_config
            .message_type_details
            .sample_layout(config.max_slice_len);
        let allocator_config = shm_allocator::pool_allocator::Config { bucket_layout: l };

        Ok(fail!(from "Publisher::create_data_segment()",
            when <<Service::SharedMemory as SharedMemory<PoolAllocator>>::Builder as NamedConceptBuilder<
            Service::SharedMemory,
                >>::new(&data_segment_name(port_id))
                .config(&data_segment_config::<Service>(global_config))
                .size(l.size() * number_of_samples + l.align() - 1)
                .create(&allocator_config),
            "Unable to create the data segment."))
    }

    /// Returns the [`UniquePublisherId`] of the [`Publisher`]
    pub fn id(&self) -> UniquePublisherId {
        self.data_segment.port_id
    }

    /// Returns the strategy the [`Publisher`] follows when a [`SampleMut`] cannot be delivered
    /// since the [`Subscriber`](crate::port::subscriber::Subscriber)s buffer is full.
    pub fn unable_to_deliver_strategy(&self) -> UnableToDeliverStrategy {
        self.data_segment.config.unable_to_deliver_strategy
    }

    /// Returns the maximum slice length configured for this [`Publisher`].
    pub fn max_slice_len(&self) -> usize {
        self.data_segment.config.max_slice_len
    }

    fn allocate(&self, layout: Layout) -> Result<ShmPointer, PublisherLoanError> {
        let msg = "Unable to allocate Sample with";

        if self.data_segment.loan_counter.load(Ordering::Relaxed)
            >= self.data_segment.config.max_loaned_samples
        {
            fail!(from self, with PublisherLoanError::ExceedsMaxLoanedSamples,
                "{} {:?} since already {} samples were loaned and it would exceed the maximum of parallel loans of {}. Release or send a loaned sample to loan another sample.",
                msg, layout, self.data_segment.loan_counter.load(Ordering::Relaxed), self.data_segment.config.max_loaned_samples);
        }

        match self.data_segment.allocate(layout) {
            Ok(chunk) => {
                self.data_segment
                    .loan_counter
                    .fetch_add(1, Ordering::Relaxed);
                Ok(chunk)
            }
            Err(ShmAllocationError::AllocationError(AllocationError::OutOfMemory)) => {
                fail!(from self, with PublisherLoanError::OutOfMemory,
                    "{} {:?} since the underlying shared memory is out of memory.", msg, layout);
            }
            Err(ShmAllocationError::AllocationError(AllocationError::SizeTooLarge))
            | Err(ShmAllocationError::AllocationError(AllocationError::AlignmentFailure)) => {
                fatal_panic!(from self, "{} {:?} since the system seems to be corrupted.", msg, layout);
            }
            Err(v) => {
                fail!(from self, with PublisherLoanError::InternalFailure,
                    "{} {:?} since an internal failure occurred ({:?}).", msg, layout, v);
            }
        }
    }

    fn sample_layout(&self, number_of_elements: usize) -> Layout {
        self.data_segment
            .subscriber_connections
            .static_config
            .message_type_details
            .sample_layout(number_of_elements)
    }

    fn user_header_ptr(&self, header: *const Header) -> *const u8 {
        self.data_segment
            .subscriber_connections
            .static_config
            .message_type_details
            .user_header_ptr_from_header(header.cast())
            .cast()
    }

    fn payload_ptr(&self, header: *const Header) -> *const u8 {
        self.data_segment
            .subscriber_connections
            .static_config
            .message_type_details
            .payload_ptr_from_header(header.cast())
            .cast()
    }

    fn payload_type_variant(&self) -> TypeVariant {
        self.data_segment
            .subscriber_connections
            .static_config
            .message_type_details
            .payload
            .variant
    }
}

////////////////////////
// BEGIN: typed API
////////////////////////
impl<Service: service::Service, Payload: Debug + Sized, UserHeader: Debug>
    Publisher<Service, Payload, UserHeader>
{
    /// Copies the input `value` into a [`crate::sample_mut::SampleMut`] and delivers it.
    /// On success it returns the number of [`crate::port::subscriber::Subscriber`]s that received
    /// the data, otherwise a [`PublisherSendError`] describing the failure.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher_builder()
    ///                          .create()?;
    ///
    /// publisher.send_copy(1234)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn send_copy(&self, value: Payload) -> Result<usize, PublisherSendError> {
        let msg = "Unable to send copy of payload";
        let sample = fail!(from self, when self.loan_uninit(),
                                    "{} since the loan of a sample failed.", msg);

        sample.write_payload(value).send()
    }

    /// Loans/allocates a [`SampleMutUninit`] from the underlying data segment of the [`Publisher`].
    /// The user has to initialize the payload before it can be sent.
    ///
    /// On failure it returns [`PublisherLoanError`] describing the failure.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher_builder()
    ///                          .create()?;
    ///
    /// let sample = publisher.loan_uninit()?;
    /// let sample = sample.write_payload(42); // alternatively `sample.payload_mut()` can be use to access the `MaybeUninit<Payload>`
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn loan_uninit(
        &self,
    ) -> Result<SampleMutUninit<Service, MaybeUninit<Payload>, UserHeader>, PublisherLoanError>
    {
        let chunk = self.allocate(self.sample_layout(1))?;
        let header_ptr = chunk.data_ptr as *mut Header;
        let user_header_ptr = self.user_header_ptr(header_ptr) as *mut UserHeader;
        let payload_ptr = self.payload_ptr(header_ptr) as *mut MaybeUninit<Payload>;

        unsafe { header_ptr.write(Header::new(self.data_segment.port_id, 1)) };

        let sample =
            unsafe { RawSampleMut::new_unchecked(header_ptr, user_header_ptr, payload_ptr) };
        Ok(
            SampleMutUninit::<Service, MaybeUninit<Payload>, UserHeader>::new(
                &self.data_segment,
                sample,
                chunk.offset,
            ),
        )
    }
}

impl<Service: service::Service, Payload: Default + Debug + Sized, UserHeader: Debug>
    Publisher<Service, Payload, UserHeader>
{
    /// Loans/allocates a [`crate::sample_mut::SampleMut`] from the underlying data segment of the [`Publisher`]
    /// and initialize it with the default value. This can be a performance hit and [`Publisher::loan_uninit`]
    /// can be used to loan a [`core::mem::MaybeUninit<Payload>`].
    ///
    /// On failure it returns [`PublisherLoanError`] describing the failure.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let mut sample = publisher.loan()?;
    /// *sample.payload_mut() = 42;
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn loan(&self) -> Result<SampleMut<Service, Payload, UserHeader>, PublisherLoanError> {
        Ok(self.loan_uninit()?.write_payload(Payload::default()))
    }
}
////////////////////////
// END: typed API
////////////////////////

////////////////////////
// BEGIN: sliced API
////////////////////////
impl<Service: service::Service, Payload: Default + Debug, UserHeader: Debug>
    Publisher<Service, [Payload], UserHeader>
{
    /// Loans/allocates a [`crate::sample_mut::SampleMut`] from the underlying data segment of the [`Publisher`]
    /// and initializes all slice elements with the default value. This can be a performance hit
    /// and [`Publisher::loan_slice_uninit()`] can be used to loan a slice of
    /// [`core::mem::MaybeUninit<Payload>`].
    ///
    /// On failure it returns [`PublisherLoanError`] describing the failure.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<[u64]>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher_builder()
    ///                          .max_slice_len(120)
    ///                          .create()?;
    ///
    /// let slice_length = 5;
    /// let mut sample = publisher.loan_slice(slice_length)?;
    /// sample.payload_mut()[2] = 42;
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn loan_slice(
        &self,
        number_of_elements: usize,
    ) -> Result<SampleMut<Service, [Payload], UserHeader>, PublisherLoanError> {
        let sample = self.loan_slice_uninit(number_of_elements)?;
        Ok(sample.write_from_fn(|_| Payload::default()))
    }
}

impl<Service: service::Service, Payload: Debug, UserHeader: Debug>
    Publisher<Service, [Payload], UserHeader>
{
    /// Loans/allocates a [`SampleMutUninit`] from the underlying data segment of the [`Publisher`].
    /// The user has to initialize the payload before it can be sent.
    ///
    /// On failure it returns [`PublisherLoanError`] describing the failure.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<[usize]>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher_builder()
    ///                          .max_slice_len(120)
    ///                          .create()?;
    ///
    /// let slice_length = 5;
    /// let sample = publisher.loan_slice_uninit(slice_length)?;
    /// let sample = sample.write_from_fn(|n| n * 2); // alternatively `sample.payload_mut()` can be use to access the `[MaybeUninit<Payload>]`
    ///
    /// sample.send()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn loan_slice_uninit(
        &self,
        slice_len: usize,
    ) -> Result<SampleMutUninit<Service, [MaybeUninit<Payload>], UserHeader>, PublisherLoanError>
    {
        // required since Rust does not support generic specializations or negative traits
        debug_assert!(TypeId::of::<Payload>() != TypeId::of::<CustomPayloadMarker>());

        unsafe { self.loan_slice_uninit_impl(slice_len, slice_len) }
    }

    unsafe fn loan_slice_uninit_impl(
        &self,
        slice_len: usize,
        underlying_number_of_slice_elements: usize,
    ) -> Result<SampleMutUninit<Service, [MaybeUninit<Payload>], UserHeader>, PublisherLoanError>
    {
        let max_slice_len = self.data_segment.config.max_slice_len;
        if max_slice_len < slice_len {
            fail!(from self, with PublisherLoanError::ExceedsMaxLoanSize,
                "Unable to loan slice with {} elements since it would exceed the max supported slice length of {}.",
                slice_len, max_slice_len);
        }

        let sample_layout = self.sample_layout(slice_len);
        let chunk = self.allocate(sample_layout)?;
        let header_ptr = chunk.data_ptr as *mut Header;
        let user_header_ptr = self.user_header_ptr(header_ptr) as *mut UserHeader;
        let payload_ptr = self.payload_ptr(header_ptr) as *mut MaybeUninit<Payload>;

        unsafe { header_ptr.write(Header::new(self.data_segment.port_id, slice_len as _)) };

        let sample = unsafe {
            RawSampleMut::new_unchecked(
                header_ptr,
                user_header_ptr,
                core::slice::from_raw_parts_mut(payload_ptr, underlying_number_of_slice_elements),
            )
        };

        Ok(
            SampleMutUninit::<Service, [MaybeUninit<Payload>], UserHeader>::new(
                &self.data_segment,
                sample,
                chunk.offset,
            ),
        )
    }
}

impl<Service: service::Service, UserHeader: Debug>
    Publisher<Service, [CustomPayloadMarker], UserHeader>
{
    /// # Safety
    ///
    ///  * slice_len != 1 only when payload TypeVariant == Dynamic
    ///  * The number_of_elements in the [`Header`](crate::service::header::publish_subscribe::Header)
    ///     is set to `slice_len`
    ///  * The [`SampleMutUninit`] will contain `slice_len` * `MessageTypeDetails::payload.size`
    ///     elements of type [`CustomPayloadMarker`].
    #[doc(hidden)]
    pub unsafe fn loan_custom_payload(
        &self,
        slice_len: usize,
    ) -> Result<
        SampleMutUninit<Service, [MaybeUninit<CustomPayloadMarker>], UserHeader>,
        PublisherLoanError,
    > {
        // TypeVariant::Dynamic == slice and only here it makes sense to loan more than one element
        debug_assert!(slice_len == 1 || self.payload_type_variant() == TypeVariant::Dynamic);

        self.loan_slice_uninit_impl(slice_len, self.payload_size * slice_len)
    }
}
////////////////////////
// END: sliced API
////////////////////////

impl<Service: service::Service, Payload: Debug + ?Sized, UserHeader: Debug> UpdateConnections
    for Publisher<Service, Payload, UserHeader>
{
    fn update_connections(&self) -> Result<(), ConnectionFailure> {
        self.data_segment.update_connections()
    }
}

pub(crate) unsafe fn remove_data_segment_of_publisher<Service: service::Service>(
    port_id: &UniquePublisherId,
    config: &config::Config,
) -> Result<(), NamedConceptRemoveError> {
    let origin = format!(
        "remove_data_segment_of_publisher::<{}>::({:?})",
        core::any::type_name::<Service>(),
        port_id
    );

    fail!(from origin, when <Service::SharedMemory as NamedConceptMgmt>::remove_cfg(
            &data_segment_name(port_id),
            &data_segment_config::<Service>(config),
        ), "Unable to remove the publishers data segment."
    );

    Ok(())
}

fn connections<Service: service::Service>(
    origin: &str,
    msg: &str,
    config: &<Service::Connection as NamedConceptMgmt>::Configuration,
) -> Result<Vec<FileName>, RemovePubSubPortFromAllConnectionsError> {
    match <Service::Connection as NamedConceptMgmt>::list_cfg(config) {
        Ok(list) => Ok(list),
        Err(NamedConceptListError::InsufficientPermissions) => {
            fail!(from origin, with RemovePubSubPortFromAllConnectionsError::InsufficientPermissions,
                    "{} due to insufficient permissions to list all connections.", msg);
        }
        Err(NamedConceptListError::InternalError) => {
            fail!(from origin, with RemovePubSubPortFromAllConnectionsError::InternalError,
                "{} due to an internal error while listing all connections.", msg);
        }
    }
}

pub(crate) unsafe fn remove_publisher_from_all_connections<Service: service::Service>(
    port_id: &UniquePublisherId,
    config: &config::Config,
) -> Result<(), RemovePubSubPortFromAllConnectionsError> {
    let origin = format!(
        "remove_publisher_from_all_connections::<{}>::({:?})",
        core::any::type_name::<Service>(),
        port_id
    );
    let msg = "Unable to remove the publisher from all connections";

    let connection_config = connection_config::<Service>(config);
    let connection_list = connections::<Service>(&origin, msg, &connection_config)?;

    let mut ret_val = Ok(());
    for connection in connection_list {
        let publisher_id = extract_publisher_id_from_connection(&connection);
        if publisher_id == *port_id {
            match <Service::Connection as NamedConceptMgmt>::remove_cfg(
                &connection,
                &connection_config,
            ) {
                Ok(_) => (),
                Err(NamedConceptRemoveError::InsufficientPermissions) => {
                    debug!(from origin, "{} due to insufficient permissions to remove the connection ({:?}).", msg, connection);
                    ret_val = Err(RemovePubSubPortFromAllConnectionsError::InsufficientPermissions);
                }
                Err(NamedConceptRemoveError::InternalError) => {
                    debug!(from origin, "{} due to insufficient permissions to remove the connection ({:?}).", msg, connection);
                    ret_val = Err(RemovePubSubPortFromAllConnectionsError::InternalError);
                }
            }
        }
    }

    ret_val
}

pub(crate) unsafe fn remove_subscriber_from_all_connections<Service: service::Service>(
    port_id: &UniqueSubscriberId,
    config: &config::Config,
) -> Result<(), RemovePubSubPortFromAllConnectionsError> {
    let origin = format!(
        "remove_subscriber_from_all_connections::<{}>::({:?})",
        core::any::type_name::<Service>(),
        port_id
    );
    let msg = "Unable to remove the subscriber from all connections";

    let connection_config = connection_config::<Service>(config);
    let connection_list = connections::<Service>(&origin, msg, &connection_config)?;

    let mut ret_val = Ok(());
    for connection in connection_list {
        let subscriber_id = extract_subscriber_id_from_connection(&connection);
        if subscriber_id == *port_id {
            match <Service::Connection as NamedConceptMgmt>::remove_cfg(
                &connection,
                &connection_config,
            ) {
                Ok(_) => (),
                Err(NamedConceptRemoveError::InsufficientPermissions) => {
                    debug!(from origin, "{} due to insufficient permissions to remove the connection ({:?}).", msg, connection);
                    ret_val = Err(RemovePubSubPortFromAllConnectionsError::InsufficientPermissions);
                }
                Err(NamedConceptRemoveError::InternalError) => {
                    debug!(from origin, "{} due to insufficient permissions to remove the connection ({:?}).", msg, connection);
                    ret_val = Err(RemovePubSubPortFromAllConnectionsError::InternalError);
                }
            }
        }
    }

    ret_val
}
