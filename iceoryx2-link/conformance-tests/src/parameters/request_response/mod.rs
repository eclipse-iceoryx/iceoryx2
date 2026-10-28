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

mod fixed_size;
mod service_creator;

pub use service_creator::{CreatesRequestResponseService, ServiceCreator};

use alloc::vec::Vec;
use core::fmt::Debug;
use core::marker::PhantomData;

use iceoryx2::active_request::ActiveRequest;
use iceoryx2::config::Config;
use iceoryx2::node::Node;
use iceoryx2::pending_response::PendingResponse;
use iceoryx2::port::client::Client;
use iceoryx2::port::server::Server;
use iceoryx2::service::Service;
use iceoryx2::service::builder::request_response::Builder;
use iceoryx2::service::messaging_pattern::MessagingPattern;
use iceoryx2::service::port_factory::request_response::PortFactory;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2::testing::generate_service_name;
use iceoryx2_bb_elementary_traits::iceoryx_send::IceoryxSend;
use iceoryx2_link_backend::service_description::ServiceDescription;

use super::{AnyName, AnyService, PayloadShape, UserHeader};
use crate::testing::describe_request_response;

/// The request payload type the services of `X` are created with.
pub type Request<X> = <<X as RequestResponseService>::Request as PayloadShape>::Type;
/// The request user header type the services of `X` are created with.
pub type RequestHeader<X> = <X as RequestResponseService>::RequestHeader;
/// The response payload type the services of `X` are created with.
pub type Response<X> = <<X as RequestResponseService>::Response as PayloadShape>::Type;
/// The response user header type the services of `X` are created with.
pub type ResponseHeader<X> = <X as RequestResponseService>::ResponseHeader;
/// A request value of the services of `X` in the suites' hands.
pub type RequestValue<X> = <<X as RequestResponseService>::Request as PayloadShape>::Value;
/// A response value of the services of `X` in the suites' hands.
pub type ResponseValue<X> = <<X as RequestResponseService>::Response as PayloadShape>::Value;

/// A source of names for request-response services the mapping covers.
pub trait RequestResponseName {
    /// A fresh service name.
    fn service_name() -> ServiceName;
}

impl RequestResponseName for AnyName {
    fn service_name() -> ServiceName {
        generate_service_name()
    }
}

/// A payload shape on the request side of a request-response service, how
/// a request is sent and read back.
pub trait RequestPayload: PayloadShape {
    /// A client on `service`, sized for the values.
    fn client<S: Service, QH: UserHeader, R: IceoryxSend + Debug + ?Sized, RH: UserHeader>(
        service: &PortFactory<S, Self::Type, QH, R, RH>,
    ) -> Client<S, Self::Type, QH, R, RH>;

    /// Sends `value` under `header` as a request.
    fn send_request<S: Service, QH: UserHeader, R: IceoryxSend + Debug + ?Sized, RH: UserHeader>(
        client: &Client<S, Self::Type, QH, R, RH>,
        header: QH,
        value: Self::Value,
    ) -> PendingResponse<S, Self::Type, QH, R, RH>;

    /// The next request pending on `server`, if any.
    fn receive_request<
        S: Service,
        QH: UserHeader,
        R: IceoryxSend + Debug + ?Sized,
        RH: UserHeader,
    >(
        server: &Server<S, Self::Type, QH, R, RH>,
    ) -> Option<ActiveRequest<S, Self::Type, QH, R, RH>>;

    /// The header `request` carries.
    fn request_header<S: Service, QH: UserHeader, R: IceoryxSend + Debug + ?Sized, RH: UserHeader>(
        request: &ActiveRequest<S, Self::Type, QH, R, RH>,
    ) -> QH;

    /// The value `request` carries.
    fn request_value<S: Service, QH: UserHeader, R: IceoryxSend + Debug + ?Sized, RH: UserHeader>(
        request: &ActiveRequest<S, Self::Type, QH, R, RH>,
    ) -> Self::Value;
}

/// A payload shape on the response side of a request-response service,
/// how a response is sent and read back.
pub trait ResponsePayload: PayloadShape {
    /// A server on `service`, sized for the values.
    fn server<S: Service, Q: IceoryxSend + Debug + ?Sized, QH: UserHeader, RH: UserHeader>(
        service: &PortFactory<S, Q, QH, Self::Type, RH>,
    ) -> Server<S, Q, QH, Self::Type, RH>;

