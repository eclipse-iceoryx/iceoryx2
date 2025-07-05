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

use std::sync::Arc;

use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use pyo3::prelude::*;

use crate::{
    allocation_strategy::AllocationStrategy,
    error::ServerCreateError,
    parc::Parc,
    port_factory_request_response::PortFactoryRequestResponseType,
    server::{Server, ServerType},
    unable_to_deliver_strategy::UnableToDeliverStrategy,
};

pub(crate) enum PortFactoryServerType {
    Ipc(
        Parc<
            iceoryx2::service::port_factory::server::PortFactoryServer<
                'static,
                crate::IpcService,
                [CustomPayloadMarker],
                CustomHeaderMarker,
                [CustomPayloadMarker],
                CustomHeaderMarker,
            >,
        >,
    ),
    Local(
        Parc<
            iceoryx2::service::port_factory::server::PortFactoryServer<
                'static,
                crate::LocalService,
                [CustomPayloadMarker],
                CustomHeaderMarker,
                [CustomPayloadMarker],
                CustomHeaderMarker,
            >,
        >,
    ),
}

#[pyclass]
pub struct PortFactoryServer {
    factory: Parc<PortFactoryRequestResponseType>,
    value: PortFactoryServerType,
}

impl PortFactoryServer {
    pub(crate) fn new(factory: Parc<PortFactoryRequestResponseType>) -> Self {
        Self {
            factory: factory.clone(),
            value: match &*factory.lock() {
                PortFactoryRequestResponseType::Ipc(v) => PortFactoryServerType::Ipc(unsafe {
                    Parc::new(core::mem::transmute(v.server_builder()))
                }),
                PortFactoryRequestResponseType::Local(v) => PortFactoryServerType::Local(unsafe {
                    Parc::new(core::mem::transmute(v.server_builder()))
                }),
            },
        }
    }
}

#[pymethods]
impl PortFactoryServer {
    /// Sets the `UnableToDeliverStrategy` which defines how the `Server` shall behave
    /// when a `Client` cannot receive a `Response` since its internal buffer is full.
    pub fn unable_to_deliver_strategy(&self, value: &UnableToDeliverStrategy) -> Self {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryServerType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.unable_to_deliver_strategy(value.clone().into());
                Self {
                    value: PortFactoryServerType::Ipc(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
            PortFactoryServerType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.unable_to_deliver_strategy(value.clone().into());
                Self {
                    value: PortFactoryServerType::Local(Parc::new(this)),
                    factory: self.factory.clone(),
                }
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
                Self {
                    value: PortFactoryServerType::Ipc(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
            PortFactoryServerType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.max_loaned_responses_per_request(value);
                Self {
                    value: PortFactoryServerType::Local(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
        }
    }

    /// Sets the maximum slice length that a user can allocate with
    /// `ActiveRequest::loan_slice()` or `ActiveRequest::loan_slice_uninit()`.
    pub fn initial_max_slice_len(&self, value: usize) -> Self {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryServerType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.initial_max_slice_len(value);
                Self {
                    value: PortFactoryServerType::Ipc(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
            PortFactoryServerType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.initial_max_slice_len(value);
                Self {
                    value: PortFactoryServerType::Local(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
        }
    }

    /// Defines the allocation strategy that is used when the provided
    /// `PortFactoryServer::initial_max_slice_len()` is exhausted. This happens when the user
    /// acquires more than max slice len in `ActiveRequest::loan_slice()` or
    /// `ActiveRequest::loan_slice_uninit()`.
    pub fn allocation_strategy(&self, value: &AllocationStrategy) -> Self {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryServerType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.allocation_strategy(value.clone().into());
                Self {
                    value: PortFactoryServerType::Ipc(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
            PortFactoryServerType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.allocation_strategy(value.clone().into());
                Self {
                    value: PortFactoryServerType::Local(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
        }
    }

    /// Creates a new `Server` or emits a `ServerCreateError` on failure.
    pub fn create(&self) -> PyResult<Server> {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryServerType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                Ok(Server(ServerType::Ipc(Arc::new(this.create().map_err(
                    |e| ServerCreateError::new_err(format!("{e:?}")),
                )?))))
            }
            PortFactoryServerType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                Ok(Server(ServerType::Local(Arc::new(this.create().map_err(
                    |e| ServerCreateError::new_err(format!("{e:?}")),
                )?))))
            }
        }
    }
}
