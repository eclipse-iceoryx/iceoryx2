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
use iceoryx2::prelude::SemanticString;
use pyo3::prelude::*;

#[pyclass(str = "{0:?}", eq)]
#[derive(PartialEq)]
/// Relocatable (inter-process shared memory compatible) `SemanticString` implementation for
/// `FilePath`. All modification operations ensure that never an
/// invalid file or path name can be generated. All strings have a fixed size so that the maximum
/// path or file name length the system supports can be stored.
pub struct FilePath(pub(crate) iceoryx2::prelude::FilePath);

#[pymethods]
impl FilePath {
    #[staticmethod]
    /// Creates a new `FilePath` when the provided `name` contains a valid path to a file,
    /// otherwise it emits a `SemanticStringError`.
    pub fn new(name: &str) -> PyResult<Self> {
        Ok(Self(
            iceoryx2::prelude::FilePath::new(name.as_bytes())
                .map_err(|e| SemanticStringError::new_err(format!("{e:?}")))?,
        ))
    }

    #[staticmethod]
    /// Returns the maximum length of a `FilePath`
    pub fn max_len() -> usize {
        iceoryx2::prelude::FilePath::max_len()
    }

    /// Converts the `FilePath` into a `String`
    #[allow(clippy::inherent_to_string)] // method required to generate this API in Python
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}
