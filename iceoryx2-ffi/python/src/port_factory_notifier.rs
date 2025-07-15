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
    error::NotifierCreateError,
    event_id::EventId,
    notifier::{Notifier, NotifierType},
    parc::Parc,
    port_factory_event::PortFactoryEventType,
};

#[derive(Clone)]
pub(crate) enum PortFactoryNotifierType {
    Ipc(iceoryx2::service::port_factory::notifier::PortFactoryNotifier<'static, crate::IpcService>),
    Local(
        iceoryx2::service::port_factory::notifier::PortFactoryNotifier<
            'static,
            crate::LocalService,
        >,
    ),
}

#[pyclass]
/// Factory to create a new `Notifier` port/endpoint for `MessagingPattern::Event` based
/// communication.
pub struct PortFactoryNotifier {
    // required to hold since the PortFactoryNotifier has a reference to it and thanks to the
    // garbage collector we do not know how long it will be available
    // also: lifetime parameters are forbidden in pyclass
    factory: Parc<PortFactoryEventType>,
    value: PortFactoryNotifierType,
}

impl PortFactoryNotifier {
    pub(crate) fn new(factory: Parc<PortFactoryEventType>) -> Self {
        Self {
            factory: factory.clone(),
            value: match &*factory.lock() {
                PortFactoryEventType::Ipc(v) => {
                    let v: *const iceoryx2::service::port_factory::event::PortFactory<
                        crate::IpcService,
                    > = v;
                    // by converting the factory into a pointer we change the lifetime into 'static
                    // and with the factory reference hold by this object we ensure that it
                    // lifes long enough
                    PortFactoryNotifierType::Ipc(unsafe { &*v }.notifier_builder())
                }
                PortFactoryEventType::Local(v) => {
                    let v: *const iceoryx2::service::port_factory::event::PortFactory<
                        crate::LocalService,
                    > = v;
                    // by converting the factory into a pointer we change the lifetime into 'static
                    // and with the factory reference hold by this object we ensure that it
                    // lifes long enough
                    PortFactoryNotifierType::Local(unsafe { &*v }.notifier_builder())
                }
            },
        }
    }
}

#[pymethods]
impl PortFactoryNotifier {
    /// Creates a new `Notifier` port or emits a `NotifierCreateError` on failure.
    pub fn create(&self) -> PyResult<Notifier> {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryNotifierType::Ipc(v) => {
                let this = v.clone();
                Ok(Notifier(NotifierType::Ipc(Some(this.create().map_err(
                    |e| NotifierCreateError::new_err(format!("{e:?}")),
                )?))))
            }
            PortFactoryNotifierType::Local(v) => {
                let this = v.clone();
                Ok(Notifier(NotifierType::Local(Some(this.create().map_err(
                    |e| NotifierCreateError::new_err(format!("{e:?}")),
                )?))))
            }
        }
    }

    /// Sets a default `EventId` for the `Notifier` that is used in `Notifier.notify()`
    pub fn default_event_id(&self, value: &EventId) -> Self {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryNotifierType::Ipc(v) => {
                let this = v.clone();
                let this = this.default_event_id(value.0);
                Self {
                    value: PortFactoryNotifierType::Ipc(this),
                    factory: self.factory.clone(),
                }
            }
            PortFactoryNotifierType::Local(v) => {
                let this = v.clone();
                let this = this.default_event_id(value.0);
                Self {
                    value: PortFactoryNotifierType::Local(this),
                    factory: self.factory.clone(),
                }
            }
        }
    }
}
