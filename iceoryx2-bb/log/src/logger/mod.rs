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

//! Trait which can be implemented by logger, see [`crate::logger::console::Logger`]
//! for instance.

pub mod buffer;
pub mod console;
#[cfg(feature = "logger_log")]
pub mod log;
#[cfg(feature = "logger_tracing")]
pub mod tracing;

use std::fmt::Arguments;

use crate::LogLevel;

pub trait Logger: Send + Sync {
    /// logs a message
    fn log(&self, log_level: LogLevel, origin: Arguments, formatted_message: Arguments);
}
