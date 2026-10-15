// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use core::fmt::Debug;

use iceoryx2::active_request::ActiveRequest;
use iceoryx2::pending_response::PendingResponse;
use iceoryx2::port::client::Client;
use iceoryx2::port::server::Server;
use iceoryx2::service::Service;
use iceoryx2::service::builder::request_response::Builder;
use iceoryx2::service::port_factory::request_response::PortFactory;
use iceoryx2_bb_elementary_traits::iceoryx_send::IceoryxSend;
use iceoryx2_bb_elementary_traits::type_name::TypeName;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

use crate::parameters::{FixedSizePayload, RequestPayload, ResponsePayload};

impl<T> RequestPayload for FixedSizePayload<T>
where
    T: IceoryxSend + ZeroCopySend + TypeName + Debug + Copy + PartialEq + From<u64> + 'static,
{
    fn create<S: Service, R: IceoryxSend + Debug>(
        builder: Builder<T, (), R, (), S>,
    ) -> PortFactory<S, T, (), R, ()> {
        builder.create().expect("service is created")
    }

    fn open<S: Service, R: IceoryxSend + Debug>(
        builder: Builder<T, (), R, (), S>,
    ) -> PortFactory<S, T, (), R, ()> {
        builder.open().expect("service is opened")
    }

    fn client<S: Service, R: IceoryxSend + Debug>(
        service: &PortFactory<S, T, (), R, ()>,
    ) -> Client<S, T, (), R, ()> {
        service
            .client_builder()
            .create()
            .expect("client is created")
    }

    fn send_request<S: Service, R: IceoryxSend + Debug>(
        client: &Client<S, T, (), R, ()>,
        value: T,
    ) -> PendingResponse<S, T, (), R, ()> {
        client.send_copy(value).expect("request is sent")
    }

    fn receive_request<S: Service, R: IceoryxSend + Debug>(
        server: &Server<S, T, (), R, ()>,
    ) -> Option<ActiveRequest<S, T, (), R, ()>> {
        server.receive().expect("receive succeeds")
    }

    fn request_value<S: Service, R: IceoryxSend + Debug>(
        request: &ActiveRequest<S, T, (), R, ()>,
    ) -> T {
        *request.payload()
    }
}

impl<T> ResponsePayload for FixedSizePayload<T>
where
    T: IceoryxSend + ZeroCopySend + TypeName + Debug + Copy + PartialEq + From<u64> + 'static,
{
    fn server<S: Service, Q: IceoryxSend + Debug>(
        service: &PortFactory<S, Q, (), T, ()>,
    ) -> Server<S, Q, (), T, ()> {
        service
            .server_builder()
            .create()
            .expect("server is created")
    }

    fn send_response<S: Service, Q: IceoryxSend + Debug>(
        request: &ActiveRequest<S, Q, (), T, ()>,
        value: T,
    ) {
        request.send_copy(value).expect("response is sent");
    }

    fn receive_response<S: Service, Q: IceoryxSend + Debug>(
        pending: &PendingResponse<S, Q, (), T, ()>,
    ) -> Option<T> {
        pending
            .receive()
            .expect("receive succeeds")
            .map(|response| *response.payload())
    }
}
