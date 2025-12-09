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

//! Concrete logger backend implementations for iceoryx2.
//!
//! This crate provides logger implementations that can be registered with
//! `iceoryx2_log` using `iceoryx2_log::set_logger()`. Each logger is
//! feature-gated, allowing users to include only what they need.
//!
//! # Architecture
//!
//! The iceoryx2 logging system is split into two crates:
//! - **`iceoryx2_log`**: The frontend providing the logging API and macros
//! - **`iceoryx2_loggers`**: This crate, providing selectable logger backends
//!
//! This separation keeps the logging API lightweight and platform-agnostic
//! while allowing flexible backend selection at runtime.
//!
//! See `iceoryx2_log` for usage examples and the complete logging API.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

#[cfg(all(feature = "logger_buffer", feature = "logger_file"))]
compile_error!("Features 'logger_buffer' and 'logger_file' are mutually exclusive");

#[cfg(all(feature = "logger_buffer", feature = "logger_console"))]
compile_error!("Features 'logger_buffer' and 'logger_console' are mutually exclusive");

#[cfg(all(feature = "logger_buffer", feature = "logger_log"))]
compile_error!("Features 'logger_buffer' and 'logger_log' are mutually exclusive");

#[cfg(all(feature = "logger_buffer", feature = "logger_tracing"))]
compile_error!("Features 'logger_buffer' and 'logger_tracing' are mutually exclusive");

#[cfg(all(feature = "logger_file", feature = "logger_console"))]
compile_error!("Features 'logger_file' and 'logger_console' are mutually exclusive");

#[cfg(all(feature = "logger_file", feature = "logger_log"))]
compile_error!("Features 'logger_file' and 'logger_log' are mutually exclusive");

#[cfg(all(feature = "logger_file", feature = "logger_tracing"))]
compile_error!("Features 'logger_file' and 'logger_tracing' are mutually exclusive");

#[cfg(all(feature = "logger_console", feature = "logger_log"))]
compile_error!("Features 'logger_console' and 'logger_log' are mutually exclusive");

#[cfg(all(feature = "logger_console", feature = "logger_tracing"))]
compile_error!("Features 'logger_console' and 'logger_tracing' are mutually exclusive");

#[cfg(all(feature = "logger_log", feature = "logger_tracing"))]
compile_error!("Features 'logger_log' and 'logger_tracing' are mutually exclusive");

use iceoryx2_log_types::Log;

#[cfg(feature = "logger_buffer")]
mod buffer;
#[cfg(feature = "logger_console")]
mod console;
#[cfg(feature = "logger_file")]
mod file;
#[cfg(feature = "logger_log")]
mod log;
#[cfg(feature = "logger_tracing")]
mod tracing;

mod null;

extern crate alloc;

#[cfg(feature = "logger_console")]
#[no_mangle]
pub extern "Rust" fn __internal_default_logger() -> &'static dyn Log {
    {
        static CONSOLE_LOGGER: console::Logger = console::Logger::new();
        &CONSOLE_LOGGER
    }
}

#[cfg(feature = "logger_buffer")]
#[no_mangle]
pub extern "Rust" fn __internal_default_logger() -> &'static dyn Log {
    {
        static BUFFER_LOGGER: buffer::Logger = buffer::Logger::new();
        &BUFFER_LOGGER
    }
}

#[cfg(feature = "logger_file")]
#[no_mangle]
pub extern "Rust" fn __internal_default_logger() -> &'static dyn Log {
    {
        static FILE_NAME: &str = "iceoryx2.log";
        static FILE_LOGGER: std::sync::LazyLock<file::Logger> =
            std::sync::LazyLock::new(|| file::Logger::new(FILE_NAME));
        &*FILE_LOGGER
    }
}

#[cfg(feature = "logger_log")]
#[no_mangle]
pub extern "Rust" fn __internal_default_logger() -> &'static dyn Log {
    {
        static LOG_LOGGER: log::Logger = log::Logger::new();
        &LOG_LOGGER
    }
}

#[cfg(feature = "logger_tracing")]
#[no_mangle]
pub extern "Rust" fn __internal_default_logger() -> &'static dyn Log {
    {
        static TRACING_LOGGER: tracing::Logger = tracing::Logger::new();
        &TRACING_LOGGER
    }
}

#[cfg(not(any(feature = "logger_console", feature = "logger_buffer")))]
#[no_mangle]
pub extern "Rust" fn __internal_default_logger() -> &'static dyn Log {
    {
        static NULL_LOGGER: null::Logger = null::Logger;
        &NULL_LOGGER
    }
}
