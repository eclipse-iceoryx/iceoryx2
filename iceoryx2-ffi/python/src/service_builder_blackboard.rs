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

use std::alloc::Layout;
use std::ptr::copy_nonoverlapping;

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

pub(crate) enum ServiceBuilderBlackboardCreatorType {
    Ipc(Option<IpcCreator>),
    Local(Option<LocalCreator>),
}

pub(crate) enum ServiceBuilderBlackboardOpenerType {
    Ipc(Option<IpcOpener>),
    Local(Option<LocalOpener>),
}

// `ServiceBuilderBlackboardCreatorType` is Send when `BuilderInternals::value_writer` and
// `BuilderInternals::internal_value_cleanup_callback` are Send. They are if the captured variables
// can be Send. To achieve this, the memory the value pointer argument of `__add()` points to is
// copied to `value_buffer` on the heap and therefore lives long enough, i.e. until
// `Creator::create()` returns.
unsafe impl Send for ServiceBuilderBlackboardCreatorType {}
unsafe impl Send for ServiceBuilderBlackboardOpenerType {}

#[pyclass]
/// Builder to create new `MessagingPattern::Blackboard` based `Service`s
pub struct ServiceBuilderBlackboardCreator {
    pub(crate) value: Parc<ServiceBuilderBlackboardCreatorType>,
    pub key_type_storage: TypeStorage,
}

impl ServiceBuilderBlackboardCreator {
    pub(crate) fn new(value: ServiceBuilderBlackboardCreatorType) -> Self {
        Self {
            value: Parc::new(value),
            key_type_storage: TypeStorage::new(),
        }
    }

    fn clone_ipc(&self, builder: IpcCreator) -> Self {
        Self {
            value: Parc::new(ServiceBuilderBlackboardCreatorType::Ipc(Some(builder))),
            key_type_storage: self.key_type_storage.clone(),
        }
    }

