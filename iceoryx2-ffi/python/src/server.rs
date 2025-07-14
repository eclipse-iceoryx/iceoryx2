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
use pyo3::prelude::*;

use crate::{
    active_request::{ActiveRequest, ActiveRequestType},
    error::{ConnectionFailure, ReceiveError},
    parc::Parc,
    type_storage::TypeStorage,
    unique_server_id::UniqueServerId,
};

type IpcServer = iceoryx2::port::server::Server<
    crate::IpcService,
    [CustomPayloadMarker],
    CustomHeaderMarker,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;
type LocalServer = iceoryx2::port::server::Server<
    crate::LocalService,
    [CustomPayloadMarker],
    CustomHeaderMarker,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;

pub(crate) enum ServerType {
    Ipc(IpcServer),
    Local(LocalServer),
}

#[pyclass]
/// Represents the receiving endpoint of an event based communication.
pub struct Server {
    pub(crate) value: ServerType,
    pub(crate) request_payload_type_details: TypeStorage,
    pub(crate) response_payload_type_details: TypeStorage,
    pub(crate) request_header_type_details: TypeStorage,
    pub(crate) response_header_type_details: TypeStorage,
}

#[pymethods]
impl Server {
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
    /// Returns the `UniqueServerId` of the `Server`
    pub fn id(&self) -> UniqueServerId {
        match &self.value {
            ServerType::Ipc(v) => UniqueServerId(v.id()),
            ServerType::Local(v) => UniqueServerId(v.id()),
        }
    }

    #[getter]
    /// Returns true if the `Server` has `RequestMut`s in its buffer.
    pub fn has_requests(&self) -> PyResult<bool> {
        match &self.value {
            ServerType::Ipc(v) => Ok(v
                .has_requests()
                .map_err(|e| ConnectionFailure::new_err(format!("{e:?}")))?),
            ServerType::Local(v) => Ok(v
                .has_requests()
                .map_err(|e| ConnectionFailure::new_err(format!("{e:?}")))?),
        }
    }

    #[getter]
    /// Returns the maximum initial slice length configured for this `Server`.
    pub fn __initial_max_slice_len(&self) -> usize {
        match &self.value {
            ServerType::Ipc(v) => v.initial_max_slice_len(),
            ServerType::Local(v) => v.initial_max_slice_len(),
        }
    }

    /// Receives a `RequestMut` that was sent by a `Client` and returns an `ActiveRequest`
    /// which can be used to respond. If no `RequestMut`s were received it returns `None`.
    pub fn receive(&self) -> PyResult<Option<ActiveRequest>> {
        match &self.value {
            ServerType::Ipc(v) => Ok(unsafe {
                v.receive_custom_payload()
                    .map_err(|e| ReceiveError::new_err(format!("{e:?}")))?
                    .map(|v| ActiveRequest {
                        value: Parc::new(ActiveRequestType::Ipc(Some(v))),
                        request_header_type_details: self.request_header_type_details.clone(),
                        request_payload_type_details: self.request_payload_type_details.clone(),
                        response_header_type_details: self.response_header_type_details.clone(),
                        response_payload_type_details: self.response_payload_type_details.clone(),
                    })
            }),
            ServerType::Local(v) => Ok(unsafe {
                v.receive_custom_payload()
                    .map_err(|e| ReceiveError::new_err(format!("{e:?}")))?
                    .map(|v| ActiveRequest {
                        value: Parc::new(ActiveRequestType::Local(Some(v))),
                        request_header_type_details: self.request_header_type_details.clone(),
                        request_payload_type_details: self.request_payload_type_details.clone(),
                        response_header_type_details: self.response_header_type_details.clone(),
                        response_payload_type_details: self.response_payload_type_details.clone(),
                    })
            }),
        }
    }
}
