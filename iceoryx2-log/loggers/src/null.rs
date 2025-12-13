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

use iceoryx2_log_types::{Log, LogLevel};

/// A logger that discards all log messages.
///
/// This is the default logger used before [`set_logger`](crate::set_logger) is called.
/// It silently discards all log messages, ensuring that logging calls have no effect
/// until a real logger is registered.
#[allow(dead_code)]
pub(crate) struct Logger;

impl Log for Logger {
    #[inline(always)]
    fn log(
        &self,
        _log_level: LogLevel,
        _origin: core::fmt::Arguments,
        _formatted_message: core::fmt::Arguments,
    ) {
        // Intentionally empty - discard all log messages
    }
}
