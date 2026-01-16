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
//! - **`iceoryx2_bb_loggers`**: This crate, providing selectable logger backends
//!   built on top of the platform abstraction
//!
//! This separation keeps the logging API lightweight and platform-agnostic
//! while allowing flexible backend selection at runtime.
//!
//! See `iceoryx2_log` for usage examples and the complete logging API.
//!
//! # Feature Flags
//!
//! Exactly one of these three features must be selected according to your
//! platform:
//!
//!  * `std` - Build for platforms that have `std` support
//!  * `posix` - Build for platforms that have a POSIX abastraction, but no `std` support
//!  * `bare_metal` - Build for bare metal platforms
//!
//! Optionally, the default logger can also be configured. If none are
//! configured, the null logger is used and all logs are discarded:
//!
//!  * `buffer` - output log messages to a buffer
//!  * `console` - output log messages to the console
//!  * `file` - output log messages to the file
//!  * `log` - utilize the `log` crate to output log messages
//!  * `tracing` - utilize the `tracing` crate to output log messages

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

use iceoryx2_log_types::Log;

#[cfg(feature = "buffer")]
mod buffer;
#[cfg(feature = "console")]
mod console;
#[cfg(feature = "file")]
mod file;
#[cfg(feature = "log")]
mod log;
#[cfg(feature = "tracing")]
mod tracing;

mod null;

extern crate alloc;

#[cfg(feature = "console")]
#[no_mangle]
pub extern "Rust" fn __internal_default_logger() -> &'static dyn Log {
    {
        static CONSOLE_LOGGER: console::Logger = console::Logger::new();
        &CONSOLE_LOGGER
    }
}

#[cfg(feature = "buffer")]
#[no_mangle]
pub extern "Rust" fn __internal_default_logger() -> &'static dyn Log {
    {
        static BUFFER_LOGGER: buffer::Logger = buffer::Logger::new();
        &BUFFER_LOGGER
    }
}

#[cfg(feature = "file")]
#[no_mangle]
pub extern "Rust" fn __internal_default_logger() -> &'static dyn Log {
    {
        static FILE_NAME: &str = "iceoryx2.log";
        static FILE_LOGGER: std::sync::LazyLock<file::Logger> =
            std::sync::LazyLock::new(|| file::Logger::new(FILE_NAME));
        &*FILE_LOGGER
    }
}

#[cfg(feature = "log")]
#[no_mangle]
pub extern "Rust" fn __internal_default_logger() -> &'static dyn Log {
    {
        static LOG_LOGGER: log::Logger = log::Logger::new();
        &LOG_LOGGER
    }
}

#[cfg(feature = "tracing")]
#[no_mangle]
pub extern "Rust" fn __internal_default_logger() -> &'static dyn Log {
    {
        static TRACING_LOGGER: tracing::Logger = tracing::Logger::new();
        &TRACING_LOGGER
    }
}

#[cfg(not(any(
    feature = "console",
    feature = "buffer",
    feature = "file",
    feature = "log",
    feature = "tracing"
)))]
#[no_mangle]
pub extern "Rust" fn __internal_default_logger() -> &'static dyn Log {
    {
        static NULL_LOGGER: null::Logger = null::Logger;
        &NULL_LOGGER
    }
}
