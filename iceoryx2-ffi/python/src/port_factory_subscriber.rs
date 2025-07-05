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

use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use pyo3::prelude::*;

use crate::{
    error::SubscriberCreateError,
    parc::Parc,
    port_factory_publish_subscribe::PortFactoryPublishSubscribeType,
    subscriber::{Subscriber, SubscriberType},
};

pub(crate) enum PortFactorySubscriberType {
    Ipc(
        Parc<
            iceoryx2::service::port_factory::subscriber::PortFactorySubscriber<
                'static,
                crate::IpcService,
                [CustomPayloadMarker],
                CustomHeaderMarker,
            >,
        >,
    ),
    Local(
        Parc<
            iceoryx2::service::port_factory::subscriber::PortFactorySubscriber<
                'static,
                crate::LocalService,
                [CustomPayloadMarker],
                CustomHeaderMarker,
            >,
        >,
    ),
}

#[pyclass]
pub struct PortFactorySubscriber {
    factory: Parc<PortFactoryPublishSubscribeType>,
    value: PortFactorySubscriberType,
}

impl PortFactorySubscriber {
    pub(crate) fn new(factory: Parc<PortFactoryPublishSubscribeType>) -> Self {
        Self {
            factory: factory.clone(),
            value: match &*factory.lock() {
                PortFactoryPublishSubscribeType::Ipc(v) => PortFactorySubscriberType::Ipc(unsafe {
                    Parc::new(core::mem::transmute(v.subscriber_builder()))
                }),
                PortFactoryPublishSubscribeType::Local(v) => {
                    PortFactorySubscriberType::Local(unsafe {
                        Parc::new(core::mem::transmute(v.subscriber_builder()))
                    })
                }
            },
        }
    }
}

#[pymethods]
impl PortFactorySubscriber {
    pub fn buffer_size(&self, value: usize) -> Self {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactorySubscriberType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.buffer_size(value);
                Self {
                    value: PortFactorySubscriberType::Ipc(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
            PortFactorySubscriberType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.buffer_size(value);
                Self {
                    value: PortFactorySubscriberType::Local(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
        }
    }

    /// Creates a new `Subscriber` or emits a `SubscriberCreateError` on failure.
    pub fn create(&self) -> PyResult<Subscriber> {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactorySubscriberType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                Ok(Subscriber(SubscriberType::Ipc(Arc::new(
                    this.create()
                        .map_err(|e| SubscriberCreateError::new_err(format!("{e:?}")))?,
                ))))
            }
            PortFactorySubscriberType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                Ok(Subscriber(SubscriberType::Local(Arc::new(
                    this.create()
                        .map_err(|e| SubscriberCreateError::new_err(format!("{e:?}")))?,
                ))))
            }
        }
    }
}
