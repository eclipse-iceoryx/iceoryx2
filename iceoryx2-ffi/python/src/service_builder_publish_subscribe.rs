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
    PublishSubscribeCreateError, PublishSubscribeOpenError, PublishSubscribeOpenOrCreateError,
};
use crate::port_factory_publish_subscribe::{
    PortFactoryPublishSubscribe, PortFactoryPublishSubscribeType,
};
use crate::type_detail::TypeDetail;
use crate::type_storage::TypeStorage;

type IpcBuilder = iceoryx2::service::builder::publish_subscribe::Builder<
    [CustomPayloadMarker],
    CustomHeaderMarker,
    crate::IpcService,
>;
type LocalBuilder = iceoryx2::service::builder::publish_subscribe::Builder<
    [CustomPayloadMarker],
    CustomHeaderMarker,
    crate::LocalService,
>;

#[derive(Clone)]
pub(crate) enum ServiceBuilderPublishSubscribeType {
    Ipc(IpcBuilder),
    Local(LocalBuilder),
}

#[pyclass]
/// Builder to create new `MessagingPattern::PublishSubscribe` based `Service`s
pub struct ServiceBuilderPublishSubscribe {
    pub(crate) value: ServiceBuilderPublishSubscribeType,
    pub payload_type_details: TypeStorage,
    pub user_header_type_details: TypeStorage,
}

impl ServiceBuilderPublishSubscribe {
    pub(crate) fn new(value: ServiceBuilderPublishSubscribeType) -> Self {
        Self {
            value,
            payload_type_details: TypeStorage::new(),
            user_header_type_details: TypeStorage::new(),
        }
    }

    fn clone_ipc(&self, builder: IpcBuilder) -> Self {
        Self {
            value: ServiceBuilderPublishSubscribeType::Ipc(builder),
            payload_type_details: self.payload_type_details.clone(),
            user_header_type_details: self.user_header_type_details.clone(),
        }
    }

    fn clone_local(&self, builder: LocalBuilder) -> Self {
        Self {
            value: ServiceBuilderPublishSubscribeType::Local(builder),
            payload_type_details: self.payload_type_details.clone(),
            user_header_type_details: self.user_header_type_details.clone(),
        }
    }
}

#[pymethods]
impl ServiceBuilderPublishSubscribe {
    pub fn __set_payload_type(&mut self, value: PyObject) {
        self.payload_type_details.value = Some(value)
    }

    pub fn __set_user_header_type(&mut self, value: PyObject) {
        self.user_header_type_details.value = Some(value)
    }

