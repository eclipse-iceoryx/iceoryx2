// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

//! # Example
//!
//! ## Typed API
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! let service = node
//!    .service_builder(&"My/Funk/ServiceName".try_into()?)
//!    .request_response::<u64, u64>()
//!    .open_or_create()?;
//!
//! let client = service.client_builder()
//!    // defines behavior when server queue is full in a non-overflowing service
//!    .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample)
//!    .create()?;
//!
//! let request = client.loan_uninit()?;
//! let request = request.write_payload(1829);
//!
//! let pending_response = request.send()?;
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
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! let service = node
//!    .service_builder(&"My/Funk/ServiceName".try_into()?)
//!    .request_response::<[usize], u64>()
//!    .open_or_create()?;
//!
//! let client = service.client_builder()
//!     // provides a hint for the max slice len, 128 means we want at
//!     // list a slice of 128 `usize`
//!     .initial_max_slice_len(128)
//!     // The underlying sample size will be increased with a power of two strategy
//!     // when [`Client::loan_slice()`] or [`Client::loan_slice_uninit()`] requires more
//!     // memory than available.
//!     .allocation_strategy(AllocationStrategy::PowerOfTwo)
//!    .create()?;
//!
//! let number_of_elements = 10;
//! let request = client.loan_slice_uninit(number_of_elements)?;
//! let request = request.write_from_fn(|idx| idx * 2 + 1);
//!
//! let pending_response = request.send()?;
//!
//! # Ok(())
//! # }
//! ```

use core::{
    any::TypeId, cell::UnsafeCell, fmt::Debug, marker::PhantomData, mem::MaybeUninit,
    sync::atomic::Ordering,
};
use iceoryx2_bb_container::{queue::Queue, slotmap::SlotMap, vec::Vec};

use iceoryx2_bb_elementary::{cyclic_tagger::CyclicTagger, CallbackProgression};
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_lock_free::mpmc::container::{ContainerHandle, ContainerState};
use iceoryx2_bb_log::{fail, fatal_panic, warn};
use iceoryx2_cal::{
    arc_sync_policy::ArcSyncPolicy,
    dynamic_storage::DynamicStorage,
    shm_allocator::{AllocationStrategy, PointerOffset},
    zero_copy_connection::ChannelId,
};
use iceoryx2_pal_concurrency_sync::iox_atomic::{IoxAtomicBool, IoxAtomicU64, IoxAtomicUsize};

use crate::{
    pending_response::PendingResponse,
    port::{
        details::data_segment::DataSegment, update_connections::UpdateConnections, UniqueClientId,
    },
    prelude::{PortFactory, UnableToDeliverStrategy},
    raw_sample::RawSampleMut,
    request_mut::RequestMut,
    request_mut_uninit::RequestMutUninit,
    service::{
        self,
        builder::{CustomHeaderMarker, CustomPayloadMarker},
        dynamic_config::request_response::{ClientDetails, ServerDetails},
        header,
        naming_scheme::data_segment_name,
        port_factory::client::{ClientCreateError, LocalClientConfig, PortFactoryClient},
        static_config::message_type_details::TypeVariant,
    },
};

use super::{
    details::{
        data_segment::DataSegmentType,
        receiver::{Receiver, SenderDetails},
        segment_state::SegmentState,
        sender::{ReceiverDetails, Sender},
    },
    update_connections::ConnectionFailure,
    LoanError, SendError,
};

/// Failure that can be emitted when a [`RequestMut`] is sent.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum RequestSendError {
    /// Sending this [`RequestMut`] exceeds the maximum supported amount of active
    /// requests. When a [`PendingResponse`] object is released another [`RequestMut`]
    /// can be sent.
    ExceedsMaxActiveRequests,

    /// Underlying [`SendError`]s.
    SendError(SendError),
}

impl From<SendError> for RequestSendError {
    fn from(value: SendError) -> Self {
        RequestSendError::SendError(value)
    }
}

impl From<LoanError> for RequestSendError {
    fn from(value: LoanError) -> Self {
        RequestSendError::SendError(SendError::LoanError(value))
    }
}