    fn clone_local(&self, builder: LocalCreator) -> Self {
        Self {
            value: Parc::new(ServiceBuilderBlackboardCreatorType::Local(Some(builder))),
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
    pub fn __set_key_type_details(&mut self, value: &TypeDetail) -> Self {
        match &mut *self.value.lock() {
            ServiceBuilderBlackboardCreatorType::Ipc(ref mut v) => {
                let this = v.take().unwrap();
                let this = unsafe { this.__internal_set_key_type_details(&value.0) };
                self.clone_ipc(this)
            }
            ServiceBuilderBlackboardCreatorType::Local(ref mut v) => {
                let this = v.take().unwrap();
                let this = unsafe { this.__internal_set_key_type_details(&value.0) };
                self.clone_local(this)
            }
        }
    }

    /// Defines the key eq comparison function needed to store and retrieve keys in the
    /// blackboard.
    pub fn __set_key_eq_cmp_func(&mut self, key_eq_func: PyObject) -> Self {
        match &mut *self.value.lock() {
            ServiceBuilderBlackboardCreatorType::Ipc(ref mut v) => {
                let this = v.take().unwrap();
                let eq_func = Box::new(move |lhs: *const u8, rhs: *const u8| -> bool {
                    Python::with_gil(|py| {
                        let result = key_eq_func.call1(py, (lhs as usize, rhs as usize)).unwrap();
                        result
                            .extract::<bool>(py)
                            .expect("Return type of key eq comparison function must be bool.")
                    })
                });
                let this = unsafe {
                    this.__internal_set_key_eq_cmp_func(Box::new(move |lhs, rhs| {
                        KeyMemory::<MAX_BLACKBOARD_KEY_SIZE>::key_eq_comparison(lhs, rhs, &*eq_func)
                    }))
                };
                self.clone_ipc(this)
            }
            ServiceBuilderBlackboardCreatorType::Local(ref mut v) => {
                let this = v.take().unwrap();
                let eq_func = Box::new(move |lhs: *const u8, rhs: *const u8| -> bool {
                    Python::with_gil(|py| {
                        let result = key_eq_func.call1(py, (lhs as usize, rhs as usize)).unwrap();
                        result
                            .extract::<bool>(py)
                            .expect("Return type of key eq comparison function must be bool.")
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
    pub fn max_readers(&mut self, value: usize) -> Self {
        match &mut *self.value.lock() {
            ServiceBuilderBlackboardCreatorType::Ipc(ref mut v) => {
                let this = v.take().unwrap();
                let this = this.max_readers(value);
                self.clone_ipc(this)
            }
            ServiceBuilderBlackboardCreatorType::Local(ref mut v) => {
                let this = v.take().unwrap();
                let this = this.max_readers(value);
                self.clone_local(this)
            }
        }
    }

    /// Defines how many `Node`s shall be able to open the `Service` in parallel.
    pub fn max_nodes(&mut self, value: usize) -> Self {
        match &mut *self.value.lock() {
            ServiceBuilderBlackboardCreatorType::Ipc(ref mut v) => {
                let this = v.take().unwrap();
                let this = this.max_nodes(value);
                self.clone_ipc(this)
            }
            ServiceBuilderBlackboardCreatorType::Local(ref mut v) => {
                let this = v.take().unwrap();
                let this = this.max_nodes(value);
                self.clone_local(this)
            }
        }
    }

    /// Adds key-value pairs to the blackboard.
    pub fn __add(&mut self, key: usize, value: usize, value_details: &TypeDetail) -> Self {
        let value_layout = unsafe {
            Layout::from_size_align_unchecked(value_details.0.size(), value_details.0.alignment())
        };
        let value_buffer = unsafe { std::alloc::alloc(value_layout) };
        unsafe { copy_nonoverlapping(value as *const u8, value_buffer, value_details.0.size()) };
        match &mut *self.value.lock() {
            ServiceBuilderBlackboardCreatorType::Ipc(ref mut v) => {
                let this = v.take().unwrap();
                let this = unsafe {
                    this.__internal_add(
                        key as *const u8,
                        value_buffer,
                        value_details.0.clone(),
                        Box::new(move || {
                            std::alloc::dealloc(value_buffer, value_layout);
                        }),
                    )
                };
                self.clone_ipc(this)
            }
            ServiceBuilderBlackboardCreatorType::Local(ref mut v) => {
                let this = v.take().unwrap();
                let this = unsafe {
                    this.__internal_add(
                        key as *const u8,
                        value_buffer,
                        value_details.0.clone(),
                        Box::new(move || {
                            std::alloc::dealloc(value_buffer, value_layout);
                        }),
                    )
                };
                self.clone_local(this)
            }
        }
    }

    /// Creates a new `Service`.
    pub fn create(&mut self) -> PyResult<PortFactoryBlackboard> {
        match &mut *self.value.lock() {
            ServiceBuilderBlackboardCreatorType::Ipc(ref mut v) => {
                let this = v.take().unwrap();
                Ok(PortFactoryBlackboard::new(
                    PortFactoryBlackboardType::Ipc(Some(
                        this.create()
                            .map_err(|e| BlackboardCreateError::new_err(format!("{e:?}")))?,
                    )),
                    self.key_type_storage.clone(),
                ))
            }
            ServiceBuilderBlackboardCreatorType::Local(ref mut v) => {
                let this = v.take().unwrap();
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
        &mut self,
        attributes: &AttributeSpecifier,
    ) -> PyResult<PortFactoryBlackboard> {
        match &mut *self.value.lock() {
            ServiceBuilderBlackboardCreatorType::Ipc(ref mut v) => {
                let this = v.take().unwrap();
                Ok(PortFactoryBlackboard::new(
                    PortFactoryBlackboardType::Ipc(Some(
                        this.create_with_attributes(&attributes.0)
                            .map_err(|e| BlackboardCreateError::new_err(format!("{e:?}")))?,
                    )),
                    self.key_type_storage.clone(),
                ))
            }
            ServiceBuilderBlackboardCreatorType::Local(ref mut v) => {
                let this = v.take().unwrap();
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

#[pyclass]
/// Builder to open new `MessagingPattern::Blackboard` based `Service`s
pub struct ServiceBuilderBlackboardOpener {
    pub(crate) value: Parc<ServiceBuilderBlackboardOpenerType>,
    pub key_type_details: TypeStorage,
}

impl ServiceBuilderBlackboardOpener {
    pub(crate) fn new(value: ServiceBuilderBlackboardOpenerType) -> Self {
        Self {
            value: Parc::new(value),
            key_type_details: TypeStorage::new(),
        }
    }

    fn clone_ipc(&self, builder: IpcOpener) -> Self {
        Self {
            value: Parc::new(ServiceBuilderBlackboardOpenerType::Ipc(Some(builder))),
            key_type_details: self.key_type_details.clone(),
        }
    }

    fn clone_local(&self, builder: LocalOpener) -> Self {
        Self {
            value: Parc::new(ServiceBuilderBlackboardOpenerType::Local(Some(builder))),
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
    pub fn __set_key_type_details(&mut self, value: &TypeDetail) -> Self {
        match &mut *self.value.lock() {
            ServiceBuilderBlackboardOpenerType::Ipc(ref mut v) => {
                let this = v.take().unwrap();
                let this = unsafe { this.__internal_set_key_type_details(&value.0) };
                self.clone_ipc(this)
            }
            ServiceBuilderBlackboardOpenerType::Local(ref mut v) => {
                let this = v.take().unwrap();
                let this = unsafe { this.__internal_set_key_type_details(&value.0) };
                self.clone_local(this)
            }
        }
    }

    /// Defines how many `Reader`s must be at least supported.
    pub fn max_readers(&mut self, value: usize) -> Self {
        match &mut *self.value.lock() {
            ServiceBuilderBlackboardOpenerType::Ipc(ref mut v) => {
                let this = v.take().unwrap();
                let this = this.max_readers(value);
                self.clone_ipc(this)
            }
            ServiceBuilderBlackboardOpenerType::Local(ref mut v) => {
                let this = v.take().unwrap();
                let this = this.max_readers(value);
                self.clone_local(this)
            }
        }
    }

    /// Defines how many `Node`s must be at least supported.
    pub fn max_nodes(&mut self, value: usize) -> Self {
        match &mut *self.value.lock() {
            ServiceBuilderBlackboardOpenerType::Ipc(ref mut v) => {
                let this = v.take().unwrap();
                let this = this.max_nodes(value);
                self.clone_ipc(this)
            }
            ServiceBuilderBlackboardOpenerType::Local(ref mut v) => {
                let this = v.take().unwrap();
                let this = this.max_nodes(value);
                self.clone_local(this)
            }
        }
    }

    /// Opens an existing `Service`.
    pub fn open(&mut self) -> PyResult<PortFactoryBlackboard> {
        match &mut *self.value.lock() {
            ServiceBuilderBlackboardOpenerType::Ipc(ref mut v) => {
                let this = v.take().unwrap();
                Ok(PortFactoryBlackboard::new(
                    PortFactoryBlackboardType::Ipc(Some(
                        this.open()
                            .map_err(|e| BlackboardOpenError::new_err(format!("{e:?}")))?,
                    )),
                    self.key_type_details.clone(),
                ))
            }
            ServiceBuilderBlackboardOpenerType::Local(ref mut v) => {
                let this = v.take().unwrap();
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
        &mut self,
        verifier: &AttributeVerifier,
    ) -> PyResult<PortFactoryBlackboard> {
        match &mut *self.value.lock() {
            ServiceBuilderBlackboardOpenerType::Ipc(ref mut v) => {
                let this = v.take().unwrap();
                Ok(PortFactoryBlackboard::new(
                    PortFactoryBlackboardType::Ipc(Some(
                        this.open_with_attributes(&verifier.0)
                            .map_err(|e| BlackboardOpenError::new_err(format!("{e:?}")))?,
                    )),
                    self.key_type_details.clone(),
                ))
            }
            ServiceBuilderBlackboardOpenerType::Local(ref mut v) => {
                let this = v.take().unwrap();
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
