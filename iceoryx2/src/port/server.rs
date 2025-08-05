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
//! #
//! let service = node
//!     .service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .request_response::<u64, u64>()
//!     .open_or_create()?;
//!
//! let server = service.server_builder()
//!    // defines behavior when client queue is full in a non-overflowing service
//!    .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample)
//!    .create()?;
//!
//! while let Some(active_request) = server.receive()? {
//!     println!("received request: {:?}", *active_request);
//!     let response = active_request.loan_uninit()?;
//!     let response = response.write_payload(871238);
//!     response.send()?;
//! }
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
//! #
//! let service = node
//!     .service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .request_response::<u64, [usize]>()
//!     .open_or_create()?;
//!
//! let server = service.server_builder()
//!     // provides a hint for the max slice len, 128 means we want at
//!     // list a slice of 128 `usize`
//!     .initial_max_slice_len(128)
//!     // The underlying sample size will be increased with a power of two strategy
//!     // when [`ActiveRequest::loan_slice()`] or [`ActiveRequest::loan_slice_uninit()`]
//!     // requires more memory than available.
//!     .create()?;
//!
//! let number_of_elements = 10;
//! while let Some(active_request) = server.receive()? {
//!     println!("received request: {:?}", *active_request);
//!     let response = active_request.loan_slice_uninit(number_of_elements)?;
//!     let response = response.write_from_fn(|idx| idx * 3 + 4);
//!     response.send()?;
//! }
//! # Ok(())
//! # }
//! ```

use alloc::sync::Arc;
use core::{cell::UnsafeCell, sync::atomic::Ordering};
use core::{fmt::Debug, marker::PhantomData};
use iceoryx2_bb_container::slotmap::SlotMap;
use iceoryx2_bb_container::vec::Vec;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_cal::zero_copy_connection::ChannelId;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicUsize;

use iceoryx2_bb_elementary::{cyclic_tagger::CyclicTagger, CallbackProgression};
use iceoryx2_bb_lock_free::mpmc::container::{ContainerHandle, ContainerState};
use iceoryx2_bb_log::{fail, warn};
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;
use iceoryx2_cal::dynamic_storage::DynamicStorage;

use crate::port::update_connections::UpdateConnections;
use crate::prelude::UnableToDeliverStrategy;
use crate::service::builder::CustomPayloadMarker;
use crate::service::naming_scheme::data_segment_name;
use crate::service::port_factory::server::LocalServerConfig;
use crate::service::NoResource;
use crate::{
    active_request::ActiveRequest,
    prelude::PortFactory,
    raw_sample::RawSample,
    service::{
        self,
        dynamic_config::request_response::{ClientDetails, ServerDetails},
        port_factory::server::{PortFactoryServer, ServerCreateError},
        ServiceState,
    },
};

use super::details::data_segment::DataSegment;
use super::details::segment_state::SegmentState;
use super::details::sender::{ReceiverDetails, Sender};
use super::{
    details::{
        chunk::Chunk,
        chunk_details::ChunkDetails,
        data_segment::DataSegmentType,
        receiver::{Receiver, SenderDetails},
    },
    update_connections::ConnectionFailure,
    ReceiveError, UniqueServerId,
};

// All requests are received via one channel with id 0
const REQUEST_CHANNEL_ID: ChannelId = ChannelId::new(0);
pub(crate) const INVALID_CONNECTION_ID: usize = usize::MAX;

#[derive(Debug)]
pub(crate) struct SharedServerState<Service: service::Service> {
    pub(crate) config: LocalServerConfig,
    pub(crate) response_sender: Sender<Service>,
    server_handle: UnsafeCell<Option<ContainerHandle>>,
    pub(crate) request_receiver: Receiver<Service>,
    client_list_state: UnsafeCell<ContainerState<ClientDetails>>,
    service_state: Arc<ServiceState<Service, NoResource>>,
}

impl<Service: service::Service> Drop for SharedServerState<Service> {
    fn drop(&mut self) {
        if let Some(handle) = unsafe { *self.server_handle.get() } {
            self.service_state
                .dynamic_storage
                .get()
                .request_response()
                .release_server_handle(handle);
        }
    }
}

impl<Service: service::Service> SharedServerState<Service> {
    pub(crate) fn update_connections(&self) -> Result<(), ConnectionFailure> {
        if unsafe {
            self.request_receiver
                .service_state
                .dynamic_storage
                .get()
                .request_response()
                .clients
                .update_state(&mut *self.client_list_state.get())
        } {
            fail!(from self,
                  when self.force_update_connections(),
                  "Connections were updated only partially since at least one connection to a client failed.");
        }

        Ok(())
    }

