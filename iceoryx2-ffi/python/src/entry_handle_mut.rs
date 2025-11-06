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

use iceoryx2_bb_log::fatal_panic;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::entry_value::{EntryValue, EntryValueType};
use crate::entry_value_uninit::{EntryValueUninit, EntryValueUninitType};
use crate::event_id::EventId;
use crate::parc::Parc;
use crate::type_detail::TypeDetail;
use crate::type_storage::TypeStorage;

pub(crate) enum EntryHandleMutType {
    Ipc(Option<iceoryx2::port::writer::__InternalEntryHandleMut<crate::IpcService>>), // TODO: Option?
    Local(Option<iceoryx2::port::writer::__InternalEntryHandleMut<crate::LocalService>>),
}

// TODO: unsendable?
#[pyclass(unsendable)]
pub struct EntryHandleMut {
    pub(crate) value: EntryHandleMutType, // TODO: better name
    // pub(crate) value: Parc<EntryHandleMutType>,
    pub(crate) value_type_storage: TypeStorage,
    pub(crate) value_type_details: TypeDetail,
}

#[pymethods]
impl EntryHandleMut {
    pub fn __set_value_type(&mut self, value: PyObject) {
        self.value_type_storage.value = Some(value)
    }

    #[getter]
    pub fn __value_type(&self) -> TypeDetail {
        self.value_type_details.clone()
    }

    pub fn __get_data_ptr(&self, value_size: usize, value_alignment: usize) -> usize {
        unsafe {
            match &self.value {
                EntryHandleMutType::Ipc(Some(v)) => {
                    v.__internal_get_ptr_to_write_cell(value_size, value_alignment) as usize
                }
                EntryHandleMutType::Local(Some(v)) => {
                    v.__internal_get_ptr_to_write_cell(value_size, value_alignment) as usize
                }
                _ => fatal_panic!(""), // TODO
            }
        }
    }

    pub fn __update_data_ptr(&self) {
        unsafe {
            match &self.value {
                EntryHandleMutType::Ipc(Some(v)) => {
                    v.__internal_update_write_cell();
                }
                EntryHandleMutType::Local(Some(v)) => {
                    v.__internal_update_write_cell();
                }
                _ => fatal_panic!(""), // TODO
            }
        };
    }

    pub fn loan_uninit(&mut self) -> EntryValueUninit {
        // TODO: use parc and lock
        match &mut self.value {
            EntryHandleMutType::Ipc(ref mut v) => {
                let entry_handle_mut = v.take().unwrap();
                let entry_value_uninit = entry_handle_mut.loan_uninit(
                    self.value_type_details.0.size(),
                    self.value_type_details.0.alignment(),
                );
                EntryValueUninit {
                    // value: EntryValueUninitType::Ipc(Some(entry_value_uninit)),
                    // value_type_details: self.value_type_details.clone(),
                    value_type_storage: self.value_type_storage.clone(),
                    entry_value: EntryValue {
                        value: EntryValueType::Ipc(Some(entry_value_uninit)),
                        value_type_details: self.value_type_details.clone(),
                        value_type_storage: self.value_type_storage.clone(),
                    },
                }
            }
            EntryHandleMutType::Local(ref mut v) => {
                let entry_handle_mut = v.take().unwrap();
                let entry_value_uninit = entry_handle_mut.loan_uninit(
                    self.value_type_details.0.size(),
                    self.value_type_details.0.alignment(),
                );
                EntryValueUninit {
                    // value: EntryValueUninitType::Local(Some(entry_value_uninit)),
                    // value_type_details: self.value_type_details.clone(),
                    value_type_storage: self.value_type_storage.clone(),
                    entry_value: EntryValue {
                        value: EntryValueType::Local(Some(entry_value_uninit)),
                        value_type_details: self.value_type_details.clone(),
                        value_type_storage: self.value_type_storage.clone(),
                    },
                }
            }
            _ => fatal_panic!(""), // TODO
        }
    }

    pub fn entry_id(&self) -> EventId {
        match &self.value {
            EntryHandleMutType::Ipc(Some(v)) => EventId::new(v.entry_id().as_value()),
            EntryHandleMutType::Local(Some(v)) => EventId::new(v.entry_id().as_value()),
            _ => fatal_panic!(""), // TODO
        }
    }

    /// Releases the `EntryHandleMut`.
    ///
    /// After this call the `EntryHandleMut` is no longer usable!
    pub fn delete(&mut self) {
        // match *self.value.lock() {
        match self.value {
            EntryHandleMutType::Ipc(ref mut v) => {
                v.take();
            }
            EntryHandleMutType::Local(ref mut v) => {
                v.take();
            }
        }
    }
}
