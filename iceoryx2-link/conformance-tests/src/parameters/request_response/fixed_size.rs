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
use iceoryx2::service::port_factory::request_response::PortFactory;
use iceoryx2_bb_elementary_traits::iceoryx_send::IceoryxSend;
use iceoryx2_bb_elementary_traits::type_name::TypeName;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

use crate::parameters::{FixedSizePayload, RequestPayload, ResponsePayload, UserHeader};

impl<T> RequestPayload for FixedSizePayload<T>
where
    T: IceoryxSend + ZeroCopySend + TypeName + Debug + Copy + PartialEq + From<u64> + 'static,
{
    fn client<S: Service, QH: UserHeader, R: IceoryxSend + Debug + ?Sized, RH: UserHeader>(
        service: &PortFactory<S, T, QH, R, RH>,
    ) -> Client<S, T, QH, R, RH> {
        service
            .client_builder()
            .create()
            .expect("client is created")
    }

    fn send_request<S: Service, QH: UserHeader, R: IceoryxSend + Debug + ?Sized, RH: UserHeader>(
        client: &Client<S, T, QH, R, RH>,
        header: QH,
        value: T,
    ) -> PendingResponse<S, T, QH, R, RH> {
        let mut request = client.loan_uninit().expect("request is loaned");
        *request.user_header_mut() = header;
        request
            .write_payload(value)
            .send()
            .expect("request is sent")
    }

    fn receive_request<
        S: Service,
        QH: UserHeader,
        R: IceoryxSend + Debug + ?Sized,
        RH: UserHeader,
    >(
        server: &Server<S, T, QH, R, RH>,
    ) -> Option<ActiveRequest<S, T, QH, R, RH>> {
        server.receive().expect("receive succeeds")
    }

    fn request_header<
        S: Service,
        QH: UserHeader,
        R: IceoryxSend + Debug + ?Sized,
        RH: UserHeader,
    >(
        request: &ActiveRequest<S, T, QH, R, RH>,
    ) -> QH {
        *request.user_header()
    }

    fn request_value<
        S: Service,
        QH: UserHeader,
        R: IceoryxSend + Debug + ?Sized,
        RH: UserHeader,
    >(
        request: &ActiveRequest<S, T, QH, R, RH>,
    ) -> T {
        *request.payload()
    }
}

impl<T> ResponsePayload for FixedSizePayload<T>
where
    T: IceoryxSend + ZeroCopySend + TypeName + Debug + Copy + PartialEq + From<u64> + 'static,
{
    fn server<S: Service, Q: IceoryxSend + Debug + ?Sized, QH: UserHeader, RH: UserHeader>(
        service: &PortFactory<S, Q, QH, T, RH>,
    ) -> Server<S, Q, QH, T, RH> {
        service
            .server_builder()
            .create()
            .expect("server is created")
    }

    fn send_response<
        S: Service,
        Q: IceoryxSend + Debug + ?Sized,
        QH: UserHeader,
        RH: UserHeader,
    >(
        request: &ActiveRequest<S, Q, QH, T, RH>,
        header: RH,
        value: T,
    ) {
        let mut response = request.loan_uninit().expect("response is loaned");
        *response.user_header_mut() = header;
        response
            .write_payload(value)
            .send()
            .expect("response is sent");
    }

    fn receive_response<
        S: Service,
        Q: IceoryxSend + Debug + ?Sized,
        QH: UserHeader,
        RH: UserHeader,
    >(
        pending: &PendingResponse<S, Q, QH, T, RH>,
    ) -> Option<(RH, T)> {
        pending
            .receive()
            .expect("receive succeeds")
            .map(|response| (*response.user_header(), *response.payload()))
    }
}
