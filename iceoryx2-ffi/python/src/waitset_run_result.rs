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
/// States why the `WaitSet::wait_and_process()` method returned.
pub enum WaitSetRunResult {
    /// A termination signal `SIGTERM` was received.
    TerminationRequest,
    /// An interrupt signal `SIGINT` was received.
    Interrupt,
    /// The users callback returned `CallbackProgression::Stop`.
    StopRequest,
    /// All events were handled.
    AllEventsHandled,
}

#[pymethods]
impl WaitSetRunResult {
    pub fn __str__(&self) -> String {
        format!("{self:?}")
    }
}

impl From<iceoryx2::waitset::WaitSetRunResult> for WaitSetRunResult {
    fn from(value: iceoryx2::waitset::WaitSetRunResult) -> Self {
        match value {
            iceoryx2::waitset::WaitSetRunResult::Interrupt => WaitSetRunResult::Interrupt,
            iceoryx2::waitset::WaitSetRunResult::StopRequest => WaitSetRunResult::StopRequest,
            iceoryx2::waitset::WaitSetRunResult::AllEventsHandled => {
                WaitSetRunResult::AllEventsHandled
            }
            iceoryx2::waitset::WaitSetRunResult::TerminationRequest => {
                WaitSetRunResult::TerminationRequest
            }
        }
    }
}

impl From<WaitSetRunResult> for iceoryx2::waitset::WaitSetRunResult {
    fn from(value: WaitSetRunResult) -> Self {
        match value {
            WaitSetRunResult::Interrupt => iceoryx2::waitset::WaitSetRunResult::Interrupt,
            WaitSetRunResult::StopRequest => iceoryx2::waitset::WaitSetRunResult::StopRequest,
            WaitSetRunResult::AllEventsHandled => {
                iceoryx2::waitset::WaitSetRunResult::AllEventsHandled
            }
            WaitSetRunResult::TerminationRequest => {
                iceoryx2::waitset::WaitSetRunResult::TerminationRequest
            }
        }
    }
}
