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
    error::ReceiveError,
    parc::Parc,
    request_header::RequestHeader,
    response::{Response, ResponseType},
    type_storage::TypeStorage,
};

type IpcPendingResponse = iceoryx2::pending_response::PendingResponse<
    crate::IpcService,
    [CustomPayloadMarker],
    CustomHeaderMarker,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;
type LocalPendingResponse = iceoryx2::pending_response::PendingResponse<
    crate::LocalService,
    [CustomPayloadMarker],
    CustomHeaderMarker,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;

pub(crate) enum PendingResponseType {
    Ipc(Option<IpcPendingResponse>),
    Local(Option<LocalPendingResponse>),
}

#[pyclass]
/// Represents an active connection to all `Server` that received the `RequestMut`. The
/// `Client` can use it to receive the corresponding `Response`s.
///
/// As soon as it goes out of scope, the connections are closed and the `Server`s are informed.
pub struct PendingResponse {
    pub(crate) value: Parc<PendingResponseType>,
    pub(crate) request_payload_type_details: TypeStorage,
    pub(crate) response_payload_type_details: TypeStorage,
    pub(crate) request_header_type_details: TypeStorage,
    pub(crate) response_header_type_details: TypeStorage,
}

#[pymethods]
impl PendingResponse {
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

    /// Returns `True` until the `ActiveRequest` goes out of scope on the `Server`s side
    /// indicating that the `Server` will no longer send `Response`s.
    /// It also returns `False` when there are no `Server`.
    #[getter]
    pub fn is_connected(&self) -> bool {
        match &*self.value.lock() {
            PendingResponseType::Ipc(Some(v)) => v.is_connected(),
            PendingResponseType::Local(Some(v)) => v.is_connected(),
            _ => fatal_panic!(from "PendingResponse::is_connected()",
                    "Accessing a released pending response."),
        }
    }

    /// Returns a reference to the iceoryx2 internal `RequestHeader` of the corresponding
    /// `RequestMut`
    #[getter]
    pub fn header(&self) -> RequestHeader {
        match &*self.value.lock() {
            PendingResponseType::Ipc(Some(v)) => RequestHeader(*v.header()),
            PendingResponseType::Local(Some(v)) => RequestHeader(*v.header()),
            _ => fatal_panic!(from "PendingResponse::header()",
                    "Accessing a released pending response."),
        }
    }

    /// Returns a pointer to the user defined request header of the corresponding
    /// `RequestMut`
    #[getter]
    pub fn user_header_ptr(&self) -> usize {
        match &*self.value.lock() {
            PendingResponseType::Ipc(Some(v)) => {
                (v.user_header() as *const CustomHeaderMarker) as usize
            }
            PendingResponseType::Local(Some(v)) => {
                (v.user_header() as *const CustomHeaderMarker) as usize
            }
            _ => fatal_panic!(from "PendingResponse::user_header_ptr()",
                    "Accessing a released pending response."),
        }
    }

    /// Returns a pointer to the request payload of the corresponding
    /// `RequestMut`
    #[getter]
    pub fn payload_ptr(&self) -> usize {
        match &*self.value.lock() {
            PendingResponseType::Ipc(Some(v)) => v.payload().as_ptr() as usize,
            PendingResponseType::Local(Some(v)) => v.payload().as_ptr() as usize,
            _ => fatal_panic!(from "PendingResponse::payload_ptr()",
                    "Accessing a released pending response."),
        }
    }

    /// Returns how many `Server`s received the corresponding `RequestMut` initially.
    #[getter]
    pub fn number_of_server_connections(&self) -> usize {
        match &*self.value.lock() {
            PendingResponseType::Ipc(Some(v)) => v.number_of_server_connections(),
            PendingResponseType::Local(Some(v)) => v.number_of_server_connections(),
            _ => fatal_panic!(from "PendingResponse::number_of_server_connections()",
                    "Accessing a released pending response."),
        }
    }

    /// Returns `True` when a `Server` has sent a `Response` otherwise `False`.
    #[getter]
    pub fn has_response(&self) -> bool {
        match &*self.value.lock() {
            PendingResponseType::Ipc(Some(v)) => v.has_response(),
            PendingResponseType::Local(Some(v)) => v.has_response(),
            _ => fatal_panic!(from "PendingResponse::has_response()",
                    "Accessing a released pending response."),
        }
    }

    /// Releases the `PendingResponse` and signals the `Server` that the `Client` is no longer
    /// interested in receiving any more `Response`s and terminates the connection.
    ///
    /// After this call the `PendingResponse` is no longer usable!
    pub fn delete(&mut self) {
        match &mut *self.value.lock() {
            PendingResponseType::Ipc(ref mut v) => {
                v.take();
            }
            PendingResponseType::Local(ref mut v) => {
                v.take();
            }
        }
    }

    /// Receives a `Response` from one of the `Server`s that received the `RequestMut`.
    pub fn receive(&self) -> PyResult<Option<Response>> {
        match &*self.value.lock() {
            PendingResponseType::Ipc(Some(v)) => Ok(unsafe {
                v.receive_custom_payload()
                    .map_err(|e| ReceiveError::new_err(format!("{e:?}")))?
                    .map(|response| Response {
                        value: Parc::new(ResponseType::Ipc(Some(response))),
                        response_header_type_details: self.response_header_type_details.clone(),
                        response_payload_type_details: self.response_payload_type_details.clone(),
                    })
            }),
            PendingResponseType::Local(Some(v)) => Ok(unsafe {
                v.receive_custom_payload()
                    .map_err(|e| ReceiveError::new_err(format!("{e:?}")))?
                    .map(|response| Response {
                        value: Parc::new(ResponseType::Local(Some(response))),
                        response_header_type_details: self.response_header_type_details.clone(),
                        response_payload_type_details: self.response_payload_type_details.clone(),
                    })
            }),
            _ => fatal_panic!(from "PendingResponse::receive()",
                    "Accessing a released pending response."),
        }
    }

    /// Marks the connection state that the `Client` wants to gracefully
    /// disconnect. When the `Server` reads this, it can send the last `Response` and drop the
    /// corresponding `ActiveRequest` to terminate the
    /// connection ensuring that no [`Response`] is lost on the `Client`
    /// side.
    pub fn set_disconnect_hint(&self) {
        match &*self.value.lock() {
            PendingResponseType::Ipc(Some(v)) => v.set_disconnect_hint(),
            PendingResponseType::Local(Some(v)) => v.set_disconnect_hint(),
            _ => fatal_panic!(from "PendingResponse::is_connected()",
                    "Accessing a released pending response."),
        }
    }
}