    fn force_update_connections(&self) -> Result<(), ConnectionFailure> {
        self.request_receiver.start_update_connection_cycle();
        self.response_sender.start_update_connection_cycle();

        let mut result = Ok(());
        unsafe {
            (*self.client_list_state.get()).for_each(|h, details| {
                // establish request connection
                let inner_result = self.request_receiver.update_connection(
                    h.index() as usize,
                    SenderDetails {
                        port_id: details.client_id.value(),
                        number_of_samples: details.number_of_requests,
                        max_number_of_segments: details.max_number_of_segments,
                        data_segment_type: details.data_segment_type,
                    },
                );
                result = result.and(inner_result);

                // establish response connection
                let inner_result = self.response_sender.update_connection(
                    h.index() as usize,
                    ReceiverDetails {
                        port_id: details.client_id.value(),
                        buffer_size: details.response_buffer_size,
                    },
                    |_| {},
                );
                if let Some(err) = inner_result.err() {
                    result = result.and(Err(err.into()));
                }

                CallbackProgression::Continue
            })
        };

        self.response_sender.finish_update_connection_cycle();
        self.request_receiver.finish_update_connection_cycle();

        result
    }
}

/// Receives [`RequestMut`](crate::request_mut::RequestMut) from a
/// [`Client`](crate::port::client::Client) and responds with
/// [`Response`](crate::response::Response) by using an
/// [`ActiveRequest`].
#[derive(Debug)]
pub struct Server<
    Service: service::Service,
    RequestPayload: Debug + ZeroCopySend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + ZeroCopySend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> {
    shared_state: Service::ArcThreadSafetyPolicy<SharedServerState<Service>>,
    max_loaned_responses_per_request: usize,
    enable_fire_and_forget: bool,
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
    > Send for Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
where
    Service::ArcThreadSafetyPolicy<SharedServerState<Service>>: Send + Sync,
{
}

unsafe impl<
        Service: service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Sync for Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
where
    Service::ArcThreadSafetyPolicy<SharedServerState<Service>>: Send + Sync,
{
}

impl<
        Service: service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > UpdateConnections
    for Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn update_connections(&self) -> Result<(), ConnectionFailure> {
        self.shared_state.lock().update_connections()
    }
}

