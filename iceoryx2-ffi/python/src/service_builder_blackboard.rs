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

use iceoryx2::service::builder::CustomKeyMarker;
use pyo3::prelude::*;

use crate::attribute_specifier::AttributeSpecifier;
use crate::error::BlackboardCreateError;
use crate::port_factory_blackboard::{PortFactoryBlackboard, PortFactoryBlackboardType};
use crate::type_detail::TypeDetail;
use crate::type_storage::TypeStorage;

type IpcCreator =
    iceoryx2::service::builder::blackboard::Creator<CustomKeyMarker, crate::IpcService>;
type LocalCreator =
    iceoryx2::service::builder::blackboard::Creator<CustomKeyMarker, crate::LocalService>;
type IpcOpener = iceoryx2::service::builder::blackboard::Opener<CustomKeyMarker, crate::IpcService>;
type LocalOpener =
    iceoryx2::service::builder::blackboard::Opener<CustomKeyMarker, crate::LocalService>;

#[derive(Clone)]
pub(crate) enum ServiceBuilderBlackboardCreatorType {
    Ipc(IpcCreator),
    Local(LocalCreator),
}

#[derive(Clone)]
pub(crate) enum ServiceBuilderBlackboardOpenerType {
    Ipc(IpcOpener),
    Local(LocalOpener),
}

// TODO: remove unsendable?
#[pyclass(unsendable)]
/// Builder to create new `MessagingPattern::Blackboard` based `Service`s
pub struct ServiceBuilderBlackboardCreator {
    pub(crate) value: ServiceBuilderBlackboardCreatorType,
    pub key_type_details: TypeStorage,
}

impl ServiceBuilderBlackboardCreator {
    pub(crate) fn new(value: ServiceBuilderBlackboardCreatorType) -> Self {
        Self {
            value,
            key_type_details: TypeStorage::new(),
        }
    }

    fn clone_ipc(&self, builder: IpcCreator) -> Self {
        Self {
            value: ServiceBuilderBlackboardCreatorType::Ipc(builder),
            key_type_details: self.key_type_details.clone(),
        }
    }

    fn clone_local(&self, builder: LocalCreator) -> Self {
        Self {
            value: ServiceBuilderBlackboardCreatorType::Local(builder),
            key_type_details: self.key_type_details.clone(),
        }
    }
}

#[pymethods]
impl ServiceBuilderBlackboardCreator {
    #[getter]
    pub fn __key_type_details(&self) -> Option<Py<PyAny>> {
        self.key_type_details.clone().value
    }

    pub fn __set_key_type(&mut self, value: PyObject) {
        self.key_type_details.value = Some(value)
    }

    pub fn __set_key_type_details(&self, value: &TypeDetail) -> Self {
        match &self.value {
            ServiceBuilderBlackboardCreatorType::Ipc(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_key_type_details(&value.0) };
                self.clone_ipc(this)
            }
            ServiceBuilderBlackboardCreatorType::Local(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_key_type_details(&value.0) };
                self.clone_local(this)
            }
        }
    }

    pub fn max_readers(&self, value: usize) -> Self {
        match &self.value {
            ServiceBuilderBlackboardCreatorType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_readers(value);
                self.clone_ipc(this)
            }
            ServiceBuilderBlackboardCreatorType::Local(v) => {
                let this = v.clone();
                let this = this.max_readers(value);
                self.clone_local(this)
            }
        }
    }

    pub fn max_nodes(&self, value: usize) -> Self {
        match &self.value {
            ServiceBuilderBlackboardCreatorType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_nodes(value);
                self.clone_ipc(this)
            }
            ServiceBuilderBlackboardCreatorType::Local(v) => {
                let this = v.clone();
                let this = this.max_nodes(value);
                self.clone_local(this)
            }
        }
    }

    pub fn __add(&mut self, key: PyObject, value: PyObject, value_details: &TypeDetail) -> Self {
        match &self.value {
            ServiceBuilderBlackboardCreatorType::Ipc(v) => {
                let this = v.clone();
                let this = unsafe {
                    this.__internal_add(
                        key.as_ptr() as *const u8,
                        value.as_ptr() as *mut u8,
                        value_details.0.clone(),
                        Box::new(|| {}),
                    )
                };
                self.clone_ipc(this)
            }
            ServiceBuilderBlackboardCreatorType::Local(v) => {
                let this = v.clone();
                let this = unsafe {
                    this.__internal_add(
                        key.as_ptr() as *const u8,
                        value.as_ptr() as *mut u8,
                        value_details.0.clone(),
                        Box::new(|| {}),
                    )
                };
                self.clone_local(this)
            }
        }
    }

    pub fn create(&self) -> PyResult<PortFactoryBlackboard> {
        match &self.value {
            ServiceBuilderBlackboardCreatorType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryBlackboard::new(
                    PortFactoryBlackboardType::Ipc(
                        this.create()
                            .map_err(|e| BlackboardCreateError::new_err(format!("{e:?}")))?,
                    ),
                    self.key_type_details.clone(),
                ))
            }
            ServiceBuilderBlackboardCreatorType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryBlackboard::new(
                    PortFactoryBlackboardType::Local(
                        this.create()
                            .map_err(|e| BlackboardCreateError::new_err(format!("{e:?}")))?,
                    ),
                    self.key_type_details.clone(),
                ))
            }
        }
    }

    pub fn create_with_attributes(
        &self,
        attributes: &AttributeSpecifier,
    ) -> PyResult<PortFactoryBlackboard> {
        match &self.value {
            ServiceBuilderBlackboardCreatorType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryBlackboard::new(
                    PortFactoryBlackboardType::Ipc(
                        this.create_with_attributes(&attributes.0)
                            .map_err(|e| BlackboardCreateError::new_err(format!("{e:?}")))?,
                    ),
                    self.key_type_details.clone(),
                ))
            }
            ServiceBuilderBlackboardCreatorType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryBlackboard::new(
                    PortFactoryBlackboardType::Local(
                        this.create_with_attributes(&attributes.0)
                            .map_err(|e| BlackboardCreateError::new_err(format!("{e:?}")))?,
                    ),
                    self.key_type_details.clone(),
                ))
            }
        }
    }
}

#[pyclass]
/// Builder to open new `MessagingPattern::Blackboard` based `Service`s
pub struct ServiceBuilderBlackboardOpener {}

impl ServiceBuilderBlackboardOpener {
    pub(crate) fn new(value: ServiceBuilderBlackboardOpenerType) -> Self {
        todo!()
    }
}
