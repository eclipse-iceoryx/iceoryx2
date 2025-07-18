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

use crate::alignment::Alignment;
use crate::attribute_specifier::AttributeSpecifier;
use crate::attribute_verifier::AttributeVerifier;
use crate::error::{
    RequestResponseCreateError, RequestResponseOpenError, RequestResponseOpenOrCreateError,
};
use crate::parc::Parc;
use crate::port_factory_request_response::{
    PortFactoryRequestResponse, PortFactoryRequestResponseType,
};
use crate::type_detail::TypeDetail;
use crate::type_storage::TypeStorage;

type IpcBuilder = iceoryx2::service::builder::request_response::Builder<
    [CustomPayloadMarker],
    CustomHeaderMarker,
    [CustomPayloadMarker],
    CustomHeaderMarker,
    crate::IpcService,
>;

type LocalBuilder = iceoryx2::service::builder::request_response::Builder<
    [CustomPayloadMarker],
    CustomHeaderMarker,
    [CustomPayloadMarker],
    CustomHeaderMarker,
    crate::LocalService,
>;

#[derive(Clone)]
pub(crate) enum ServiceBuilderRequestResponseType {
    Ipc(IpcBuilder),
    Local(LocalBuilder),
}

#[pyclass]
/// Builder to create new `MessagingPattern::RequestResponse` based `Service`s
pub struct ServiceBuilderRequestResponse {
    pub(crate) value: ServiceBuilderRequestResponseType,
    pub(crate) request_payload_type_details: TypeStorage,
    pub(crate) response_payload_type_details: TypeStorage,
    pub(crate) request_header_type_details: TypeStorage,
    pub(crate) response_header_type_details: TypeStorage,
}

impl ServiceBuilderRequestResponse {
    pub(crate) fn new(value: ServiceBuilderRequestResponseType) -> Self {
        Self {
            value,
            request_payload_type_details: TypeStorage::new(),
            response_payload_type_details: TypeStorage::new(),
            request_header_type_details: TypeStorage::new(),
            response_header_type_details: TypeStorage::new(),
        }
    }

    fn clone_ipc(&self, builder: IpcBuilder) -> Self {
        Self {
            value: ServiceBuilderRequestResponseType::Ipc(builder),
            request_payload_type_details: self.request_payload_type_details.clone(),
            response_payload_type_details: self.response_payload_type_details.clone(),
            request_header_type_details: self.request_header_type_details.clone(),
            response_header_type_details: self.response_header_type_details.clone(),
        }
    }

    fn clone_local(&self, builder: LocalBuilder) -> Self {
        Self {
            value: ServiceBuilderRequestResponseType::Local(builder),
            request_payload_type_details: self.request_payload_type_details.clone(),
            response_payload_type_details: self.response_payload_type_details.clone(),
            request_header_type_details: self.request_header_type_details.clone(),
            response_header_type_details: self.response_header_type_details.clone(),
        }
    }
}

#[pymethods]
impl ServiceBuilderRequestResponse {
    pub fn __set_request_payload_type(&mut self, value: PyObject) {
        self.request_payload_type_details.value = Some(value)
    }

    pub fn __set_response_payload_type(&mut self, value: PyObject) {
        self.response_payload_type_details.value = Some(value)
    }

    pub fn __set_request_header_type(&mut self, value: PyObject) {
        self.request_header_type_details.value = Some(value)
    }

    pub fn __set_response_header_type(&mut self, value: PyObject) {
        self.response_header_type_details.value = Some(value)
    }