impl From<ConnectionFailure> for RequestSendError {
    fn from(value: ConnectionFailure) -> Self {
        RequestSendError::SendError(SendError::ConnectionError(value))
    }
}

impl core::fmt::Display for RequestSendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "RequestSendError::{self:?}")
    }
}

impl core::error::Error for RequestSendError {}

#[derive(Debug)]
pub(crate) struct ClientSharedState<Service: service::Service> {
    pub(crate) config: LocalClientConfig,
    pub(crate) request_sender: Sender<Service>,
    pub(crate) response_receiver: Receiver<Service>,
    client_handle: UnsafeCell<Option<ContainerHandle>>,
    server_list_state: UnsafeCell<ContainerState<ServerDetails>>,
    pub(crate) active_request_counter: IoxAtomicUsize,
    pub(crate) available_channel_ids: UnsafeCell<Queue<ChannelId>>,
}

impl<Service: service::Service> Drop for ClientSharedState<Service> {
    fn drop(&mut self) {
        if let Some(handle) = unsafe { *self.client_handle.get() } {
            self.request_sender
                .service_state
                .dynamic_storage
                .get()
                .request_response()
                .release_client_handle(handle)
        }
    }
}

impl<Service: service::Service> ClientSharedState<Service> {
    fn prepare_channel_to_receive_responses(&self, channel_id: ChannelId, request_id: u64) {
        self.response_receiver
            .set_channel_state(channel_id, request_id);
    }

    pub(crate) fn send_request(
        &self,
        offset: PointerOffset,
        sample_size: usize,
        channel_id: ChannelId,
        request_id: u64,
    ) -> Result<usize, RequestSendError> {
        let msg = "Unable to send request";

        let active_request_counter = self.active_request_counter.load(Ordering::Relaxed);
        if self
            .request_sender
            .service_state
            .static_config
            .request_response()
            .max_active_requests_per_client
            <= active_request_counter
        {
            fail!(from self, with RequestSendError::ExceedsMaxActiveRequests,
                    "{} since the number of active requests is limited to {} and sending this request would exceed the limit.", msg, active_request_counter);
        }

        fail!(from self, when self.update_connections(),
            "{} since the connections could not be updated.", msg);

        self.prepare_channel_to_receive_responses(channel_id, request_id);

        self.active_request_counter.fetch_add(1, Ordering::Relaxed);
        Ok(self.request_sender.deliver_offset(
            offset,
            sample_size,
            // All requests are delivered on the same channel, therefore we can use
            // ChannelId::new(0).
            ChannelId::new(0),
        )?)
    }

    pub(crate) fn update_connections(
        &self,
    ) -> Result<(), super::update_connections::ConnectionFailure> {
        if unsafe {
            self.request_sender
                .service_state
                .dynamic_storage
                .get()
                .request_response()
                .servers
                .update_state(&mut *self.server_list_state.get())
        } {
            fail!(from self, when self.force_update_connections(),
                "Connections were updated only partially since at least one connection to a Server port failed.");
        }

        Ok(())
    }

    fn force_update_connections(&self) -> Result<(), ConnectionFailure> {
        let mut result = Ok(());
        self.request_sender.start_update_connection_cycle();
        self.response_receiver.start_update_connection_cycle();

        unsafe {
            (*self.server_list_state.get()).for_each(|h, port| {
                // establish response connection
                let inner_result = self.response_receiver.update_connection(
                    h.index() as usize,
                    SenderDetails {
                        port_id: port.server_id.value(),
                        max_number_of_segments: port.max_number_of_segments,
                        data_segment_type: port.data_segment_type,
                        number_of_samples: port.number_of_responses,
                    },
                );
                result = result.and(inner_result);

                // establish request connection
                let inner_result = self.request_sender.update_connection(
                    h.index() as usize,
                    ReceiverDetails {
                        port_id: port.server_id.value(),
                        buffer_size: port.request_buffer_size,
                    },
                    |_| {},
                );
                if let Some(err) = inner_result.err() {
                    result = result.and(Err(err.into()));
                }

                CallbackProgression::Continue
            })
        };

        self.response_receiver.finish_update_connection_cycle();
        self.request_sender.finish_update_connection_cycle();

        result
    }
}

