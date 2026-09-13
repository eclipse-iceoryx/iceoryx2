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
mod slice;

use alloc::vec::Vec;
use core::fmt::Debug;
use core::marker::PhantomData;

use iceoryx2::config::Config;
use iceoryx2::node::Node;
use iceoryx2::port::publisher::Publisher;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::prelude::ZeroCopySend;
use iceoryx2::service::Service;
use iceoryx2::service::builder::publish_subscribe::Builder;
use iceoryx2::service::messaging_pattern::MessagingPattern;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2::testing::generate_service_name;
use iceoryx2_bb_elementary_traits::type_name::TypeName;
use iceoryx2_link_backend::description::ServiceDescription;

use super::{AnyName, AnyService, FixedSizePayload, PayloadShape};
use crate::testing::describe;

/// The payload type the services of `X` are created with.
pub type Payload<X> = <<X as PublishSubscribeService>::Payload as PayloadShape>::Type;
/// The user header type the services of `X` are created with.
pub type Header<X> = <X as PublishSubscribeService>::Header;
/// A payload value of the services of `X` in the suites' hands.
pub type Value<X> = <<X as PublishSubscribeService>::Payload as PayloadShape>::Value;

/// A source of names for publish-subscribe services.
pub trait PublishSubscribeName {
    /// A fresh service name.
    fn service_name() -> ServiceName;
}

impl PublishSubscribeName for AnyName {
    fn service_name() -> ServiceName {
        generate_service_name()
    }
}

/// A user header the suites' services carry.
pub trait UserHeader:
    TypeName + ZeroCopySend + Debug + Copy + PartialEq + Default + 'static
{
}

impl<T: TypeName + ZeroCopySend + Debug + Copy + PartialEq + Default + 'static> UserHeader for T {}

/// A payload shape on a publish-subscribe service, how the service is
/// built and how a value is sent and read back.
pub trait PublishSubscribePayload: PayloadShape {
    /// Creates the service `builder` describes.
    fn create<S: Service, H: ZeroCopySend + Debug>(
        builder: Builder<Self::Type, H, S>,
    ) -> PortFactory<S, Self::Type, H>;

    /// Opens the service `builder` describes.
    fn open<S: Service, H: ZeroCopySend + Debug>(
        builder: Builder<Self::Type, H, S>,
    ) -> PortFactory<S, Self::Type, H>;

    /// A publisher on `service`, sized for the values.
    fn publisher<S: Service, H: ZeroCopySend + Debug + Default>(
        service: &PortFactory<S, Self::Type, H>,
    ) -> Publisher<S, Self::Type, H>;

    /// Sends `value` under `header`.
    fn send<S: Service, H: ZeroCopySend + Debug + Default>(
        publisher: &Publisher<S, Self::Type, H>,
        header: H,
        value: Self::Value,
    );

    /// The next sample pending on `subscriber`, if any, its header and its
    /// value.
    fn receive<S: Service, H: ZeroCopySend + Debug + Copy>(
        subscriber: &Subscriber<S, Self::Type, H>,
    ) -> Option<(H, Self::Value)>;
}

/// A payload type no fixture can share. It is private to this crate, and
/// its type name on the wire is its full path, so a look-alike declared
/// elsewhere still differs by name.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, ZeroCopySend)]
pub(crate) struct ForeignPayload(u32);

impl From<u64> for ForeignPayload {
    fn from(n: u64) -> Self {
        Self(n as u32)
    }
}

/// A publish-subscribe service of a type no fixture shares, for scenarios
/// pitting the fixture's service against one of other types.
pub(crate) type Foreign = PublishSubscribe<AnyName, FixedSizePayload<ForeignPayload>, ()>;

/// A publish-subscribe service the suites run over.
pub trait PublishSubscribeService: AnyService {
    /// The payload of the service.
    type Payload: PublishSubscribePayload;
    /// The user header of the service when it has one.
    type Header: UserHeader;

    /// Creates the service `name` on `node` under the user header `H`.
    fn create_service<H: UserHeader, S: Service>(
        node: &Node<S>,
        name: &ServiceName,
    ) -> PortFactory<S, Payload<Self>, H> {
        Self::create_service_with::<H, S>(node, name, |builder| builder)
    }

