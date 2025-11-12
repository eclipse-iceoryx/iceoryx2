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

use iceoryx2::constants::MAX_BLACKBOARD_KEY_SIZE;
use iceoryx2::service::builder::blackboard::KeyMemory;
use iceoryx2::service::builder::CustomKeyMarker;
use pyo3::prelude::*;

use crate::attribute_specifier::AttributeSpecifier;
use crate::attribute_verifier::AttributeVerifier;
use crate::error::{BlackboardCreateError, BlackboardOpenError};
use crate::parc::Parc;
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

// TODO: remove unsendable
#[pyclass(unsendable)]
/// Builder to create new `MessagingPattern::Blackboard` based `Service`s
pub struct ServiceBuilderBlackboardCreator {
    // pub(crate) value: Parc<ServiceBuilderBlackboardCreatorType>,
    pub(crate) value: ServiceBuilderBlackboardCreatorType,
    pub key_type_storage: TypeStorage,
}

impl ServiceBuilderBlackboardCreator {
    pub(crate) fn new(value: ServiceBuilderBlackboardCreatorType) -> Self {
        Self {
            value,
            key_type_storage: TypeStorage::new(),
        }
    }

    fn clone_ipc(&self, builder: IpcCreator) -> Self {
        Self {
            value: ServiceBuilderBlackboardCreatorType::Ipc(builder),
            key_type_storage: self.key_type_storage.clone(),
        }
    }

    fn clone_local(&self, builder: LocalCreator) -> Self {
        Self {
            value: ServiceBuilderBlackboardCreatorType::Local(builder),
            key_type_storage: self.key_type_storage.clone(),
        }
    }
}

#[pymethods]
impl ServiceBuilderBlackboardCreator {
    #[getter]
    pub fn __key_type_details(&self) -> Option<Py<PyAny>> {
        self.key_type_storage.clone().value
    }

    pub fn __set_key_type(&mut self, value: PyObject) {
        self.key_type_storage.value = Some(value)
    }

    /// Defines the key type. To be able to connect to a `Service`, the `TypeDetail` must be
    /// indentical in all participants since the communication is always strongly typed.
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

    /// Defines the key eq comparison function needed to store and retrieve keys in the
    /// blackboard.
    pub fn __set_key_eq_cmp_func(&self, key_eq_func: PyObject) -> Self {
        match &self.value {
            ServiceBuilderBlackboardCreatorType::Ipc(v) => {
                let this = v.clone();
                let eq_func = Box::new(move |lhs: *const u8, rhs: *const u8| -> bool {
                    Python::with_gil(|py| {
                        let x = key_eq_func.call1(py, (lhs as usize, rhs as usize)).unwrap();
                        x.extract::<bool>(py).expect("")
                    })
                });
                let this = unsafe {
                    this.__internal_set_key_eq_cmp_func(Box::new(move |lhs, rhs| {
                        KeyMemory::<MAX_BLACKBOARD_KEY_SIZE>::key_eq_comparison(lhs, rhs, &*eq_func)
                    }))
                };
                self.clone_ipc(this)
            }
            ServiceBuilderBlackboardCreatorType::Local(v) => {
                let this = v.clone();
                let eq_func = Box::new(move |lhs: *const u8, rhs: *const u8| -> bool {
                    Python::with_gil(|py| {
                        let x = key_eq_func.call1(py, (lhs as usize, rhs as usize)).unwrap();
                        x.extract::<bool>(py).expect("")
                    })
                });
                let this = unsafe {
                    this.__internal_set_key_eq_cmp_func(Box::new(move |lhs, rhs| {
                        KeyMemory::<MAX_BLACKBOARD_KEY_SIZE>::key_eq_comparison(lhs, rhs, &*eq_func)
                    }))
                };
                self.clone_local(this)
            }
        }
    }

    /// Defines how many `Reader`s shall be supported at most.
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

    /// Defines how many `Node`s shall be able to open the `Service` in parallel.
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

    /// Adds key-value pairs to the blackboard.
    pub fn __add(&mut self, key: usize, value: usize, value_details: &TypeDetail) -> Self {
        match &self.value {
            ServiceBuilderBlackboardCreatorType::Ipc(v) => {
                let this = v.clone();
                let this = unsafe {
                    this.__internal_add(
                        key as *const u8,
                        value as *mut u8,
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
                        key as *const u8,
                        value as *mut u8,
                        value_details.0.clone(),
                        Box::new(|| {}),
                    )
                };
                self.clone_local(this)
            }
        }
    }

    /// Creates a new `Service`.
    pub fn create(&self) -> PyResult<PortFactoryBlackboard> {
        match &self.value {
            ServiceBuilderBlackboardCreatorType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryBlackboard::new(
                    PortFactoryBlackboardType::Ipc(Some(
                        this.create()
                            .map_err(|e| BlackboardCreateError::new_err(format!("{e:?}")))?,
                    )),
                    self.key_type_storage.clone(),
                ))
            }
            ServiceBuilderBlackboardCreatorType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryBlackboard::new(
                    PortFactoryBlackboardType::Local(Some(
                        this.create()
                            .map_err(|e| BlackboardCreateError::new_err(format!("{e:?}")))?,
                    )),
                    self.key_type_storage.clone(),
                ))
            }
        }
    }

    /// Creates a new `Service` with a set of attributes.
    pub fn create_with_attributes(
        &self,
        attributes: &AttributeSpecifier,
    ) -> PyResult<PortFactoryBlackboard> {
        match &self.value {
            ServiceBuilderBlackboardCreatorType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryBlackboard::new(
                    PortFactoryBlackboardType::Ipc(Some(
                        this.create_with_attributes(&attributes.0)
                            .map_err(|e| BlackboardCreateError::new_err(format!("{e:?}")))?,
                    )),
                    self.key_type_storage.clone(),
                ))
            }
            ServiceBuilderBlackboardCreatorType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryBlackboard::new(
                    PortFactoryBlackboardType::Local(Some(
                        this.create_with_attributes(&attributes.0)
                            .map_err(|e| BlackboardCreateError::new_err(format!("{e:?}")))?,
                    )),
                    self.key_type_storage.clone(),
                ))
            }
        }
    }
}

