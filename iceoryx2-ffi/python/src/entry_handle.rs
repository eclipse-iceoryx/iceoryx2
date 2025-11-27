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

use crate::event_id::EventId;
use crate::parc::Parc;
use crate::type_detail::TypeDetail;
use crate::type_storage::TypeStorage;

pub(crate) enum EntryHandleType {
    Ipc(iceoryx2::port::reader::__InternalEntryHandle<crate::IpcService>),
    Local(iceoryx2::port::reader::__InternalEntryHandle<crate::LocalService>),
}

#[pyclass]
pub struct EntryHandle {
    pub(crate) value: Parc<EntryHandleType>,
    pub(crate) value_type_details: TypeDetail,
    pub(crate) value_type_storage: TypeStorage,
    pub(crate) value_ptr: Parc<InternalValueStorage>,
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

    pub fn __set_value_ptr(&mut self) {
        let value_size = self.value_type_details.0.size();
        let value_alignment = self.value_type_details.0.alignment();
        let layout =
            unsafe { core::alloc::Layout::from_size_align_unchecked(value_size, value_alignment) };
        // The corresponding dealloc is implemented in InternalValueStorage::drop().
        let value_buffer = unsafe { std::alloc::alloc(layout) };
        self.value_ptr = Parc::new(InternalValueStorage {
            value_buffer,
            value_type_details: self.value_type_details.clone(),
        });
    }

    // Stores a copy of the blackboard value into `value_ptr` and returns a tuple containing the
    // pointer as usize and the value's generation counter.
    pub fn __get(&self) -> (usize, u64) {
        let value_size = self.value_type_details.0.size();
        let value_alignment = self.value_type_details.0.alignment();
        let value_buffer = (self.value_ptr.lock()).value_buffer;
        let mut generation_counter: u64 = 0;
        let generation_counter_ptr: *mut u64 = &mut generation_counter;
        match &*self.value.lock() {
            EntryHandleType::Ipc(v) => {
                unsafe {
                    v.get(
                        value_buffer,
                        value_size,
                        value_alignment,
                        generation_counter_ptr,
                    )
                };
                (value_buffer as usize, generation_counter)
            }
            EntryHandleType::Local(v) => {
                unsafe {
                    v.get(
                        value_buffer,
                        value_size,
                        value_alignment,
                        generation_counter_ptr,
                    )
                };
                (value_buffer as usize, generation_counter)
            }
        }
    }

    pub fn __is_up_to_date(&self, generation_counter: u64) -> bool {
        match &*self.value.lock() {
            EntryHandleType::Ipc(v) => v.is_up_to_date(generation_counter),
            EntryHandleType::Local(v) => v.is_up_to_date(generation_counter),
        }
    }

    /// Returns an ID corresponding to the entry which can be used in an event based communication
    /// setup.
    #[getter]
    pub fn entry_id(&self) -> EventId {
        match &*self.value.lock() {
            EntryHandleType::Ipc(v) => EventId::new(v.entry_id().as_value()),
            EntryHandleType::Local(v) => EventId::new(v.entry_id().as_value()),
        }
    }
}

pub struct InternalValueStorage {
    pub value_buffer: *mut u8,
    pub value_type_details: TypeDetail,
}

// `InternalValueStorage` is only used as member of `EntryHandle` and the memory that
// `value_buffer` is pointing to remains valid until the `EntryHandle` goes out of scope.
unsafe impl Send for InternalValueStorage {}

impl Drop for InternalValueStorage {
    fn drop(&mut self) {
        unsafe {
            let value_layout = core::alloc::Layout::from_size_align_unchecked(
                self.value_type_details.0.size(),
                self.value_type_details.0.alignment(),
            );
            std::alloc::dealloc(self.value_buffer, value_layout);
        }
    }
}
