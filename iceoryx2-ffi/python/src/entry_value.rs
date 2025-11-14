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

use pyo3::prelude::*;

use crate::entry_handle_mut::{EntryHandleMut, EntryHandleMutType};
use crate::parc::Parc;
use crate::type_detail::TypeDetail;
use crate::type_storage::TypeStorage;

pub(crate) enum EntryValueType {
    Ipc(Option<iceoryx2::port::writer::__InternalEntryValueUninit<crate::IpcService>>),
    Local(Option<iceoryx2::port::writer::__InternalEntryValueUninit<crate::LocalService>>),
}

#[pyclass]
pub struct EntryValue {
    pub(crate) value: Parc<EntryValueType>,
    pub(crate) value_type_details: TypeDetail,
    pub(crate) value_type_storage: TypeStorage,
}

#[pymethods]
impl EntryValue {
    /// Makes new value readable for `Reader`s, consumes the `EntryValue` and
    /// returns the original `EntryHandleMut`.
    pub fn update(&mut self) -> EntryHandleMut {
        match &mut *self.value.lock() {
            EntryValueType::Ipc(v) => {
                let entry_value_uninit = v.take().unwrap();
                let entry_handle_mut = entry_value_uninit.update();
                EntryHandleMut {
                    value: Parc::new(EntryHandleMutType::Ipc(Some(entry_handle_mut))),
                    value_type_storage: self.value_type_storage.clone(),
                    value_type_details: self.value_type_details.clone(),
                }
            }
            EntryValueType::Local(v) => {
                let entry_value_uninit = v.take().unwrap();
                let entry_handle_mut = entry_value_uninit.update();
                EntryHandleMut {
                    value: Parc::new(EntryHandleMutType::Local(Some(entry_handle_mut))),
                    value_type_storage: self.value_type_storage.clone(),
                    value_type_details: self.value_type_details.clone(),
                }
            }
        }
    }

    /// Discards the `EntryValue` and returns the original `EntryHandleMut`.
    pub fn discard(&mut self) -> EntryHandleMut {
        match &mut *self.value.lock() {
            EntryValueType::Ipc(v) => {
                let entry_value_uninit = v.take().unwrap();
                let entry_handle_mut = entry_value_uninit.discard();
                EntryHandleMut {
                    value: Parc::new(EntryHandleMutType::Ipc(Some(entry_handle_mut))),
                    value_type_storage: self.value_type_storage.clone(),
                    value_type_details: self.value_type_details.clone(),
                }
            }
            EntryValueType::Local(v) => {
                let entry_value_uninit = v.take().unwrap();
                let entry_handle_mut = entry_value_uninit.discard();
                EntryHandleMut {
                    value: Parc::new(EntryHandleMutType::Local(Some(entry_handle_mut))),
                    value_type_storage: self.value_type_storage.clone(),
                    value_type_details: self.value_type_details.clone(),
                }
            }
        }
    }
}
