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

use super::nodes;
use super::writer::PortFactoryWriter;
use crate::node::NodeListFailure;
use crate::service::attribute::AttributeSet;
use crate::service::service_id::ServiceId;
use crate::service::service_name::ServiceName;
use crate::service::{self, dynamic_config, static_config, ServiceState};
use core::fmt::Debug;
use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_cal::dynamic_storage::DynamicStorage;

extern crate alloc;
use alloc::sync::Arc;

/// The factory for
/// [`MessagingPattern::Blackboard`](crate::service::messaging_pattern::MessagingPattern::Blackboard).
/// It can acquire dynamic and static service informations and create
/// [`crate::port::reader::Reader`] or [`crate::port::writer::Writer`] ports.
#[derive(Debug)]
pub struct PortFactory<Service: service::Service, T: Send + Sync + Debug + 'static> {
    pub(crate) service: Arc<ServiceState<Service>>,
    pub(crate) mgmt: Service::BlackboardMgmt<T>,
}

impl<Service: service::Service, T: Send + Sync + Debug + 'static>
    crate::service::port_factory::PortFactory for PortFactory<Service, T>
{
    type Service = Service;
    type StaticConfig = static_config::blackboard::StaticConfig;
    type DynamicConfig = dynamic_config::blackboard::DynamicConfig;

    fn name(&self) -> &ServiceName {
        //println!("service has ownership: {}", self.mgmt.has_ownership());
        self.service.static_config.name()
    }

    fn service_id(&self) -> &ServiceId {
        self.service.static_config.service_id()
    }

    fn attributes(&self) -> &AttributeSet {
        self.service.static_config.attributes()
    }

    fn static_config(&self) -> &static_config::blackboard::StaticConfig {
        self.service.static_config.blackboard()
    }

    fn dynamic_config(&self) -> &dynamic_config::blackboard::DynamicConfig {
        self.service.dynamic_storage.get().blackboard()
    }

    fn nodes<F: FnMut(crate::node::NodeState<Service>) -> CallbackProgression>(
        &self,
        callback: F,
    ) -> Result<(), NodeListFailure> {
        nodes(
            self.service.dynamic_storage.get(),
            self.service.shared_node.config(),
            callback,
        )
    }
}

impl<Service: service::Service, T: Send + Sync + Debug + 'static> PortFactory<Service, T> {
    pub(crate) fn new(service: ServiceState<Service>, mgmt: Service::BlackboardMgmt<T>) -> Self {
        Self {
            service: Arc::new(service),
            mgmt,
        }
    }

    pub fn writer_builder(&self) -> PortFactoryWriter<Service, T> {
        println!("WriterBuilder");
        PortFactoryWriter::new(self)
    }
}
