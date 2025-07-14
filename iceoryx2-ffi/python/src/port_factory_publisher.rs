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
    type_storage::TypeStorage,
    unable_to_deliver_strategy::UnableToDeliverStrategy,
};

type IpcPortFactoryPublisher<'a> = iceoryx2::service::port_factory::publisher::PortFactoryPublisher<
    'a,
    crate::IpcService,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;
type LocalPortFactoryPublisher<'a> =
    iceoryx2::service::port_factory::publisher::PortFactoryPublisher<
        'a,
        crate::LocalService,
        [CustomPayloadMarker],
        CustomHeaderMarker,
    >;

pub(crate) enum PortFactoryPublisherType {
    Ipc(Parc<IpcPortFactoryPublisher<'static>>),
    Local(Parc<LocalPortFactoryPublisher<'static>>),
}

#[pyclass]
/// Factory to create a new `Publisher` port/endpoint for `MessagingPattern::PublishSubscribe`
/// based communication.
pub struct PortFactoryPublisher {
    factory: Parc<PortFactoryPublishSubscribeType>,
    value: PortFactoryPublisherType,
    payload_type_details: TypeStorage,
    user_header_type_details: TypeStorage,
}

impl PortFactoryPublisher {
    pub(crate) fn new(
        factory: Parc<PortFactoryPublishSubscribeType>,
        payload_type_details: TypeStorage,
        user_header_type_details: TypeStorage,
    ) -> Self {
        Self {
            factory: factory.clone(),
            value: match &*factory.lock() {
                PortFactoryPublishSubscribeType::Ipc(v) => PortFactoryPublisherType::Ipc(unsafe {
                    Parc::new(core::mem::transmute::<
                        IpcPortFactoryPublisher<'_>,
                        IpcPortFactoryPublisher<'static>,
                    >(v.publisher_builder()))
                }),
                PortFactoryPublishSubscribeType::Local(v) => {
                    PortFactoryPublisherType::Local(unsafe {
                        Parc::new(core::mem::transmute::<
                            LocalPortFactoryPublisher<'_>,
                            LocalPortFactoryPublisher<'static>,
                        >(v.publisher_builder()))
                    })
                }
            },
            payload_type_details,
            user_header_type_details,
        }
    }

    fn clone_ipc(&self, value: IpcPortFactoryPublisher<'static>) -> Self {
        Self {
            factory: self.factory.clone(),
            value: PortFactoryPublisherType::Ipc(Parc::new(value)),
            payload_type_details: self.payload_type_details.clone(),
            user_header_type_details: self.user_header_type_details.clone(),
        }
    }

    fn clone_local(&self, value: LocalPortFactoryPublisher<'static>) -> Self {
        Self {
            factory: self.factory.clone(),
            value: PortFactoryPublisherType::Local(Parc::new(value)),
            payload_type_details: self.payload_type_details.clone(),
            user_header_type_details: self.user_header_type_details.clone(),
        }
    }
}

#[pymethods]
impl PortFactoryPublisher {
    #[getter]
    pub fn __payload_type_details(&self) -> Option<Py<PyAny>> {
        self.payload_type_details.clone().value
    }

    /// Defines how many `SampleMut` the `Publisher` can loan with `Publisher::loan()` or
    /// `Publisher::loan_uninit()` in parallel.
    pub fn max_loaned_samples(&self, value: usize) -> Self {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryPublisherType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.max_loaned_samples(value);
                self.clone_ipc(this)
            }
            PortFactoryPublisherType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.max_loaned_samples(value);
                self.clone_local(this)
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
                self.clone_ipc(this)
            }
            PortFactoryPublisherType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.unable_to_deliver_strategy(value.clone().into());
                self.clone_local(this)
            }
        }
    }

    /// Sets the maximum slice length that a user can allocate with
    /// `ActiveRequest::loan_slice()` or `ActiveRequest::loan_slice_uninit()`.
    pub fn __initial_max_slice_len(&self, value: usize) -> Self {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryPublisherType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.initial_max_slice_len(value);
                self.clone_ipc(this)
            }
            PortFactoryPublisherType::Local(v) => {
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
            PortFactoryPublisherType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.allocation_strategy(value.clone().into());
                self.clone_ipc(this)
            }
            PortFactoryPublisherType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                let this = this.allocation_strategy(value.clone().into());
                self.clone_local(this)
            }
        }
    }

    /// Creates a new `Publisher` or emits a `PublisherCreateError` on failure.
    pub fn create(&self) -> PyResult<Publisher> {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryPublisherType::Ipc(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                Ok(Publisher {
                    value: Parc::new(PublisherType::Ipc(Some(
                        this.create()
                            .map_err(|e| PublisherCreateError::new_err(format!("{e:?}")))?,
                    ))),
                    payload_type_details: self.payload_type_details.clone(),
                    user_header_type_details: self.user_header_type_details.clone(),
                })
            }
            PortFactoryPublisherType::Local(v) => {
                let this = unsafe { (*v.lock()).__internal_partial_clone() };
                Ok(Publisher {
                    value: Parc::new(PublisherType::Local(Some(
                        this.create()
                            .map_err(|e| PublisherCreateError::new_err(format!("{e:?}")))?,
                    ))),
                    payload_type_details: self.payload_type_details.clone(),
                    user_header_type_details: self.user_header_type_details.clone(),
                })
            }
        }
    }
}
