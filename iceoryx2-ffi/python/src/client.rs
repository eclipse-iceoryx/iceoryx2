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
    request_mut_uninit::{RequestMutUninit, RequestMutUninitType},
    type_storage::TypeStorage,
    unable_to_deliver_strategy::UnableToDeliverStrategy,
    unique_client_id::UniqueClientId,
};

type IpcClient = iceoryx2::port::client::Client<
    crate::IpcService,
    [CustomPayloadMarker],
    CustomHeaderMarker,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;
type LocalClient = iceoryx2::port::client::Client<
    crate::LocalService,
    [CustomPayloadMarker],
    CustomHeaderMarker,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;

pub(crate) enum ClientType {
    Ipc(Option<IpcClient>),
    Local(Option<LocalClient>),
}

#[pyclass]
/// Represents the receiving endpoint of an event based communication.
pub struct Client {
    pub(crate) value: ClientType,
    pub(crate) request_payload_type_details: TypeStorage,
    pub(crate) response_payload_type_details: TypeStorage,
    pub(crate) request_header_type_details: TypeStorage,
    pub(crate) response_header_type_details: TypeStorage,
}

#[pymethods]
impl Client {
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
    /// Returns the `UniqueClientId` of the `Client`
    pub fn id(&self) -> UniqueClientId {
        match &self.value {
            ClientType::Ipc(Some(v)) => UniqueClientId(v.id()),
            ClientType::Local(Some(v)) => UniqueClientId(v.id()),
            _ => fatal_panic!(from "Client::id()", "Accessing a released client."),
        }
    }

    /// Releases the `Client`.
    ///
    /// After this call the `Client` is no longer usable!
    pub fn delete(&mut self) {
        match self.value {
            ClientType::Ipc(ref mut v) => {
                v.take();
            }
            ClientType::Local(ref mut v) => {
                v.take();
            }
        }
    }

    #[getter]
    /// Returns the strategy the `Client` follows when a `RequestMut` cannot be delivered
    /// if the `Server`s buffer is full.
    pub fn unable_to_deliver_strategy(&self) -> UnableToDeliverStrategy {
        match &self.value {
            ClientType::Ipc(Some(v)) => v.unable_to_deliver_strategy().into(),
            ClientType::Local(Some(v)) => v.unable_to_deliver_strategy().into(),
            _ => {
                fatal_panic!(from "Client::unable_to_deliver_strategy()",
                    "Accessing a released client.")
            }
        }
    }

    /// Acquires an `RequestMutUninit` to store payload. This API shall be used
    /// by default to avoid unnecessary copies.
    pub fn __loan_uninit(&self) -> PyResult<RequestMutUninit> {
        self.__loan_slice_uninit(1)
    }

    /// Loans/allocates a `RequestMutUninit` from the underlying data segment of the `Client`.
    /// The user has to initialize the payload before it can be sent.
    ///
    /// On failure it emits a `LoanError` describing the failure.
    pub fn __loan_slice_uninit(&self, slice_len: usize) -> PyResult<RequestMutUninit> {
        match &self.value {
            ClientType::Ipc(Some(v)) => Ok(RequestMutUninit {
                value: Parc::new(RequestMutUninitType::Ipc(Some(unsafe {
                    v.loan_custom_payload(slice_len)
                        .map_err(|e| LoanError::new_err(format!("{e:?}")))?
                }))),
                request_payload_type_details: self.request_payload_type_details.clone(),
                response_payload_type_details: self.response_payload_type_details.clone(),
                request_header_type_details: self.request_header_type_details.clone(),
                response_header_type_details: self.response_header_type_details.clone(),
            }),
            ClientType::Local(Some(v)) => Ok(RequestMutUninit {
                value: Parc::new(RequestMutUninitType::Local(Some(unsafe {
                    v.loan_custom_payload(slice_len)
                        .map_err(|e| LoanError::new_err(format!("{e:?}")))?
                }))),
                request_payload_type_details: self.request_payload_type_details.clone(),
                response_payload_type_details: self.response_payload_type_details.clone(),
                request_header_type_details: self.request_header_type_details.clone(),
                response_header_type_details: self.response_header_type_details.clone(),
            }),
            _ => {
                fatal_panic!(from "Client::loan_slice_uninit()",
                    "Accessing a released client.")
            }
        }
    }

    /// Returns the maximum initial slice length configured for this `Client`.
    pub fn initial_max_slice_len(&self) -> usize {
        match &self.value {
            ClientType::Ipc(Some(v)) => v.initial_max_slice_len(),
            ClientType::Local(Some(v)) => v.initial_max_slice_len(),
            _ => {
                fatal_panic!(from "Client::loan_slice_uninit()",
                    "Accessing a released client.")
            }
        }
    }
}