impl<
        Service: service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    pub(crate) fn new(
        server_factory: PortFactoryServer<
            Service,
            RequestPayload,
            RequestHeader,
            ResponsePayload,
            ResponseHeader,
        >,
    ) -> Result<Self, ServerCreateError> {
        let msg = "Failed to create Server port";
        let origin = "Server::new()";
        let server_id = UniqueServerId::new();
        let service = &server_factory.factory.service;
        let static_config = server_factory.factory.static_config();
        let number_of_requests_per_client =
            unsafe { service.static_config.messaging_pattern.request_response() }
                .required_amount_of_chunks_per_client_data_segment(
                    static_config.max_loaned_requests,
                );

        let number_of_responses =
            unsafe { service.static_config.messaging_pattern.request_response() }
                .required_amount_of_chunks_per_server_data_segment(
                    server_factory.max_loaned_responses_per_request,
                    number_of_requests_per_client,
                );

        let client_list = &service.dynamic_storage.get().request_response().clients;

        let number_of_to_be_removed_connections = service
            .shared_node
            .config()
            .defaults
            .request_response
            .server_expired_connection_buffer;
        let number_of_active_connections = client_list.capacity();
        let number_of_connections =
            number_of_to_be_removed_connections + number_of_active_connections;

        let request_receiver = Receiver {
            connections: Vec::from_fn(number_of_active_connections, |_| UnsafeCell::new(None)),
            receiver_port_id: server_id.value(),
            service_state: service.clone(),
            message_type_details: static_config.request_message_type_details.clone(),
            receiver_max_borrowed_samples: static_config.max_active_requests_per_client,
            enable_safe_overflow: static_config.enable_safe_overflow_for_requests,
            buffer_size: static_config.max_active_requests_per_client,
            tagger: CyclicTagger::new(),
            to_be_removed_connections: if static_config.enable_fire_and_forget_requests {
                Some(UnsafeCell::new(Vec::new(
                    number_of_to_be_removed_connections,
                )))
            } else {
                None
            },
            degradation_callback: server_factory.request_degradation_callback,
            number_of_channels: 1,
            connection_storage: UnsafeCell::new(SlotMap::new(number_of_connections)),
        };

        let global_config = service.shared_node.config();
        let data_segment_type = DataSegmentType::new_from_allocation_strategy(
            server_factory.config.allocation_strategy,
        );
        let segment_name = data_segment_name(server_id.value());
        let max_number_of_segments =
            DataSegment::<Service>::max_number_of_segments(data_segment_type);
        let sample_layout = static_config
            .response_message_type_details
            .sample_layout(server_factory.config.initial_max_slice_len);
        let data_segment = match data_segment_type {
            DataSegmentType::Static => DataSegment::<Service>::create_static_segment(
                &segment_name,
                sample_layout,
                global_config,
                number_of_responses,
            ),
            DataSegmentType::Dynamic => DataSegment::<Service>::create_dynamic_segment(
                &segment_name,
                sample_layout,
                global_config,
                number_of_responses,
                server_factory.config.allocation_strategy,
            ),
        };

        let data_segment = fail!(from origin,
            when data_segment,
            with ServerCreateError::UnableToCreateDataSegment,
            "{} since the server data segment could not be created.", msg);

        let response_sender = Sender {
            segment_states: {
                let mut v =
                    alloc::vec::Vec::<SegmentState>::with_capacity(max_number_of_segments as usize);
                for _ in 0..max_number_of_segments {
                    v.push(SegmentState::new(number_of_responses))
                }
                v
            },
            data_segment,
            connections: (0..client_list.capacity())
                .map(|_| UnsafeCell::new(None))
                .collect(),
            sender_port_id: server_id.value(),
            shared_node: service.shared_node.clone(),
            receiver_max_buffer_size: static_config.max_response_buffer_size,
            receiver_max_borrowed_samples: static_config
                .max_borrowed_responses_per_pending_response,
            sender_max_borrowed_samples: server_factory.max_loaned_responses_per_request
                * number_of_requests_per_client
                * static_config.max_clients,
            enable_safe_overflow: static_config.enable_safe_overflow_for_responses,
            number_of_samples: number_of_responses,
            max_number_of_segments,
            degradation_callback: server_factory.response_degradation_callback,
            service_state: service.clone(),
            tagger: CyclicTagger::new(),
            loan_counter: IoxAtomicUsize::new(0),
            unable_to_deliver_strategy: server_factory.config.unable_to_deliver_strategy,
            message_type_details: static_config.response_message_type_details.clone(),
            number_of_channels: number_of_requests_per_client,
        };

        let shared_state = Service::ArcThreadSafetyPolicy::new(SharedServerState {
            config: server_factory.config,
            request_receiver,
            client_list_state: UnsafeCell::new(unsafe { client_list.get_state() }),
            server_handle: UnsafeCell::new(None),
            service_state: service.clone(),
            response_sender,
        });

        let shared_state = match shared_state {
            Ok(v) => v,
            Err(e) => {
                fail!(from origin, with ServerCreateError::FailedToDeployThreadsafetyPolicy,
                      "{msg} since the threadsafety policy could not be instantiated ({e:?}).");
            }
        };

        let new_self = Self {
            max_loaned_responses_per_request: server_factory.max_loaned_responses_per_request,
            enable_fire_and_forget: service
                .static_config
                .request_response()
                .enable_fire_and_forget_requests,
            shared_state,
            _request_payload: PhantomData,
            _request_header: PhantomData,
            _response_payload: PhantomData,
            _response_header: PhantomData,
        };

        if let Err(e) = new_self.shared_state.lock().force_update_connections() {
            warn!(from new_self, "The new server is unable to connect to every client, caused by {:?}.", e);
        }

        core::sync::atomic::compiler_fence(Ordering::SeqCst);

        // !MUST! be the last task otherwise a server is added to the dynamic config without the
        // creation of all required resources
        unsafe {
            *new_self.shared_state.lock().server_handle.get() = match service
                .dynamic_storage
                .get()
                .request_response()
                .add_server_id(ServerDetails {
                    server_id,
                    node_id: *service.shared_node.id(),
                    request_buffer_size: static_config.max_active_requests_per_client,
                    number_of_responses,
                    max_slice_len: server_factory.config.initial_max_slice_len,
                    data_segment_type,
                    max_number_of_segments,
                }) {
                Some(v) => Some(v),
                None => {
                    fail!(from origin,
                    with ServerCreateError::ExceedsMaxSupportedServers,
                    "{} since it would exceed the maximum supported amount of servers of {}.",
                    msg, service.static_config.request_response().max_servers());
                }
            }
        };

        Ok(new_self)
    }

    /// Returns the [`UniqueServerId`] of the [`Server`]
    pub fn id(&self) -> UniqueServerId {
        UniqueServerId(UniqueSystemId::from(
            self.shared_state.lock().request_receiver.receiver_port_id,
        ))
    }

    /// Returns true if the [`Server`] has [`RequestMut`](crate::request_mut::RequestMut)s in its buffer.
    pub fn has_requests(&self) -> Result<bool, ConnectionFailure> {
        let shared_state = self.shared_state.lock();
        fail!(from self, when shared_state.update_connections(),
                "Some requests are not being received since not all connections to clients could be established.");
        if self.enable_fire_and_forget {
            Ok(shared_state
                .request_receiver
                .has_samples(REQUEST_CHANNEL_ID))
        } else {
            Ok(shared_state
                .request_receiver
                .has_samples_in_active_connection(REQUEST_CHANNEL_ID))
        }
    }

    /// Returns the strategy the [`Server`] follows when a
    /// [`ResponseMut`](crate::response_mut::ResponseMut) cannot be delivered
    /// if the [`Client`](crate::port::client::Client)s buffer is full.
    pub fn unable_to_deliver_strategy(&self) -> UnableToDeliverStrategy {
        self.shared_state
            .lock()
            .response_sender
            .unable_to_deliver_strategy
    }

    fn receive_impl(&self) -> Result<Option<(ChunkDetails, Chunk)>, ReceiveError> {
        let shared_state = self.shared_state.lock();
        if let Err(e) = shared_state.update_connections() {
            fail!(from self,
                  with ReceiveError::ConnectionFailure(e),
                  "Some requests are not being received since not all connections to the clients could be established.");
        }

        shared_state.request_receiver.receive(REQUEST_CHANNEL_ID)
    }
}

