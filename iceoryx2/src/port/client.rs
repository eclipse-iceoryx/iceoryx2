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

use core::{cell::UnsafeCell, fmt::Debug, marker::PhantomData, sync::atomic::Ordering};
use std::sync::Arc;

use iceoryx2_bb_elementary::{visitor::Visitor, CallbackProgression};
use iceoryx2_bb_lock_free::mpmc::container::{ContainerHandle, ContainerState};
use iceoryx2_bb_log::{fail, warn};
use iceoryx2_cal::{dynamic_storage::DynamicStorage, zero_copy_connection::ZeroCopyCreationError};
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicUsize;

use crate::{
    port::{details::data_segment::DataSegment, UniqueClientId},
    prelude::PortFactory,
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
        outgoing_connections::{OutgoingConnections, ReceiverDetails},
        segment_state::SegmentState,
    },
    update_connections::UpdateConnections,
};

#[derive(Debug)]
pub struct Client<
    Service: service::Service,
    RequestPayload: Debug,
    RequestHeader: Debug,
    ResponsePayload: Debug,
    ResponseHeader: Debug,
> {
    client_handle: Option<ContainerHandle>,
    server_list_state: UnsafeCell<ContainerState<ServerDetails>>,
    server_connections: OutgoingConnections<Service>,
    service_state: Arc<ServiceState<Service>>,
    client_port_id: UniqueClientId,
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
    > Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    pub(crate) fn new(
        client_factory: &PortFactoryClient<
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
            server_list_state: UnsafeCell::new(unsafe { server_list.get_state() }),
            server_connections: OutgoingConnections {
                data_segment,
                segment_states: vec![SegmentState::new(number_of_requests)],
                sender_port_id: client_port_id.value(),
                shared_node: service.__internal_state().shared_node.clone(),
                connections: (0..server_list.capacity())
                    .map(|_| UnsafeCell::new(None))
                    .collect(),
                receiver_max_buffer_size: static_config.max_request_buffer_size,
                receiver_max_borrowed_samples: static_config.max_borrowed_requests,
                enable_safe_overflow: static_config.enable_safe_overflow_for_requests,
                degration_callback: None,
                number_of_samples: number_of_requests,
                max_number_of_segments,
                service_state: service.__internal_state().clone(),
                visitor: Visitor::new(),
                loan_counter: IoxAtomicUsize::new(0),
                sender_max_borrowed_samples: client_factory.max_loaned_requests,
                unable_to_deliver_strategy: client_factory.unable_to_deliver_strategy,
            },
            client_port_id,
            service_state: service.__internal_state().clone(),
            _request_payload: PhantomData,
            _request_header: PhantomData,
            _response_payload: PhantomData,
            _response_header: PhantomData,
        };

        if let Err(e) = new_self.force_update_connections() {
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

    pub fn id(&self) -> UniqueClientId {
        self.client_port_id
    }

    fn force_update_connections(&self) -> Result<(), ZeroCopyCreationError> {
        let mut result = Ok(());
        self.server_connections.start_update_connection_cycle();
        unsafe {
            (*self.server_list_state.get()).for_each(|h, port| {
                let inner_result = self.server_connections.update_connection(
                    h.index() as usize,
                    ReceiverDetails {
                        port_id: port.server_port_id.value(),
                        buffer_size: port.buffer_size,
                    },
                    |_| {},
                );

                if result.is_ok() {
                    result = inner_result;
                }

                CallbackProgression::Continue
            })
        };

        self.server_connections.finish_update_connection_cycle();

        result
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
}
