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

use crate::error::UniqueIdGeneratorDetailsError;
use pyo3::prelude::*;

use crate::service_type::ServiceType;

#[pyclass(str = "{0:?}", from_py_object)]
#[derive(Clone, PartialEq)]
/// The system-wide unique id of a `Node`
pub struct UniqueNodeId(pub(crate) iceoryx2::identifiers::UniqueNodeId);

#[pymethods]
impl UniqueNodeId {
    pub fn __eq__(&self, other: &Self) -> bool {
        self == other
    }

    #[getter]
    /// Returns the underlying integer value of the `UniqueNodeId`.
    pub fn value(&self) -> u128 {
        self.0.value()
    }

    /// Returns the process id of the process that owns the `Node`.
    pub fn pid(&self, service_type: &ServiceType) -> PyResult<u32> {
        match service_type {
            ServiceType::Ipc => match self.0.pid::<crate::IpcService>() {
                Ok(v) => Ok(v.value() as _),
                Err(e) => Err(UniqueIdGeneratorDetailsError::new_err(format!("{e:?}"))),
            },
            ServiceType::Local => match self.0.pid::<crate::LocalService>() {
                Ok(v) => Ok(v.value() as _),
                Err(e) => Err(UniqueIdGeneratorDetailsError::new_err(format!("{e:?}"))),
            },
        }
    }
}
