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

#[pyclass(eq, eq_int)]
#[derive(PartialEq, Clone, Debug)]
/// Defines the type of the `Service` and what kind of resources and operating system mechanisms
/// it shall use.
pub enum ServiceType {
    /// Optimized for inter-thread communication does not not support inter-process communication.
    Local,
    /// Optimized for inter-process communication.
    Ipc,
}

pub(crate) type IpcService = iceoryx2::prelude::ipc_threadsafe::Service;
pub(crate) type LocalService = iceoryx2::prelude::local_threadsafe::Service;

#[pymethods]
impl ServiceType {
    pub fn __str__(&self) -> String {
        format!("{self:?}")
    }
}
