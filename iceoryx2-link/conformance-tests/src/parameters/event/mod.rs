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

use alloc::vec::Vec;
use core::marker::PhantomData;

use iceoryx2::config::Config;
use iceoryx2::node::Node;
use iceoryx2::port::event_id::EventId;
use iceoryx2::port::listener::Listener;
use iceoryx2::port::notifier::Notifier;
use iceoryx2::service::Service;
use iceoryx2::service::messaging_pattern::MessagingPattern;
use iceoryx2::service::port_factory::event::PortFactory;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2::testing::generate_service_name;
use iceoryx2_link_backend::description::{
    EventSettings, PatternSettings, ServiceDescription, ServiceSettings, ServiceTypes,
};

use super::{AnyName, AnyService};
use crate::testing::notifications_of;

/// A source of names for event services.
pub trait EventName {
    /// A fresh service name.
    fn service_name() -> ServiceName;
}

impl EventName for AnyName {
    fn service_name() -> ServiceName {
        generate_service_name()
    }
}

/// An event service the suites run over.
pub trait EventService: AnyService {
    /// Creates the service `name` on `node`.
    fn create_service<S: Service>(node: &Node<S>, name: &ServiceName) -> PortFactory<S> {
        node.service_builder(name)
            .event()
            .create()
            .expect("service is created")
    }

    /// Opens the service `name` on `node`.
    fn open_service<S: Service>(node: &Node<S>, name: &ServiceName) -> PortFactory<S> {
        node.service_builder(name)
            .event()
            .open()
            .expect("service is opened")
    }

    /// A notifier on `service`.
    fn create_notifier<S: Service>(service: &PortFactory<S>) -> Notifier<S> {
        service
            .notifier_builder()
            .create()
            .expect("notifier is created")
    }

    /// A listener on `service`.
    fn create_listener<S: Service>(service: &PortFactory<S>) -> Listener<S> {
        service
            .listener_builder()
            .create()
            .expect("listener is created")
    }

    /// Sends the notification `id` through `notifier`.
    fn notify<S: Service>(notifier: &Notifier<S>, id: EventId) {
        notifier
            .notify_with_custom_event_id(id)
            .expect("notification is sent");
    }

    /// The ids of the notifications pending on `listener`.
    fn notifications<S: Service>(listener: &Listener<S>) -> Vec<EventId> {
        notifications_of(listener)
    }
}

/// The event service named by `N`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Event<N>(PhantomData<N>);

impl<N: EventName> EventService for Event<N> {}

impl<N: EventName> AnyService for Event<N> {
    const PATTERN: MessagingPattern = MessagingPattern::Event;
    type Handle<S: Service> = PortFactory<S>;
    type Port<S: Service> = Notifier<S>;

    fn service_name() -> ServiceName {
        N::service_name()
    }

    fn describe<S: Service>(name: &ServiceName, config: &Config) -> ServiceDescription {
        ServiceDescription::compose::<S>(
            ServiceSettings::new(
                *name,
                PatternSettings::Event(EventSettings::from_config(config)),
            ),
            ServiceTypes::Event,
        )
        .expect("halves of one pattern")
    }

    fn create<S: Service>(node: &Node<S>, name: &ServiceName) -> Self::Handle<S> {
        Self::create_service::<S>(node, name)
    }

    fn open<S: Service>(node: &Node<S>, name: &ServiceName) -> Self::Handle<S> {
        Self::open_service::<S>(node, name)
    }

    fn create_port<S: Service>(service: &Self::Handle<S>) -> Self::Port<S> {
        Self::create_notifier(service)
    }

    fn occupy_ports<S: Service>(service: &Self::Handle<S>, config: &Config) -> Vec<Self::Port<S>> {
        (0..config.defaults.event.max_notifiers)
            .map(|_| Self::create_port(service))
            .collect()
    }
}
