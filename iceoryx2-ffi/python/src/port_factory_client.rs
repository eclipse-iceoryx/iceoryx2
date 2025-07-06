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

use crate::error::ClientCreateError;
use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use pyo3::prelude::*;

use crate::{
    allocation_strategy::AllocationStrategy,
    client::{Client, ClientType},
    parc::Parc,
    port_factory_request_response::PortFactoryRequestResponseType,
    unable_to_deliver_strategy::UnableToDeliverStrategy,
};

type IpcPortFactoryClient<'a> = iceoryx2::service::port_factory::client::PortFactoryClient<
    'a,
    crate::IpcService,
    [CustomPayloadMarker],
    CustomHeaderMarker,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;
type LocalPortFactoryClient<'a> = iceoryx2::service::port_factory::client::PortFactoryClient<
    'a,
    crate::LocalService,
    [CustomPayloadMarker],
    CustomHeaderMarker,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;

pub(crate) enum PortFactoryClientType {
    Ipc(Parc<IpcPortFactoryClient<'static>>),
    Local(Parc<LocalPortFactoryClient<'static>>),
}

#[pyclass]
/// Factory to create a new `Client` port/endpoint for `MessagingPattern::RequestResponse`
/// based communication.
pub struct PortFactoryClient {
    factory: Parc<PortFactoryRequestResponseType>,
    value: PortFactoryClientType,
}

impl PortFactoryClient {
    pub(crate) fn new(factory: Parc<PortFactoryRequestResponseType>) -> Self {
        Self {
            factory: factory.clone(),
            value: match &*factory.lock() {
                PortFactoryRequestResponseType::Ipc(v) => PortFactoryClientType::Ipc(unsafe {
                    Parc::new(core::mem::transmute::<
                        IpcPortFactoryClient<'_>,
                        IpcPortFactoryClient<'static>,
                    >(v.client_builder()))
                }),
                PortFactoryRequestResponseType::Local(v) => PortFactoryClientType::Local(unsafe {
                    Parc::new(core::mem::transmute::<
                        LocalPortFactoryClient<'_>,
                        LocalPortFactoryClient<'static>,
                    >(v.client_builder()))
                }),
            },
        }
    }
}

#[pymethods]
impl PortFactoryClient {
    /// Sets the `UnableToDeliverStrategy` which defines how the `Client` shall behave
    /// when a `Server` cannot receive a `RequestMut` since its internal buffer is full.
    pub fn unable_to_deliver_strategy(&self, value: &UnableToDeliverStrategy) -> Self {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryClientType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.unable_to_deliver_strategy(value.clone().into());
                Self {
                    value: PortFactoryClientType::Ipc(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
            PortFactoryClientType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.unable_to_deliver_strategy(value.clone().into());
                Self {
                    value: PortFactoryClientType::Local(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
        }
    }

    /// Sets the maximum slice length that a user can allocate with
    /// `Client::loan_slice()` or `Client::loan_slice_uninit()`.
    pub fn initial_max_slice_len(&self, value: usize) -> Self {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryClientType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.initial_max_slice_len(value);
                Self {
                    value: PortFactoryClientType::Ipc(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
            PortFactoryClientType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.initial_max_slice_len(value);
                Self {
                    value: PortFactoryClientType::Local(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
        }
    }

    /// Defines the allocation strategy that is used when the provided
    /// `PortFactoryClient::initial_max_slice_len()` is exhausted. This happens when the user
    /// acquires more than max slice len in `Client::loan_slice()` or `Client::loan_slice_uninit()`.
    pub fn allocation_strategy(&self, value: &AllocationStrategy) -> Self {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryClientType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.allocation_strategy(value.clone().into());
                Self {
                    value: PortFactoryClientType::Ipc(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
            PortFactoryClientType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.allocation_strategy(value.clone().into());
                Self {
                    value: PortFactoryClientType::Local(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
        }
    }

    /// Creates a new `Client` or emits a `ClientCreateError` on failure.
    pub fn create(&self) -> PyResult<Client> {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryClientType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                Ok(Client(ClientType::Ipc(Arc::new(this.create().map_err(
                    |e| ClientCreateError::new_err(format!("{e:?}")),
                )?))))
            }
            PortFactoryClientType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                Ok(Client(ClientType::Local(Arc::new(this.create().map_err(
                    |e| ClientCreateError::new_err(format!("{e:?}")),
                )?))))
            }
        }
    }
}
