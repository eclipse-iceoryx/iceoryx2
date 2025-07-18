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
    allocation_strategy::AllocationStrategy,
    error::ServerCreateError,
    parc::Parc,
    port_factory_request_response::PortFactoryRequestResponseType,
    server::{Server, ServerType},
    type_storage::TypeStorage,
    unable_to_deliver_strategy::UnableToDeliverStrategy,
};

type IpcPortFactoryServer<'a> = iceoryx2::service::port_factory::server::PortFactoryServer<
    'a,
    crate::IpcService,
    [CustomPayloadMarker],
    CustomHeaderMarker,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;
type LocalPortFactoryServer<'a> = iceoryx2::service::port_factory::server::PortFactoryServer<
    'a,
    crate::LocalService,
    [CustomPayloadMarker],
    CustomHeaderMarker,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;

pub(crate) enum PortFactoryServerType {
    Ipc(Parc<IpcPortFactoryServer<'static>>),
    Local(Parc<LocalPortFactoryServer<'static>>),
}

#[pyclass]
/// Factory to create a new `Server` port/endpoint for `MessagingPattern::RequestResponse` based
/// communication.
pub struct PortFactoryServer {
    factory: Parc<PortFactoryRequestResponseType>,
    value: PortFactoryServerType,
    request_payload_type_details: TypeStorage,
    response_payload_type_details: TypeStorage,
    request_header_type_details: TypeStorage,
    response_header_type_details: TypeStorage,
}

impl PortFactoryServer {
    pub(crate) fn new(
        factory: Parc<PortFactoryRequestResponseType>,
        request_payload_type_details: TypeStorage,
        response_payload_type_details: TypeStorage,
        request_header_type_details: TypeStorage,
        response_header_type_details: TypeStorage,
    ) -> Self {
        Self {
            factory: factory.clone(),
            value: match &*factory.lock() {
                PortFactoryRequestResponseType::Ipc(v) => PortFactoryServerType::Ipc(unsafe {
                    Parc::new(core::mem::transmute::<
                        IpcPortFactoryServer<'_>,
                        IpcPortFactoryServer<'static>,
                    >(v.server_builder()))
                }),
                PortFactoryRequestResponseType::Local(v) => PortFactoryServerType::Local(unsafe {
                    Parc::new(core::mem::transmute::<
                        LocalPortFactoryServer<'_>,
                        LocalPortFactoryServer<'static>,
                    >(v.server_builder()))
                }),
            },
            request_header_type_details,
            request_payload_type_details,
            response_header_type_details,
            response_payload_type_details,
        }
    }

    fn clone_ipc(&self, value: IpcPortFactoryServer<'static>) -> Self {
        Self {
            factory: self.factory.clone(),
            value: PortFactoryServerType::Ipc(Parc::new(value)),
            request_payload_type_details: self.request_payload_type_details.clone(),
            response_payload_type_details: self.response_payload_type_details.clone(),
            request_header_type_details: self.request_header_type_details.clone(),
            response_header_type_details: self.response_header_type_details.clone(),
        }
    }

    fn clone_local(&self, value: LocalPortFactoryServer<'static>) -> Self {
        Self {
            factory: self.factory.clone(),
            value: PortFactoryServerType::Local(Parc::new(value)),
            request_payload_type_details: self.request_payload_type_details.clone(),
            response_payload_type_details: self.response_payload_type_details.clone(),
            request_header_type_details: self.request_header_type_details.clone(),
            response_header_type_details: self.response_header_type_details.clone(),
        }
    }
}

#[pymethods]
impl PortFactoryServer {
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

    /// Sets the `UnableToDeliverStrategy` which defines how the `Server` shall behave
    /// when a `Client` cannot receive a `Response` since its internal buffer is full.
    pub fn unable_to_deliver_strategy(&self, value: &UnableToDeliverStrategy) -> Self {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryServerType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.unable_to_deliver_strategy(value.clone().into());
                self.clone_ipc(this)
            }
            PortFactoryServerType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.unable_to_deliver_strategy(value.clone().into());
                self.clone_local(this)
            }
        }
    }

    /// Defines the maximum number of `ResponseMut` that
    /// the `Server` can loan in parallel per
    /// `ActiveRequest`.
    pub fn max_loaned_responses_per_request(&self, value: usize) -> Self {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryServerType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.max_loaned_responses_per_request(value);
                self.clone_ipc(this)
            }
            PortFactoryServerType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.max_loaned_responses_per_request(value);
                self.clone_local(this)
            }
        }
    }

    /// Sets the maximum slice length that a user can allocate with
    /// `ActiveRequest::loan_slice()` or `ActiveRequest::loan_slice_uninit()`.
    pub fn __initial_max_slice_len(&self, value: usize) -> Self {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryServerType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.initial_max_slice_len(value);
                self.clone_ipc(this)
            }
            PortFactoryServerType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.initial_max_slice_len(value);
                self.clone_local(this)
            }
        }
    }

    /// Defines the allocation strategy that is used when the provided
    /// `PortFactoryServer::initial_max_slice_len()` is exhausted. This happens when the user
    /// acquires more than max slice len in `ActiveRequest::loan_slice()` or
    /// `ActiveRequest::loan_slice_uninit()`.
    pub fn __allocation_strategy(&self, value: &AllocationStrategy) -> Self {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryServerType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.allocation_strategy(value.clone().into());
                self.clone_ipc(this)
            }
            PortFactoryServerType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.allocation_strategy(value.clone().into());
                self.clone_local(this)
            }
        }
    }

    /// Creates a new `Server` or emits a `ServerCreateError` on failure.
    pub fn create(&self) -> PyResult<Server> {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryServerType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                Ok(Server {
                    value: ServerType::Ipc(Some(
                        this.create()
                            .map_err(|e| ServerCreateError::new_err(format!("{e:?}")))?,
                    )),
                    request_header_type_details: self.request_header_type_details.clone(),
                    request_payload_type_details: self.request_payload_type_details.clone(),
                    response_header_type_details: self.response_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                })
            }
            PortFactoryServerType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                Ok(Server {
                    value: ServerType::Local(Some(
                        this.create()
                            .map_err(|e| ServerCreateError::new_err(format!("{e:?}")))?,
                    )),
                    request_header_type_details: self.request_header_type_details.clone(),
                    request_payload_type_details: self.request_payload_type_details.clone(),
                    response_header_type_details: self.response_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                })
            }
        }
    }
}