    /// Creates the service `name` on `node` under the user header `H`,
    /// with the settings `configure` sets on the builder.
    fn create_service_with<H: UserHeader, S: Service>(
        node: &Node<S>,
        name: &ServiceName,
        configure: impl FnOnce(Builder<Payload<Self>, H, S>) -> Builder<Payload<Self>, H, S>,
    ) -> PortFactory<S, Payload<Self>, H> {
        <Self::Payload as PublishSubscribePayload>::create(configure(
            node.service_builder(name)
                .publish_subscribe::<Payload<Self>>()
                .user_header::<H>(),
        ))
    }

    /// Opens the service `name` on `node` under the user header `H`.
    fn open_service<H: UserHeader, S: Service>(
        node: &Node<S>,
        name: &ServiceName,
    ) -> PortFactory<S, Payload<Self>, H> {
        <Self::Payload as PublishSubscribePayload>::open(
            node.service_builder(name)
                .publish_subscribe::<Payload<Self>>()
                .user_header::<H>(),
        )
    }

    /// A publisher on `service`, sized for the values.
    fn create_publisher<S: Service, H: UserHeader>(
        service: &PortFactory<S, Payload<Self>, H>,
    ) -> Publisher<S, Payload<Self>, H> {
        <Self::Payload as PublishSubscribePayload>::publisher(service)
    }

    /// A subscriber on `service`.
    fn create_subscriber<S: Service, H: UserHeader>(
        service: &PortFactory<S, Payload<Self>, H>,
    ) -> Subscriber<S, Payload<Self>, H> {
        service
            .subscriber_builder()
            .create()
            .expect("subscriber is created")
    }

    /// Sends `value` under `header`.
    fn send<S: Service, H: UserHeader>(
        publisher: &Publisher<S, Payload<Self>, H>,
        header: H,
        value: Value<Self>,
    ) {
        <Self::Payload as PublishSubscribePayload>::send(publisher, header, value)
    }

    /// The next sample pending on `subscriber`, if any, its header and its
    /// value.
    fn receive<S: Service, H: UserHeader>(
        subscriber: &Subscriber<S, Payload<Self>, H>,
    ) -> Option<(H, Value<Self>)> {
        <Self::Payload as PublishSubscribePayload>::receive(subscriber)
    }
}

/// The publish-subscribe service named by `N`, of payload `P` under the
/// user header `H`.
#[derive(Debug, Default, Clone, Copy)]
pub struct PublishSubscribe<N, P, H>(PhantomData<(N, P, H)>);

impl<N: PublishSubscribeName, P: PublishSubscribePayload, H: UserHeader> PublishSubscribeService
    for PublishSubscribe<N, P, H>
{
    type Payload = P;
    type Header = H;
}

impl<N: PublishSubscribeName, P: PublishSubscribePayload, H: UserHeader> AnyService
    for PublishSubscribe<N, P, H>
{
    const PATTERN: MessagingPattern = MessagingPattern::PublishSubscribe;
    type Handle<S: Service> = PortFactory<S, Payload<Self>, Header<Self>>;
    type Port<S: Service> = Publisher<S, Payload<Self>, Header<Self>>;

    fn service_name() -> ServiceName {
        N::service_name()
    }

    fn describe<S: Service>(name: &ServiceName, config: &Config) -> ServiceDescription {
        describe::<S, P, H>(name, config)
    }

    fn create<S: Service>(node: &Node<S>, name: &ServiceName) -> Self::Handle<S> {
        Self::create_service::<Header<Self>, S>(node, name)
    }

    fn open<S: Service>(node: &Node<S>, name: &ServiceName) -> Self::Handle<S> {
        Self::open_service::<Header<Self>, S>(node, name)
    }

    fn create_port<S: Service>(service: &Self::Handle<S>) -> Self::Port<S> {
        Self::create_publisher(service)
    }

    fn occupy_ports<S: Service>(service: &Self::Handle<S>, config: &Config) -> Vec<Self::Port<S>> {
        (0..config.defaults.publish_subscribe.max_publishers)
            .map(|_| Self::create_port(service))
            .collect()
    }
}
