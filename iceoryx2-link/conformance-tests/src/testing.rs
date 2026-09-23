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

use alloc::collections::BTreeSet;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::time::Duration;

use iceoryx2::config::Config;
use iceoryx2::node::{Node, NodeBuilder};
use iceoryx2::port::event_id::EventId;
use iceoryx2::port::listener::Listener;
use iceoryx2::service::Service;
use iceoryx2::service::local;
use iceoryx2::service::messaging_pattern::MessagingPattern;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern as Pattern;
use iceoryx2::testing::generate_isolated_config;
use iceoryx2_bb_concurrency::cell::Cell;
use iceoryx2_bb_elementary_traits::type_name::TypeName;
use iceoryx2_bb_posix::adaptive_wait::AdaptiveWaitBuilder;
use iceoryx2_link::Link;

use crate::parameters::{PayloadShape, PublishSubscribeService};
use iceoryx2_link_adapter::{LoanError, LoanableSample, Region, UnsupportedLength, WritableSample};
use iceoryx2_link_backend::service_description::{
    PublishSubscribeSettings, SampleTypes, ServiceDescription, ServiceDescriptor, ServiceTypes,
    TypeDescription,
};
use iceoryx2_link_backend::{Backend, WakeHandle, WakeService};

/// Calls `attempt` until it succeeds or `timeout` passes. On timeout the
/// error lists every distinct failure seen.
pub fn retry(
    mut attempt: impl FnMut() -> Result<(), &'static str>,
    timeout: Duration,
) -> Result<(), String> {
    let mut failures = BTreeSet::<&'static str>::new();
    let mut wait = AdaptiveWaitBuilder::new()
        .create()
        .expect("adaptive wait is created");
    let succeeded = wait
        .timed_wait_while(
            || -> Result<bool, ()> {
                match attempt() {
                    Ok(()) => Ok(false),
                    Err(failure) => {
                        failures.insert(failure);
                        Ok(true)
                    }
                }
            },
            timeout,
        )
        .expect("waiting succeeds");
    if succeeded {
        return Ok(());
    }
    failures.insert("timeout exceeded");
    Err(failures.into_iter().collect::<Vec<_>>().join(", "))
}

/// The description of a publish-subscribe service of `Payload` under
/// `Header`, as an application creating it with `config` would.
pub fn describe<S: Service, Payload: PayloadShape, Header: TypeName>(
    name: &ServiceName,
    config: &Config,
) -> ServiceDescription {
    ServiceDescription::compose_publish_subscribe::<S>(
        *name,
        PublishSubscribeSettings::from_config(config),
        SampleTypes {
            payload: TypeDescription::from(&Payload::type_detail()),
            user_header: TypeDescription::from(&TypeDetail::new::<Header>(TypeVariant::FixedSize)),
        },
    )
}

/// The hash of the publish-subscribe service `name`.
pub fn hash(name: &str) -> ServiceHash {
    let name = ServiceName::new(name).expect("valid service name");
    ServiceHash::new::<<local::Service as Service>::ServiceNameHasher>(
        &name,
        MessagingPattern::PublishSubscribe,
    )
}

/// The descriptor of the publish-subscribe service `name` with `payload`
/// as its payload type.
pub fn descriptor(name: &str, payload: &str) -> ServiceDescriptor {
    ServiceDescriptor {
        name: ServiceName::new(name).expect("valid service name"),
        hash: hash(name),
        types: types_of(payload),
    }
}

/// The types of a publish-subscribe service with `payload` as its
/// payload type and no user header.
pub fn types_of(payload: &str) -> ServiceTypes {
    ServiceTypes::PublishSubscribe(SampleTypes {
        payload: TypeDescription {
            variant: TypeVariant::FixedSize,
            type_name: String::from(payload),
            size: 8,
            alignment: 8,
        },
        user_header: TypeDescription::from(&TypeDetail::new::<()>(TypeVariant::FixedSize)),
    })
}

/// A descriptor of the event service `name`.
pub fn event_descriptor(name: &str) -> ServiceDescriptor {
    ServiceDescriptor {
        name: ServiceName::new(name).expect("valid service name"),
        hash: hash(name),
        types: ServiceTypes::Event,
    }
}

/// A descriptor of the service `name` with `payload` as its payload type
/// and a user header of `header_size` bytes.
pub fn descriptor_with_header(name: &str, payload: &str, header_size: usize) -> ServiceDescriptor {
    let ServiceTypes::PublishSubscribe(mut types) = types_of(payload) else {
        unreachable!("types_of describes a publish-subscribe service");
    };
    types.user_header = TypeDescription {
        variant: TypeVariant::FixedSize,
        type_name: String::from("header"),
        size: header_size,
        alignment: 1,
    };
    ServiceDescriptor {
        name: ServiceName::new(name).expect("valid service name"),
        hash: hash(name),
        types: ServiceTypes::PublishSubscribe(types),
    }
}

/// A link filter rejecting only the service `name`, and whether it has
/// rejected it yet, so a scenario knows the link got as far as deciding.
pub fn rejecting(name: ServiceName) -> (impl Fn(&ServiceName) -> bool + 'static, Rc<Cell<bool>>) {
    let rejected = Rc::new(Cell::new(false));
    let filter = {
        let rejected = rejected.clone();
        move |candidate: &ServiceName| {
            let admitted = *candidate != name;
            if !admitted {
                rejected.set(true);
            }
            admitted
        }
    };
    (filter, rejected)
}