// TODO: remove unsendable
#[pyclass(unsendable)]
/// Builder to open new `MessagingPattern::Blackboard` based `Service`s
pub struct ServiceBuilderBlackboardOpener {
    pub(crate) value: ServiceBuilderBlackboardOpenerType,
    pub key_type_details: TypeStorage,
}

impl ServiceBuilderBlackboardOpener {
    pub(crate) fn new(value: ServiceBuilderBlackboardOpenerType) -> Self {
        Self {
            value,
            key_type_details: TypeStorage::new(),
        }
    }

    fn clone_ipc(&self, builder: IpcOpener) -> Self {
        Self {
            value: ServiceBuilderBlackboardOpenerType::Ipc(builder),
            key_type_details: self.key_type_details.clone(),
        }
    }

    fn clone_local(&self, builder: LocalOpener) -> Self {
        Self {
            value: ServiceBuilderBlackboardOpenerType::Local(builder),
            key_type_details: self.key_type_details.clone(),
        }
    }
}

#[pymethods]
impl ServiceBuilderBlackboardOpener {
    #[getter]
    pub fn __key_type_details(&self) -> Option<Py<PyAny>> {
        self.key_type_details.clone().value
    }

    pub fn __set_key_type(&mut self, value: PyObject) {
        self.key_type_details.value = Some(value)
    }

    /// Defines the key type. To be able to connect to a `Service`, the `TypeDetail` must be
    /// indentical in all participants since the communication is always strongly typed.
    pub fn __set_key_type_details(&self, value: &TypeDetail) -> Self {
        match &self.value {
            ServiceBuilderBlackboardOpenerType::Ipc(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_key_type_details(&value.0) };
                self.clone_ipc(this)
            }
            ServiceBuilderBlackboardOpenerType::Local(v) => {
                let this = v.clone();
                let this = unsafe { this.__internal_set_key_type_details(&value.0) };
                self.clone_local(this)
            }
        }
    }

    /// Defines how many `Reader`s must be at least supported.
    pub fn max_readers(&self, value: usize) -> Self {
        match &self.value {
            ServiceBuilderBlackboardOpenerType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_readers(value);
                self.clone_ipc(this)
            }
            ServiceBuilderBlackboardOpenerType::Local(v) => {
                let this = v.clone();
                let this = this.max_readers(value);
                self.clone_local(this)
            }
        }
    }

    /// Defines how many `Node`s must be at least supported.
    pub fn max_nodes(&self, value: usize) -> Self {
        match &self.value {
            ServiceBuilderBlackboardOpenerType::Ipc(v) => {
                let this = v.clone();
                let this = this.max_nodes(value);
                self.clone_ipc(this)
            }
            ServiceBuilderBlackboardOpenerType::Local(v) => {
                let this = v.clone();
                let this = this.max_nodes(value);
                self.clone_local(this)
            }
        }
    }

    /// Opens an existing `Service`.
    pub fn open(&self) -> PyResult<PortFactoryBlackboard> {
        match &self.value {
            ServiceBuilderBlackboardOpenerType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryBlackboard::new(
                    PortFactoryBlackboardType::Ipc(Some(
                        this.open()
                            .map_err(|e| BlackboardOpenError::new_err(format!("{e:?}")))?,
                    )),
                    self.key_type_details.clone(),
                ))
            }
            ServiceBuilderBlackboardOpenerType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryBlackboard::new(
                    PortFactoryBlackboardType::Local(Some(
                        this.open()
                            .map_err(|e| BlackboardOpenError::new_err(format!("{e:?}")))?,
                    )),
                    self.key_type_details.clone(),
                ))
            }
        }
    }

    /// Opens an existing `Service` with attribute requirements. If the defined attribute
    /// requirements are not satisfied the open process will fail.
    pub fn open_with_attributes(
        &self,
        verifier: &AttributeVerifier,
    ) -> PyResult<PortFactoryBlackboard> {
        match &self.value {
            ServiceBuilderBlackboardOpenerType::Ipc(v) => {
                let this = v.clone();
                Ok(PortFactoryBlackboard::new(
                    PortFactoryBlackboardType::Ipc(Some(
                        this.open_with_attributes(&verifier.0)
                            .map_err(|e| BlackboardOpenError::new_err(format!("{e:?}")))?,
                    )),
                    self.key_type_details.clone(),
                ))
            }
            ServiceBuilderBlackboardOpenerType::Local(v) => {
                let this = v.clone();
                Ok(PortFactoryBlackboard::new(
                    PortFactoryBlackboardType::Local(Some(
                        this.open_with_attributes(&verifier.0)
                            .map_err(|e| BlackboardOpenError::new_err(format!("{e:?}")))?,
                    )),
                    self.key_type_details.clone(),
                ))
            }
        }
    }
}
