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

use crate::error::InvalidAlignmentValue;

#[pyclass(str = "{0:?}", eq)]
#[derive(PartialEq)]
/// Contains the alignment memory can have.
pub struct Alignment(pub(crate) iceoryx2::prelude::Alignment);

#[pymethods]
impl Alignment {
    #[staticmethod]
    /// Creates a new `Alignment`. If the value is zero or not a power of 2
    /// it emits an `InvalidAlignmentValue`.
    pub fn new(value: usize) -> PyResult<Alignment> {
        match iceoryx2::prelude::Alignment::new(value) {
            Some(v) => Ok(Alignment(v)),
            None => Err(InvalidAlignmentValue::new_err(format!(
                "This: ({value}) is not a valid alignment"
            ))),
        }
    }

    /// Returns the value of the `Alignment`
    pub fn value(&self) -> usize {
        self.0.value()
    }
}
