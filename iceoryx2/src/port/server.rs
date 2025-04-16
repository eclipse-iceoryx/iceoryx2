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
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//! #
//! let service = node
//!     .service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .request_response::<u64, u64>()
//!     .open_or_create()?;
//!
//! let server = service.server_builder().create()?;
//!
//! while let Some(active_request) = server.receive()? {
//!     println!("received request: {:?}", *active_request);
//! }
//! # Ok(())
//! # }
//! ```

extern crate alloc;

use alloc::sync::Arc;
use core::{cell::UnsafeCell, sync::atomic::Ordering};
use core::{fmt::Debug, marker::PhantomData};
use iceoryx2_bb_container::vec::Vec;
use iceoryx2_cal::zero_copy_connection::ChannelId;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicUsize;

use iceoryx2_bb_elementary::{cyclic_tagger::CyclicTagger, CallbackProgression};
use iceoryx2_bb_lock_free::mpmc::container::{ContainerHandle, ContainerState};
use iceoryx2_bb_log::{fail, warn};
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_cal::dynamic_storage::DynamicStorage;

use crate::service::naming_scheme::data_segment_name;
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

#[derive(Debug)]
pub(crate) struct SharedServerState<Service: service::Service> {
    pub(crate) response_sender: Sender<Service>,
    server_handle: UnsafeCell<Option<ContainerHandle>>,
    request_receiver: Receiver<Service>,
    client_list_state: UnsafeCell<ContainerState<ClientDetails>>,
    service_state: Arc<ServiceState<Service>>,
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
                        port_id: details.client_port_id.value(),
                        number_of_samples: details.number_of_requests,
                        max_number_of_segments: 1,
                        data_segment_type: DataSegmentType::Static,
                    },
                );
                result = result.and(inner_result);

                // establish response connection
                let inner_result = self.response_sender.update_connection(
                    h.index() as usize,
                    ReceiverDetails {
                        port_id: details.client_port_id.value(),
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
    RequestPayload: Debug,
    RequestHeader: Debug,
    ResponsePayload: Debug,
    ResponseHeader: Debug,
> {
    shared_state: Arc<SharedServerState<Service>>,
    _request_payload: PhantomData<RequestPayload>,
    _request_header: PhantomData<RequestHeader>,
    _response_payload: PhantomData<ResponsePayload>,
    _response_header: PhantomData<ResponseHeader>,
}

impl<
        Service: service::Service,
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
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
        let server_port_id = UniqueServerId::new();
        let service = &server_factory.factory.service;
        let static_config = server_factory.factory.static_config();
        let number_of_requests = unsafe {
            service
                .__internal_state()
                .static_config
                .messaging_pattern
                .request_response()
        }
        .required_amount_of_chunks_per_client_data_segment(
            static_config.client_max_loaned_requests,
        );

        let number_of_responses = unsafe {
            service
                .__internal_state()
                .static_config
                .messaging_pattern
                .request_response()
        }
        .required_amount_of_chunks_per_server_data_segment(
            server_factory.max_loaned_responses_per_request,
            number_of_requests,
        );

        let client_list = &service
            .__internal_state()
            .dynamic_storage
            .get()
            .request_response()
            .clients;

        let request_receiver = Receiver {
            connections: Vec::from_fn(client_list.capacity(), |_| UnsafeCell::new(None)),
            receiver_port_id: server_port_id.value(),
            service_state: service.__internal_state().clone(),
            message_type_details: static_config.request_message_type_details.clone(),
            receiver_max_borrowed_samples: static_config.max_active_requests_per_client,
            enable_safe_overflow: static_config.enable_safe_overflow_for_requests,
            buffer_size: static_config.max_active_requests_per_client,
            tagger: CyclicTagger::new(),
            to_be_removed_connections: None,
            degradation_callback: server_factory.request_degradation_callback,
            number_of_channels: 1,
        };

        let global_config = service.__internal_state().shared_node.config();
        let data_segment_type = DataSegmentType::Static;
        let segment_name = data_segment_name(server_port_id.value());
        let max_number_of_segments =
            DataSegment::<Service>::max_number_of_segments(data_segment_type);
        let data_segment = DataSegment::<Service>::create_static_segment(
            &segment_name,
            static_config.response_message_type_details.sample_layout(1),
            global_config,
            number_of_responses,
        );

        let data_segment = fail!(from origin,
            when data_segment,
            with ServerCreateError::UnableToCreateDataSegment,
            "{} since the server data segment could not be created.", msg);

        let response_sender = Sender {
            segment_states: vec![SegmentState::new(number_of_responses)],
            data_segment,
            connections: (0..client_list.capacity())
                .map(|_| UnsafeCell::new(None))
                .collect(),
            sender_port_id: server_port_id.value(),
            shared_node: service.__internal_state().shared_node.clone(),
            receiver_max_buffer_size: static_config.max_response_buffer_size,
            receiver_max_borrowed_samples: static_config
                .max_borrowed_responses_per_pending_response,
            sender_max_borrowed_samples: server_factory.max_loaned_responses_per_request,
            enable_safe_overflow: static_config.enable_safe_overflow_for_responses,
            number_of_samples: number_of_responses,
            max_number_of_segments,
            degradation_callback: server_factory.response_degradation_callback,
            service_state: service.__internal_state().clone(),
            tagger: CyclicTagger::new(),
            loan_counter: IoxAtomicUsize::new(0),
            unable_to_deliver_strategy: server_factory.unable_to_deliver_strategy,
            message_type_details: static_config.response_message_type_details.clone(),
            number_of_channels: number_of_requests,
        };

        let new_self = Self {
            shared_state: Arc::new(SharedServerState {
                request_receiver,
                client_list_state: UnsafeCell::new(unsafe { client_list.get_state() }),
                response_sender,
                server_handle: UnsafeCell::new(None),
                service_state: service.__internal_state().clone(),
            }),
            _request_payload: PhantomData,
            _request_header: PhantomData,
            _response_payload: PhantomData,
            _response_header: PhantomData,
        };

        if let Err(e) = new_self.shared_state.force_update_connections() {
            warn!(from new_self, "The new server is unable to connect to every client, caused by {:?}.", e);
        }

        core::sync::atomic::compiler_fence(Ordering::SeqCst);

        // !MUST! be the last task otherwise a server is added to the dynamic config without the
        // creation of all required resources
        unsafe {
            *new_self.shared_state.server_handle.get() = match service
                .__internal_state()
                .dynamic_storage
                .get()
                .request_response()
                .add_server_id(ServerDetails {
                    server_port_id,
                    request_buffer_size: static_config.max_active_requests_per_client,
                    number_of_responses,
                }) {
                Some(v) => Some(v),
                None => {
                    fail!(from origin,
                    with ServerCreateError::ExceedsMaxSupportedServers,
                    "{} since it would exceed the maximum supported amount of servers of {}.",
                    msg, service.__internal_state().static_config.request_response().max_servers());
                }
            }
        };

        Ok(new_self)
    }

    /// Returns the [`UniqueServerId`] of the [`Server`]
    pub fn id(&self) -> UniqueServerId {
        UniqueServerId(UniqueSystemId::from(
            self.shared_state.request_receiver.receiver_port_id,
        ))
    }

    /// Returns true if the [`Server`] has [`RequestMut`](crate::request_mut::RequestMut)s in its buffer.
    pub fn has_requests(&self) -> Result<bool, ConnectionFailure> {
        fail!(from self, when self.shared_state.update_connections(),
                "Some requests are not being received since not all connections to clients could be established.");
        self.shared_state
            .request_receiver
            .has_samples(ChannelId::new(0))
    }

    fn receive_impl(&self) -> Result<Option<(ChunkDetails<Service>, Chunk)>, ReceiveError> {
        if let Err(e) = self.shared_state.update_connections() {
            fail!(from self,
                  with ReceiveError::ConnectionFailure(e),
                  "Some requests are not being received since not all connections to the clients could be established.");
        }

        self.shared_state
            .request_receiver
            .receive(ChannelId::new(0))
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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

                    match self
                        .shared_state
                        .response_sender
                        .get_connection_id_of(header.client_port_id.value())
                    {
                        Some(connection_id) => {
                            let active_request = ActiveRequest {
                                details,
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
                            };

                            if !active_request.is_connected() {
                                continue;
                            }

                            return Ok(Some(active_request));
                        }

                        None => (),
                    }
                }
                None => return Ok(None),
            }
        }
    }
}
