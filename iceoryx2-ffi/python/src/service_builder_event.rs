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

use pyo3::prelude::*;

use crate::{
    attribute_specifier::AttributeSpecifier,
    attribute_verifier::AttributeVerifier,
    duration::Duration,
    error::{EventCreateError, EventOpenError, EventOpenOrCreateError},
    event_id::EventId,
    parc::Parc,
    port_factory_event::{PortFactoryEvent, PortFactoryEventType},
};

pub(crate) enum ServiceBuilderEventType {
    Ipc(iceoryx2::service::builder::event::Builder<crate::IpcService>),
    Local(iceoryx2::service::builder::event::Builder<crate::LocalService>),
}

#[pyclass]
/// Builder to create new `MessagingPattern::Event` based `Service`s
pub struct ServiceBuilderEvent(pub(crate) ServiceBuilderEventType);

#[pymethods]
impl ServiceBuilderEvent {
    /// Enables the deadline property of the service. There must be a notification emitted by any
    /// `Notifier` after at least the provided `deadline`.
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

    /// Disables the deadline property of the service. `Notifier` can signal notifications at any
    /// rate.
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

    /// If the `Service` is created it defines how many `Node`s shall be able to open it in
    /// parallel. If an existing `Service` is opened it defines how many `Node`s must be at least
    /// supported.
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

    /// If the `Service` is created it set the greatest supported `NodeId` value
    /// If an existing `Service` is opened it defines the value size the `NodeId`
    /// must at least support.
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

    /// If the `Service` is created it defines how many `Notifier` shall be supported at most. If
    /// an existing `Service` is opened it defines how many `Notifier` must be at least supported.
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

    /// If the `Service` is created it defines how many `Listener` shall be supported at most. If
    /// an existing `Service` is opened it defines how many `Listener` must be at least supported.
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

    /// If the `Service` is created it defines the event that shall be emitted by every newly
    /// created `Notifier`.
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

    /// If the `Service` is created it disables the event that shall be emitted by every newly
    /// created `Notifier`.
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

    /// If the `Service` is created it defines the event that shall be emitted by every
    /// `Notifier` before it is dropped.
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

    /// If the `Service` is created it disables the event that shall be emitted by every
    /// `Notifier` before it is dropped.
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

    /// If the `Service` is created it defines the event that shall be emitted when a
    /// `Notifier` is identified as dead.
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

    /// If the `Service` is created it disables the event that shall be emitted when a
    /// `Notifier` is identified as dead.
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

    /// If the `Service` exists, it will be opened otherwise a new `Service` will be
    /// created. On failure it emits an `EventOpenOrCreateError`
    pub fn open_or_create(&self) -> PyResult<PortFactoryEvent> {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(Parc::new(PortFactoryEventType::Ipc(
                    this.open_or_create()
                        .map_err(|e| EventOpenOrCreateError::new_err(format!("{e:?}")))?,
                ))))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(Parc::new(PortFactoryEventType::Local(
                    this.open_or_create()
                        .map_err(|e| EventOpenOrCreateError::new_err(format!("{e:?}")))?,
                ))))
            }
        }
    }

    /// If the `Service` exists, it will be opened otherwise a new `Service` will be
    /// created. It defines a set of attributes. If the `Service` already exists all attribute
    /// requirements must be satisfied otherwise the open process will fail. If the `Service`
    /// does not exist the required attributes will be defined in the `Service`.
    /// Emits and `EventOpenOrCreateError` on failure.
    pub fn open_or_create_with_attributes(
        &self,
        verifier: &AttributeVerifier,
    ) -> PyResult<PortFactoryEvent> {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(Parc::new(PortFactoryEventType::Ipc(
                    this.open_or_create_with_attributes(&verifier.0)
                        .map_err(|e| EventOpenOrCreateError::new_err(format!("{e:?}")))?,
                ))))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(Parc::new(PortFactoryEventType::Local(
                    this.open_or_create_with_attributes(&verifier.0)
                        .map_err(|e| EventOpenOrCreateError::new_err(format!("{e:?}")))?,
                ))))
            }
        }
    }

    /// Opens an existing `Service`. Emits an `EventOpenError` on failure.
    pub fn open(&self) -> PyResult<PortFactoryEvent> {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(Parc::new(PortFactoryEventType::Ipc(
                    this.open()
                        .map_err(|e| EventOpenError::new_err(format!("{e:?}")))?,
                ))))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(Parc::new(PortFactoryEventType::Local(
                    this.open()
                        .map_err(|e| EventOpenError::new_err(format!("{e:?}")))?,
                ))))
            }
        }
    }

    /// Opens an existing `Service` with attribute requirements. If the defined attribute
    /// requirements are not satisfied the open process will fail. Emits an `EventOpenError`
    /// on failure.
    pub fn open_with_attributes(&self, verifier: &AttributeVerifier) -> PyResult<PortFactoryEvent> {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(Parc::new(PortFactoryEventType::Ipc(
                    this.open_with_attributes(&verifier.0)
                        .map_err(|e| EventOpenError::new_err(format!("{e:?}")))?,
                ))))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(Parc::new(PortFactoryEventType::Local(
                    this.open_with_attributes(&verifier.0)
                        .map_err(|e| EventOpenError::new_err(format!("{e:?}")))?,
                ))))
            }
        }
    }

    /// Creates a new `Service`.
    pub fn create(&self) -> PyResult<PortFactoryEvent> {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(Parc::new(PortFactoryEventType::Ipc(
                    this.create()
                        .map_err(|e| EventCreateError::new_err(format!("{e:?}")))?,
                ))))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(Parc::new(PortFactoryEventType::Local(
                    this.create()
                        .map_err(|e| EventCreateError::new_err(format!("{e:?}")))?,
                ))))
            }
        }
    }

    /// Creates a new `Service` with a set of attributes.
    pub fn create_with_attributes(
        &self,
        attributes: &AttributeSpecifier,
    ) -> PyResult<PortFactoryEvent> {
        match &self.0 {
            ServiceBuilderEventType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(Parc::new(PortFactoryEventType::Ipc(
                    this.create_with_attributes(&attributes.0)
                        .map_err(|e| EventCreateError::new_err(format!("{e:?}")))?,
                ))))
            }
            ServiceBuilderEventType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryEvent(Parc::new(PortFactoryEventType::Local(
                    this.create_with_attributes(&attributes.0)
                        .map_err(|e| EventCreateError::new_err(format!("{e:?}")))?,
                ))))
            }
        }
    }
}
