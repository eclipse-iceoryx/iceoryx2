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
    error::LoanError,
    parc::Parc,
    request_header::RequestHeader,
    response_mut_uninit::{ResponseMutUninit, ResponseMutUninitType},
    type_storage::TypeStorage,
    unique_client_id::UniqueClientId,
};

type IpcActiveRequest = iceoryx2::active_request::ActiveRequest<
    crate::IpcService,
    [CustomPayloadMarker],
    CustomHeaderMarker,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;
type LocalActiveRequest = iceoryx2::active_request::ActiveRequest<
    crate::LocalService,
    [CustomPayloadMarker],
    CustomHeaderMarker,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;

pub(crate) enum ActiveRequestType {
    Ipc(Option<IpcActiveRequest>),
    Local(Option<LocalActiveRequest>),
}

#[pyclass]
/// The `ActiveRequest` represents the object that contains the payload that the `Client` sends to the
/// `Server`.
pub struct ActiveRequest {
    pub(crate) value: Parc<ActiveRequestType>,
    pub(crate) request_payload_type_details: TypeStorage,
    pub(crate) response_payload_type_details: TypeStorage,
    pub(crate) request_header_type_details: TypeStorage,
    pub(crate) response_header_type_details: TypeStorage,
}

#[pymethods]
impl ActiveRequest {
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
            ActiveRequestType::Ipc(Some(v)) => v.payload().len(),
            ActiveRequestType::Local(Some(v)) => v.payload().len(),
            _ => fatal_panic!(from "RequestMutUninit::__slice_len()",
                "Accessing a released request."),
        }
    }

    #[getter]
    /// Returns `True` until the `PendingResponse` goes out of scope on the `Client`s side
    /// indicating that the `Client` no longer receives the `ResponseMut`.
    pub fn is_connected(&self) -> bool {
        match &*self.value.lock() {
            ActiveRequestType::Ipc(Some(v)) => v.is_connected(),
            ActiveRequestType::Local(Some(v)) => v.is_connected(),
            _ => fatal_panic!(from "ActiveRequest::is_connected()",
                    "Accessing a released active request"),
        }
    }

    #[getter]
    /// Returns a pointer to the payload of the received `RequestMut`.
    pub fn payload_ptr(&self) -> usize {
        match &*self.value.lock() {
            ActiveRequestType::Ipc(Some(v)) => v.payload().as_ptr() as usize,
            ActiveRequestType::Local(Some(v)) => v.payload().as_ptr() as usize,
            _ => fatal_panic!(from "ActiveRequest::payload_ptr()",
                    "Accessing a released active request"),
        }
    }

    #[getter]
    /// Returns a pointer to the user_header of the received `RequestMut`
    pub fn user_header_ptr(&self) -> usize {
        match &*self.value.lock() {
            ActiveRequestType::Ipc(Some(v)) => {
                (v.user_header() as *const CustomHeaderMarker) as usize
            }
            ActiveRequestType::Local(Some(v)) => {
                (v.user_header() as *const CustomHeaderMarker) as usize
            }
            _ => fatal_panic!(from "ActiveRequest::user_header_ptr()",
                    "Accessing a released active request"),
        }
    }

    #[getter]
    /// Returns the `RequestHeader` of the received `RequestMut`
    pub fn header(&self) -> RequestHeader {
        match &*self.value.lock() {
            ActiveRequestType::Ipc(Some(v)) => RequestHeader(*v.header()),
            ActiveRequestType::Local(Some(v)) => RequestHeader(*v.header()),
            _ => fatal_panic!(from "ActiveRequest::header()",
                    "Accessing a released active request"),
        }
    }

    #[getter]
    /// Returns the `UniqueClientId` of the `Client`
    pub fn origin(&self) -> UniqueClientId {
        match &*self.value.lock() {
            ActiveRequestType::Ipc(Some(v)) => UniqueClientId(v.origin()),
            ActiveRequestType::Local(Some(v)) => UniqueClientId(v.origin()),
            _ => fatal_panic!(from "ActiveRequest::origin()",
                    "Accessing a released active request"),
        }
    }

    #[getter]
    /// Returns `True` if the `Client` wants to gracefully disconnect.
    /// This allows the `Server` to send its last response and then
    /// drop the `ActiveRequest` to signal the `Client` that no more
    /// `ResponseMut` will be sent.
    pub fn has_disconnect_hint(&self) -> bool {
        match &*self.value.lock() {
            ActiveRequestType::Ipc(Some(v)) => v.has_disconnect_hint(),
            ActiveRequestType::Local(Some(v)) => v.has_disconnect_hint(),
            _ => fatal_panic!(from "ActiveRequest::has_requested_graceful_disconnect()",
                    "Accessing a released active request"),
        }
    }

    /// Releases the `ActiveRequest` and terminates the connection.
    ///
    /// After this call the `ActiveRequest` is no longer usable!
    pub fn delete(&mut self) {
        match &mut *self.value.lock() {
            ActiveRequestType::Ipc(ref mut v) => {
                v.take();
            }
            ActiveRequestType::Local(ref mut v) => {
                v.take();
            }
        }
    }

    /// Loans uninitialized memory for a `ResponseMut` where the user can write its payload to.
    pub fn __loan_uninit(&self) -> PyResult<ResponseMutUninit> {
        self.__loan_slice_uninit(1)
    }

    /// Loans/allocates a `ResponseMutUninit` from the underlying data segment of the
    /// `Server`. The user has to initialize the payload before it can be sent.
    ///
    /// On failure it emits `LoanError` describing the failure.
    pub fn __loan_slice_uninit(&self, slice_len: usize) -> PyResult<ResponseMutUninit> {
        match &*self.value.lock() {
            ActiveRequestType::Ipc(Some(v)) => Ok(ResponseMutUninit {
                value: Parc::new(ResponseMutUninitType::Ipc(Some(unsafe {
                    v.loan_custom_payload(slice_len)
                        .map_err(|e| LoanError::new_err(format!("{e:?}")))?
                }))),
                response_header_type_details: self.response_header_type_details.clone(),
                response_payload_type_details: self.response_payload_type_details.clone(),
            }),
            ActiveRequestType::Local(Some(v)) => Ok(ResponseMutUninit {
                value: Parc::new(ResponseMutUninitType::Local(Some(unsafe {
                    v.loan_custom_payload(slice_len)
                        .map_err(|e| LoanError::new_err(format!("{e:?}")))?
                }))),
                response_header_type_details: self.response_header_type_details.clone(),
                response_payload_type_details: self.response_payload_type_details.clone(),
            }),
            _ => fatal_panic!(from "ActiveRequest::loan_slice_uninit()",
                    "Accessing a released active request"),
        }
    }
}
