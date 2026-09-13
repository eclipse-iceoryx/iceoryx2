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
use iceoryx2::config::Config;
use iceoryx2::identifiers::UniqueNodeId;
use iceoryx2::node::{Node, NodeBuilder};
use iceoryx2::port::listener::Listener;
use iceoryx2::service::port_factory::event::PortFactory;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2_link_backend::origin;
use iceoryx2_link_backend::{WakeHandle, WakeService};
use iceoryx2_log::fail;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum WakeCreationError {
    Node,
    Service,
    Notifier,
    Listener,
}

impl core::fmt::Display for WakeCreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "WakeCreationError::{self:?}")
    }
}

impl core::error::Error for WakeCreationError {}

/// The wake service of a reactive link. On a node of its own, since the
/// service must be of a thread-safe variant and the link's node is not.
pub(crate) struct Wake {
    service: PortFactory<WakeService>,
    // Placed last on purpose, required for clean-up order.
    _node: Node<WakeService>,
}

impl Wake {
    /// Creates the wake service of the link running as `link`, named after
    /// it, within `config`.
    pub(crate) fn create(config: &Config, link: &UniqueNodeId) -> Result<Self, WakeCreationError> {
        let origin = origin!("Wake::create");
        let node = fail!(
            from origin,
            when NodeBuilder::new().config(config).create::<WakeService>(),
            with WakeCreationError::Node,
            "Failed to create the node of the wake service"
        );
        let service = fail!(
            from origin,
            when node.service_builder(&wake_service_name(link)).event().create(),
            with WakeCreationError::Service,
            "Failed to create the wake service"
        );
        Ok(Self {
            service,
            _node: node,
        })
    }

    /// A handle signalling the service.
    pub(crate) fn handle(&self) -> Result<WakeHandle, WakeCreationError> {
        let origin = origin!("Wake::handle");

        let notifier = fail!(
            from origin,
            when self.service.notifier_builder().create(),
            with WakeCreationError::Notifier,
            "Failed to create the wake service's notifier"
        );
        Ok(WakeHandle::new(notifier))
    }

    /// A listener on the service.
    pub(crate) fn listener(&self) -> Result<Listener<WakeService>, WakeCreationError> {
        let origin = origin!("Wake::listener");

        let listener = fail!(
            from origin,
            when self.service.listener_builder().create(),
            with WakeCreationError::Listener,
            "Failed to create the wake service's listener"
        );
        Ok(listener)
    }
}

/// The name of the wake service of the link running as `link`.
fn wake_service_name(link: &UniqueNodeId) -> ServiceName {
    ServiceName::new(&alloc::format!("link/wake/{link}")).expect("a node id fits a service name")
}