    /// Defines the payload type. To be able to connect to a `Service` the `TypeDetail` must be
    /// identical in all participants since the communication is always strongly typed.
    pub fn __payload_type_details(&self, value: &TypeDetail) -> Self {
        match &self.value {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_payload_type_details(&value.0) };
                self.clone_ipc(this)
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_payload_type_details(&value.0) };
                self.clone_local(this)
            }
        }
    }

    /// Defines the user header type. To be able to connect to a `Service` the `TypeDetail` must be
    /// identical in all participants since the communication is always strongly typed.
    pub fn __user_header_type_details(&self, value: &TypeDetail) -> Self {
        match &self.value {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_user_header_type_details(&value.0) };
                self.clone_ipc(this)
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_user_header_type_details(&value.0) };
                self.clone_local(this)
            }
        }
    }

    /// Overrides and increases the alignment of the payload - useful when the payload is used in
    /// SIMD operations. To be able to connect to a `Service` the payload alignment must be
    /// identical in all participants since the communication is always strongly typed.
    pub fn payload_alignment(&self, value: &Alignment) -> Self {
        match &self.value {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = this.payload_alignment(value.0);
                self.clone_ipc(this)
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = this.payload_alignment(value.0);
                self.clone_local(this)
            }
        }
    }

    /// If the `Service` is created, defines the overflow behavior of the service. If an existing
    /// `Service` is opened it requires the service to have the defined overflow behavior.
    pub fn enable_safe_overflow(&self, value: bool) -> Self {
        match &self.value {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = this.enable_safe_overflow(value);
                self.clone_ipc(this)
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = this.enable_safe_overflow(value);
                self.clone_local(this)
            }
        }
    }

    /// If the `Service` is created it defines how many `Sample`s a
    /// `Subscriber` can borrow at most in parallel. If an existing
    /// `Service` is opened it defines the minimum required.
    pub fn subscriber_max_borrowed_samples(&self, value: usize) -> Self {
        match &self.value {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = this.subscriber_max_borrowed_samples(value);
                self.clone_ipc(this)
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = this.subscriber_max_borrowed_samples(value);
                self.clone_local(this)
            }
        }
    }

    /// If the `Service` is created it defines the maximum history size a `Subscriber` can request
    /// on connection. If an existing `Service` is opened it defines the minimum required.
    pub fn history_size(&self, value: usize) -> Self {
        match &self.value {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = this.history_size(value);
                self.clone_ipc(this)
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = this.history_size(value);
                self.clone_local(this)
            }
        }
    }

    /// If the `Service` is created it defines how many `Sample` a `Subscriber` can store in its
    /// internal buffer. If an existing `Service` is opened it defines the minimum required.
    pub fn subscriber_max_buffer_size(&self, value: usize) -> Self {
        match &self.value {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = this.subscriber_max_buffer_size(value);
                self.clone_ipc(this)
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = this.subscriber_max_buffer_size(value);
                self.clone_local(this)
            }
        }
    }

    /// If the `Service` is created it defines how many `Subscriber` shall be supported at
    /// most. If an existing `Service` is opened it defines how many `Subscriber` must be at
    /// least supported.
    pub fn max_subscribers(&self, value: usize) -> Self {
        match &self.value {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_subscribers(value);
                self.clone_ipc(this)
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = this.max_subscribers(value);
                self.clone_local(this)
            }
        }
    }

    /// If the `Service` is created it defines how many `Publisher` shall be supported at
    /// most. If an existing `Service` is opened it defines how many `Publisher` must be at
    /// least supported.
    pub fn max_publishers(&self, value: usize) -> Self {
        match &self.value {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_publishers(value);
                self.clone_ipc(this)
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = this.max_publishers(value);
                self.clone_local(this)
            }
        }
    }

    /// If the `Service` is created it defines how many `Node`s shall be able to open it in
    /// parallel. If an existing `Service` is opened it defines how many `Node`s must be at
    /// least supported.
    pub fn max_nodes(&self, value: usize) -> Self {
        match &self.value {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_nodes(value);
                self.clone_ipc(this)
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = this.max_nodes(value);
                self.clone_local(this)
            }
        }
    }

    /// If the `Service` exists, it will be opened otherwise a new `Service` will be created.
    /// On failure it emits `PublishSubscribeOpenOrCreateError`
    pub fn open_or_create(&self) -> PyResult<PortFactoryPublishSubscribe> {
        match &self.value {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe::new(
                    PortFactoryPublishSubscribeType::Ipc(this.open_or_create().map_err(|e| {
                        PublishSubscribeOpenOrCreateError::new_err(format!("{e:?}"))
                    })?),
                    self.payload_type_details.clone(),
                    self.user_header_type_details.clone(),
                ))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe::new(
                    PortFactoryPublishSubscribeType::Local(this.open_or_create().map_err(|e| {
                        PublishSubscribeOpenOrCreateError::new_err(format!("{e:?}"))
                    })?),
                    self.payload_type_details.clone(),
                    self.user_header_type_details.clone(),
                ))
            }
        }
    }

    /// If the `Service` exists, it will be opened otherwise a new `Service` will be
    /// created. It defines a set of attributes. If the `Service` already exists all attribute
    /// requirements must be satisfied otherwise the open process will fail. If the `Service`
    /// does not exist the required attributes will be defined in the `Service`.
    /// On failure it emits `PublishSubscribeOpenOrCreateError`
    pub fn open_or_create_with_attributes(
        &self,
        verifier: &AttributeVerifier,
    ) -> PyResult<PortFactoryPublishSubscribe> {
        match &self.value {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe::new(
                    PortFactoryPublishSubscribeType::Ipc(
                        this.open_or_create_with_attributes(&verifier.0)
                            .map_err(|e| {
                                PublishSubscribeOpenOrCreateError::new_err(format!("{e:?}"))
                            })?,
                    ),
                    self.payload_type_details.clone(),
                    self.user_header_type_details.clone(),
                ))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe::new(
                    PortFactoryPublishSubscribeType::Local(
                        this.open_or_create_with_attributes(&verifier.0)
                            .map_err(|e| {
                                PublishSubscribeOpenOrCreateError::new_err(format!("{e:?}"))
                            })?,
                    ),
                    self.payload_type_details.clone(),
                    self.user_header_type_details.clone(),
                ))
            }
        }
    }

    /// Opens an existing `Service`.
    /// On failure it emits `PublishSubscribeOpenError`.
    pub fn open(&self) -> PyResult<PortFactoryPublishSubscribe> {
        match &self.value {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe::new(
                    PortFactoryPublishSubscribeType::Ipc(
                        this.open()
                            .map_err(|e| PublishSubscribeOpenError::new_err(format!("{e:?}")))?,
                    ),
                    self.payload_type_details.clone(),
                    self.user_header_type_details.clone(),
                ))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe::new(
                    PortFactoryPublishSubscribeType::Local(
                        this.open()
                            .map_err(|e| PublishSubscribeOpenError::new_err(format!("{e:?}")))?,
                    ),
                    self.payload_type_details.clone(),
                    self.user_header_type_details.clone(),
                ))
            }
        }
    }

    /// Opens an existing `Service` with attribute requirements. If the defined attribute
    /// requirements are not satisfied the open process will fail.
    /// On failure it emits `PublishSubscribeOpenError`.
    pub fn open_with_attributes(
        &self,
        verifier: &AttributeVerifier,
    ) -> PyResult<PortFactoryPublishSubscribe> {
        match &self.value {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe::new(
                    PortFactoryPublishSubscribeType::Ipc(
                        this.open_with_attributes(&verifier.0)
                            .map_err(|e| PublishSubscribeOpenError::new_err(format!("{e:?}")))?,
                    ),
                    self.payload_type_details.clone(),
                    self.user_header_type_details.clone(),
                ))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe::new(
                    PortFactoryPublishSubscribeType::Local(
                        this.open_with_attributes(&verifier.0)
                            .map_err(|e| PublishSubscribeOpenError::new_err(format!("{e:?}")))?,
                    ),
                    self.payload_type_details.clone(),
                    self.user_header_type_details.clone(),
                ))
            }
        }
    }

    /// Creates a new `Service`.
    /// On failure it emits `PublishSubscribeCreateError`.
    pub fn create(&self) -> PyResult<PortFactoryPublishSubscribe> {
        match &self.value {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe::new(
                    PortFactoryPublishSubscribeType::Ipc(
                        this.create()
                            .map_err(|e| PublishSubscribeCreateError::new_err(format!("{e:?}")))?,
                    ),
                    self.payload_type_details.clone(),
                    self.user_header_type_details.clone(),
                ))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe::new(
                    PortFactoryPublishSubscribeType::Local(
                        this.create()
                            .map_err(|e| PublishSubscribeCreateError::new_err(format!("{e:?}")))?,
                    ),
                    self.payload_type_details.clone(),
                    self.user_header_type_details.clone(),
                ))
            }
        }
    }

    /// Creates a new `Service` with a set of attributes.
    /// On failure it emits `PublishSubscribeCreateError`.
    pub fn create_with_attributes(
        &self,
        attributes: &AttributeSpecifier,
    ) -> PyResult<PortFactoryPublishSubscribe> {
        match &self.value {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe::new(
                    PortFactoryPublishSubscribeType::Ipc(
                        this.create_with_attributes(&attributes.0)
                            .map_err(|e| PublishSubscribeCreateError::new_err(format!("{e:?}")))?,
                    ),
                    self.payload_type_details.clone(),
                    self.user_header_type_details.clone(),
                ))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe::new(
                    PortFactoryPublishSubscribeType::Local(
                        this.create_with_attributes(&attributes.0)
                            .map_err(|e| PublishSubscribeCreateError::new_err(format!("{e:?}")))?,
                    ),
                    self.payload_type_details.clone(),
                    self.user_header_type_details.clone(),
                ))
            }
        }
    }
}
