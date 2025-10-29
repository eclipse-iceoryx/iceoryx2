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

use crate::parc::Parc;
use crate::type_storage::TypeStorage;

pub(crate) enum EntryHandleType {
    Ipc(Option<iceoryx2::port::reader::__InternalEntryHandle<crate::IpcService>>),
    Local(Option<iceoryx2::port::reader::__InternalEntryHandle<crate::LocalService>>),
}

// TODO: unsendable?
#[pyclass(unsendable)]
pub struct EntryHandle {
    pub(crate) value: EntryHandleType,
    // pub(crate) key_type_details: TypeStorage,
    // value_type_details?
}
