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

use iceoryx2::prelude::{ipc, local};
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

#[derive(Clone)]
pub(crate) enum ServiceBuilderPublishSubscribeType {
    Ipc(
        iceoryx2::service::builder::publish_subscribe::Builder<
            [CustomPayloadMarker],
            CustomHeaderMarker,
            ipc::Service,
        >,
    ),
    Local(
        iceoryx2::service::builder::publish_subscribe::Builder<
            [CustomPayloadMarker],
            CustomHeaderMarker,
            local::Service,
        >,
    ),
}

#[pyclass]
pub struct ServiceBuilderPublishSubscribe(pub(crate) ServiceBuilderPublishSubscribeType);

#[pymethods]
impl ServiceBuilderPublishSubscribe {
    pub fn payload_type_details(&self, value: &TypeDetail) -> Self {
        match &self.0 {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_payload_type_details(&value.0) };
                Self(ServiceBuilderPublishSubscribeType::Ipc(this))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_payload_type_details(&value.0) };
                Self(ServiceBuilderPublishSubscribeType::Local(this))
            }
        }
    }

    pub fn user_header_type_details(&self, value: &TypeDetail) -> Self {
        match &self.0 {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_user_header_type_details(&value.0) };
                Self(ServiceBuilderPublishSubscribeType::Ipc(this))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_user_header_type_details(&value.0) };
                Self(ServiceBuilderPublishSubscribeType::Local(this))
            }
        }
    }

    pub fn payload_alignment(&self, value: &Alignment) -> Self {
        match &self.0 {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = this.payload_alignment(value.0);
                Self(ServiceBuilderPublishSubscribeType::Ipc(this))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = this.payload_alignment(value.0);
                Self(ServiceBuilderPublishSubscribeType::Local(this))
            }
        }
    }

    pub fn enable_safe_overflow(&self, value: bool) -> Self {
        match &self.0 {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = this.enable_safe_overflow(value);
                Self(ServiceBuilderPublishSubscribeType::Ipc(this))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = this.enable_safe_overflow(value);
                Self(ServiceBuilderPublishSubscribeType::Local(this))
            }
        }
    }

    pub fn subscriber_max_borrowed_samples(&self, value: usize) -> Self {
        match &self.0 {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = this.subscriber_max_borrowed_samples(value);
                Self(ServiceBuilderPublishSubscribeType::Ipc(this))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = this.subscriber_max_borrowed_samples(value);
                Self(ServiceBuilderPublishSubscribeType::Local(this))
            }
        }
    }

    pub fn history_size(&self, value: usize) -> Self {
        match &self.0 {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = this.history_size(value);
                Self(ServiceBuilderPublishSubscribeType::Ipc(this))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = this.history_size(value);
                Self(ServiceBuilderPublishSubscribeType::Local(this))
            }
        }
    }

    pub fn subscriber_max_buffer_size(&self, value: usize) -> Self {
        match &self.0 {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = this.subscriber_max_buffer_size(value);
                Self(ServiceBuilderPublishSubscribeType::Ipc(this))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = this.subscriber_max_buffer_size(value);
                Self(ServiceBuilderPublishSubscribeType::Local(this))
            }
        }
    }

    pub fn max_subscribers(&self, value: usize) -> Self {
        match &self.0 {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_subscribers(value);
                Self(ServiceBuilderPublishSubscribeType::Ipc(this))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = this.max_subscribers(value);
                Self(ServiceBuilderPublishSubscribeType::Local(this))
            }
        }
    }

    pub fn max_publishers(&self, value: usize) -> Self {
        match &self.0 {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_publishers(value);
                Self(ServiceBuilderPublishSubscribeType::Ipc(this))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = this.max_publishers(value);
                Self(ServiceBuilderPublishSubscribeType::Local(this))
            }
        }
    }

    pub fn max_nodes(&self, value: usize) -> Self {
        match &self.0 {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_nodes(value);
                Self(ServiceBuilderPublishSubscribeType::Ipc(this))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                let this = this.max_nodes(value);
                Self(ServiceBuilderPublishSubscribeType::Local(this))
            }
        }
    }

    pub fn open_or_create(&self) -> PyResult<PortFactoryPublishSubscribe> {
        match &self.0 {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe(
                    PortFactoryPublishSubscribeType::Ipc(this.open_or_create().map_err(|e| {
                        PublishSubscribeOpenOrCreateError::new_err(format!("{:?}", e))
                    })?),
                ))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe(
                    PortFactoryPublishSubscribeType::Local(this.open_or_create().map_err(|e| {
                        PublishSubscribeOpenOrCreateError::new_err(format!("{:?}", e))
                    })?),
                ))
            }
        }
    }

    pub fn open_or_create_with_attributes(
        &self,
        verifier: &AttributeVerifier,
    ) -> PyResult<PortFactoryPublishSubscribe> {
        match &self.0 {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe(
                    PortFactoryPublishSubscribeType::Ipc(
                        this.open_or_create_with_attributes(&verifier.0)
                            .map_err(|e| {
                                PublishSubscribeOpenOrCreateError::new_err(format!("{:?}", e))
                            })?,
                    ),
                ))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe(
                    PortFactoryPublishSubscribeType::Local(
                        this.open_or_create_with_attributes(&verifier.0)
                            .map_err(|e| {
                                PublishSubscribeOpenOrCreateError::new_err(format!("{:?}", e))
                            })?,
                    ),
                ))
            }
        }
    }

    pub fn open(&self) -> PyResult<PortFactoryPublishSubscribe> {
        match &self.0 {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe(
                    PortFactoryPublishSubscribeType::Ipc(
                        this.open()
                            .map_err(|e| PublishSubscribeOpenError::new_err(format!("{:?}", e)))?,
                    ),
                ))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe(
                    PortFactoryPublishSubscribeType::Local(
                        this.open()
                            .map_err(|e| PublishSubscribeOpenError::new_err(format!("{:?}", e)))?,
                    ),
                ))
            }
        }
    }

    pub fn open_with_attributes(
        &self,
        verifier: &AttributeVerifier,
    ) -> PyResult<PortFactoryPublishSubscribe> {
        match &self.0 {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe(
                    PortFactoryPublishSubscribeType::Ipc(
                        this.open_with_attributes(&verifier.0)
                            .map_err(|e| PublishSubscribeOpenError::new_err(format!("{:?}", e)))?,
                    ),
                ))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe(
                    PortFactoryPublishSubscribeType::Local(
                        this.open_with_attributes(&verifier.0)
                            .map_err(|e| PublishSubscribeOpenError::new_err(format!("{:?}", e)))?,
                    ),
                ))
            }
        }
    }

    pub fn create(&self) -> PyResult<PortFactoryPublishSubscribe> {
        match &self.0 {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe(
                    PortFactoryPublishSubscribeType::Ipc(
                        this.create().map_err(|e| {
                            PublishSubscribeCreateError::new_err(format!("{:?}", e))
                        })?,
                    ),
                ))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe(
                    PortFactoryPublishSubscribeType::Local(
                        this.create().map_err(|e| {
                            PublishSubscribeCreateError::new_err(format!("{:?}", e))
                        })?,
                    ),
                ))
            }
        }
    }

    pub fn create_with_attributes(
        &self,
        attributes: &AttributeSpecifier,
    ) -> PyResult<PortFactoryPublishSubscribe> {
        match &self.0 {
            ServiceBuilderPublishSubscribeType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe(
                    PortFactoryPublishSubscribeType::Ipc(
                        this.create_with_attributes(&attributes.0).map_err(|e| {
                            PublishSubscribeCreateError::new_err(format!("{:?}", e))
                        })?,
                    ),
                ))
            }
            ServiceBuilderPublishSubscribeType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryPublishSubscribe(
                    PortFactoryPublishSubscribeType::Local(
                        this.create_with_attributes(&attributes.0).map_err(|e| {
                            PublishSubscribeCreateError::new_err(format!("{:?}", e))
                        })?,
                    ),
                ))
            }
        }
    }
}
