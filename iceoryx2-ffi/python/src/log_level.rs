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
/// Describes the log level.
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

#[pymethods]
impl LogLevel {
    pub fn __str__(&self) -> String {
        format!("{self:?}")
    }
}

impl From<iceoryx2::prelude::LogLevel> for LogLevel {
    fn from(value: iceoryx2::prelude::LogLevel) -> Self {
        match value {
            iceoryx2::prelude::LogLevel::Trace => LogLevel::Trace,
            iceoryx2::prelude::LogLevel::Debug => LogLevel::Debug,
            iceoryx2::prelude::LogLevel::Info => LogLevel::Info,
            iceoryx2::prelude::LogLevel::Warn => LogLevel::Warn,
            iceoryx2::prelude::LogLevel::Error => LogLevel::Error,
            iceoryx2::prelude::LogLevel::Fatal => LogLevel::Fatal,
        }
    }
}

impl From<LogLevel> for iceoryx2::prelude::LogLevel {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => iceoryx2::prelude::LogLevel::Trace,
            LogLevel::Debug => iceoryx2::prelude::LogLevel::Debug,
            LogLevel::Info => iceoryx2::prelude::LogLevel::Info,
            LogLevel::Warn => iceoryx2::prelude::LogLevel::Warn,
            LogLevel::Error => iceoryx2::prelude::LogLevel::Error,
            LogLevel::Fatal => iceoryx2::prelude::LogLevel::Fatal,
        }
    }
}
