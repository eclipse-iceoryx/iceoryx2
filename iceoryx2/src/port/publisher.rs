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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<[usize]>()
//!     .open_or_create()?;
//!
//! let publisher = service
//!     .publisher_builder()
//!     // defines the maximum length of a slice
//!     .initial_max_slice_len(128)
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

use super::details::data_segment::{DataSegment, DataSegmentType};
use super::details::segment_state::SegmentState;
use super::port_identifiers::UniquePublisherId;
use super::{LoanError, SendError};
use crate::port::details::sender::*;
use crate::port::update_connections::{ConnectionFailure, UpdateConnections};
use crate::prelude::UnableToDeliverStrategy;
use crate::raw_sample::RawSampleMut;
use crate::sample_mut::SampleMut;
use crate::sample_mut_uninit::SampleMutUninit;
use crate::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use crate::service::dynamic_config::publish_subscribe::{PublisherDetails, SubscriberDetails};
use crate::service::header::publish_subscribe::Header;
use crate::service::naming_scheme::data_segment_name;
use crate::service::port_factory::publisher::LocalPublisherConfig;
use crate::service::static_config::message_type_details::TypeVariant;
use crate::service::static_config::publish_subscribe;
use crate::service::{self, NoResource, ServiceState};
use alloc::sync::Arc;
use core::any::TypeId;
use core::cell::UnsafeCell;
use core::fmt::Debug;
use core::sync::atomic::Ordering;
use core::{marker::PhantomData, mem::MaybeUninit};
use iceoryx2_bb_container::queue::Queue;
use iceoryx2_bb_elementary::cyclic_tagger::CyclicTagger;
use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_lock_free::mpmc::container::{ContainerHandle, ContainerState};
use iceoryx2_bb_log::{fail, warn};
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::shm_allocator::{AllocationStrategy, PointerOffset};
use iceoryx2_cal::zero_copy_connection::{
    ChannelId, ZeroCopyCreationError, ZeroCopyPortDetails, ZeroCopySender,
};
use iceoryx2_pal_concurrency_sync::iox_atomic::{IoxAtomicBool, IoxAtomicUsize};

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
    /// Caused by a failure when instantiating a [`ArcSyncPolicy`] defined in the
    /// [`Service`](crate::service::Service) as `ArcThreadSafetyPolicy`.
    FailedToDeployThreadsafetyPolicy,
}

impl core::fmt::Display for PublisherCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PublisherCreateError::{self:?}")
    }
}

impl core::error::Error for PublisherCreateError {}

#[derive(Debug, Clone, Copy)]
struct OffsetAndSize {
    offset: u64,
    size: usize,
}

#[derive(Debug)]
pub(crate) struct PublisherSharedState<Service: service::Service> {
    config: LocalPublisherConfig,
    pub(crate) sender: Sender<Service>,
    subscriber_list_state: UnsafeCell<ContainerState<SubscriberDetails>>,
    history: Option<UnsafeCell<Queue<OffsetAndSize>>>,
    is_active: IoxAtomicBool,
}

impl<Service: service::Service> PublisherSharedState<Service> {
    fn add_sample_to_history(&self, offset: PointerOffset, sample_size: usize) {
        match &self.history {
            None => (),
            Some(history) => {
                let history = unsafe { &mut *history.get() };
                self.sender.borrow_sample(offset);
                match history.push_with_overflow(OffsetAndSize {
                    offset: offset.as_value(),
                    size: sample_size,
                }) {
                    None => (),
                    Some(old) => self
                        .sender
                        .release_sample(PointerOffset::from_value(old.offset)),
                }
            }
        }
    }

    fn force_update_connections(&self) -> Result<(), ZeroCopyCreationError> {
        let mut result = Ok(());
        self.sender.start_update_connection_cycle();
        unsafe {
            (*self.subscriber_list_state.get()).for_each(|h, port| {
                let inner_result = self.sender.update_connection(
                    h.index() as usize,
                    ReceiverDetails {
                        port_id: port.subscriber_id.value(),
                        buffer_size: port.buffer_size,
                    },
                    |connection| self.deliver_sample_history(connection),
                );

                if result.is_ok() {
                    result = inner_result;
                }

                CallbackProgression::Continue
            })
        };

        self.sender.finish_update_connection_cycle();

        result
    }

