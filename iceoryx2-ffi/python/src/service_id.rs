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
/// The unique id of a `Service`
pub struct ServiceId(pub(crate) iceoryx2::service::service_id::ServiceId);

#[pymethods]
impl ServiceId {
    #[staticmethod]
    /// Returns the maximum string length of a `ServiceId`
    pub fn max_number_of_characters() -> usize {
        iceoryx2::service::service_id::ServiceId::max_number_of_characters()
    }

    #[getter]
    /// Returns a String containing the `ServiceId` value
    pub fn as_str(&self) -> String {
        self.0.as_str().to_string()
    }
}
