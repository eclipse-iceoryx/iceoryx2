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
use alloc::string::String;
use alloc::vec::Vec;
use core::time::Duration;

use iceoryx2::node::{Node, NodeBuilder};
use iceoryx2::port::listener::Listener;
use iceoryx2::service::Service;
use iceoryx2::service::local;
use iceoryx2::service::messaging_pattern::MessagingPattern;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2::testing::generate_isolated_config;
use iceoryx2_bb_posix::adaptive_wait::AdaptiveWaitBuilder;

use iceoryx2_link_backend::description::{
    PublishSubscribeTypes, ServiceDescriptor, ServiceTypes, TypeDescription,
};
use iceoryx2_link_backend::{WakeHandle, WakeService};

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
    ServiceTypes::PublishSubscribe(PublishSubscribeTypes {
        payload: TypeDescription {
            variant: TypeVariant::FixedSize,
            type_name: String::from(payload),
            size: 8,
            alignment: 8,
        },
        user_header: TypeDescription::from(&TypeDetail::new::<()>(TypeVariant::FixedSize)),
    })
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
