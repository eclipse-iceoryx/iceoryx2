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
//!
//! let service = node
//!    .service_builder(&"My/Funk/ServiceName".try_into()?)
//!    .request_response::<u64, u64>()
//!    .open_or_create()?;
//!
//! let client = service.client_builder().create()?;
//!
//! let request = client.loan_uninit()?;
//! let request = request.write_payload(1829);
//!
//! let pending_response = request.send()?;
//!
//! # Ok(())
//! # }
//! ```

extern crate alloc;

use alloc::sync::Arc;
use core::{
    cell::UnsafeCell, fmt::Debug, marker::PhantomData, mem::MaybeUninit, sync::atomic::Ordering,
};

use iceoryx2_bb_elementary::{cyclic_tagger::CyclicTagger, CallbackProgression};
use iceoryx2_bb_lock_free::mpmc::container::{ContainerHandle, ContainerState};
use iceoryx2_bb_log::{fail, warn};
use iceoryx2_cal::{
    dynamic_storage::DynamicStorage, shm_allocator::PointerOffset,
    zero_copy_connection::ZeroCopyCreationError,
};
use iceoryx2_pal_concurrency_sync::iox_atomic::{IoxAtomicBool, IoxAtomicUsize};

use crate::{
    pending_response::PendingResponse,
    port::{details::data_segment::DataSegment, UniqueClientId},
    prelude::{PortFactory, UnableToDeliverStrategy},
    raw_sample::RawSampleMut,
    request_mut::RequestMut,
    request_mut_uninit::RequestMutUninit,
    service::{
        self,
        dynamic_config::request_response::{ClientDetails, ServerDetails},
        naming_scheme::data_segment_name,
        port_factory::client::{ClientCreateError, PortFactoryClient},
        ServiceState,
    },
};

use super::{
    details::{
        data_segment::DataSegmentType,
        segment_state::SegmentState,
        sender::{ReceiverDetails, Sender},
    },
    update_connections::UpdateConnections,
    LoanError, SendError,
};

#[derive(Debug)]
pub(crate) struct ClientBackend<Service: service::Service> {
    pub(crate) sender: Sender<Service>,
    is_active: IoxAtomicBool,
    server_list_state: UnsafeCell<ContainerState<ServerDetails>>,
    service_state: Arc<ServiceState<Service>>,
}

impl<Service: service::Service> ClientBackend<Service> {
    pub(crate) fn send_request(
        &self,
        offset: PointerOffset,
        sample_size: usize,
    ) -> Result<usize, SendError> {
        let msg = "Unable to send request";
        if !self.is_active.load(Ordering::Relaxed) {
            fail!(from self, with SendError::ConnectionBrokenSinceSenderNoLongerExists,
                "{} since the connections could not be updated.", msg);
        }

        fail!(from self, when self.update_connections(),
            "{} since the connections could not be updated.", msg);

        self.sender.deliver_offset(offset, sample_size)
    }

