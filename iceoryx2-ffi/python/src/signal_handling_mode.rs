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
/// Defines how signals are handled by constructs that might register a custom
/// `SignalHandler`
pub enum SignalHandlingMode {
    /// The signals `Signal::Interrupt` and `Signal::Terminate` are registered and
    /// handled. If such a `Signal` is received the user will be notified.
    HandleTerminationRequests,
    /// No signal handler will be registered.
    Disabled,
}

#[pymethods]
impl SignalHandlingMode {
    pub fn __str__(&self) -> String {
        format!("{self:?}")
    }
}

impl From<iceoryx2::prelude::SignalHandlingMode> for SignalHandlingMode {
    fn from(value: iceoryx2::prelude::SignalHandlingMode) -> Self {
        match value {
            iceoryx2::prelude::SignalHandlingMode::Disabled => SignalHandlingMode::Disabled,
            iceoryx2::prelude::SignalHandlingMode::HandleTerminationRequests => {
                SignalHandlingMode::HandleTerminationRequests
            }
        }
    }
}

impl From<SignalHandlingMode> for iceoryx2::prelude::SignalHandlingMode {
    fn from(value: SignalHandlingMode) -> Self {
        match value {
            SignalHandlingMode::Disabled => iceoryx2::prelude::SignalHandlingMode::Disabled,
            SignalHandlingMode::HandleTerminationRequests => {
                iceoryx2::prelude::SignalHandlingMode::HandleTerminationRequests
            }
        }
    }
}
