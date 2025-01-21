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

use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_cal::dynamic_storage::DynamicStorage;

use crate::{
    node::NodeListFailure,
    prelude::AttributeSet,
    service::{
        self, dynamic_config, service_id::ServiceId, service_name::ServiceName, static_config,
    },
};

use super::nodes;

#[derive(Debug)]
pub struct PortFactory<Service: service::Service> {
    pub(crate) service: Service,
}

unsafe impl<Service: service::Service> Send for PortFactory<Service> {}
unsafe impl<Service: service::Service> Sync for PortFactory<Service> {}

impl<Service: service::Service> crate::service::port_factory::PortFactory for PortFactory<Service> {
    type Service = Service;
    type StaticConfig = static_config::request_response::StaticConfig;
    type DynamicConfig = dynamic_config::request_response::DynamicConfig;

    fn name(&self) -> &ServiceName {
        self.service.__internal_state().static_config.name()
    }

    fn service_id(&self) -> &ServiceId {
        self.service.__internal_state().static_config.service_id()
    }

    fn attributes(&self) -> &AttributeSet {
        self.service.__internal_state().static_config.attributes()
    }

    fn static_config(&self) -> &Self::StaticConfig {
        self.service
            .__internal_state()
            .static_config
            .request_response()
    }

    fn dynamic_config(&self) -> &Self::DynamicConfig {
        self.service
            .__internal_state()
            .dynamic_storage
            .get()
            .request_response()
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
