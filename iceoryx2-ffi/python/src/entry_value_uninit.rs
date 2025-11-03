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

pub(crate) enum EntryValueUninitType {
    Ipc(iceoryx2::port::writer::__InternalEntryValueUninit<crate::IpcService>),
    Local(iceoryx2::port::writer::__InternalEntryValueUninit<crate::LocalService>),
}

#[pyclass(unsendable)]
pub struct EntryValueUninit {
    pub(crate) value: EntryValueUninitType, // TODO: better name
}