/// Sends [`RequestMut`]s to a [`Server`](crate::port::server::Server) in a
/// request-response based communication.
#[derive(Debug)]
pub struct Client<
    Service: service::Service,
    RequestPayload: Debug + ZeroCopySend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + ZeroCopySend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> {
    client_id: UniqueClientId,
    client_shared_state: Service::ArcThreadSafetyPolicy<ClientSharedState<Service>>,
    request_id_counter: IoxAtomicU64,
    _request_payload: PhantomData<RequestPayload>,
    _request_header: PhantomData<RequestHeader>,
    _response_payload: PhantomData<ResponsePayload>,
    _response_header: PhantomData<ResponseHeader>,
}

unsafe impl<
        Service: service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Send for Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
where
    Service::ArcThreadSafetyPolicy<ClientSharedState<Service>>: Send + Sync,
{
}

unsafe impl<
        Service: service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Sync for Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
where
    Service::ArcThreadSafetyPolicy<ClientSharedState<Service>>: Send + Sync,
{
}

impl<
        Service: service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    pub(crate) fn new(
        client_factory: PortFactoryClient<
            Service,
            RequestPayload,
            RequestHeader,
            ResponsePayload,
            ResponseHeader,
        >,
    ) -> Result<Self, ClientCreateError> {
        let msg = "Unable to create Client port";
        let origin = "Client::new()";
        let service = &client_factory.factory.service;
        let client_id = UniqueClientId::new();
        let static_config = client_factory.factory.static_config();
        let number_of_requests =
            unsafe { service.static_config.messaging_pattern.request_response() }
                .required_amount_of_chunks_per_client_data_segment(
                    static_config.max_loaned_requests,
                );
        let server_list = &service.dynamic_storage.get().request_response().servers;

        let global_config = service.shared_node.config();
        let segment_name = data_segment_name(client_id.value());
        let data_segment_type = DataSegmentType::new_from_allocation_strategy(
            client_factory.config.allocation_strategy,
        );
        let max_number_of_segments =
            DataSegment::<Service>::max_number_of_segments(data_segment_type);

        let sample_layout = static_config
            .request_message_type_details
            .sample_layout(client_factory.config.initial_max_slice_len);

        let data_segment = match data_segment_type {
            DataSegmentType::Static => DataSegment::<Service>::create_static_segment(
                &segment_name,
                sample_layout,
                global_config,
                number_of_requests,
            ),
            DataSegmentType::Dynamic => DataSegment::<Service>::create_dynamic_segment(
                &segment_name,
                sample_layout,
                global_config,
                number_of_requests,
                client_factory.config.allocation_strategy,
            ),
        };

        let data_segment = fail!(from origin,
            when data_segment,
            with ClientCreateError::UnableToCreateDataSegment,
            "{} since the client data segment could not be created.", msg);

        let client_details = ClientDetails {
            client_id,
            node_id: *service.shared_node.id(),
            number_of_requests,
            response_buffer_size: static_config.max_response_buffer_size,
            max_slice_len: client_factory.config.initial_max_slice_len,
            data_segment_type,
            max_number_of_segments,
        };

        let request_sender = Sender {
            data_segment,
            segment_states: {
                let mut v =
                    alloc::vec::Vec::<SegmentState>::with_capacity(max_number_of_segments as usize);
                for _ in 0..max_number_of_segments {
                    v.push(SegmentState::new(number_of_requests))
                }
                v
            },
            sender_port_id: client_id.value(),
            shared_node: service.shared_node.clone(),
            connections: (0..server_list.capacity())
                .map(|_| UnsafeCell::new(None))
                .collect(),
            receiver_max_buffer_size: static_config.max_active_requests_per_client,
            receiver_max_borrowed_samples: static_config.max_active_requests_per_client,
            enable_safe_overflow: static_config.enable_safe_overflow_for_requests,
            degradation_callback: client_factory.request_degradation_callback,
            number_of_samples: number_of_requests,
            max_number_of_segments,
            service_state: service.clone(),
            tagger: CyclicTagger::new(),
            loan_counter: IoxAtomicUsize::new(0),
            sender_max_borrowed_samples: static_config.max_loaned_requests,
            unable_to_deliver_strategy: client_factory.config.unable_to_deliver_strategy,
            message_type_details: static_config.request_message_type_details.clone(),
            // all requests are sent via one channel, only the responses require different
            // channels to guarantee that one response does not fill the buffer of another
            // response.
            // but the requests have one shared buffer that the user can configure, therefore
            // one channel suffices
            number_of_channels: 1,
        };

        let number_of_to_be_removed_connections = service
            .shared_node
            .config()
            .defaults
            .request_response
            .client_expired_connection_buffer;
        let number_of_active_connections = server_list.capacity();
        let number_of_connections =
            number_of_to_be_removed_connections + number_of_active_connections;

        let response_receiver = Receiver {
            connections: Vec::from_fn(number_of_active_connections, |_| UnsafeCell::new(None)),
            receiver_port_id: client_id.value(),
            service_state: service.clone(),
            buffer_size: static_config.max_response_buffer_size,
            tagger: CyclicTagger::new(),
            to_be_removed_connections: Some(UnsafeCell::new(Vec::new(
                number_of_to_be_removed_connections,
            ))),
            degradation_callback: client_factory.response_degradation_callback,
            message_type_details: static_config.response_message_type_details.clone(),
            receiver_max_borrowed_samples: static_config
                .max_borrowed_responses_per_pending_response,
            enable_safe_overflow: static_config.enable_safe_overflow_for_responses,
            number_of_channels: number_of_requests,
            connection_storage: UnsafeCell::new(SlotMap::new(number_of_connections)),
        };

        let client_shared_state = Service::ArcThreadSafetyPolicy::new(ClientSharedState {
            config: client_factory.config,
            client_handle: UnsafeCell::new(None),
            available_channel_ids: {
                let mut queue = Queue::new(number_of_requests);
                for n in 0..number_of_requests {
                    queue.push(ChannelId::new(n));
                }
                UnsafeCell::new(queue)
            },
            request_sender,
            response_receiver,
            server_list_state: UnsafeCell::new(unsafe { server_list.get_state() }),
            active_request_counter: IoxAtomicUsize::new(0),
        });

        let client_shared_state = match client_shared_state {
            Ok(v) => v,
            Err(e) => {
                fail!(from origin, with ClientCreateError::FailedToDeployThreadsafetyPolicy,
                            "{msg} since the threadsafety policy could not be instantiated ({e:?}).");
            }
        };

        let new_self = Self {
            request_id_counter: IoxAtomicU64::new(0),
            client_shared_state,
            client_id,
            _request_payload: PhantomData,
            _request_header: PhantomData,
            _response_payload: PhantomData,
            _response_header: PhantomData,
        };

        if let Err(e) = new_self
            .client_shared_state
            .lock()
            .force_update_connections()
        {
            warn!(from new_self,
                "The new Client port is unable to connect to every Server port, caused by {:?}.", e);
        }

        core::sync::atomic::compiler_fence(Ordering::SeqCst);

        // !MUST! be the last task otherwise a client is added to the dynamic config without the
        // creation of all required resources
        unsafe {
            *new_self.client_shared_state.lock().client_handle.get() = match service
                .dynamic_storage
                .get()
                .request_response()
                .add_client_id(client_details)
            {
                Some(handle) => Some(handle),
                None => {
                    fail!(from origin,
                      with ClientCreateError::ExceedsMaxSupportedClients,
                      "{} since it would exceed the maximum support amount of clients of {}.",
                      msg, service.static_config.request_response().max_clients());
                }
            }
        };

        Ok(new_self)
    }

    /// Returns the [`UniqueClientId`] of the [`Client`]
    pub fn id(&self) -> UniqueClientId {
        self.client_id
    }

    /// Returns the strategy the [`Client`] follows when a [`RequestMut`] cannot be delivered
    /// if the [`Server`](crate::port::server::Server)s buffer is full.
    pub fn unable_to_deliver_strategy(&self) -> UnableToDeliverStrategy {
        self.client_shared_state
            .lock()
            .request_sender
            .unable_to_deliver_strategy
    }
}

