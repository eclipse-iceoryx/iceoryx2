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

use iceoryx2_bb_container::queue::Queue;
use iceoryx2_bb_elementary::{cyclic_tagger::CyclicTagger, CallbackProgression};
use iceoryx2_bb_lock_free::mpmc::container::{ContainerHandle, ContainerState};
use iceoryx2_bb_log::{fail, warn};
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_cal::dynamic_storage::DynamicStorage;

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

use super::{
    details::{
        chunk::Chunk,
        chunk_details::ChunkDetails,
        data_segment::DataSegmentType,
        receiver::{Receiver, SenderDetails},
    },
    update_connections::{ConnectionFailure, UpdateConnections},
    ReceiveError, UniqueServerId,
};

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
    receiver: Receiver<Service>,
    server_handle: Option<ContainerHandle>,
    client_list_state: UnsafeCell<ContainerState<ClientDetails>>,
    service_state: Arc<ServiceState<Service>>,
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
    > Drop for Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn drop(&mut self) {
        if let Some(handle) = self.server_handle {
            self.service_state
                .dynamic_storage
                .get()
                .request_response()
                .release_server_handle(handle);
        }
    }
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

        let client_list = &service
            .__internal_state()
            .dynamic_storage
            .get()
            .request_response()
            .clients;

        let static_config = server_factory.factory.static_config();
        let receiver = Receiver {
            connections: (0..client_list.capacity())
                .map(|_| UnsafeCell::new(None))
                .collect(),
            receiver_port_id: server_port_id.value(),
            service_state: service.__internal_state().clone(),
            message_type_details: static_config.request_message_type_details.clone(),
            receiver_max_borrowed_samples: static_config.max_active_requests_per_client,
            enable_safe_overflow: static_config.enable_safe_overflow_for_requests,
            buffer_size: static_config.max_active_requests_per_client,
            tagger: CyclicTagger::new(),
            to_be_removed_connections: UnsafeCell::new(Queue::new(0)),
            degradation_callback: server_factory.degradation_callback,
        };

        let mut new_self = Self {
            receiver,
            server_handle: None,
            client_list_state: UnsafeCell::new(unsafe { client_list.get_state() }),
            service_state: service.__internal_state().clone(),
            _request_payload: PhantomData,
            _request_header: PhantomData,
            _response_payload: PhantomData,
            _response_header: PhantomData,
        };

        if let Err(e) = new_self.force_update_connections() {
            warn!(from new_self, "The new server is unable to connect to every client, caused by {:?}.", e);
        }

        core::sync::atomic::compiler_fence(Ordering::SeqCst);

        // !MUST! be the last task otherwise a server is added to the dynamic config without the
        // creation of all required resources
        new_self.server_handle = match service
            .__internal_state()
            .dynamic_storage
            .get()
            .request_response()
            .add_server_id(ServerDetails {
                server_port_id,
                buffer_size: static_config.max_active_requests_per_client,
            }) {
            Some(v) => Some(v),
            None => {
                fail!(from origin,
                    with ServerCreateError::ExceedsMaxSupportedServers,
                    "{} since it would exceed the maximum supported amount of servers of {}.",
                    msg, service.__internal_state().static_config.request_response().max_servers());
            }
        };

        Ok(new_self)
    }

    /// Returns the [`UniqueServerId`] of the [`Server`]
    pub fn id(&self) -> UniqueServerId {
        UniqueServerId(UniqueSystemId::from(self.receiver.receiver_port_id))
    }

    /// Returns true if the [`Server`] has [`RequestMut`](crate::request_mut::RequestMut)s in its buffer.
    pub fn has_requests(&self) -> Result<bool, ConnectionFailure> {
        fail!(from self, when self.update_connections(),
                "Some requests are not being received since not all connections to clients could be established.");
        self.receiver.has_samples()
    }

    fn force_update_connections(&self) -> Result<(), ConnectionFailure> {
        self.receiver.start_update_connection_cycle();

        let mut result = Ok(());
        unsafe {
            (*self.client_list_state.get()).for_each(|h, details| {
                let inner_result = self.receiver.update_connection(
                    h.index() as usize,
                    SenderDetails {
                        port_id: details.client_port_id.value(),
                        number_of_samples: details.number_of_requests,
                        max_number_of_segments: 1,
                        data_segment_type: DataSegmentType::Static,
                    },
                );

                if result.is_ok() {
                    result = inner_result;
                }
                CallbackProgression::Continue
            })
        };

        self.receiver.finish_update_connection_cycle();

        result
    }

    fn receive_impl(&self) -> Result<Option<(ChunkDetails<Service>, Chunk)>, ReceiveError> {
        if let Err(e) = self.update_connections() {
            fail!(from self,
                  with ReceiveError::ConnectionFailure(e),
                  "Some requests are not being received since not all connections to the clients could be established.");
        }

        self.receiver.receive()
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
        Ok(self.receive_impl()?.map(|(details, chunk)| ActiveRequest {
            details,
            ptr: unsafe {
                RawSample::new_unchecked(
                    chunk.header.cast(),
                    chunk.user_header.cast(),
                    chunk.payload.cast(),
                )
            },
            _response_payload: PhantomData,
            _response_header: PhantomData,
        }))
    }
}

impl<
        Service: service::Service,
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
    > UpdateConnections
    for Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn update_connections(&self) -> Result<(), ConnectionFailure> {
        if unsafe {
            self.receiver
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
}
