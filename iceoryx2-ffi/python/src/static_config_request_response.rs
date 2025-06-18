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
/// The static configuration of an `MessagingPattern::RequestResponse` based service. Contains all
/// parameters that do not change during the lifetime of a `Service`.
pub struct StaticConfigRequestResponse(
    pub(crate) iceoryx2::service::static_config::request_response::StaticConfig,
);

#[pymethods]
impl StaticConfigRequestResponse {
    #[getter]
    /// Returns the request type details of the `Service`.
    pub fn request_message_type_details(&self) -> MessageTypeDetails {
        MessageTypeDetails(self.0.request_message_type_details().clone())
    }

    #[getter]
    /// Returns the response type details of the `Service`.
    pub fn response_message_type_details(&self) -> MessageTypeDetails {
        MessageTypeDetails(self.0.response_message_type_details().clone())
    }

    #[getter]
    /// Returns true if the request buffer of the `Service` safely overflows, otherwise false.
    /// Safe overflow means that the `Client` will recycle the oldest requests from the
    /// `Server` when its buffer is full.
    pub fn has_safe_overflow_for_requests(&self) -> bool {
        self.0.has_safe_overflow_for_requests()
    }

    #[getter]
    /// Returns true if the response buffer of the `Service` safely overflows, otherwise false.
    /// Safe overflow means that the `Server` will recycle the oldest responses from the
    /// `Client` when its buffer is full.
    pub fn has_safe_overflow_for_responses(&self) -> bool {
        self.0.has_safe_overflow_for_responses()
    }

    #[getter]
    /// Returns true if fire and forget `RequestMut`s can be sent from the `Client`, otherwise
    /// false.
    pub fn does_support_fire_and_forget_requests(&self) -> bool {
        self.0.does_support_fire_and_forget_requests()
    }

    #[getter]
    /// Returns the maximum number of borrowed `Response`s a `Client` can hold in parallel per
    /// `PendingResponse`
    pub fn max_borrowed_responses_per_pending_response(&self) -> usize {
        self.0.max_borrowed_responses_per_pending_response()
    }

    #[getter]
    /// Returns the maximum of active requests a `Server` can hold in parallel per `Client`.
    pub fn max_active_requests_per_client(&self) -> usize {
        self.0.max_active_requests_per_client()
    }

    #[getter]
    /// Returns the maximum buffer size for responses for a `PendingResponse`.
    pub fn max_response_buffer_size(&self) -> usize {
        self.0.max_response_buffer_size()
    }

    #[getter]
    /// Returns the maximum number of `RequestMut` a `Client` can loan in parallel.
    pub fn max_loaned_requests(&self) -> usize {
        self.0.max_loaned_requests()
    }

    #[getter]
    /// Returns the maximum number of supported `Server` ports for the `Service`.
    pub fn max_servers(&self) -> usize {
        self.0.max_servers()
    }

    #[getter]
    /// Returns the maximum number of supported `Client` ports for the `Service`.
    pub fn max_clients(&self) -> usize {
        self.0.max_clients()
    }

    #[getter]
    /// Returns the maximum number of supported `Node`s for the `Service`.
    pub fn max_nodes(&self) -> usize {
        self.0.max_nodes()
    }
}
