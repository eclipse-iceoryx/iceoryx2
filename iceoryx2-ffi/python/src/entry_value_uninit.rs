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

use crate::entry_handle_mut::{EntryHandleMut, EntryHandleMutType};
use crate::entry_value::{EntryValue, EntryValueType};
use crate::type_storage::TypeStorage;

pub(crate) enum EntryValueUninitType {
    Ipc(Option<iceoryx2::port::writer::__InternalEntryValueUninit<crate::IpcService>>), // TODO: Option?
    Local(Option<iceoryx2::port::writer::__InternalEntryValueUninit<crate::LocalService>>),
}

#[pyclass(unsendable)]
pub struct EntryValueUninit {
    // pub(crate) value: EntryValueUninitType, // TODO: better name
    pub(crate) value_type_storage: TypeStorage,
    pub(crate) entry_value: EntryValue,
}

#[pymethods]
impl EntryValueUninit {
    #[getter]
    pub fn __value_type(&self) -> Option<Py<PyAny>> {
        self.value_type_storage.clone().value
    }

    pub fn __get_write_cell(&self) -> usize {
        match &self.entry_value.value {
            EntryValueType::Ipc(Some(v)) => v.write_cell() as usize,
            EntryValueType::Local(Some(v)) => v.write_cell() as usize,
            _ => fatal_panic!(""), // TODO
        }
    }

    pub fn __assume_init(&mut self) -> EntryValue {
        match &mut self.entry_value.value {
            EntryValueType::Ipc(v) => {
                let value_type_details = self.entry_value.value_type_details.clone();
                let entry_value_uninit = v.take().unwrap();
                EntryValue {
                    value: EntryValueType::Ipc(Some(entry_value_uninit)),
                    value_type_details,
                    value_type_storage: self.value_type_storage.clone(),
                }
            }
            EntryValueType::Local(v) => {
                let value_type_details = self.entry_value.value_type_details.clone();
                let entry_value_uninit = v.take().unwrap();
                EntryValue {
                    value: EntryValueType::Local(Some(entry_value_uninit)),
                    value_type_details,
                    value_type_storage: self.value_type_storage.clone(),
                }
            }
        }
    }

    pub fn discard(&mut self) -> EntryHandleMut {
        match &mut self.entry_value.value {
            EntryValueType::Ipc(v) => {
                let entry_value_uninit = v.take().unwrap();
                let entry_handle_mut = entry_value_uninit.discard();
                EntryHandleMut {
                    value: EntryHandleMutType::Ipc(Some(entry_handle_mut)),
                    value_type_storage: self.value_type_storage.clone(),
                    value_type_details: self.entry_value.value_type_details.clone(),
                }
            }
            EntryValueType::Local(v) => {
                let entry_value_uninit = v.take().unwrap();
                let entry_handle_mut = entry_value_uninit.discard();
                EntryHandleMut {
                    value: EntryHandleMutType::Local(Some(entry_handle_mut)),
                    value_type_storage: self.value_type_storage.clone(),
                    value_type_details: self.entry_value.value_type_details.clone(),
                }
            }
        }
    }
}
