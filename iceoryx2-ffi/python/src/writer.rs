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
use iceoryx2_log::fatal_panic;
use pyo3::prelude::*;

use crate::entry_handle_mut::{EntryHandleMut, EntryHandleMutType};
use crate::error::EntryHandleMutError;
use crate::parc::Parc;
use crate::type_detail::TypeDetail;
use crate::type_storage::TypeStorage;
use crate::unique_writer_id::UniqueWriterId;

pub(crate) enum WriterType {
    Ipc(Option<iceoryx2::port::writer::Writer<crate::IpcService, CustomKeyMarker>>),
    Local(Option<iceoryx2::port::writer::Writer<crate::LocalService, CustomKeyMarker>>),
}

#[pyclass]
/// Represents the writing endpoint of a blackboard based communication.
pub struct Writer {
    pub(crate) value: Parc<WriterType>,
    pub(crate) key_type_storage: TypeStorage,
}

#[pymethods]
impl Writer {
    #[getter]
    pub fn __key_type_details(&self) -> Option<Py<PyAny>> {
        self.key_type_storage.clone().value
    }

    #[getter]
    /// Returns the `UniqueWriterId` of the `Writer`
    pub fn id(&self) -> UniqueWriterId {
        match &*self.value.lock() {
            WriterType::Ipc(Some(v)) => UniqueWriterId(v.id()),
            WriterType::Local(Some(v)) => UniqueWriterId(v.id()),
            _ => fatal_panic!(from "Writer::id()",
                    "Accessing a deleted writer."),
        }
    }

    /// Creates an `EntryHandleMut` for direct write access to the value. There can be only one
    /// `EntryHandleMut` per value. On failure it returns `EntryHandleMutError` describing the
    /// failure.
    pub fn __entry(&self, key: usize, value_type_details: TypeDetail) -> PyResult<EntryHandleMut> {
        match &*self.value.lock() {
            WriterType::Ipc(Some(v)) => {
                let entry_handle = unsafe {
                    v.__internal_entry(key as *const u8, &value_type_details.0)
                        .map_err(|e| EntryHandleMutError::new_err(format!("{e:?}")))?
                };
                Ok(EntryHandleMut {
                    value: Parc::new(EntryHandleMutType::Ipc(Some(entry_handle))),
                    value_type_storage: TypeStorage::new(),
                    value_type_details,
                })
            }
            WriterType::Local(Some(v)) => {
                let entry_handle = unsafe {
                    v.__internal_entry(key as *const u8, &value_type_details.0)
                        .map_err(|e| EntryHandleMutError::new_err(format!("{e:?}")))?
                };
                Ok(EntryHandleMut {
                    value: Parc::new(EntryHandleMutType::Local(Some(entry_handle))),
                    value_type_storage: TypeStorage::new(),
                    value_type_details,
                })
            }
            _ => fatal_panic!(from "Writer::entry()",
                    "Accessing a deleted writer."),
        }
    }

    /// Releases the `Writer`.
    ///
    /// After this call the `Writer` is no longer usable!
    pub fn delete(&mut self) {
        match *self.value.lock() {
            WriterType::Ipc(ref mut v) => {
                v.take();
            }
            WriterType::Local(ref mut v) => {
                v.take();
            }
        }
    }
}