impl<
        Service: service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > UpdateConnections
    for Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn update_connections(&self) -> Result<(), ConnectionFailure> {
        self.client_shared_state.lock().update_connections()
    }
}

////////////////////////
// BEGIN: typed API
////////////////////////
impl<
        Service: service::Service,
        RequestPayload: Debug + ZeroCopySend,
        RequestHeader: Default + Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    /// Acquires an [`RequestMutUninit`] to store payload. This API shall be used
    /// by default to avoid unnecessary copies.
    ///
    /// # Example
    ///
    /// ## True Zero Copy
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node
    /// #    .service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #    .request_response::<u64, u64>()
    /// #    .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    ///
    /// let mut request = client.loan_uninit()?;
    ///
    /// // Use MaybeUninit API to populate the underlying payload
    /// request.payload_mut().write(1234);
    /// // Promise that we have initialized everything and initialize request
    /// let request = unsafe { request.assume_init() };
    /// // Send request
    /// let pending_response = request.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Copy Payload
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node
    /// #    .service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #    .request_response::<u64, u64>()
    /// #    .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    ///
    /// let request = client.loan_uninit()?;
    /// // we write the payload by copying the data into the request and retrieve
    /// // an initialized RequestMut that can be sent
    /// let request = request.write_payload(123);
    /// // Send request
    /// let pending_response = request.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn loan_uninit(
        &self,
    ) -> Result<
        RequestMutUninit<
            Service,
            MaybeUninit<RequestPayload>,
            RequestHeader,
            ResponsePayload,
            ResponseHeader,
        >,
        LoanError,
    > {
        let client_shared_state = self.client_shared_state.lock();
        let chunk = client_shared_state
            .request_sender
            .allocate(client_shared_state.request_sender.sample_layout(1))?;

        let channel_id =
            match unsafe { &mut *client_shared_state.available_channel_ids.get() }.pop() {
                Some(channel_id) => channel_id,
                None => {
                    fatal_panic!(from self,
                    "This should never happen! There are no more available response channels.");
                }
            };

        let header_ptr: *mut service::header::request_response::RequestHeader = chunk.header.cast();
        let user_header_ptr: *mut RequestHeader = chunk.user_header.cast();
        unsafe {
            header_ptr.write(service::header::request_response::RequestHeader {
                client_id: self.id(),
                channel_id,
                request_id: self.request_id_counter.fetch_add(1, Ordering::Relaxed),
                number_of_elements: 1,
            })
        };
        unsafe { user_header_ptr.write(RequestHeader::default()) };

        let ptr = unsafe {
            RawSampleMut::<
                service::header::request_response::RequestHeader,
                RequestHeader,
                MaybeUninit<RequestPayload>,
            >::new_unchecked(header_ptr, user_header_ptr, chunk.payload.cast())
        };

        Ok(RequestMutUninit {
            request: RequestMut {
                ptr,
                sample_size: chunk.size,
                channel_id,
                offset_to_chunk: chunk.offset,
                client_shared_state: self.client_shared_state.clone(),
                _response_payload: PhantomData,
                _response_header: PhantomData,
                was_sample_sent: IoxAtomicBool::new(false),
            },
        })
    }

    /// Copies the input value into a [`RequestMut`] and sends it. On success it
    /// returns a [`PendingResponse`] that can be used to receive a stream of
    /// [`Response`](crate::response::Response)s from the
    /// [`Server`](crate::port::server::Server).
    pub fn send_copy(
        &self,
        value: RequestPayload,
    ) -> Result<
        PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        RequestSendError,
    > {
        let msg = "Unable to send copy of request";
        let request = fail!(from self,
                            when self.loan_uninit(),
                            "{} since the loan of the request failed.", msg);

        request.write_payload(value).send()
    }
}