    /// Sends `value` under `header` as a response to `request`.
    fn send_response<S: Service, Q: IceoryxSend + Debug + ?Sized, QH: UserHeader, RH: UserHeader>(
        request: &ActiveRequest<S, Q, QH, Self::Type, RH>,
        header: RH,
        value: Self::Value,
    );

    /// The next response pending on `pending`, if any, its header and its
    /// value.
    fn receive_response<
        S: Service,
        Q: IceoryxSend + Debug + ?Sized,
        QH: UserHeader,
        RH: UserHeader,
    >(
        pending: &PendingResponse<S, Q, QH, Self::Type, RH>,
    ) -> Option<(RH, Self::Value)>;
}

/// A request-response service the suites run over.
pub trait RequestResponseService: AnyService {
    /// The request payload of the service.
    type Request: RequestPayload;
    /// The user header of the service's requests when they have one.
    type RequestHeader: UserHeader;
    /// The response payload of the service.
    type Response: ResponsePayload;
    /// The user header of the service's responses when they have one.
    type ResponseHeader: UserHeader;
    /// Creates and opens the service for its pair of payload types.
    type ServiceCreator: CreatesRequestResponseService<Request = Request<Self>, Response = Response<Self>>;

    /// Creates the service `name` on `node` under the user headers `QH`
    /// and `RH`.
    fn create_service<QH: UserHeader, RH: UserHeader, S: Service>(
        node: &Node<S>,
        name: &ServiceName,
    ) -> PortFactory<S, Request<Self>, QH, Response<Self>, RH> {
        Self::create_service_with::<QH, RH, S>(node, name, |builder| builder)
    }

    /// Creates the service `name` on `node` under the user headers `QH`
    /// and `RH`, with the settings `configure` sets on the builder.
    fn create_service_with<QH: UserHeader, RH: UserHeader, S: Service>(
        node: &Node<S>,
        name: &ServiceName,
        configure: impl FnOnce(
            Builder<Request<Self>, QH, Response<Self>, RH, S>,
        ) -> Builder<Request<Self>, QH, Response<Self>, RH, S>,
    ) -> PortFactory<S, Request<Self>, QH, Response<Self>, RH> {
        <Self::ServiceCreator as CreatesRequestResponseService>::create(configure(
            node.service_builder(name)
                .request_response::<Request<Self>, Response<Self>>()
                .request_user_header::<QH>()
                .response_user_header::<RH>(),
        ))
    }

    /// Opens the service `name` on `node` under the user headers `QH` and
    /// `RH`.
    fn open_service<QH: UserHeader, RH: UserHeader, S: Service>(
        node: &Node<S>,
        name: &ServiceName,
    ) -> PortFactory<S, Request<Self>, QH, Response<Self>, RH> {
        <Self::ServiceCreator as CreatesRequestResponseService>::open(
            node.service_builder(name)
                .request_response::<Request<Self>, Response<Self>>()
                .request_user_header::<QH>()
                .response_user_header::<RH>(),
        )
    }

    /// A client on `service`, sized for the values.
    fn create_client<S: Service, QH: UserHeader, RH: UserHeader>(
        service: &PortFactory<S, Request<Self>, QH, Response<Self>, RH>,
    ) -> Client<S, Request<Self>, QH, Response<Self>, RH> {
        <Self::Request as RequestPayload>::client(service)
    }

    /// A server on `service`, sized for the values.
    fn create_server<S: Service, QH: UserHeader, RH: UserHeader>(
        service: &PortFactory<S, Request<Self>, QH, Response<Self>, RH>,
    ) -> Server<S, Request<Self>, QH, Response<Self>, RH> {
        <Self::Response as ResponsePayload>::server(service)
    }

    /// Sends `value` under `header` as a request.
    fn send_request<S: Service, QH: UserHeader, RH: UserHeader>(
        client: &Client<S, Request<Self>, QH, Response<Self>, RH>,
        header: QH,
        value: RequestValue<Self>,
    ) -> PendingResponse<S, Request<Self>, QH, Response<Self>, RH> {
        <Self::Request as RequestPayload>::send_request(client, header, value)
    }

