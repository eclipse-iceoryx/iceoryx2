// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use core::ffi::c_int;

use iceoryx2::signal_handling_mode::SignalHandlingMode;

use super::{IntoCInt, IOX2_OK};

/// Defines how signals are handled by constructs that might register a custom
/// signal handler.
#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_signal_handling_mode_e {
    /// The signals `SIGINT` and `SIGTERM` are registered and handled. If such a signal is received
    /// the user will be notified.
    HANDLE_TERMINATION_REQUESTS = IOX2_OK as isize + 1,
    /// No signal handler will be registered.
    DISABLED,
}

impl IntoCInt for SignalHandlingMode {
    fn into_c_int(self) -> c_int {
        (match self {
            SignalHandlingMode::HandleTerminationRequests => {
                iox2_signal_handling_mode_e::HANDLE_TERMINATION_REQUESTS
            }
            SignalHandlingMode::Disabled => iox2_signal_handling_mode_e::DISABLED,
        }) as c_int
    }
}

impl From<iox2_signal_handling_mode_e> for SignalHandlingMode {
    fn from(value: iox2_signal_handling_mode_e) -> Self {
        match value {
            iox2_signal_handling_mode_e::HANDLE_TERMINATION_REQUESTS => {
                SignalHandlingMode::HandleTerminationRequests
            }
            iox2_signal_handling_mode_e::DISABLED => SignalHandlingMode::Disabled,
        }
    }
}

impl From<SignalHandlingMode> for iox2_signal_handling_mode_e {
    fn from(value: SignalHandlingMode) -> Self {
        match value {
            SignalHandlingMode::Disabled => iox2_signal_handling_mode_e::DISABLED,
            SignalHandlingMode::HandleTerminationRequests => {
                iox2_signal_handling_mode_e::HANDLE_TERMINATION_REQUESTS
            }
        }
    }
}
