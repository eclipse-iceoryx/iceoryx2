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

use super::{AnyName, AnyService, PayloadShape};
use crate::testing::describe_request_response;

/// The request payload type the services of `X` are created with.
pub type Request<X> = <<X as RequestResponseService>::Request as PayloadShape>::Type;
/// The response payload type the services of `X` are created with.
pub type Response<X> = <<X as RequestResponseService>::Response as PayloadShape>::Type;
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
/// the service is built and how a request is sent and read back.
pub trait RequestPayload: PayloadShape {
    /// Creates the service `builder` describes.
    fn create<S: Service, R: IceoryxSend + Debug>(
        builder: Builder<Self::Type, (), R, (), S>,
    ) -> PortFactory<S, Self::Type, (), R, ()>;

    /// Opens the service `builder` describes.
    fn open<S: Service, R: IceoryxSend + Debug>(
        builder: Builder<Self::Type, (), R, (), S>,
    ) -> PortFactory<S, Self::Type, (), R, ()>;

    /// A client on `service`, sized for the values.
    fn client<S: Service, R: IceoryxSend + Debug>(
        service: &PortFactory<S, Self::Type, (), R, ()>,
    ) -> Client<S, Self::Type, (), R, ()>;

    /// Sends `value` as a request.
    fn send_request<S: Service, R: IceoryxSend + Debug>(
        client: &Client<S, Self::Type, (), R, ()>,
        value: Self::Value,
    ) -> PendingResponse<S, Self::Type, (), R, ()>;

    /// The next request pending on `server`, if any.
    fn receive_request<S: Service, R: IceoryxSend + Debug>(
        server: &Server<S, Self::Type, (), R, ()>,
    ) -> Option<ActiveRequest<S, Self::Type, (), R, ()>>;

    /// The value `request` carries.
    fn request_value<S: Service, R: IceoryxSend + Debug>(
        request: &ActiveRequest<S, Self::Type, (), R, ()>,
    ) -> Self::Value;
}

/// A payload shape on the response side of a request-response service,
/// how a response is sent and read back.
pub trait ResponsePayload: PayloadShape {
    /// A server on `service`, sized for the values.
    fn server<S: Service, Q: IceoryxSend + Debug>(
        service: &PortFactory<S, Q, (), Self::Type, ()>,
    ) -> Server<S, Q, (), Self::Type, ()>;

    /// Sends `value` as a response to `request`.
    fn send_response<S: Service, Q: IceoryxSend + Debug>(
        request: &ActiveRequest<S, Q, (), Self::Type, ()>,
        value: Self::Value,
    );

    /// The next response pending on `pending`, if any.
    fn receive_response<S: Service, Q: IceoryxSend + Debug>(
        pending: &PendingResponse<S, Q, (), Self::Type, ()>,
    ) -> Option<Self::Value>;
}

/// A request-response service the suites run over.
pub trait RequestResponseService: AnyService {
    /// The request payload of the service.
    type Request: RequestPayload<Type: Sized>;
    /// The response payload of the service.
    type Response: ResponsePayload<Type: Sized>;

    /// Creates the service `name` on `node`.
    fn create_service<S: Service>(
        node: &Node<S>,
        name: &ServiceName,
    ) -> PortFactory<S, Request<Self>, (), Response<Self>, ()> {
        Self::create_service_with::<S>(node, name, |builder| builder)
    }

    /// Creates the service `name` on `node` with the settings `configure`
    /// sets on the builder.
    fn create_service_with<S: Service>(
        node: &Node<S>,
        name: &ServiceName,
        configure: impl FnOnce(
            Builder<Request<Self>, (), Response<Self>, (), S>,
        ) -> Builder<Request<Self>, (), Response<Self>, (), S>,
    ) -> PortFactory<S, Request<Self>, (), Response<Self>, ()> {
        <Self::Request as RequestPayload>::create(configure(
            node.service_builder(name)
                .request_response::<Request<Self>, Response<Self>>(),
        ))
    }