    fn update_connections(&self) -> Result<(), super::update_connections::ConnectionFailure> {
        if unsafe {
            self.service_state
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

    fn force_update_connections(&self) -> Result<(), ZeroCopyCreationError> {
        let mut result = Ok(());
        self.sender.start_update_connection_cycle();
        unsafe {
            (*self.server_list_state.get()).for_each(|h, port| {
                let inner_result = self.sender.update_connection(
                    h.index() as usize,
                    ReceiverDetails {
                        port_id: port.server_port_id.value(),
                        buffer_size: port.buffer_size,
                    },
                    |_| {},
                );

                result = result.and(inner_result);
                CallbackProgression::Continue
            })
        };

        self.sender.finish_update_connection_cycle();

        result
    }
}

/// Sends [`RequestMut`]s to a [`Server`](crate::port::server::Server) in a
/// request-response based communication.
#[derive(Debug)]
pub struct Client<
    Service: service::Service,
    RequestPayload: Debug,
    RequestHeader: Debug,
    ResponsePayload: Debug,
    ResponseHeader: Debug,
> {
    client_handle: Option<ContainerHandle>,
    client_port_id: UniqueClientId,
    backend: Arc<ClientBackend<Service>>,
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
    > Drop for Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn drop(&mut self) {
        if let Some(handle) = self.client_handle {
            self.backend
                .service_state
                .dynamic_storage
                .get()
                .request_response()
                .release_client_handle(handle)
        }
        self.backend.is_active.store(false, Ordering::Relaxed);
    }
}

impl<
        Service: service::Service,
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
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
        let client_port_id = UniqueClientId::new();
        let number_of_requests = unsafe {
            service
                .__internal_state()
                .static_config
                .messaging_pattern
                .request_response()
        }
        .required_amount_of_chunks_per_client_data_segment(client_factory.max_loaned_requests);
        let server_list = &service
            .__internal_state()
            .dynamic_storage
            .get()
            .request_response()
            .servers;

        let static_config = client_factory.factory.static_config();
        let global_config = service.__internal_state().shared_node.config();
        let segment_name = data_segment_name(client_port_id.value());
        let data_segment_type = DataSegmentType::Static;
        let max_number_of_segments =
            DataSegment::<Service>::max_number_of_segments(data_segment_type);
        let data_segment = DataSegment::<Service>::create_static_segment(
            &segment_name,
            static_config.request_message_type_details.sample_layout(1),
            global_config,
            number_of_requests,
        );

        let data_segment = fail!(from origin,
            when data_segment,
            with ClientCreateError::UnableToCreateDataSegment,
            "{} since the client data segment could not be created.", msg);

        let client_details = ClientDetails {
            client_port_id,
            node_id: *service.__internal_state().shared_node.id(),
            number_of_requests,
        };

        let mut new_self = Self {
            client_handle: None,
            backend: Arc::new(ClientBackend {
                sender: Sender {
                    data_segment,
                    segment_states: vec![SegmentState::new(number_of_requests)],
                    sender_port_id: client_port_id.value(),
                    shared_node: service.__internal_state().shared_node.clone(),
                    connections: (0..server_list.capacity())
                        .map(|_| UnsafeCell::new(None))
                        .collect(),
                    receiver_max_buffer_size: static_config.max_request_buffer_size,
                    receiver_max_borrowed_samples: static_config.max_active_requests_per_client,
                    enable_safe_overflow: static_config.enable_safe_overflow_for_requests,
                    degration_callback: client_factory.degration_callback,
                    number_of_samples: number_of_requests,
                    max_number_of_segments,
                    service_state: service.__internal_state().clone(),
                    tagger: CyclicTagger::new(),
                    loan_counter: IoxAtomicUsize::new(0),
                    sender_max_borrowed_samples: client_factory.max_loaned_requests,
                    unable_to_deliver_strategy: client_factory.unable_to_deliver_strategy,
                    message_type_details: static_config.request_message_type_details.clone(),
                },
                is_active: IoxAtomicBool::new(true),
                server_list_state: UnsafeCell::new(unsafe { server_list.get_state() }),
                service_state: service.__internal_state().clone(),
            }),
            client_port_id,
            _request_payload: PhantomData,
            _request_header: PhantomData,
            _response_payload: PhantomData,
            _response_header: PhantomData,
        };

        if let Err(e) = new_self.backend.force_update_connections() {
            warn!(from new_self,
                "The new Client port is unable to connect to every Server port, caused by {:?}.", e);
        }

        core::sync::atomic::compiler_fence(Ordering::SeqCst);

        // !MUST! be the last task otherwise a client is added to the dynamic config without the
        // creation of all required resources
        new_self.client_handle = match service
            .__internal_state()
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
                      msg, service.__internal_state().static_config.request_response().max_clients());
            }
        };

        Ok(new_self)
    }

    /// Returns the [`UniqueClientId`] of the [`Client`]
    pub fn id(&self) -> UniqueClientId {
        self.client_port_id
    }

    /// Returns the strategy the [`Client`] follows when a [`RequestMut`] cannot be delivered
    /// if the [`Server`](crate::port::server::Server)s buffer is full.
    pub fn unable_to_deliver_strategy(&self) -> UnableToDeliverStrategy {
        self.backend.sender.unable_to_deliver_strategy
    }

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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        let chunk = self
            .backend
            .sender
            .allocate(self.backend.sender.sample_layout(1))?;

        unsafe {
            (chunk.header as *mut service::header::request_response::RequestHeader).write(
                service::header::request_response::RequestHeader {
                    client_port_id: self.id(),
                },
            )
        };

        let ptr = unsafe {
            RawSampleMut::<
                service::header::request_response::RequestHeader,
                RequestHeader,
                MaybeUninit<RequestPayload>,
            >::new_unchecked(
                chunk.header.cast(),
                chunk.user_header.cast(),
                chunk.payload.cast(),
            )
        };

        Ok(RequestMutUninit {
            request: RequestMut {
                ptr,
                sample_size: chunk.size,
                offset_to_chunk: chunk.offset,
                client_backend: self.backend.clone(),
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
        SendError,
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
        RequestPayload: Debug + Default,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node
    /// #    .service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #    .request_response::<u64, u64>()
    /// #    .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    ///
    /// // Acquire request that is initialized with by `Default::default()`.
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

impl<
        Service: service::Service,
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
    > UpdateConnections
    for Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn update_connections(&self) -> Result<(), super::update_connections::ConnectionFailure> {
        self.backend.update_connections()
    }
}
