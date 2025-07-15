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

use std::sync::Arc;

use pyo3::prelude::*;

use crate::{
    error::ListenerCreateError,
    listener::{Listener, ListenerType},
    parc::Parc,
    port_factory_event::PortFactoryEventType,
};

#[derive(Clone)]
pub(crate) enum PortFactoryListenerType {
    Ipc(iceoryx2::service::port_factory::listener::PortFactoryListener<'static, crate::IpcService>),
    Local(
        iceoryx2::service::port_factory::listener::PortFactoryListener<
            'static,
            crate::LocalService,
        >,
    ),
}

#[pyclass]
/// Factory to create a new `Listener` port/endpoint for `MessagingPattern::Event` based
/// communication.
pub struct PortFactoryListener {
    // required to hold since the PortFactoryListener has a reference to it and thanks to the
    // garbage collector we do not know how long it will be available
    // also: lifetime parameters are forbidden in pyclass
    factory: Parc<PortFactoryEventType>,
    value: PortFactoryListenerType,
}

impl PortFactoryListener {
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
                    PortFactoryListenerType::Ipc(unsafe { &*v }.listener_builder())
                }
                PortFactoryEventType::Local(v) => {
                    let v: *const iceoryx2::service::port_factory::event::PortFactory<
                        crate::LocalService,
                    > = v;
                    // by converting the factory into a pointer we change the lifetime into 'static
                    // and with the factory reference hold by this object we ensure that it
                    // lifes long enough
                    PortFactoryListenerType::Local(unsafe { &*v }.listener_builder())
                }
            },
        }
    }
}

#[pymethods]
impl PortFactoryListener {
    /// Creates the `Listener` port or emits a `ListenerCreateError` on failure.
    pub fn create(&self) -> PyResult<Listener> {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryListenerType::Ipc(v) => {
                let this = v.clone();
                Ok(Listener(ListenerType::Ipc(Some(Arc::new(
                    this.create()
                        .map_err(|e| ListenerCreateError::new_err(format!("{e:?}")))?,
                )))))
            }
            PortFactoryListenerType::Local(v) => {
                let this = v.clone();
                Ok(Listener(ListenerType::Local(Some(Arc::new(
                    this.create()
                        .map_err(|e| ListenerCreateError::new_err(format!("{e:?}")))?,
                )))))
            }
        }
    }
}
