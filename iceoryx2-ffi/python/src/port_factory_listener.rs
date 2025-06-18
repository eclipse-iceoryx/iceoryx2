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
    listener::{Listener, ListenerType},
    parc::Parc,
    port_factory_event::PortFactoryEventType,
};

#[derive(Clone)]
pub(crate) enum PortFactoryListenerType {
    Ipc(iceoryx2::service::port_factory::listener::PortFactoryListener<'static, ipc::Service>),
    Local(iceoryx2::service::port_factory::listener::PortFactoryListener<'static, local::Service>),
}

#[pyclass]
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
                        ipc::Service,
                    > = v;
                    // by converting the factory into a pointer we change the lifetime into 'static
                    // and with the factory reference hold by this object we ensure that it
                    // lifes long enough
                    PortFactoryListenerType::Ipc(unsafe { &*v }.listener_builder())
                }
                PortFactoryEventType::Local(v) => {
                    let v: *const iceoryx2::service::port_factory::event::PortFactory<
                        local::Service,
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
    pub fn create(&self) -> PyResult<Listener> {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryListenerType::Ipc(v) => {
                let this = v.clone();
                Ok(Listener(ListenerType::Ipc(this.create().unwrap())))
            }
            PortFactoryListenerType::Local(v) => {
                let this = v.clone();
                Ok(Listener(ListenerType::Local(this.create().unwrap())))
            }
        }
    }
}
