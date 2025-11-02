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

use crate::event_id::EventId;
use crate::parc::Parc;
use crate::type_detail::TypeDetail;

pub(crate) enum EntryHandleType {
    Ipc(Option<iceoryx2::port::reader::__InternalEntryHandle<crate::IpcService>>), // TODO: Option?
    Local(Option<iceoryx2::port::reader::__InternalEntryHandle<crate::LocalService>>),
}

// TODO: unsendable?
#[pyclass(unsendable)]
pub struct EntryHandle {
    pub(crate) value: EntryHandleType, // TODO: better name
    // pub(crate) value: Parc<EntryHandleType>,
    pub(crate) value_type_details: TypeDetail,
}

#[pymethods]
impl EntryHandle {
    pub fn get(&self) -> Py<PyBytes> {
        let value_size = self.value_type_details.0.size();
        let value_alignment = self.value_type_details.0.alignment();
        let layout =
            unsafe { core::alloc::Layout::from_size_align_unchecked(value_size, value_alignment) };
        let ptr = unsafe { std::alloc::alloc(layout) };
        match &self.value {
            EntryHandleType::Ipc(Some(v)) => {
                unsafe { v.get(ptr, value_size, value_alignment) };
                Python::with_gil(|py| {
                    let value_bytes = unsafe { PyBytes::from_ptr(py, ptr, value_size) };
                    value_bytes.unbind()
                })
            }
            EntryHandleType::Local(Some(v)) => {
                unsafe { v.get(ptr, value_size, value_alignment) };
                Python::with_gil(|py| {
                    let value_bytes = unsafe { PyBytes::from_ptr(py, ptr, value_size) };
                    value_bytes.unbind()
                })
            }
            _ => fatal_panic!(""), // TODO
        }
    }

    // extensions: get(value: bytes, value_type: Type[t])
    // __get(value: PyBytes, value_type: &TypeDetail)
    //      call get with value.as_bytes()
    // user: EntryHandle.get(value_bytes, c_int32)
    //      value_bytes = bytes(value.size)
    //      convert to original type with from_bytes (see service_builder_blackboard_tests.py)
    //
    // Question: can this be changed to
    // user: EntryHandle.get(c_int32)
    // extensions: get(value_type: Type[T])
    // __get(value_type: &TypeDetail)
    //      alloc memory with Layout from value_type
    //      call get
    //      create PyBytes from ptr
    //
    // Question: if this works, can we get rid of value_type argument in extension by storing the the type information in EntryHandle?

    pub fn entry_id(&self) -> EventId {
        match &self.value {
            EntryHandleType::Ipc(Some(v)) => EventId::new(v.entry_id().as_value()),
            EntryHandleType::Local(Some(v)) => EventId::new(v.entry_id().as_value()),
            _ => fatal_panic!(""), // TODO
        }
    }
}
