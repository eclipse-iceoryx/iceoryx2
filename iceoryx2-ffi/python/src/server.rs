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
    active_request::{ActiveRequest, ActiveRequestType},
    error::{ConnectionFailure, ReceiveError},
    parc::Parc,
    type_storage::TypeStorage,
    unable_to_deliver_strategy::UnableToDeliverStrategy,
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
    Ipc(Option<IpcServer>),
    Local(Option<LocalServer>),
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
            ServerType::Ipc(Some(v)) => UniqueServerId(v.id()),
            ServerType::Local(Some(v)) => UniqueServerId(v.id()),
            _ => fatal_panic!(from "Server::id()",
                "Accessing a released server."),
        }
    }

    #[getter]
    /// Returns true if the `Server` has `RequestMut`s in its buffer.
    pub fn has_requests(&self) -> PyResult<bool> {
        match &self.value {
            ServerType::Ipc(Some(v)) => Ok(v
                .has_requests()
                .map_err(|e| ConnectionFailure::new_err(format!("{e:?}")))?),
            ServerType::Local(Some(v)) => Ok(v
                .has_requests()
                .map_err(|e| ConnectionFailure::new_err(format!("{e:?}")))?),
            _ => fatal_panic!(from "Server::has_requests()",
                "Accessing a released server."),
        }
    }

    #[getter]
    /// Returns the maximum initial slice length configured for this `Server`.
    pub fn __initial_max_slice_len(&self) -> usize {
        match &self.value {
            ServerType::Ipc(Some(v)) => v.initial_max_slice_len(),
            ServerType::Local(Some(v)) => v.initial_max_slice_len(),
            _ => fatal_panic!(from "Server::initial_max_slice_len()",
                "Accessing a released server."),
        }
    }

    /// Receives a `RequestMut` that was sent by a `Client` and returns an `ActiveRequest`
    /// which can be used to respond. If no `RequestMut`s were received it returns `None`.
    pub fn receive(&self) -> PyResult<Option<ActiveRequest>> {
        match &self.value {
            ServerType::Ipc(Some(v)) => Ok(unsafe {
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
            ServerType::Local(Some(v)) => Ok(unsafe {
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
            _ => fatal_panic!(from "Server::receive()",
                "Accessing a released server."),
        }
    }

    /// Releases the `Server`.
    ///
    /// After this call the `Server` is no longer usable!
    pub fn delete(&mut self) {
        match self.value {
            ServerType::Ipc(ref mut v) => {
                v.take();
            }
            ServerType::Local(ref mut v) => {
                v.take();
            }
        }
    }

    #[getter]
    /// Returns the strategy the `Server` follows when a `ResponseMut` cannot be delivered
    /// if the `Client`s buffer is full.
    pub fn unable_to_deliver_strategy(&self) -> UnableToDeliverStrategy {
        match &self.value {
            ServerType::Ipc(Some(v)) => v.unable_to_deliver_strategy().into(),
            ServerType::Local(Some(v)) => v.unable_to_deliver_strategy().into(),
            _ => {
                fatal_panic!(from "Server::unable_to_deliver_strategy()",
                    "Accessing a released client.")
            }
        }
    }
}
