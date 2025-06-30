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
#[derive(PartialEq, Clone)]
/// Represents the string name of a type. The name shall uniquely identify the type in the
/// communication system.
pub struct TypeName(
    pub(crate) iceoryx2::service::static_config::message_type_details::TypeNameString,
);

#[pymethods]
impl TypeName {
    #[staticmethod]
    /// Creates a new `TypeName`. If the provided `name` exceeds the maximum supported length
    /// it emits an `SemanticStringError`.
    pub fn new(name: &str) -> PyResult<Self> {
        Ok(Self(
            iceoryx2::service::static_config::message_type_details::TypeNameString::from_bytes(
                name.as_bytes(),
            )
            .map_err(|e| SemanticStringError::new_err(format!("{e:?}")))?,
        ))
    }

    #[staticmethod]
    /// The maximum supported length of a `TypeName`
    pub fn max_len() -> usize {
        iceoryx2::service::static_config::message_type_details::TypeNameString::capacity()
    }

    #[allow(clippy::inherent_to_string)] // method required to generate this API in Python
    /// Returns the underlying `String` of the `TypeName`
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}