impl<
        Service: service::Service,
        RequestPayload: Debug + Default + ZeroCopySend,
        RequestHeader: Default + Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    /// Acquires the payload for the request and initializes the underlying memory
    /// with default. This can be very expensive when the payload is large, therefore
    /// prefer [`Client::loan_uninit()`] when possible.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node
    /// #    .service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #    .request_response::<u64, u64>()
    /// #    .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    ///
    /// // Acquire request that is initialized with `Default::default()`.
    /// let mut request = client.loan()?;
    /// // Assign a value to the request
    /// *request = 456;
    ///
    /// let pending_response = request.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn loan(
        &self,
    ) -> Result<
        RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        LoanError,
    > {
        Ok(self.loan_uninit()?.write_payload(RequestPayload::default()))
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
        RequestPayload: Default + Debug + ZeroCopySend + 'static,
        RequestHeader: Default + Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Client<Service, [RequestPayload], RequestHeader, ResponsePayload, ResponseHeader>
{
    /// Loans/allocates a [`RequestMut`] from the underlying data segment of the [`Client`]
    /// and initializes all slice elements with the default value. This can be a performance hit
    /// and [`Client::loan_slice_uninit()`] can be used to loan a slice of
    /// [`core::mem::MaybeUninit<Payload>`].
    ///
    /// On failure it returns [`LoanError`] describing the failure.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// let service = node
    ///    .service_builder(&"My/Funk/ServiceName".try_into()?)
    ///    .request_response::<[u64], u64>()
    ///    .open_or_create()?;
    ///
    /// let client = service.client_builder()
    ///                     .initial_max_slice_len(32)
    ///                     .create()?;
    ///
    /// let slice_length = 13;
    /// let mut request = client.loan_slice(slice_length)?;
    /// for element in request.payload_mut() {
    ///     *element = 1234;
    /// }
    ///
    /// let pending_response = request.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn loan_slice(
        &self,
        slice_len: usize,
    ) -> Result<
        RequestMut<Service, [RequestPayload], RequestHeader, ResponsePayload, ResponseHeader>,
        LoanError,
    > {
        let request = self.loan_slice_uninit(slice_len)?;
        Ok(request.write_from_fn(|_| RequestPayload::default()))
    }
}

