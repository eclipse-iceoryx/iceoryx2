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

use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use pyo3::prelude::*;

use crate::{
    error::SubscriberCreateError,
    parc::Parc,
    port_factory_publish_subscribe::PortFactoryPublishSubscribeType,
    subscriber::{Subscriber, SubscriberType},
};

type IpcPortFactorySubscriber<'a> =
    iceoryx2::service::port_factory::subscriber::PortFactorySubscriber<
        'a,
        crate::IpcService,
        [CustomPayloadMarker],
        CustomHeaderMarker,
    >;
type LocalPortFactorySubscriber<'a> =
    iceoryx2::service::port_factory::subscriber::PortFactorySubscriber<
        'a,
        crate::LocalService,
        [CustomPayloadMarker],
        CustomHeaderMarker,
    >;

pub(crate) enum PortFactorySubscriberType {
    Ipc(Parc<IpcPortFactorySubscriber<'static>>),
    Local(Parc<LocalPortFactorySubscriber<'static>>),
}

#[pyclass]
/// Factory to create a new `Subscriber` port/endpoint for
/// `MessagingPattern::PublishSubscribe` based communication.
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
                    Parc::new(core::mem::transmute::<
                        IpcPortFactorySubscriber<'_>,
                        IpcPortFactorySubscriber<'static>,
                    >(v.subscriber_builder()))
                }),
                PortFactoryPublishSubscribeType::Local(v) => {
                    PortFactorySubscriberType::Local(unsafe {
                        Parc::new(core::mem::transmute::<
                            LocalPortFactorySubscriber<'_>,
                            LocalPortFactorySubscriber<'static>,
                        >(v.subscriber_builder()))
                    })
                }
            },
        }
    }
}

#[pymethods]
impl PortFactorySubscriber {
    /// Defines the buffer size of the `Subscriber`. Smallest possible value is `1`.
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
                Ok(Subscriber(Parc::new(SubscriberType::Ipc(Some(
                    this.create()
                        .map_err(|e| SubscriberCreateError::new_err(format!("{e:?}")))?,
                )))))
            }
            PortFactorySubscriberType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                Ok(Subscriber(Parc::new(SubscriberType::Local(Some(
                    this.create()
                        .map_err(|e| SubscriberCreateError::new_err(format!("{e:?}")))?,
                )))))
            }
        }
    }
}
