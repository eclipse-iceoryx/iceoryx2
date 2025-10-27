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

use crate::LogLevel;

pub struct Logger;

impl Logger {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for Logger {
    fn default() -> Self {
        Logger::new()
    }
}

impl crate::Log for Logger {
    fn log(
        &self,
        _log_level: LogLevel,
        _origin: core::fmt::Arguments,
        _formatted_message: core::fmt::Arguments,
    ) {
        // Do nothing
    }
}
