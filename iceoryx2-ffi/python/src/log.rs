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

use crate::log_level::LogLevel;

#[pyfunction]
/// Sets the current log level. This is ignored for external frameworks like `log` or `tracing`.
/// Here you have to use the log-level settings of that framework.
pub fn set_log_level(value: LogLevel) {
    iceoryx2::prelude::set_log_level(value.into());
}

#[pyfunction]
/// Sets the log level by reading environment variable "IOX2_LOG_LEVEL", and if the environment variable
/// doesn't exit it sets it with a user-defined logging level
pub fn set_log_level_from_env_or(value: LogLevel) {
    iceoryx2::prelude::set_log_level_from_env_or(value.into());
}

#[pyfunction]
/// Sets the log level by reading environment variable "IOX2_LOG_LEVEL" or default it with LogLevel::INFO
pub fn set_log_level_from_env_or_default() {
    iceoryx2::prelude::set_log_level_from_env_or_default();
}
