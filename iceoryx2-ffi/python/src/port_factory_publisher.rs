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

use crate::{parc::Parc, port_factory_publish_subscribe::PortFactoryPublishSubscribeType};

pub(crate) enum PortFactoryPublisherType {
    Ipc(
        Parc<
            iceoryx2::service::port_factory::publisher::PortFactoryPublisher<
                'static,
                crate::IpcService,
                [CustomPayloadMarker],
                CustomHeaderMarker,
            >,
        >,
    ),
    Local(
        Parc<
            iceoryx2::service::port_factory::publisher::PortFactoryPublisher<
                'static,
                crate::LocalService,
                [CustomPayloadMarker],
                CustomHeaderMarker,
            >,
        >,
    ),
}

#[pyclass]
pub struct PortFactoryPublisher {
    factory: Parc<PortFactoryPublishSubscribeType>,
    value: PortFactoryPublisherType,
}

impl PortFactoryPublisher {
    pub(crate) fn new(factory: Parc<PortFactoryPublishSubscribeType>) -> Self {
        Self {
            factory: factory.clone(),
            value: match &*factory.lock() {
                PortFactoryPublishSubscribeType::Ipc(v) => PortFactoryPublisherType::Ipc(unsafe {
                    Parc::new(core::mem::transmute(v.publisher_builder()))
                }),
                PortFactoryPublishSubscribeType::Local(v) => {
                    PortFactoryPublisherType::Local(unsafe {
                        Parc::new(core::mem::transmute(v.publisher_builder()))
                    })
                }
            },
        }
    }
}
