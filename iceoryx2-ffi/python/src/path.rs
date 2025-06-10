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

use crate::semantic_string_error::SemanticStringError;
use iceoryx2::prelude::SemanticString;
use pyo3::prelude::*;

#[pyclass(name = "Path")]
pub struct Path {
    value: iceoryx2::prelude::Path,
}

#[pymethods]
impl Path {
    #[staticmethod]
    pub fn new(name: &str) -> PyResult<Self> {
        Ok(Self {
            value: iceoryx2::prelude::Path::new(name.as_bytes())
                .map_err(|e| SemanticStringError::new_err(format!("{:?}", e)))?,
        })
    }

    #[staticmethod]
    pub fn max_len() -> usize {
        iceoryx2::prelude::Path::max_len()
    }

    pub fn to_string(&self) -> String {
        self.value.to_string()
    }
}