    /// The next request pending on `server`, if any.
    #[allow(clippy::type_complexity)]
    fn receive_request<S: Service, QH: UserHeader, RH: UserHeader>(
        server: &Server<S, Request<Self>, QH, Response<Self>, RH>,
    ) -> Option<ActiveRequest<S, Request<Self>, QH, Response<Self>, RH>> {
        <Self::Request as RequestPayload>::receive_request(server)
    }

    /// The header `request` carries.
    fn request_header<S: Service, QH: UserHeader, RH: UserHeader>(
        request: &ActiveRequest<S, Request<Self>, QH, Response<Self>, RH>,
    ) -> QH {
        <Self::Request as RequestPayload>::request_header(request)
    }

    /// The value `request` carries.
    fn request_value<S: Service, QH: UserHeader, RH: UserHeader>(
        request: &ActiveRequest<S, Request<Self>, QH, Response<Self>, RH>,
    ) -> RequestValue<Self> {
        <Self::Request as RequestPayload>::request_value(request)
    }

    /// Sends `value` under `header` as a response to `request`.
    fn send_response<S: Service, QH: UserHeader, RH: UserHeader>(
        request: &ActiveRequest<S, Request<Self>, QH, Response<Self>, RH>,
        header: RH,
        value: ResponseValue<Self>,
    ) {
        <Self::Response as ResponsePayload>::send_response(request, header, value)
    }

    /// The next response pending on `pending`, if any, its header and its
    /// value.
    fn receive_response<S: Service, QH: UserHeader, RH: UserHeader>(
        pending: &PendingResponse<S, Request<Self>, QH, Response<Self>, RH>,
    ) -> Option<(RH, ResponseValue<Self>)> {
        <Self::Response as ResponsePayload>::receive_response(pending)
    }
}

/// The request-response service named by `N`, of request payload `Q`
/// under the user header `QH` and response payload `R` under the user
/// header `RH`.
#[derive(Debug, Default, Clone, Copy)]
pub struct RequestResponse<N, Q, QH, R, RH>(PhantomData<(N, Q, QH, R, RH)>);

impl<N, Q, QH, R, RH> RequestResponseService for RequestResponse<N, Q, QH, R, RH>
where
    N: RequestResponseName,
    Q: RequestPayload,
    QH: UserHeader,
    R: ResponsePayload,
    RH: UserHeader,
    ServiceCreator<Q::Type, R::Type>:
        CreatesRequestResponseService<Request = Q::Type, Response = R::Type>,
{
    type Request = Q;
    type RequestHeader = QH;
    type Response = R;
    type ResponseHeader = RH;
    type ServiceCreator = ServiceCreator<Q::Type, R::Type>;
}

impl<N, Q, QH, R, RH> AnyService for RequestResponse<N, Q, QH, R, RH>
where
    N: RequestResponseName,
    Q: RequestPayload,
    QH: UserHeader,
    R: ResponsePayload,
    RH: UserHeader,
    ServiceCreator<Q::Type, R::Type>:
        CreatesRequestResponseService<Request = Q::Type, Response = R::Type>,
{
    const PATTERN: MessagingPattern = MessagingPattern::RequestResponse;
    type Handle<S: Service> = PortFactory<S, Request<Self>, QH, Response<Self>, RH>;
    type Port<S: Service> = Client<S, Request<Self>, QH, Response<Self>, RH>;

    fn service_name() -> ServiceName {
        N::service_name()
    }

    fn describe<S: Service>(name: &ServiceName, config: &Config) -> ServiceDescription {
        describe_request_response::<S, Q, QH, R, RH>(name, config)
    }

    fn create<S: Service>(node: &Node<S>, name: &ServiceName) -> Self::Handle<S> {
        Self::create_service::<QH, RH, S>(node, name)
    }

    fn open<S: Service>(node: &Node<S>, name: &ServiceName) -> Self::Handle<S> {
        Self::open_service::<QH, RH, S>(node, name)
    }

    fn create_port<S: Service>(service: &Self::Handle<S>) -> Self::Port<S> {
        Self::create_client(service)
    }

    fn occupy_ports<S: Service>(service: &Self::Handle<S>, config: &Config) -> Vec<Self::Port<S>> {
        (0..config.defaults.request_response.max_clients)
            .map(|_| Self::create_port(service))
            .collect()
    }
}
