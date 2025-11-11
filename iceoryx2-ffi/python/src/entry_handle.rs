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

use crate::event_id::EventId;
use crate::parc::Parc;
use crate::type_detail::TypeDetail;
use crate::type_storage::TypeStorage;

pub(crate) enum EntryHandleType {
    Ipc(Option<iceoryx2::port::reader::__InternalEntryHandle<crate::IpcService>>), // TODO: Option?
    Local(Option<iceoryx2::port::reader::__InternalEntryHandle<crate::LocalService>>),
}

#[pyclass]
pub struct EntryHandle {
    pub(crate) value: Parc<EntryHandleType>, // TODO: better name
    pub(crate) value_type_details: TypeDetail,
    pub(crate) value_type_storage: TypeStorage,
}

#[pymethods]
impl EntryHandle {
    #[getter]
    pub fn __value_type(&self) -> Option<Py<PyAny>> {
        self.value_type_storage.clone().value
    }

    pub fn __set_value_type(&mut self, value: PyObject) {
        self.value_type_storage.value = Some(value)
    }

    pub fn __get(&self) -> usize {
        let value_size = self.value_type_details.0.size();
        let value_alignment = self.value_type_details.0.alignment();
        let layout =
            unsafe { core::alloc::Layout::from_size_align_unchecked(value_size, value_alignment) };
        let ptr = unsafe { std::alloc::alloc(layout) };
        match &*self.value.lock() {
            EntryHandleType::Ipc(Some(v)) => {
                unsafe { v.get(ptr, value_size, value_alignment) };
                ptr as usize
            }
            EntryHandleType::Local(Some(v)) => {
                unsafe { v.get(ptr, value_size, value_alignment) };
                ptr as usize
            }
            _ => fatal_panic!(""), // TODO
        }
    }

    pub fn entry_id(&self) -> EventId {
        match &*self.value.lock() {
            EntryHandleType::Ipc(Some(v)) => EventId::new(v.entry_id().as_value()),
            EntryHandleType::Local(Some(v)) => EventId::new(v.entry_id().as_value()),
            _ => fatal_panic!(""), // TODO
        }
    }
}
