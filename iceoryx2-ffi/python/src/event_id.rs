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

#[pyclass(eq, str = "{0:?}")]
#[derive(PartialEq)]
/// User defined identifier that can be provided in `Notifier.notify()` to signal a specific
/// kind of event.
pub struct EventId(pub(crate) iceoryx2::prelude::EventId);

#[pymethods]
impl EventId {
    #[staticmethod]
    /// Creates a new `EventId` from a given integer value
    pub fn new(value: usize) -> Self {
        EventId(iceoryx2::prelude::EventId::new(value))
    }

    #[getter]
    /// Returns the integer value of the `EventId`
    pub fn as_value(&self) -> usize {
        self.0.as_value()
    }
}
