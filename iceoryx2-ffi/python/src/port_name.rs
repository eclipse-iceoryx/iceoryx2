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

use crate::error::SemanticStringError;
use pyo3::prelude::*;

#[pyclass(str = "{0:?}", eq)]
#[derive(PartialEq)]
/// Represent the name for a port.
pub struct PortName(pub(crate) iceoryx2::prelude::PortName);

#[pymethods]
impl PortName {
    #[staticmethod]
    /// Creates a new `PortName`.
    /// If the provided name does not contain a valid `PortName` it will emit
    /// `SemanticStringError`, otherwise the `PortName`.
    pub fn new(name: &str) -> PyResult<Self> {
        Ok(Self(iceoryx2::prelude::PortName::new(name).map_err(
            |e| SemanticStringError::new_err(format!("{e:?}")),
        )?))
    }

    #[staticmethod]
    /// Returns the maximum length of a `PortName`
    pub fn max_len() -> usize {
        iceoryx2::prelude::PortName::max_len()
    }

    /// Converts the `PortName` into a `String`
    pub fn as_str(&self) -> String {
        self.0.as_str().to_string()
    }
}
