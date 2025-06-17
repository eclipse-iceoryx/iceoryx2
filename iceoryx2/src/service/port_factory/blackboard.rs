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

use crate::node::NodeListFailure;
use crate::service::attribute::AttributeSet;
use crate::service::service_id::ServiceId;
use crate::service::service_name::ServiceName;
use crate::service::{self, dynamic_config, static_config};
use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_cal::dynamic_storage::DynamicStorage;

use super::nodes;

#[derive(Debug)]
pub struct PortFactory<Service: service::Service> {
    pub(crate) service: Service,
}

impl<Service: service::Service> crate::service::port_factory::PortFactory for PortFactory<Service> {
    type Service = Service;
    type StaticConfig = static_config::blackboard::StaticConfig;
    type DynamicConfig = dynamic_config::blackboard::DynamicConfig;

    fn name(&self) -> &ServiceName {
        self.service.__internal_state().static_config.name()
    }

    fn service_id(&self) -> &ServiceId {
        self.service.__internal_state().static_config.service_id()
    }

    fn attributes(&self) -> &AttributeSet {
        self.service.__internal_state().static_config.attributes()
    }

    fn static_config(&self) -> &static_config::blackboard::StaticConfig {
        self.service.__internal_state().static_config.blackboard()
    }

    fn dynamic_config(&self) -> &dynamic_config::blackboard::DynamicConfig {
        self.service
            .__internal_state()
            .dynamic_storage
            .get()
            .blackboard()
    }

    fn nodes<F: FnMut(crate::node::NodeState<Service>) -> CallbackProgression>(
        &self,
        callback: F,
    ) -> Result<(), NodeListFailure> {
        nodes(
            self.service.__internal_state().dynamic_storage.get(),
            self.service.__internal_state().shared_node.config(),
            callback,
        )
    }
}

impl<Service: service::Service> PortFactory<Service> {
    pub(crate) fn new(service: Service) -> Self {
        Self { service }
    }
}
