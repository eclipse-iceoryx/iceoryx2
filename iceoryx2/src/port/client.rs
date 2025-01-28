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

use core::{fmt::Debug, marker::PhantomData};

use iceoryx2_bb_lock_free::mpmc::container::ContainerHandle;
use iceoryx2_bb_log::fail;
use iceoryx2_cal::dynamic_storage::DynamicStorage;

use crate::{
    port::{details::data_segment::DataSegment, UniqueClientId},
    prelude::PortFactory,
    service::{
        self,
        dynamic_config::request_response::ClientDetails,
        naming_scheme::data_segment_name,
        port_factory::client::{ClientCreateError, PortFactoryClient},
    },
};

pub struct Client<
    Service: service::Service,
    RequestPayload: Debug,
    RequestHeader: Debug,
    ResponsePayload: Debug,
    ResponseHeader: Debug,
> {
    data_segment: DataSegment<Service>,
    client_handle: ContainerHandle,
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
        let client_id = UniqueClientId::new();
        let number_of_requests = unsafe {
            service
                .__internal_state()
                .static_config
                .messaging_pattern
                .request_response()
        }
        .required_amount_of_chunks_per_client_data_segment(client_factory.max_loaned_requests);

        let static_config = client_factory.factory.static_config();
        let global_config = service.__internal_state().shared_node.config();
        let segment_name = data_segment_name(client_id.value());
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
            client_id,
            node_id: *service.__internal_state().shared_node.id(),
            number_of_requests,
        };

        let client_handle = match service
            .__internal_state()
            .dynamic_storage
            .get()
            .request_response()
            .add_client_id(client_details)
        {
            Some(handle) => handle,
            None => {
                fail!(from origin,
                      with ClientCreateError::ExceedsMaxSupportedClients,
                      "{} since it would exceed the maximum support amount of clients of {}.",
                      msg, service.__internal_state().static_config.request_response().max_clients());
            }
        };

        Ok(Self {
            data_segment,
            client_handle,
            _request_payload: PhantomData,
            _request_header: PhantomData,
            _response_payload: PhantomData,
            _response_header: PhantomData,
        })
    }
}
