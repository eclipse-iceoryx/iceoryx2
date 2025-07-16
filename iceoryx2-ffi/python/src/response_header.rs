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

use crate::unique_server_id::UniqueServerId;

#[pyclass(str = "{0:?}")]
/// Response header used by `MessagingPattern::RequestResponse`
pub struct ResponseHeader(pub(crate) iceoryx2::service::header::request_response::ResponseHeader);

#[pymethods]
impl ResponseHeader {
    #[getter]
    /// Returns the `UniqueServerId` of the `Server` which sent the `Response`
    pub fn server_id(&self) -> UniqueServerId {
        UniqueServerId(self.0.server_id())
    }

    #[getter]
    /// Returns how many elements are stored inside the `Response`s payload.
    pub fn number_of_elements(&self) -> u64 {
        self.0.number_of_elements()
    }
}
