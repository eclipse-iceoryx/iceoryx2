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
use crate::port_factory_blackboard::PortFactoryBlackboardType;

#[pyclass]
/// The dynamic configuration of a `MessagingPattern::Blackboard` service.
/// Port counts reflect the current state of the service when accessed.
/// The view keeps the service open while it is referenced.
/// Explicitly deleting the blackboard factory invalidates the view; subsequent
/// count queries raise `RuntimeError`.
pub struct DynamicConfigBlackboard(pub(crate) Parc<PortFactoryBlackboardType>);

#[pymethods]
impl DynamicConfigBlackboard {
    #[getter]
    /// Returns the number of writers currently connected to the service.
    pub fn number_of_writers(&self) -> PyResult<usize> {
        match &*self.0.lock() {
            PortFactoryBlackboardType::Ipc(Some(v)) => Ok(v.dynamic_config().number_of_writers()),
            PortFactoryBlackboardType::Local(Some(v)) => Ok(v.dynamic_config().number_of_writers()),
            _ => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "The blackboard service has been deleted.",
            )),
        }
    }

    #[getter]
    /// Returns the number of readers currently connected to the service.
    pub fn number_of_readers(&self) -> PyResult<usize> {
        match &*self.0.lock() {
            PortFactoryBlackboardType::Ipc(Some(v)) => Ok(v.dynamic_config().number_of_readers()),
            PortFactoryBlackboardType::Local(Some(v)) => Ok(v.dynamic_config().number_of_readers()),
            _ => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "The blackboard service has been deleted.",
            )),
        }
    }
}
