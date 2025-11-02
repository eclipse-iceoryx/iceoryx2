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

pub(crate) enum EntryHandleMutType {
    Ipc(Option<iceoryx2::port::writer::__InternalEntryHandleMut<crate::IpcService>>), // TODO: Option?
    Local(Option<iceoryx2::port::writer::__InternalEntryHandleMut<crate::LocalService>>),
}

// TODO: unsendable?
#[pyclass(unsendable)]
pub struct EntryHandleMut {
    pub(crate) value: EntryHandleMutType, // TODO: better name
    // pub(crate) value: Parc<EntryHandleMutType>,
    pub(crate) value_type_details: TypeDetail,
}

#[pymethods]
impl EntryHandleMut {
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
