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

use iceoryx2::prelude::{ipc, local};
use pyo3::prelude::*;

use crate::{
    attribute_specifier::AttributeSpecifier,
    attribute_verifier::AttributeVerifier,
    duration::Duration,
    error::{EventCreateError, EventOpenError, EventOpenOrCreateError},
    event_id::EventId,
    port_factory_event::{PortFactoryEvent, PortFactoryEventType},
};

pub(crate) enum ServiceBuilderEventType {
    Ipc(iceoryx2::service::builder::event::Builder<ipc::Service>),
    Local(iceoryx2::service::builder::event::Builder<local::Service>),
}

#[pyclass]
pub struct ServiceBuilderEvent(pub(crate) ServiceBuilderEventType);

#[pymethods]
impl ServiceBuilderEvent {
    pub fn deadline(&self, deadline: &Duration) -> Self {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                let this = this.deadline(deadline.0);
                ServiceBuilderEvent(ServiceBuilderEventType::Ipc(this))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                let this = this.deadline(deadline.0);
                ServiceBuilderEvent(ServiceBuilderEventType::Local(this))
            }
        }
    }

    pub fn disable_deadline(&self) -> Self {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                let this = this.disable_deadline();
                ServiceBuilderEvent(ServiceBuilderEventType::Ipc(this))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                let this = this.disable_deadline();
                ServiceBuilderEvent(ServiceBuilderEventType::Local(this))
            }
        }
    }

    pub fn max_nodes(&self, value: usize) -> Self {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_nodes(value);
                ServiceBuilderEvent(ServiceBuilderEventType::Ipc(this))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                let this = this.max_nodes(value);
                ServiceBuilderEvent(ServiceBuilderEventType::Local(this))
            }
        }
    }

    pub fn event_id_max_value(&self, value: usize) -> Self {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                let this = this.event_id_max_value(value);
                ServiceBuilderEvent(ServiceBuilderEventType::Ipc(this))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                let this = this.event_id_max_value(value);
                ServiceBuilderEvent(ServiceBuilderEventType::Local(this))
            }
        }
    }

    pub fn max_notifiers(&self, value: usize) -> Self {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_notifiers(value);
                ServiceBuilderEvent(ServiceBuilderEventType::Ipc(this))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                let this = this.max_notifiers(value);
                ServiceBuilderEvent(ServiceBuilderEventType::Local(this))
            }
        }
    }

    pub fn max_listeners(&self, value: usize) -> Self {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_listeners(value);
                ServiceBuilderEvent(ServiceBuilderEventType::Ipc(this))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                let this = this.max_listeners(value);
                ServiceBuilderEvent(ServiceBuilderEventType::Local(this))
            }
        }
    }

    pub fn notifier_created_event(&self, value: &EventId) -> Self {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                let this = this.notifier_created_event(value.0);
                ServiceBuilderEvent(ServiceBuilderEventType::Ipc(this))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                let this = this.notifier_created_event(value.0);
                ServiceBuilderEvent(ServiceBuilderEventType::Local(this))
            }
        }
    }

    pub fn disable_notifier_created_event(&self) -> Self {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                let this = this.disable_notifier_created_event();
                ServiceBuilderEvent(ServiceBuilderEventType::Ipc(this))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                let this = this.disable_notifier_created_event();
                ServiceBuilderEvent(ServiceBuilderEventType::Local(this))
            }
        }
    }

    pub fn notifier_dropped_event(&self, value: &EventId) -> Self {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                let this = this.notifier_dropped_event(value.0);
                ServiceBuilderEvent(ServiceBuilderEventType::Ipc(this))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                let this = this.notifier_dropped_event(value.0);
                ServiceBuilderEvent(ServiceBuilderEventType::Local(this))
            }
        }
    }

    pub fn disable_notifier_dropped_event(&self) -> Self {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                let this = this.disable_notifier_dropped_event();
                ServiceBuilderEvent(ServiceBuilderEventType::Ipc(this))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                let this = this.disable_notifier_dropped_event();
                ServiceBuilderEvent(ServiceBuilderEventType::Local(this))
            }
        }
    }

    pub fn notifier_dead_event(&self, value: &EventId) -> Self {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                let this = this.notifier_dead_event(value.0);
                ServiceBuilderEvent(ServiceBuilderEventType::Ipc(this))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                let this = this.notifier_dead_event(value.0);
                ServiceBuilderEvent(ServiceBuilderEventType::Local(this))
            }
        }
    }

    pub fn disable_notifier_dead_event(&self) -> Self {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                let this = this.disable_notifier_dead_event();
                ServiceBuilderEvent(ServiceBuilderEventType::Ipc(this))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                let this = this.disable_notifier_dead_event();
                ServiceBuilderEvent(ServiceBuilderEventType::Local(this))
            }
        }
    }

    pub fn open_or_create(&self) -> PyResult<PortFactoryEvent> {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(PortFactoryEventType::Ipc(
                    this.open_or_create()
                        .map_err(|e| EventOpenOrCreateError::new_err(format!("{:?}", e)))?,
                )))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(PortFactoryEventType::Local(
                    this.open_or_create()
                        .map_err(|e| EventOpenOrCreateError::new_err(format!("{:?}", e)))?,
                )))
            }
        }
    }

    pub fn open_or_create_with_attributes(
        &self,
        verifier: &AttributeVerifier,
    ) -> PyResult<PortFactoryEvent> {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(PortFactoryEventType::Ipc(
                    this.open_or_create_with_attributes(&verifier.0)
                        .map_err(|e| EventOpenOrCreateError::new_err(format!("{:?}", e)))?,
                )))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(PortFactoryEventType::Local(
                    this.open_or_create_with_attributes(&verifier.0)
                        .map_err(|e| EventOpenOrCreateError::new_err(format!("{:?}", e)))?,
                )))
            }
        }
    }

    pub fn open(&self) -> PyResult<PortFactoryEvent> {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(PortFactoryEventType::Ipc(
                    this.open()
                        .map_err(|e| EventOpenError::new_err(format!("{:?}", e)))?,
                )))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(PortFactoryEventType::Local(
                    this.open()
                        .map_err(|e| EventOpenError::new_err(format!("{:?}", e)))?,
                )))
            }
        }
    }

    pub fn open_with_attributes(&self, verifier: &AttributeVerifier) -> PyResult<PortFactoryEvent> {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(PortFactoryEventType::Ipc(
                    this.open_with_attributes(&verifier.0)
                        .map_err(|e| EventOpenError::new_err(format!("{:?}", e)))?,
                )))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(PortFactoryEventType::Local(
                    this.open_with_attributes(&verifier.0)
                        .map_err(|e| EventOpenError::new_err(format!("{:?}", e)))?,
                )))
            }
        }
    }

    pub fn create(&self) -> PyResult<PortFactoryEvent> {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(PortFactoryEventType::Ipc(
                    this.create()
                        .map_err(|e| EventCreateError::new_err(format!("{:?}", e)))?,
                )))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(PortFactoryEventType::Local(
                    this.create()
                        .map_err(|e| EventCreateError::new_err(format!("{:?}", e)))?,
                )))
            }
        }
    }

    pub fn create_with_attributes(
        &self,
        attributes: &AttributeSpecifier,
    ) -> PyResult<PortFactoryEvent> {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(PortFactoryEventType::Ipc(
                    this.create_with_attributes(&attributes.0)
                        .map_err(|e| EventCreateError::new_err(format!("{:?}", e)))?,
                )))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(PortFactoryEventType::Local(
                    this.create_with_attributes(&attributes.0)
                        .map_err(|e| EventCreateError::new_err(format!("{:?}", e)))?,
                )))
            }
        }
    }
}
