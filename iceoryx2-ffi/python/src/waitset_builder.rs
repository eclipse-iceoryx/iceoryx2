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

use crate::{
    error::WaitSetCreateError,
    parc::Parc,
    service_type::ServiceType,
    signal_handling_mode::SignalHandlingMode,
    waitset::{WaitSet, WaitSetType},
};

#[derive(Default)]
#[pyclass(str = "{0:?}")]
/// Creates a new `WaitSet`.
pub struct WaitSetBuilder(iceoryx2::prelude::WaitSetBuilder);

#[pymethods]
impl WaitSetBuilder {
    #[staticmethod]
    /// Instantiates a new `WaitSetBuilder`
    pub fn new() -> Self {
        Self::default()
    }

    /// Defines the `SignalHandlingMode` for the `WaitSet`. It affects the
    /// `WaitSet::wait_and_process()` and `WaitSet::wait_and_process_once()` calls
    /// that returns any received `Signal` via its `WaitSetRunResult` return value.
    pub fn signal_handling_mode(&mut self, value: &SignalHandlingMode) -> Self {
        let this = self.0.clone();
        let this = this.signal_handling_mode(value.clone().into());
        Self(this)
    }

    /// Creates the `WaitSet`.
    pub fn create(&mut self, service_type: &ServiceType) -> PyResult<WaitSet> {
        let this = self.0.clone();
        match service_type {
            ServiceType::Ipc => Ok(WaitSet(Parc::new(WaitSetType::Ipc(
                this.create::<crate::IpcService>()
                    .map_err(|e| WaitSetCreateError::new_err(format!("{e:?}")))?,
            )))),
            ServiceType::Local => Ok(WaitSet(Parc::new(WaitSetType::Local(
                this.create::<crate::LocalService>()
                    .map_err(|e| WaitSetCreateError::new_err(format!("{e:?}")))?,
            )))),
        }
    }
}
