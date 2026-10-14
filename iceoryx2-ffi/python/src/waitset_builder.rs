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
    signal_handling_mode::SignalHandlingMode,
    waitset::{WaitSet, WaitSetType},
};

pub(crate) enum WaitSetBuilderType {
    Ipc(iceoryx2::prelude::WaitSetBuilder<crate::IpcService>),
    Local(iceoryx2::prelude::WaitSetBuilder<crate::LocalService>),
}

#[pyclass]
/// Creates a new `WaitSet`.
pub struct WaitSetBuilder(pub(crate) Parc<WaitSetBuilderType>);

#[pymethods]
impl WaitSetBuilder {
    /// Defines the `SignalHandlingMode` for the `WaitSet`. It affects the
    /// `WaitSet.wait_and_process` and `WaitSet.wait_and_process_with_timeout` calls
    /// that returns any received ``Signal`` via its `WaitSetRunResult` return value.
    pub fn signal_handling_mode(&mut self, value: &SignalHandlingMode) -> Self {
        match &*self.0.lock() {
            WaitSetBuilderType::Ipc(v) => {
                let this = v.clone();
                let this = this.signal_handling_mode(value.clone().into());
                Self(Parc::new(WaitSetBuilderType::Ipc(this)))
            }
            WaitSetBuilderType::Local(v) => {
                let this = v.clone();
                let this = this.signal_handling_mode(value.clone().into());
                Self(Parc::new(WaitSetBuilderType::Local(this)))
            }
        }
    }

    /// Creates the `WaitSet`.
    pub fn create(&mut self) -> PyResult<WaitSet> {
        match &*self.0.lock() {
            WaitSetBuilderType::Ipc(v) => {
                let this = v.clone();
                Ok(WaitSet(Parc::new(WaitSetType::Ipc(Some(
                    this.create()
                        .map_err(|e| WaitSetCreateError::new_err(format!("{e:?}")))?,
                )))))
            }
            WaitSetBuilderType::Local(v) => {
                let this = v.clone();
                Ok(WaitSet(Parc::new(WaitSetType::Local(Some(
                    this.create()
                        .map_err(|e| WaitSetCreateError::new_err(format!("{e:?}")))?,
                )))))
            }
        }
    }
}