/// One side of the boundary, an isolated local system with an
/// application's node and a link over a backend.
pub struct Side<S: Service, B: Backend<S>> {
    pub config: Config,
    pub node: Node<S>,
    pub link: Link<S, B>,
}

/// Creates one side of a link using the provided config.
pub fn side<S: Service, B: Backend<S>>(
    config: Config,
    backend: impl FnOnce(&Config) -> B,
) -> Side<S, B> {
    let node = NodeBuilder::new()
        .config(&config)
        .create::<S>()
        .expect("node is created");
    let link_node = NodeBuilder::new()
        .config(&config)
        .create::<S>()
        .expect("node is created");
    let link = Link::new(link_node, backend(&config));
    Side { config, node, link }
}

/// The hash of the publish-subscribe service `name`, with `S`'s hasher.
pub fn hash_of<S: Service>(name: &ServiceName) -> ServiceHash {
    ServiceHash::new::<S::ServiceNameHasher>(name, MessagingPattern::PublishSubscribe)
}

/// Whether the service `name` of `pattern` exists in the local system
/// `config` configures.
pub fn service_exists<S: Service>(
    name: &ServiceName,
    config: &Config,
    pattern: MessagingPattern,
) -> bool {
    matches!(S::details(name, config, pattern), Ok(Some(_)))
}

/// The ids of the notifications pending on `listener`.
pub fn notifications_of<S: Service>(listener: &Listener<S>) -> Vec<EventId> {
    let mut ids = Vec::new();
    listener
        .try_wait(|activation| ids.push(activation.id))
        .expect("waiting succeeds");
    ids
}

/// The name of the payload type of the services of `X`.
pub fn payload_type_name<X: PublishSubscribeService>() -> String {
    String::from_utf8_lossy(X::Payload::type_detail().type_name()).into_owned()
}

/// The history size of the publish-subscribe service `name` in the local
/// system `config` configures, if it exists there.
pub fn history_size_of<S: Service>(name: &ServiceName, config: &Config) -> Option<usize> {
    let details = S::details(name, config, MessagingPattern::PublishSubscribe).ok()??;
    match details.static_details.messaging_pattern() {
        Pattern::PublishSubscribe(config) => Some(config.history_size()),
        _ => None,
    }
}

/// The payload type name of the publish-subscribe service `name` in the
/// local system `config` configures, if it exists there.
pub fn payload_type_of<S: Service>(name: &ServiceName, config: &Config) -> Option<String> {
    let details = S::details(name, config, MessagingPattern::PublishSubscribe).ok()??;
    match details.static_details.messaging_pattern() {
        Pattern::PublishSubscribe(config) => Some(
            String::from_utf8_lossy(config.message_type_details().payload.type_name()).into_owned(),
        ),
        _ => None,
    }
}

/// A wake service of a suite's own, the wake a source is given and the
/// listener that hears it.
pub struct WakeSource {
    pub wake: WakeHandle,
    pub listener: Listener<WakeService>,
    /// Last, so the ports are dropped before their node.
    _node: Node<WakeService>,
}

impl WakeSource {
    pub fn new() -> Self {
        let node = NodeBuilder::new()
            .config(&generate_isolated_config())
            .create::<WakeService>()
            .expect("node is created");
        let service = node
            .service_builder(&ServiceName::new("link/conformance/wake").expect("valid name"))
            .event()
            .open_or_create()
            .expect("wake service is created");
        let wake = WakeHandle::new(
            service
                .notifier_builder()
                .create()
                .expect("notifier is created"),
        );
        let listener = service
            .listener_builder()
            .create()
            .expect("listener is created");
        Self {
            wake,
            listener,
            _node: node,
        }
    }

    /// Whether the source was signalled since the last call.
    pub fn woken(&self) -> bool {
        let mut woken = false;
        self.listener
            .try_wait(|_| woken = true)
            .expect("waiting succeeds");
        woken
    }
}

impl Default for WakeSource {
    fn default() -> Self {
        Self::new()
    }
}

/// A stand-in for an unloaned sample that loans heap buffers instead of
/// shared memory, so a writer can be tested without a port.
pub struct UnloanedBuffers {
    pub header_size: usize,
}

impl LoanableSample for UnloanedBuffers {
    type Sample = LoanedBuffers;

    fn header_size(&self) -> usize {
        self.header_size
    }

    fn loan(self, payload_len: usize) -> Result<Self::Sample, LoanError> {
        Ok(LoanedBuffers {
            header: Vec::new(),
            payload: vec![0; payload_len],
        })
    }
}

/// Mocks a loan that is refused.
pub struct NotLoanable;

impl LoanableSample for NotLoanable {
    type Sample = LoanedBuffers;

    fn header_size(&self) -> usize {
        0
    }

    fn loan(self, _: usize) -> Result<Self::Sample, LoanError> {
        Err(LoanError::Malformed)
    }
}

/// The heap buffers a writer wrote, to compare against what was sent.
pub struct LoanedBuffers {
    pub header: Vec<u8>,
    pub payload: Vec<u8>,
}

impl WritableSample for LoanedBuffers {
    fn payload(&mut self) -> &mut [u8] {
        &mut self.payload
    }

    fn header(&mut self, len: usize) -> Result<&mut [u8], UnsupportedLength> {
        self.header.for_length(len)
    }
}
