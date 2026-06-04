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

use pyo3::prelude::*;

#[pyclass(eq, str = "{0:?}")]
#[derive(PartialEq)]
/// User defined identifier that can be provided in `Notifier.notify()` to signal a specific
/// kind of event.
pub struct EventActivation(pub(crate) iceoryx2::prelude::EventActivation);

#[pymethods]
impl EventActivation {
    #[getter]
    /// Returns the `EventId`
    pub fn id(&self) -> crate::event_id::EventId {
        crate::event_id::EventId::new(self.0.id.as_value())
    }

    #[getter]
    /// Returns how often the `EventId` was notified
    pub fn count(&self) -> u64 {
        self.0.count
    }
}
