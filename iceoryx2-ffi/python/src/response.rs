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

use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use iceoryx2_bb_log::fatal_panic;
use pyo3::prelude::*;

use crate::{
    parc::Parc, response_header::ResponseHeader, type_storage::TypeStorage,
    unique_server_id::UniqueServerId,
};

type IpcResponse =
    iceoryx2::response::Response<crate::IpcService, [CustomPayloadMarker], CustomHeaderMarker>;
type LocalResponse =
    iceoryx2::response::Response<crate::LocalService, [CustomPayloadMarker], CustomHeaderMarker>;

pub(crate) enum ResponseType {
    Ipc(Option<IpcResponse>),
    Local(Option<LocalResponse>),
}

#[pyclass]
/// It stores the payload and can be received by the `PendingResponse` after a `RequestMut` was
/// sent to a `Server` via the `Client`.
pub struct Response {
    pub(crate) value: Parc<ResponseType>,
    pub(crate) response_payload_type_details: TypeStorage,
    pub(crate) response_header_type_details: TypeStorage,
}

#[pymethods]
impl Response {
    #[getter]
    pub fn __response_payload_type_details(&self) -> Option<Py<PyAny>> {
        self.response_payload_type_details.clone().value
    }

    #[getter]
    pub fn __response_header_type_details(&self) -> Option<Py<PyAny>> {
        self.response_header_type_details.clone().value
    }

    #[getter]
    pub fn __slice_len(&self) -> usize {
        match &*self.value.lock() {
            ResponseType::Ipc(Some(v)) => v.payload().len(),
            ResponseType::Local(Some(v)) => v.payload().len(),
            _ => fatal_panic!(from "RequestMutUninit::__slice_len()",
                "Accessing a released request."),
        }
    }

    #[getter]
    /// Returns the `ResponseHeader`
    pub fn header(&self) -> ResponseHeader {
        match &*self.value.lock() {
            ResponseType::Ipc(Some(v)) => ResponseHeader(*v.header()),
            ResponseType::Local(Some(v)) => ResponseHeader(*v.header()),
            _ => fatal_panic!(from "Response::header()",
                    "Accessing a released response."),
        }
    }

    #[getter]
    /// Returns a pointer to the user header of the response.
    pub fn user_header_ptr(&self) -> usize {
        match &*self.value.lock() {
            ResponseType::Ipc(Some(v)) => (v.user_header() as *const CustomHeaderMarker) as usize,
            ResponseType::Local(Some(v)) => (v.user_header() as *const CustomHeaderMarker) as usize,
            _ => fatal_panic!(from "Response::user_header_ptr()",
                    "Accessing a released response."),
        }
    }

    #[getter]
    /// Returns a pointer to the payload of the response.
    pub fn payload_ptr(&self) -> usize {
        match &*self.value.lock() {
            ResponseType::Ipc(Some(v)) => v.payload().as_ptr() as usize,
            ResponseType::Local(Some(v)) => v.payload().as_ptr() as usize,
            _ => fatal_panic!(from "Response::payload_ptr()",
                    "Accessing a released response."),
        }
    }

    #[getter]
    /// Returns the `UniqueServerId` of the `Server` which sent the `Response`.
    pub fn origin(&self) -> UniqueServerId {
        match &*self.value.lock() {
            ResponseType::Ipc(Some(v)) => UniqueServerId(v.origin()),
            ResponseType::Local(Some(v)) => UniqueServerId(v.origin()),
            _ => fatal_panic!(from "Response::origin()",
                    "Accessing a released response."),
        }
    }

    /// Releases the `Response`.
    ///
    /// After this call the `Response` is no longer usable!
    pub fn delete(&mut self) {
        match &mut *self.value.lock() {
            ResponseType::Ipc(ref mut v) => {
                v.take();
            }
            ResponseType::Local(ref mut v) => {
                v.take();
            }
        }
    }
}
