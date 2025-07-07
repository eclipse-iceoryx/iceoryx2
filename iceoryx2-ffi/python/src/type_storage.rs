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

use pyo3::prelude::*;

#[pyclass]
pub struct TypeStorage {
    pub value: Option<PyObject>,
}

impl Default for TypeStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for TypeStorage {
    fn clone(&self) -> Self {
        Self {
            value: self
                .value
                .as_ref()
                .map(|v| Python::with_gil(|py| v.clone_ref(py))),
        }
    }
}

impl TypeStorage {
    pub fn new() -> Self {
        Self { value: None }
    }
}
