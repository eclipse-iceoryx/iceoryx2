// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use iceoryx2::prelude::PortFactory;
use pyo3::prelude::*;

use crate::parc::Parc;
use crate::port_factory_publish_subscribe::PortFactoryPublishSubscribeType;

#[pyclass]
/// The dynamic configuration of a `MessagingPattern::PublishSubscribe` service.
/// Port counts reflect the current state of the service when accessed.
/// The view keeps the service open while it is referenced.
pub struct DynamicConfigPublishSubscribe(pub(crate) Parc<PortFactoryPublishSubscribeType>);

#[pymethods]
impl DynamicConfigPublishSubscribe {
    #[getter]
    /// Returns the number of publishers currently connected to the service.
    pub fn number_of_publishers(&self) -> usize {
        match &*self.0.lock() {
            PortFactoryPublishSubscribeType::Ipc(v) => v.dynamic_config().number_of_publishers(),
            PortFactoryPublishSubscribeType::Local(v) => v.dynamic_config().number_of_publishers(),
        }
    }

    #[getter]
    /// Returns the number of subscribers currently connected to the service.
    pub fn number_of_subscribers(&self) -> usize {
        match &*self.0.lock() {
            PortFactoryPublishSubscribeType::Ipc(v) => v.dynamic_config().number_of_subscribers(),
            PortFactoryPublishSubscribeType::Local(v) => v.dynamic_config().number_of_subscribers(),
        }
    }
}
