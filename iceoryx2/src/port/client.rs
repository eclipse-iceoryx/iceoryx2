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

use crate::{
    port::UniqueClientId,
    service::{
        self,
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
    _service: PhantomData<Service>,
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
        let port_id = UniqueClientId::new();
        let number_of_chunks = unsafe {
            service
                .__internal_state()
                .static_config
                .messaging_pattern
                .request_response()
        }
        .required_amount_of_chunks_per_client_data_segment(client_factory.max_loaned_requests);

        todo!()
    }
}