    /// Opens the service `name` on `node`.
    fn open_service<S: Service>(
        node: &Node<S>,
        name: &ServiceName,
    ) -> PortFactory<S, Request<Self>, (), Response<Self>, ()> {
        <Self::Request as RequestPayload>::open(
            node.service_builder(name)
                .request_response::<Request<Self>, Response<Self>>(),
        )
    }

    /// A client on `service`, sized for the values.
    fn create_client<S: Service>(
        service: &PortFactory<S, Request<Self>, (), Response<Self>, ()>,
    ) -> Client<S, Request<Self>, (), Response<Self>, ()> {
        <Self::Request as RequestPayload>::client(service)
    }

    /// A server on `service`, sized for the values.
    fn create_server<S: Service>(
        service: &PortFactory<S, Request<Self>, (), Response<Self>, ()>,
    ) -> Server<S, Request<Self>, (), Response<Self>, ()> {
        <Self::Response as ResponsePayload>::server(service)
    }

    /// Sends `value` as a request.
    fn send_request<S: Service>(
        client: &Client<S, Request<Self>, (), Response<Self>, ()>,
        value: RequestValue<Self>,
    ) -> PendingResponse<S, Request<Self>, (), Response<Self>, ()> {
        <Self::Request as RequestPayload>::send_request(client, value)
    }

    /// The next request pending on `server`, if any.
    fn receive_request<S: Service>(
        server: &Server<S, Request<Self>, (), Response<Self>, ()>,
    ) -> Option<ActiveRequest<S, Request<Self>, (), Response<Self>, ()>> {
        <Self::Request as RequestPayload>::receive_request(server)
    }

    /// The value `request` carries.
    fn request_value<S: Service>(
        request: &ActiveRequest<S, Request<Self>, (), Response<Self>, ()>,
    ) -> RequestValue<Self> {
        <Self::Request as RequestPayload>::request_value(request)
    }

    /// Sends `value` as a response to `request`.
    fn send_response<S: Service>(
        request: &ActiveRequest<S, Request<Self>, (), Response<Self>, ()>,
        value: ResponseValue<Self>,
    ) {
        <Self::Response as ResponsePayload>::send_response(request, value)
    }

    /// The next response pending on `pending`, if any.
    fn receive_response<S: Service>(
        pending: &PendingResponse<S, Request<Self>, (), Response<Self>, ()>,
    ) -> Option<ResponseValue<Self>> {
        <Self::Response as ResponsePayload>::receive_response(pending)
    }
}

/// The request-response service named by `N`, of request payload `Q` and
/// response payload `R`.
#[derive(Debug, Default, Clone, Copy)]
pub struct RequestResponse<N, Q, R>(PhantomData<(N, Q, R)>);

impl<N: RequestResponseName, Q: RequestPayload<Type: Sized>, R: ResponsePayload<Type: Sized>>
    RequestResponseService for RequestResponse<N, Q, R>
{
    type Request = Q;
    type Response = R;
}

impl<N: RequestResponseName, Q: RequestPayload<Type: Sized>, R: ResponsePayload<Type: Sized>>
    AnyService for RequestResponse<N, Q, R>
{
    const PATTERN: MessagingPattern = MessagingPattern::RequestResponse;
    type Handle<S: Service> = PortFactory<S, Request<Self>, (), Response<Self>, ()>;
    type Port<S: Service> = Client<S, Request<Self>, (), Response<Self>, ()>;

    fn service_name() -> ServiceName {
        N::service_name()
    }

    fn describe<S: Service>(name: &ServiceName, config: &Config) -> ServiceDescription {
        describe_request_response::<S, Q, R>(name, config)
    }

    fn create<S: Service>(node: &Node<S>, name: &ServiceName) -> Self::Handle<S> {
        Self::create_service::<S>(node, name)
    }

    fn open<S: Service>(node: &Node<S>, name: &ServiceName) -> Self::Handle<S> {
        Self::open_service::<S>(node, name)
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
