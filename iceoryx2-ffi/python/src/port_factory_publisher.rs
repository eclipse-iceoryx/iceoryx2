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
    error::PublisherCreateError,
    parc::Parc,
    port_factory_publish_subscribe::PortFactoryPublishSubscribeType,
    publisher::{Publisher, PublisherType},
    unable_to_deliver_strategy::UnableToDeliverStrategy,
};

pub(crate) enum PortFactoryPublisherType {
    Ipc(
        Parc<
            iceoryx2::service::port_factory::publisher::PortFactoryPublisher<
                'static,
                crate::IpcService,
                [CustomPayloadMarker],
                CustomHeaderMarker,
            >,
        >,
    ),
    Local(
        Parc<
            iceoryx2::service::port_factory::publisher::PortFactoryPublisher<
                'static,
                crate::LocalService,
                [CustomPayloadMarker],
                CustomHeaderMarker,
            >,
        >,
    ),
}

#[pyclass]
pub struct PortFactoryPublisher {
    factory: Parc<PortFactoryPublishSubscribeType>,
    value: PortFactoryPublisherType,
}

impl PortFactoryPublisher {
    pub(crate) fn new(factory: Parc<PortFactoryPublishSubscribeType>) -> Self {
        Self {
            factory: factory.clone(),
            value: match &*factory.lock() {
                PortFactoryPublishSubscribeType::Ipc(v) => PortFactoryPublisherType::Ipc(unsafe {
                    Parc::new(core::mem::transmute(v.publisher_builder()))
                }),
                PortFactoryPublishSubscribeType::Local(v) => {
                    PortFactoryPublisherType::Local(unsafe {
                        Parc::new(core::mem::transmute(v.publisher_builder()))
                    })
                }
            },
        }
    }
}

#[pymethods]
impl PortFactoryPublisher {
    /// Defines how many `SampleMut` the `Publisher` can loan with `Publisher::loan()` or
    /// `Publisher::loan_uninit()` in parallel.
    pub fn max_loaned_samples(&self, value: usize) -> Self {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryPublisherType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.max_loaned_samples(value);
                Self {
                    value: PortFactoryPublisherType::Ipc(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
            PortFactoryPublisherType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.max_loaned_samples(value);
                Self {
                    value: PortFactoryPublisherType::Local(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
        }
    }

    /// Sets the `UnableToDeliverStrategy`.
    pub fn unable_to_deliver_strategy(&self, value: &UnableToDeliverStrategy) -> Self {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryPublisherType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.unable_to_deliver_strategy(value.clone().into());
                Self {
                    value: PortFactoryPublisherType::Ipc(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
            PortFactoryPublisherType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.unable_to_deliver_strategy(value.clone().into());
                Self {
                    value: PortFactoryPublisherType::Local(Parc::new(this)),
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
            PortFactoryPublisherType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.initial_max_slice_len(value);
                Self {
                    value: PortFactoryPublisherType::Ipc(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
            PortFactoryPublisherType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.initial_max_slice_len(value);
                Self {
                    value: PortFactoryPublisherType::Local(Parc::new(this)),
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
            PortFactoryPublisherType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.allocation_strategy(value.clone().into());
                Self {
                    value: PortFactoryPublisherType::Ipc(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
            PortFactoryPublisherType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.allocation_strategy(value.clone().into());
                Self {
                    value: PortFactoryPublisherType::Local(Parc::new(this)),
                    factory: self.factory.clone(),
                }
            }
        }
    }

    /// Creates a new `Publisher` or emits a `PublisherCreateError` on failure.
    pub fn create(&self) -> PyResult<Publisher> {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryPublisherType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                Ok(Publisher(Parc::new(PublisherType::Ipc(Some(
                    this.create()
                        .map_err(|e| PublisherCreateError::new_err(format!("{e:?}")))?,
                )))))
            }
            PortFactoryPublisherType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                Ok(Publisher(Parc::new(PublisherType::Local(Some(
                    this.create()
                        .map_err(|e| PublisherCreateError::new_err(format!("{e:?}")))?,
                )))))
            }
        }
    }
}
