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
use crate::port_factory_request_response::PortFactoryRequestResponseType;

#[pyclass]
/// The dynamic configuration of a `MessagingPattern::RequestResponse` service.
/// Port counts reflect the current state of the service when accessed.
/// The view keeps the service open while it is referenced.
pub struct DynamicConfigRequestResponse(pub(crate) Parc<PortFactoryRequestResponseType>);

#[pymethods]
impl DynamicConfigRequestResponse {
    #[getter]
    /// Returns the number of clients currently connected to the service.
    pub fn number_of_clients(&self) -> usize {
        match &*self.0.lock() {
            PortFactoryRequestResponseType::Ipc(v) => v.dynamic_config().number_of_clients(),
            PortFactoryRequestResponseType::Local(v) => v.dynamic_config().number_of_clients(),
        }
    }

    #[getter]
    /// Returns the number of servers currently connected to the service.
    pub fn number_of_servers(&self) -> usize {
        match &*self.0.lock() {
            PortFactoryRequestResponseType::Ipc(v) => v.dynamic_config().number_of_servers(),
            PortFactoryRequestResponseType::Local(v) => v.dynamic_config().number_of_servers(),
        }
    }
}
