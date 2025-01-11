// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

pub struct Logger {
    _priv: (),
}

impl Logger {
    pub const fn new() -> Self {
        Self { _priv: () }
    }
}

impl crate::Log for Logger {
    fn log(
        &self,
        log_level: crate::LogLevel,
        origin: core::fmt::Arguments,
        formatted_message: core::fmt::Arguments,
    ) {
        let origin = format!("{}", origin);
        match log_level {
            LogLevel::Trace => log::trace!(target: &origin, "{}", formatted_message),
            LogLevel::Debug => log::debug!(target: &origin, "{}", formatted_message),
            LogLevel::Info => log::info!(target: &origin, "{}", formatted_message),
            LogLevel::Warn => log::warn!(target: &origin, "{}", formatted_message),
            LogLevel::Error => log::error!(target: &origin, "{}", formatted_message),
            LogLevel::Fatal => log::error!(target: &origin, "{}", formatted_message),
        }
    }
}
