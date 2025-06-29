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

use crate::error::SemanticStringError;
use pyo3::prelude::*;

#[pyclass(str = "{0:?}", eq)]
#[derive(PartialEq)]
/// Represent the name for a `Node`.
pub struct NodeName(pub(crate) iceoryx2::prelude::NodeName);

#[pymethods]
impl NodeName {
    #[staticmethod]
    /// Creates a new `NodeName`.
    /// If the provided name does not contain a valid `NodeName` it will emit
    /// `SemanticStringError`, otherwise the `NodeName`.
    pub fn new(name: &str) -> PyResult<Self> {
        Ok(Self(iceoryx2::prelude::NodeName::new(name).map_err(
            |e| SemanticStringError::new_err(format!("{e:?}")),
        )?))
    }

    #[staticmethod]
    /// Returns the maximum length of a `NodeName`
    pub fn max_len() -> usize {
        iceoryx2::prelude::NodeName::max_len()
    }

    /// Converts the `NodeName` into a `String`
    pub fn as_str(&self) -> String {
        self.0.as_str().to_string()
    }
}