    fn update_connections(&self) -> Result<(), ConnectionFailure> {
        if unsafe {
            self.sender
                .service_state
                .dynamic_storage
                .get()
                .publish_subscribe()
                .subscribers
                .update_state(&mut *self.subscriber_list_state.get())
        } {
            fail!(from self, when self.force_update_connections(),
                "Connections were updated only partially since at least one connection to a Subscriber port failed.");
        }

        Ok(())
    }

    fn deliver_sample_history(&self, connection: &Connection<Service>) {
        match &self.history {
            None => (),
            Some(history) => {
                let history = unsafe { &mut *history.get() };
                let buffer_size = connection.sender.buffer_size();
                let history_start = history.len().saturating_sub(buffer_size);

                for i in history_start..history.len() {
                    let old_sample = unsafe { history.get_unchecked(i) };
                    self.sender.retrieve_returned_samples();

                    let offset = PointerOffset::from_value(old_sample.offset);
                    match connection
                        .sender
                        .try_send(offset, old_sample.size, ChannelId::new(0))
                    {
                        Ok(overflow) => {
                            self.sender.borrow_sample(offset);

                            if let Some(old) = overflow {
                                self.sender.release_sample(old);
                            }
                        }
                        Err(e) => {
                            warn!(from self, "Failed to deliver history to new subscriber via {:?} due to {:?}", connection, e);
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn send_sample(
        &self,
        offset: PointerOffset,
        sample_size: usize,
    ) -> Result<usize, SendError> {
        let msg = "Unable to send sample";
        if !self.is_active.load(Ordering::Relaxed) {
            fail!(from self, with SendError::ConnectionBrokenSinceSenderNoLongerExists,
                "{} since the corresponding publisher is already disconnected.", msg);
        }

        fail!(from self, when self.update_connections(),
            "{} since the connections could not be updated.", msg);

        self.add_sample_to_history(offset, sample_size);
        self.sender
            .deliver_offset(offset, sample_size, ChannelId::new(0))
    }
}

/// Sending endpoint of a publish-subscriber based communication.
#[derive(Debug)]
pub struct Publisher<
    Service: service::Service,
    Payload: Debug + ZeroCopySend + ?Sized + 'static,
    UserHeader: Debug + ZeroCopySend,
> {
    pub(crate) publisher_shared_state:
        Service::ArcThreadSafetyPolicy<PublisherSharedState<Service>>,
    dynamic_publisher_handle: Option<ContainerHandle>,
    _payload: PhantomData<Payload>,
    _user_header: PhantomData<UserHeader>,
}

unsafe impl<
        Service: service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: Debug + ZeroCopySend,
    > Send for Publisher<Service, Payload, UserHeader>
where
    Service::ArcThreadSafetyPolicy<PublisherSharedState<Service>>: Send + Sync,
{
}

unsafe impl<
        Service: service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: Debug + ZeroCopySend,
    > Sync for Publisher<Service, Payload, UserHeader>
where
    Service::ArcThreadSafetyPolicy<PublisherSharedState<Service>>: Send + Sync,
{
}

impl<
        Service: service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: Debug + ZeroCopySend,
    > Drop for Publisher<Service, Payload, UserHeader>
{
    fn drop(&mut self) {
        let shared_state = self.publisher_shared_state.lock();
        shared_state.is_active.store(false, Ordering::Relaxed);
        if let Some(handle) = self.dynamic_publisher_handle {
            shared_state
                .sender
                .service_state
                .dynamic_storage
                .get()
                .publish_subscribe()
                .release_publisher_handle(handle)
        }
    }
}

impl<
        Service: service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: Debug + ZeroCopySend,
    > Publisher<Service, Payload, UserHeader>
{
    pub(crate) fn new(
        service: Arc<ServiceState<Service, NoResource>>,
        static_config: &publish_subscribe::StaticConfig,
        config: LocalPublisherConfig,
    ) -> Result<Self, PublisherCreateError> {
        let msg = "Unable to create Publisher port";
        let origin = "Publisher::new()";
        let port_id = UniquePublisherId::new();
        let subscriber_list = &service
            .dynamic_storage
            .get()
            .publish_subscribe()
            .subscribers;

        let number_of_samples =
            unsafe { service.static_config.messaging_pattern.publish_subscribe() }
                .required_amount_of_samples_per_data_segment(config.max_loaned_samples);

        let data_segment_type =
            DataSegmentType::new_from_allocation_strategy(config.allocation_strategy);

        let sample_layout = static_config
            .message_type_details
            .sample_layout(config.initial_max_slice_len);

        let max_slice_len = config.initial_max_slice_len;
        let max_number_of_segments =
            DataSegment::<Service>::max_number_of_segments(data_segment_type);
        let publisher_details = PublisherDetails {
            data_segment_type,
            publisher_id: port_id,
            number_of_samples,
            max_slice_len,
            node_id: *service.shared_node.id(),
            max_number_of_segments,
        };
        let global_config = service.shared_node.config();

        let segment_name = data_segment_name(publisher_details.publisher_id.value());
        let data_segment = match data_segment_type {
            DataSegmentType::Static => DataSegment::create_static_segment(
                &segment_name,
                sample_layout,
                global_config,
                number_of_samples,
            ),
            DataSegmentType::Dynamic => DataSegment::create_dynamic_segment(
                &segment_name,
                sample_layout,
                global_config,
                number_of_samples,
                config.allocation_strategy,
            ),
        };

        let data_segment = fail!(from origin,
                when data_segment,
                with PublisherCreateError::UnableToCreateDataSegment,
                "{} since the data segment could not be acquired.", msg);

        let publisher_shared_state =
            <Service as service::Service>::ArcThreadSafetyPolicy::new(PublisherSharedState {
                is_active: IoxAtomicBool::new(true),
                sender: Sender {
                    data_segment,
                    segment_states: {
                        let mut v: Vec<SegmentState> =
                            Vec::with_capacity(max_number_of_segments as usize);
                        for _ in 0..max_number_of_segments {
                            v.push(SegmentState::new(number_of_samples))
                        }
                        v
                    },
                    connections: (0..subscriber_list.capacity())
                        .map(|_| UnsafeCell::new(None))
                        .collect(),
                    sender_port_id: port_id.value(),
                    shared_node: service.shared_node.clone(),
                    receiver_max_buffer_size: static_config.subscriber_max_buffer_size,
                    receiver_max_borrowed_samples: static_config.subscriber_max_borrowed_samples,
                    enable_safe_overflow: static_config.enable_safe_overflow,
                    number_of_samples,
                    max_number_of_segments,
                    degradation_callback: None,
                    service_state: service.clone(),
                    tagger: CyclicTagger::new(),
                    loan_counter: IoxAtomicUsize::new(0),
                    sender_max_borrowed_samples: config.max_loaned_samples,
                    unable_to_deliver_strategy: config.unable_to_deliver_strategy,
                    message_type_details: static_config.message_type_details.clone(),
                    number_of_channels: 1,
                },
                config,
                subscriber_list_state: UnsafeCell::new(unsafe { subscriber_list.get_state() }),
                history: match static_config.history_size == 0 {
                    true => None,
                    false => Some(UnsafeCell::new(Queue::new(static_config.history_size))),
                },
            });

        let publisher_shared_state = match publisher_shared_state {
            Ok(v) => v,
            Err(e) => {
                fail!(from origin,
                            with PublisherCreateError::FailedToDeployThreadsafetyPolicy,
                            "{msg} since the threadsafety policy could not be instantiated ({e:?}).");
            }
        };

        let mut new_self = Self {
            publisher_shared_state,
            dynamic_publisher_handle: None,
            _payload: PhantomData,
            _user_header: PhantomData,
        };

        if let Err(e) = new_self
            .publisher_shared_state
            .lock()
            .force_update_connections()
        {
            warn!(from new_self,
                "The new Publisher port is unable to connect to every Subscriber port, caused by {:?}.", e);
        }

        core::sync::atomic::compiler_fence(Ordering::SeqCst);

        // !MUST! be the last task otherwise a publisher is added to the dynamic config without the
        // creation of all required resources
        let dynamic_publisher_handle = match service
            .dynamic_storage
            .get()
            .publish_subscribe()
            .add_publisher_id(publisher_details)
        {
            Some(unique_index) => unique_index,
            None => {
                fail!(from origin, with PublisherCreateError::ExceedsMaxSupportedPublishers,
                            "{} since it would exceed the maximum supported amount of publishers of {}.",
                            msg, service.static_config.publish_subscribe().max_publishers);
            }
        };

        new_self.dynamic_publisher_handle = Some(dynamic_publisher_handle);

        Ok(new_self)
    }

    /// Returns the [`UniquePublisherId`] of the [`Publisher`]
    pub fn id(&self) -> UniquePublisherId {
        UniquePublisherId(UniqueSystemId::from(
            self.publisher_shared_state.lock().sender.sender_port_id,
        ))
    }

    /// Returns the strategy the [`Publisher`] follows when a [`SampleMut`] cannot be delivered
    /// since the [`Subscriber`](crate::port::subscriber::Subscriber)s buffer is full.
    pub fn unable_to_deliver_strategy(&self) -> UnableToDeliverStrategy {
        self.publisher_shared_state
            .lock()
            .sender
            .unable_to_deliver_strategy
    }
}

////////////////////////
// BEGIN: typed API
////////////////////////
impl<
        Service: service::Service,
        Payload: Debug + ZeroCopySend + Sized,
        UserHeader: Default + Debug + ZeroCopySend,
    > Publisher<Service, Payload, UserHeader>
{
    /// Copies the input `value` into a [`crate::sample_mut::SampleMut`] and delivers it.
    /// On success it returns the number of [`crate::port::subscriber::Subscriber`]s that received
    /// the data, otherwise a [`SendError`] describing the failure.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher_builder()
    /// #                        .create()?;
    ///
    /// publisher.send_copy(1234)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn send_copy(&self, value: Payload) -> Result<usize, SendError> {
        let msg = "Unable to send copy of payload";
        let sample = fail!(from self, when self.loan_uninit(),
                                    "{} since the loan of a sample failed.", msg);

        sample.write_payload(value).send()
    }

    /// Loans/allocates a [`SampleMutUninit`] from the underlying data segment of the [`Publisher`].
    /// The user has to initialize the payload before it can be sent.
    ///
    /// On failure it returns [`LoanError`] describing the failure.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher_builder()
    /// #                        .create()?;
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
    ) -> Result<SampleMutUninit<Service, MaybeUninit<Payload>, UserHeader>, LoanError> {
        let shared_state = self.publisher_shared_state.lock();
        let chunk = shared_state
            .sender
            .allocate(shared_state.sender.sample_layout(1))?;
        let node_id = shared_state.sender.service_state.shared_node.id();
        let header_ptr = chunk.header as *mut Header;
        let user_header_ptr: *mut UserHeader = chunk.user_header.cast();
        unsafe { header_ptr.write(Header::new(*node_id, self.id(), 1)) };
        unsafe { user_header_ptr.write(UserHeader::default()) };

        let sample = unsafe {
            RawSampleMut::new_unchecked(header_ptr, user_header_ptr, chunk.payload.cast())
        };
        Ok(
            SampleMutUninit::<Service, MaybeUninit<Payload>, UserHeader>::new(
                &self.publisher_shared_state,
                sample,
                chunk.offset,
                chunk.size,
            ),
        )
    }
}

impl<
        Service: service::Service,
        Payload: Default + Debug + ZeroCopySend + Sized,
        UserHeader: Default + Debug + ZeroCopySend,
    > Publisher<Service, Payload, UserHeader>
{
    /// Loans/allocates a [`crate::sample_mut::SampleMut`] from the underlying data segment of the [`Publisher`]
    /// and initialize it with the default value. This can be a performance hit and [`Publisher::loan_uninit`]
    /// can be used to loan a [`core::mem::MaybeUninit<Payload>`].
    ///
    /// On failure it returns [`LoanError`] describing the failure.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
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
    pub fn loan(&self) -> Result<SampleMut<Service, Payload, UserHeader>, LoanError> {
        Ok(self.loan_uninit()?.write_payload(Payload::default()))
    }
}
////////////////////////
// END: typed API
////////////////////////

////////////////////////
// BEGIN: sliced API
////////////////////////
impl<
        Service: service::Service,
        Payload: Default + Debug + ZeroCopySend,
        UserHeader: Default + Debug + ZeroCopySend,
    > Publisher<Service, [Payload], UserHeader>
{
    /// Loans/allocates a [`crate::sample_mut::SampleMut`] from the underlying data segment of the [`Publisher`]
    /// and initializes all slice elements with the default value. This can be a performance hit
    /// and [`Publisher::loan_slice_uninit()`] can be used to loan a slice of
    /// [`core::mem::MaybeUninit<Payload>`].
    ///
    /// On failure it returns [`LoanError`] describing the failure.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<[u64]>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher_builder()
    /// #                        .initial_max_slice_len(120)
    /// #                        .create()?;
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
    ) -> Result<SampleMut<Service, [Payload], UserHeader>, LoanError> {
        let sample = self.loan_slice_uninit(number_of_elements)?;
        Ok(sample.write_from_fn(|_| Payload::default()))
    }
}

impl<
        Service: service::Service,
        Payload: Debug + ZeroCopySend,
        UserHeader: Debug + ZeroCopySend,
    > Publisher<Service, [Payload], UserHeader>
{
    /// Returns the maximum initial slice length configured for this [`Publisher`].
    pub fn initial_max_slice_len(&self) -> usize {
        self.publisher_shared_state
            .lock()
            .config
            .initial_max_slice_len
    }
}

impl<
        Service: service::Service,
        Payload: Debug + ZeroCopySend,
        UserHeader: Default + Debug + ZeroCopySend,
    > Publisher<Service, [Payload], UserHeader>
{
    /// Loans/allocates a [`SampleMutUninit`] from the underlying data segment of the [`Publisher`].
    /// The user has to initialize the payload before it can be sent.
    ///
    /// On failure it returns [`LoanError`] describing the failure.
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
    /// #                        .initial_max_slice_len(120)
    /// #                        .create()?;
    ///
    /// let slice_length = 5;
    /// let sample = publisher.loan_slice_uninit(slice_length)?;
    /// let sample = sample.write_from_fn(|n| n * 2); // alternatively `sample.payload_mut()` can be use to access the `[MaybeUninit<Payload>]`
    ///
    /// sample.send()?;
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn loan_slice_uninit(
        &self,
        slice_len: usize,
    ) -> Result<SampleMutUninit<Service, [MaybeUninit<Payload>], UserHeader>, LoanError> {
        // required since Rust does not support generic specializations or negative traits
        debug_assert!(TypeId::of::<Payload>() != TypeId::of::<CustomPayloadMarker>());

        self.loan_slice_uninit_impl(slice_len, slice_len)
    }

    fn loan_slice_uninit_impl(
        &self,
        slice_len: usize,
        underlying_number_of_slice_elements: usize,
    ) -> Result<SampleMutUninit<Service, [MaybeUninit<Payload>], UserHeader>, LoanError> {
        let shared_state = self.publisher_shared_state.lock();
        let max_slice_len = shared_state.config.initial_max_slice_len;
        if shared_state.config.allocation_strategy == AllocationStrategy::Static
            && max_slice_len < slice_len
        {
            fail!(from self, with LoanError::ExceedsMaxLoanSize,
                "Unable to loan slice with {} elements since it would exceed the max supported slice length of {}.",
                slice_len, max_slice_len);
        }

        let sample_layout = shared_state.sender.sample_layout(slice_len);
        let chunk = shared_state.sender.allocate(sample_layout)?;
        let user_header_ptr: *mut UserHeader = chunk.user_header.cast();
        let header_ptr = chunk.header as *mut Header;
        let node_id = shared_state.sender.service_state.shared_node.id();
        unsafe { header_ptr.write(Header::new(*node_id, self.id(), slice_len as _)) };
        unsafe { user_header_ptr.write(UserHeader::default()) };

        let sample = unsafe {
            RawSampleMut::new_unchecked(
                header_ptr,
                user_header_ptr,
                core::slice::from_raw_parts_mut(
                    chunk.payload.cast(),
                    underlying_number_of_slice_elements,
                ),
            )
        };

        Ok(
            SampleMutUninit::<Service, [MaybeUninit<Payload>], UserHeader>::new(
                &self.publisher_shared_state,
                sample,
                chunk.offset,
                chunk.size,
            ),
        )
    }
}

impl<Service: service::Service> Publisher<Service, [CustomPayloadMarker], CustomHeaderMarker> {
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
        SampleMutUninit<Service, [MaybeUninit<CustomPayloadMarker>], CustomHeaderMarker>,
        LoanError,
    > {
        let shared_state = self.publisher_shared_state.lock();

        // TypeVariant::Dynamic == slice and only here it makes sense to loan more than one element
        debug_assert!(
            slice_len == 1 || shared_state.sender.payload_type_variant() == TypeVariant::Dynamic
        );

        self.loan_slice_uninit_impl(slice_len, shared_state.sender.payload_size() * slice_len)
    }
}
////////////////////////
// END: sliced API
////////////////////////

impl<
        Service: service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: Debug + ZeroCopySend,
    > UpdateConnections for Publisher<Service, Payload, UserHeader>
{
    fn update_connections(&self) -> Result<(), ConnectionFailure> {
        self.publisher_shared_state.lock().update_connections()
    }
}