impl<
        Service: service::Service,
        RequestPayload: Debug + ZeroCopySend + 'static,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Client<Service, [RequestPayload], RequestHeader, ResponsePayload, ResponseHeader>
{
    /// Returns the maximum initial slice length configured for this [`Client`].
    pub fn initial_max_slice_len(&self) -> usize {
        self.client_shared_state.lock().config.initial_max_slice_len
    }
}

impl<
        Service: service::Service,
        RequestPayload: Debug + ZeroCopySend + 'static,
        RequestHeader: Default + Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Client<Service, [RequestPayload], RequestHeader, ResponsePayload, ResponseHeader>
{
    /// Loans/allocates a [`RequestMutUninit`] from the underlying data segment of the [`Client`].
    /// The user has to initialize the payload before it can be sent.
    ///
    /// On failure it returns [`LoanError`] describing the failure.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// let service = node
    ///    .service_builder(&"My/Funk/ServiceName".try_into()?)
    ///    .request_response::<[u64], u64>()
    ///    .open_or_create()?;
    ///
    /// let client = service.client_builder()
    ///                     .initial_max_slice_len(32)
    ///                     .create()?;
    ///
    /// let slice_length = 13;
    /// let mut request = client.loan_slice_uninit(slice_length)?;
    /// for element in request.payload_mut() {
    ///     element.write(1234);
    /// }
    /// // we have written the payload, initialize the request
    /// let request = unsafe { request.assume_init() };
    ///
    /// let pending_response = request.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::type_complexity)] // type alias would require 5 generic parameters which hardly reduces complexity
    pub fn loan_slice_uninit(
        &self,
        slice_len: usize,
    ) -> Result<
        RequestMutUninit<
            Service,
            [MaybeUninit<RequestPayload>],
            RequestHeader,
            ResponsePayload,
            ResponseHeader,
        >,
        LoanError,
    > {
        debug_assert!(TypeId::of::<RequestPayload>() != TypeId::of::<CustomPayloadMarker>());
        unsafe { self.loan_slice_uninit_impl(slice_len, slice_len) }
    }

    #[allow(clippy::type_complexity)] // type alias would require 5 generic parameters which hardly reduces complexity
    unsafe fn loan_slice_uninit_impl(
        &self,
        slice_len: usize,
        underlying_number_of_slice_elements: usize,
    ) -> Result<
        RequestMutUninit<
            Service,
            [MaybeUninit<RequestPayload>],
            RequestHeader,
            ResponsePayload,
            ResponseHeader,
        >,
        LoanError,
    > {
        let client_shared_state = self.client_shared_state.lock();
        let max_slice_len = client_shared_state.config.initial_max_slice_len;

        if client_shared_state.config.allocation_strategy == AllocationStrategy::Static
            && max_slice_len < slice_len
        {
            fail!(from self, with LoanError::ExceedsMaxLoanSize,
                "Unable to loan slice with {} elements since it would exceed the max supported slice length of {}.",
                slice_len, max_slice_len);
        }

        let request_layout = client_shared_state.request_sender.sample_layout(slice_len);
        let chunk = client_shared_state
            .request_sender
            .allocate(request_layout)?;

        let channel_id =
            match unsafe { &mut *client_shared_state.available_channel_ids.get() }.pop() {
                Some(channel_id) => channel_id,
                None => {
                    fatal_panic!(from self,
                    "This should never happen! There are no more available response channels.");
                }
            };

        let user_header_ptr: *mut RequestHeader = chunk.user_header.cast();
        let header_ptr = chunk.header as *mut header::request_response::RequestHeader;
        unsafe {
            header_ptr.write(header::request_response::RequestHeader {
                client_id: self.id(),
                channel_id,
                request_id: self.request_id_counter.fetch_add(1, Ordering::Relaxed),
                number_of_elements: slice_len as _,
            })
        };
        unsafe { user_header_ptr.write(RequestHeader::default()) };

        let ptr = unsafe {
            RawSampleMut::<
                service::header::request_response::RequestHeader,
                RequestHeader,
                [MaybeUninit<RequestPayload>],
            >::new_unchecked(
                header_ptr,
                user_header_ptr,
                core::slice::from_raw_parts_mut(
                    chunk.payload.cast(),
                    underlying_number_of_slice_elements,
                ),
            )
        };

        Ok(RequestMutUninit {
            request: RequestMut {
                ptr,
                sample_size: chunk.size,
                channel_id,
                offset_to_chunk: chunk.offset,
                client_shared_state: self.client_shared_state.clone(),
                _response_payload: PhantomData,
                _response_header: PhantomData,
                was_sample_sent: IoxAtomicBool::new(false),
            },
        })
    }
}

impl<Service: service::Service>
    Client<
        Service,
        [CustomPayloadMarker],
        CustomHeaderMarker,
        [CustomPayloadMarker],
        CustomHeaderMarker,
    >
{
    #[doc(hidden)]
    #[allow(clippy::type_complexity)] // type alias would require 5 generic parameters which hardly reduces complexity
    pub unsafe fn loan_custom_payload(
        &self,
        slice_len: usize,
    ) -> Result<
        RequestMutUninit<
            Service,
            [MaybeUninit<CustomPayloadMarker>],
            CustomHeaderMarker,
            [CustomPayloadMarker],
            CustomHeaderMarker,
        >,
        LoanError,
    > {
        let client_shared_state = self.client_shared_state.lock();
        // TypeVariant::Dynamic == slice and only here it makes sense to loan more than one element
        debug_assert!(
            slice_len == 1
                || client_shared_state.request_sender.payload_type_variant()
                    == TypeVariant::Dynamic
        );

        self.loan_slice_uninit_impl(
            slice_len,
            client_shared_state.request_sender.payload_size() * slice_len,
        )
    }
}
////////////////////////
// END: sliced API
////////////////////////
