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

use crate::type_detail::TypeDetail;

#[pyclass]
/// Contains all type information to the header and payload type.
pub struct MessageTypeDetails(
    pub(crate) iceoryx2::service::static_config::message_type_details::MessageTypeDetails,
);

#[pymethods]
impl MessageTypeDetails {
    #[getter]
    /// The `TypeDetail` of the header of a message, the first iceoryx2 internal part.
    pub fn header(&self) -> TypeDetail {
        TypeDetail(self.0.header.clone())
    }

    #[getter]
    /// The `TypeDetail` of the user_header or the custom header, is located directly after the
    /// header.
    pub fn user_header(&self) -> TypeDetail {
        TypeDetail(self.0.user_header.clone())
    }

    #[getter]
    /// The `TypeDetail` of the payload of the message, the last part.
    pub fn payload(&self) -> TypeDetail {
        TypeDetail(self.0.payload.clone())
    }
}
