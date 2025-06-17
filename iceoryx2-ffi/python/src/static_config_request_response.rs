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

use crate::message_type_details::MessageTypeDetails;

#[pyclass]
pub struct StaticConfigRequestResponse(
    pub(crate) iceoryx2::service::static_config::request_response::StaticConfig,
);

#[pymethods]
impl StaticConfigRequestResponse {
    #[getter]
    pub fn request_message_type_details(&self) -> MessageTypeDetails {
        MessageTypeDetails(self.0.request_message_type_details().clone())
    }

    #[getter]
    pub fn response_message_type_details(&self) -> MessageTypeDetails {
        MessageTypeDetails(self.0.response_message_type_details().clone())
    }

    #[getter]
    pub fn has_safe_overflow_for_requests(&self) -> bool {
        self.0.has_safe_overflow_for_requests()
    }

    #[getter]
    pub fn has_safe_overflow_for_responses(&self) -> bool {
        self.0.has_safe_overflow_for_responses()
    }

    #[getter]
    pub fn does_support_fire_and_forget_requests(&self) -> bool {
        self.0.does_support_fire_and_forget_requests()
    }

    #[getter]
    pub fn max_borrowed_responses_per_pending_response(&self) -> usize {
        self.0.max_borrowed_responses_per_pending_response()
    }

    #[getter]
    pub fn max_active_requests_per_client(&self) -> usize {
        self.0.max_active_requests_per_client()
    }

    #[getter]
    pub fn max_response_buffer_size(&self) -> usize {
        self.0.max_response_buffer_size()
    }

    #[getter]
    pub fn max_loaned_requests(&self) -> usize {
        self.0.max_loaned_requests()
    }

    #[getter]
    pub fn max_servers(&self) -> usize {
        self.0.max_servers()
    }

    #[getter]
    pub fn max_clients(&self) -> usize {
        self.0.max_clients()
    }

    #[getter]
    pub fn max_nodes(&self) -> usize {
        self.0.max_nodes()
    }
}