impl<
        Service: service::Service,
        RequestPayload: Debug + ZeroCopySend,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn create_active_request(
        &self,
        details: ChunkDetails,
        chunk: Chunk,
        connection_id: usize,
    ) -> ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
    {
        let header =
            unsafe { &*(chunk.header as *const service::header::request_response::RequestHeader) };

        ActiveRequest {
            details,
            shared_loan_counter: Arc::new(IoxAtomicUsize::new(0)),
            max_loan_count: self.max_loaned_responses_per_request,
            request_id: header.request_id,
            channel_id: header.channel_id,
            connection_id,
            shared_state: self.shared_state.clone(),
            ptr: unsafe {
                RawSample::new_unchecked(
                    chunk.header.cast(),
                    chunk.user_header.cast(),
                    chunk.payload.cast::<RequestPayload>(),
                )
            },

            _response_payload: PhantomData,
            _response_header: PhantomData,
        }
    }

    /// Receives a [`RequestMut`](crate::request_mut::RequestMut) that was sent by a
    /// [`Client`](crate::port::client::Client) and returns an [`ActiveRequest`] which
    /// can be used to respond.
    /// If no [`RequestMut`](crate::request_mut::RequestMut)s were received it
    /// returns [`None`].
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
    ///     .service_builder(&"My/Funk/ServiceName".try_into()?)
    ///     .request_response::<u64, u64>()
    ///     .open_or_create()?;
    ///
    /// let server = service.server_builder().create()?;
    ///
    /// while let Some(active_request) = server.receive()? {
    ///     println!("received request: {:?}", *active_request);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::type_complexity)] // type alias would require 5 generic parameters which hardly reduces complexity
    pub fn receive(
        &self,
    ) -> Result<
        Option<
            ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        >,
        ReceiveError,
    > {
        loop {
            match self.receive_impl()? {
                Some((details, chunk)) => {
                    let header = unsafe {
                        &*(chunk.header as *const service::header::request_response::RequestHeader)
                    };

                    if let Some(connection_id) = self
                        .shared_state
                        .lock()
                        .response_sender
                        .get_connection_id_of(header.client_id.value())
                    {
                        let active_request =
                            self.create_active_request(details, chunk, connection_id);

                        if !self.enable_fire_and_forget && !active_request.is_connected() {
                            continue;
                        }

                        return Ok(Some(active_request));
                    } else if self.enable_fire_and_forget {
                        let active_request =
                            self.create_active_request(details, chunk, INVALID_CONNECTION_ID);
                        return Ok(Some(active_request));
                    }
                }
                None => return Ok(None),
            }
        }
    }
}