    /// Defines the payload type for requests. To be able to connect to a `Service` the
    /// `TypeDetail` must be identical in all participants since the communication is always
    /// strongly typed.
    pub fn __request_payload_type_details(&self, value: &TypeDetail) -> Self {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_request_payload_type_details(&value.0) };
                self.clone_ipc(this)
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_request_payload_type_details(&value.0) };
                self.clone_local(this)
            }
        }
    }

    /// Defines the request header type. To be able to connect to a `Service` the `TypeDetail` must
    /// be identical in all participants since the communication is always strongly typed.
    pub fn __request_header_type_details(&self, value: &TypeDetail) -> Self {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_request_header_type_details(&value.0) };
                self.clone_ipc(this)
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_request_header_type_details(&value.0) };
                self.clone_local(this)
            }
        }
    }

    /// Defines the payload type for responses. To be able to connect to a `Service` the
    /// `TypeDetail` must be identical in all participants since the communication is always
    /// strongly typed.
    pub fn __response_payload_type_details(&self, value: &TypeDetail) -> Self {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_response_payload_type_details(&value.0) };
                self.clone_ipc(this)
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_response_payload_type_details(&value.0) };
                self.clone_local(this)
            }
        }
    }

    /// Defines the response header type. To be able to connect to a `Service` the `TypeDetail`
    /// must be identical in all participants since the communication is always strongly typed.
    pub fn __response_header_type_details(&self, value: &TypeDetail) -> Self {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_response_header_type_details(&value.0) };
                self.clone_ipc(this)
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_response_header_type_details(&value.0) };
                self.clone_local(this)
            }
        }
    }

    /// Overrides and increases the alignment of the request payload - useful when the payload is
    /// used in SIMD operations. To be able to connect to a `Service` the payload alignment must be
    /// identical in all participants since the communication is always strongly typed.
    pub fn request_payload_alignment(&self, value: &Alignment) -> Self {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                let this = this.request_payload_alignment(value.0);
                self.clone_ipc(this)
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                let this = this.request_payload_alignment(value.0);
                self.clone_local(this)
            }
        }
    }

    /// Overrides and increases the alignment of the response payload - useful when the payload is
    /// used in SIMD operations. To be able to connect to a `Service` the payload alignment must be
    /// identical in all participants since the communication is always strongly typed.
    pub fn response_payload_alignment(&self, value: &Alignment) -> Self {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                let this = this.response_payload_alignment(value.0);
                self.clone_ipc(this)
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                let this = this.response_payload_alignment(value.0);
                self.clone_local(this)
            }
        }
    }

    /// If the `Service` is created, defines the overflow behavior of the service for requests.
    /// If an existing `Service` is opened it requires the service to have the defined overflow
    /// behavior.
    pub fn enable_safe_overflow_for_requests(&self, value: bool) -> Self {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                let this = this.enable_safe_overflow_for_requests(value);
                self.clone_ipc(this)
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                let this = this.enable_safe_overflow_for_requests(value);
                self.clone_local(this)
            }
        }
    }

    /// If the `Service` is created, defines the overflow behavior of the service for responses.
    /// If an existing `Service` is opened it requires the service to have the defined overflow
    /// behavior.
    pub fn enable_safe_overflow_for_responses(&self, value: bool) -> Self {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                let this = this.enable_safe_overflow_for_responses(value);
                self.clone_ipc(this)
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                let this = this.enable_safe_overflow_for_responses(value);
                self.clone_local(this)
            }
        }
    }

    /// If the `Service` is created, defines the fire-and-forget behavior of the service for
    /// requests.
    pub fn enable_fire_and_forget_requests(&self, value: bool) -> Self {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                let this = this.enable_fire_and_forget_requests(value);
                self.clone_ipc(this)
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                let this = this.enable_fire_and_forget_requests(value);
                self.clone_local(this)
            }
        }
    }

    /// Defines how many active requests a `Server` can hold in
    /// parallel per `Client`. The objects are used to send answers to a request that was
    /// received earlier from a `Client`.
    pub fn max_active_requests_per_client(&self, value: usize) -> Self {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_active_requests_per_client(value);
                self.clone_ipc(this)
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                let this = this.max_active_requests_per_client(value);
                self.clone_local(this)
            }
        }
    }

    /// If the `Service` is created it defines how many `RequestMut` a
    /// `Client` can loan in parallel.
    pub fn max_loaned_requests(&self, value: usize) -> Self {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_loaned_requests(value);
                self.clone_ipc(this)
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                let this = this.max_loaned_requests(value);
                self.clone_local(this)
            }
        }
    }

    /// If the `Service` is created it defines how many responses fit in the
    /// `Clients`s buffer. If an existing `Service` is opened it defines the minimum required.
    pub fn max_response_buffer_size(&self, value: usize) -> Self {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_response_buffer_size(value);
                self.clone_ipc(this)
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                let this = this.max_response_buffer_size(value);
                self.clone_local(this)
            }
        }
    }

    /// If the `Service` is created it defines how many `Server`s shall
    /// be supported at most. If an existing `Service` is opened it defines how many
    /// `Server`s must be at least supported.
    pub fn max_servers(&self, value: usize) -> Self {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_servers(value);
                self.clone_ipc(this)
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                let this = this.max_servers(value);
                self.clone_local(this)
            }
        }
    }

    /// If the `Service` is created it defines how many `Client`s shall
    /// be supported at most. If an existing `Service` is opened it defines how many
    /// `Client`s must be at least supported.
    pub fn max_clients(&self, value: usize) -> Self {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_clients(value);
                self.clone_ipc(this)
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                let this = this.max_clients(value);
                self.clone_local(this)
            }
        }
    }

    /// If the `Service` is created it defines how many `Node`s shall
    /// be able to open it in parallel. If an existing `Service` is opened it defines how many
    /// `Node`s must be at least supported.
    pub fn max_nodes(&self, value: usize) -> Self {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_nodes(value);
                self.clone_ipc(this)
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                let this = this.max_nodes(value);
                self.clone_local(this)
            }
        }
    }

    /// If the `Service` is created it defines how many `Response`s shall
    /// be able to be borrowed in parallel per `PendingResponse`. If an
    /// existing `Service` is opened it defines how many borrows must be at least supported.
    pub fn max_borrowed_responses_per_pending_response(&self, value: usize) -> Self {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_borrowed_responses_per_pending_response(value);
                self.clone_ipc(this)
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                let this = this.max_borrowed_responses_per_pending_response(value);
                self.clone_local(this)
            }
        }
    }

    /// If the `Service` exists, it will be opened otherwise a new `Service` will be created.
    /// On failure `RequestResponseOpenOrCreateError` will be emitted.
    pub fn open_or_create(&self) -> PyResult<PortFactoryRequestResponse> {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryRequestResponse {
                    value: Parc::new(PortFactoryRequestResponseType::Ipc(
                        this.open_or_create().map_err(|e| {
                            RequestResponseOpenOrCreateError::new_err(format!("{e:?}"))
                        })?,
                    )),
                    request_payload_type_details: self.request_payload_type_details.clone(),
                    request_header_type_details: self.request_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                    response_header_type_details: self.response_header_type_details.clone(),
                })
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryRequestResponse {
                    value: Parc::new(PortFactoryRequestResponseType::Local(
                        this.open_or_create().map_err(|e| {
                            RequestResponseOpenOrCreateError::new_err(format!("{e:?}"))
                        })?,
                    )),
                    request_payload_type_details: self.request_payload_type_details.clone(),
                    request_header_type_details: self.request_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                    response_header_type_details: self.response_header_type_details.clone(),
                })
            }
        }
    }

    /// If the `Service` exists, it will be opened otherwise a new `Service` will be
    /// created. It defines a set of attributes.
    ///
    /// If the `Service` already exists all attribute requirements must be satisfied,
    /// and service payload type must be the same, otherwise the open process will fail.
    /// If the `Service` does not exist the required attributes will be defined in the `Service`.
    /// On failure `RequestResponseOpenOrCreateError` will be emitted.
    pub fn open_or_create_with_attributes(
        &self,
        verifier: &AttributeVerifier,
    ) -> PyResult<PortFactoryRequestResponse> {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryRequestResponse {
                    value: Parc::new(PortFactoryRequestResponseType::Ipc(
                        this.open_or_create_with_attributes(&verifier.0)
                            .map_err(|e| {
                                RequestResponseOpenOrCreateError::new_err(format!("{e:?}"))
                            })?,
                    )),
                    request_payload_type_details: self.request_payload_type_details.clone(),
                    request_header_type_details: self.request_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                    response_header_type_details: self.response_header_type_details.clone(),
                })
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryRequestResponse {
                    value: Parc::new(PortFactoryRequestResponseType::Local(
                        this.open_or_create_with_attributes(&verifier.0)
                            .map_err(|e| {
                                RequestResponseOpenOrCreateError::new_err(format!("{e:?}"))
                            })?,
                    )),
                    request_payload_type_details: self.request_payload_type_details.clone(),
                    request_header_type_details: self.request_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                    response_header_type_details: self.response_header_type_details.clone(),
                })
            }
        }
    }

    /// Opens an existing `Service`.
    /// On failure `RequestResponseOpenError` will be emitted.
    pub fn open(&self) -> PyResult<PortFactoryRequestResponse> {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryRequestResponse {
                    value: Parc::new(PortFactoryRequestResponseType::Ipc(
                        this.open()
                            .map_err(|e| RequestResponseOpenError::new_err(format!("{e:?}")))?,
                    )),
                    request_payload_type_details: self.request_payload_type_details.clone(),
                    request_header_type_details: self.request_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                    response_header_type_details: self.response_header_type_details.clone(),
                })
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryRequestResponse {
                    value: Parc::new(PortFactoryRequestResponseType::Local(
                        this.open()
                            .map_err(|e| RequestResponseOpenError::new_err(format!("{e:?}")))?,
                    )),
                    request_payload_type_details: self.request_payload_type_details.clone(),
                    request_header_type_details: self.request_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                    response_header_type_details: self.response_header_type_details.clone(),
                })
            }
        }
    }

    /// Opens an existing `Service` with attribute requirements. If the defined attribute
    /// requirements are not satisfied the open process will fail.
    /// On failure `RequestResponseOpenError` will be emitted.
    pub fn open_with_attributes(
        &self,
        verifier: &AttributeVerifier,
    ) -> PyResult<PortFactoryRequestResponse> {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryRequestResponse {
                    value: Parc::new(PortFactoryRequestResponseType::Ipc(
                        this.open_with_attributes(&verifier.0)
                            .map_err(|e| RequestResponseOpenError::new_err(format!("{e:?}")))?,
                    )),
                    request_payload_type_details: self.request_payload_type_details.clone(),
                    request_header_type_details: self.request_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                    response_header_type_details: self.response_header_type_details.clone(),
                })
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryRequestResponse {
                    value: Parc::new(PortFactoryRequestResponseType::Local(
                        this.open_with_attributes(&verifier.0)
                            .map_err(|e| RequestResponseOpenError::new_err(format!("{e:?}")))?,
                    )),
                    request_payload_type_details: self.request_payload_type_details.clone(),
                    request_header_type_details: self.request_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                    response_header_type_details: self.response_header_type_details.clone(),
                })
            }
        }
    }

    /// Creates a new `Service`.
    /// On failure `RequestResponseCreateError` will be emitted.
    pub fn create(&self) -> PyResult<PortFactoryRequestResponse> {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryRequestResponse {
                    value: Parc::new(PortFactoryRequestResponseType::Ipc(
                        this.create()
                            .map_err(|e| RequestResponseCreateError::new_err(format!("{e:?}")))?,
                    )),
                    request_payload_type_details: self.request_payload_type_details.clone(),
                    request_header_type_details: self.request_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                    response_header_type_details: self.response_header_type_details.clone(),
                })
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryRequestResponse {
                    value: Parc::new(PortFactoryRequestResponseType::Local(
                        this.create()
                            .map_err(|e| RequestResponseCreateError::new_err(format!("{e:?}")))?,
                    )),
                    request_payload_type_details: self.request_payload_type_details.clone(),
                    request_header_type_details: self.request_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                    response_header_type_details: self.response_header_type_details.clone(),
                })
            }
        }
    }

    /// Creates a new `Service` with a set of attributes.
    /// On failure `RequestResponseCreateError` will be emitted.
    pub fn create_with_attributes(
        &self,
        attributes: &AttributeSpecifier,
    ) -> PyResult<PortFactoryRequestResponse> {
        match &self.value {
            ServiceBuilderRequestResponseType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryRequestResponse {
                    value: Parc::new(PortFactoryRequestResponseType::Ipc(
                        this.create_with_attributes(&attributes.0)
                            .map_err(|e| RequestResponseCreateError::new_err(format!("{e:?}")))?,
                    )),
                    request_payload_type_details: self.request_payload_type_details.clone(),
                    request_header_type_details: self.request_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                    response_header_type_details: self.response_header_type_details.clone(),
                })
            }
            ServiceBuilderRequestResponseType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryRequestResponse {
                    value: Parc::new(PortFactoryRequestResponseType::Local(
                        this.create_with_attributes(&attributes.0)
                            .map_err(|e| RequestResponseCreateError::new_err(format!("{e:?}")))?,
                    )),
                    request_payload_type_details: self.request_payload_type_details.clone(),
                    request_header_type_details: self.request_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                    response_header_type_details: self.response_header_type_details.clone(),
                })
            }
        }
    }
}
