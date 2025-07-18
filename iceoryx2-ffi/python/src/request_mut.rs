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
    error::RequestSendError,
    parc::Parc,
    pending_response::{PendingResponse, PendingResponseType},
    request_header::RequestHeader,
    type_storage::TypeStorage,
};

type IpcRequestMut = iceoryx2::request_mut::RequestMut<
    crate::IpcService,
    [CustomPayloadMarker],
    CustomHeaderMarker,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;
type LocalRequestMut = iceoryx2::request_mut::RequestMut<
    crate::LocalService,
    [CustomPayloadMarker],
    CustomHeaderMarker,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;

pub(crate) enum RequestMutType {
    Ipc(Option<IpcRequestMut>),
    Local(Option<LocalRequestMut>),
}

#[pyclass]
/// The `RequestMut` represents the object that contains the payload that the `Client` sends to the
/// `Server`.
pub struct RequestMut {
    pub(crate) value: Parc<RequestMutType>,
    pub(crate) request_payload_type_details: TypeStorage,
    pub(crate) response_payload_type_details: TypeStorage,
    pub(crate) request_header_type_details: TypeStorage,
    pub(crate) response_header_type_details: TypeStorage,
}

#[pymethods]
impl RequestMut {
    #[getter]
    pub fn __request_payload_type_details(&self) -> Option<Py<PyAny>> {
        self.request_payload_type_details.clone().value
    }

    #[getter]
    pub fn __request_header_type_details(&self) -> Option<Py<PyAny>> {
        self.request_header_type_details.clone().value
    }

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
            RequestMutType::Ipc(Some(v)) => v.payload().len(),
            RequestMutType::Local(Some(v)) => v.payload().len(),
            _ => fatal_panic!(from "RequestMutUninit::__slice_len()",
                "Accessing a released request."),
        }
    }

    #[getter]
    /// Returns the iceoryx2 internal `RequestHeader`
    pub fn header(&self) -> RequestHeader {
        match &*self.value.lock() {
            RequestMutType::Ipc(Some(v)) => RequestHeader(*v.header()),
            RequestMutType::Local(Some(v)) => RequestHeader(*v.header()),
            _ => fatal_panic!(from "RequestMut::header()", "Accessing a released request."),
        }
    }

    #[getter]
    /// Returns a pointer to the user defined request header.
    pub fn user_header_ptr(&self) -> usize {
        match &mut *self.value.lock() {
            RequestMutType::Ipc(Some(v)) => {
                (v.user_header_mut() as *mut CustomHeaderMarker) as usize
            }
            RequestMutType::Local(Some(v)) => {
                (v.user_header_mut() as *mut CustomHeaderMarker) as usize
            }
            _ => {
                fatal_panic!(from "RequestMut::user_header_ptr()", "Accessing a released request.")
            }
        }
    }

    #[getter]
    /// Returns a pointer to the user defined request payload.
    pub fn payload_ptr(&self) -> usize {
        match &mut *self.value.lock() {
            RequestMutType::Ipc(Some(v)) => (v.payload_mut().as_mut_ptr()) as usize,
            RequestMutType::Local(Some(v)) => (v.payload_mut().as_mut_ptr()) as usize,
            _ => fatal_panic!(from "RequestMut::payload_ptr()", "Accessing a released request."),
        }
    }

    /// Releases the `RequestMut`.
    ///
    /// After this call the `RequestMut` is no longer usable!
    pub fn delete(&mut self) {
        match &mut *self.value.lock() {
            RequestMutType::Ipc(ref mut v) => {
                v.take();
            }
            RequestMutType::Local(ref mut v) => {
                v.take();
            }
        }
    }

    /// Sends the `RequestMut` to all connected `Server`s of the `Service`.
    pub fn send(&self) -> PyResult<PendingResponse> {
        match &mut *self.value.lock() {
            RequestMutType::Ipc(ref mut v) => {
                let request = v.take().unwrap();
                Ok(PendingResponse {
                    value: Parc::new(PendingResponseType::Ipc(Some(
                        request
                            .send()
                            .map_err(|e| RequestSendError::new_err(format!("{e:?}")))?,
                    ))),
                    request_header_type_details: self.request_header_type_details.clone(),
                    request_payload_type_details: self.request_payload_type_details.clone(),
                    response_header_type_details: self.response_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                })
            }
            RequestMutType::Local(ref mut v) => {
                let request = v.take().unwrap();
                Ok(PendingResponse {
                    value: Parc::new(PendingResponseType::Local(Some(
                        request
                            .send()
                            .map_err(|e| RequestSendError::new_err(format!("{e:?}")))?,
                    ))),
                    request_header_type_details: self.request_header_type_details.clone(),
                    request_payload_type_details: self.request_payload_type_details.clone(),
                    response_header_type_details: self.response_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                })
            }
        }
    }
}