impl<
        Service: service::Service,
        RequestPayload: Debug + ZeroCopySend,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Server<Service, [RequestPayload], RequestHeader, ResponsePayload, ResponseHeader>
{
    fn create_active_request(
        &self,
        details: ChunkDetails,
        chunk: Chunk,
        connection_id: usize,
        number_of_elements: usize,
    ) -> ActiveRequest<Service, [RequestPayload], RequestHeader, ResponsePayload, ResponseHeader>
    {
        let header =
            unsafe { &*(chunk.header as *const service::header::request_response::RequestHeader) };

        ActiveRequest {
            details,
            shared_loan_counter: Arc::new(IoxAtomicUsize::new(0)),
            max_loan_count: self.max_loaned_responses_per_request,
            request_id: header.request_id,
            channel_id: header.channel_id,
            connection_id,
            shared_state: self.shared_state.clone(),
            ptr: unsafe {
                RawSample::new_slice_unchecked(
                    chunk.header.cast(),
                    chunk.user_header.cast(),
                    core::slice::from_raw_parts(
                        chunk.payload.cast::<RequestPayload>(),
                        number_of_elements as _,
                    ),
                )
            },
            _response_payload: PhantomData,
            _response_header: PhantomData,
        }
    }

    /// Receives a [`RequestMut`](crate::request_mut::RequestMut) that was sent by a
    /// [`Client`](crate::port::client::Client) and returns an [`ActiveRequest`] which
    /// can be used to respond.
    /// If no [`RequestMut`](crate::request_mut::RequestMut)s were received it
    /// returns [`None`].
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
    ///     .service_builder(&"My/Funk/ServiceName".try_into()?)
    ///     .request_response::<[u64], u64>()
    ///     .open_or_create()?;
    ///
    /// let server = service.server_builder().create()?;
    ///
    /// while let Some(active_request) = server.receive()? {
    ///     println!("received request: {:?}", active_request);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::type_complexity)] // type alias would require 5 generic parameters which hardly reduces complexity
    pub fn receive(
        &self,
    ) -> Result<
        Option<
            ActiveRequest<
                Service,
                [RequestPayload],
                RequestHeader,
                ResponsePayload,
                ResponseHeader,
            >,
        >,
        ReceiveError,
    > {
        loop {
            match self.receive_impl()? {
                Some((details, chunk)) => {
                    let header = unsafe {
                        &*(chunk.header as *const service::header::request_response::RequestHeader)
                    };

                    if let Some(connection_id) = self
                        .shared_state
                        .lock()
                        .response_sender
                        .get_connection_id_of(header.client_id.value())
                    {
                        let active_request = self.create_active_request(
                            details,
                            chunk,
                            connection_id,
                            header.number_of_elements() as _,
                        );

                        if !self.enable_fire_and_forget && !active_request.is_connected() {
                            continue;
                        }

                        return Ok(Some(active_request));
                    }
                }
                None => return Ok(None),
            }
        }
    }
}

impl<
        Service: service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend,
        ResponseHeader: Debug + ZeroCopySend,
    > Server<Service, RequestPayload, RequestHeader, [ResponsePayload], ResponseHeader>
{
    /// Returns the maximum initial slice length configured for this [`Server`].
    pub fn initial_max_slice_len(&self) -> usize {
        self.shared_state.lock().config.initial_max_slice_len
    }
}

impl<
        Service: service::Service,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Server<Service, [CustomPayloadMarker], RequestHeader, ResponsePayload, ResponseHeader>
{
    #[doc(hidden)]
    #[allow(clippy::type_complexity)] // type alias would require 5 generic parameters which hardly reduces complexity
    pub unsafe fn receive_custom_payload(
        &self,
    ) -> Result<
        Option<
            ActiveRequest<
                Service,
                [CustomPayloadMarker],
                RequestHeader,
                ResponsePayload,
                ResponseHeader,
            >,
        >,
        ReceiveError,
    > {
        let shared_state = self.shared_state.lock();
        loop {
            match self.receive_impl()? {
                Some((details, chunk)) => {
                    let header = unsafe {
                        &*(chunk.header as *const service::header::request_response::RequestHeader)
                    };
                    let number_of_elements = (*header).number_of_elements();
                    let number_of_bytes =
                        number_of_elements as usize * shared_state.request_receiver.payload_size();

                    if let Some(connection_id) = shared_state
                        .response_sender
                        .get_connection_id_of(header.client_id.value())
                    {
                        let active_request = self.create_active_request(
                            details,
                            chunk,
                            connection_id,
                            number_of_bytes,
                        );

                        if !self.enable_fire_and_forget && !active_request.is_connected() {
                            continue;
                        }

                        return Ok(Some(active_request));
                    }
                }
                None => return Ok(None),
            }
        }
    }
}
