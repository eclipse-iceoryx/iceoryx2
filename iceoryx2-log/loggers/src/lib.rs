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
//! [`iceoryx2_log`] using [`iceoryx2_log::set_logger()`]. Each logger is
//! feature-gated, allowing users to include only what they need.
//!
//! # Architecture
//!
//! The iceoryx2 logging system is split into two crates:
//! - **[`iceoryx2_log`]**: The frontend providing the logging API and macros
//! - **`iceoryx2-loggers`**: This crate, providing selectable logger backends
//!
//! This separation keeps the logging API lightweight and platform-agnostic
//! while allowing flexible backend selection at runtime.
//!
//! # Available Loggers
//!
//! Each logger is enabled via a corresponding Cargo feature:
//!
//! | Logger | Feature | Description |
//! |--------|---------|-------------|
//! | [`console::Logger`] | `logger_console` | Writes to stderr with optional color support |
//! | [`file::Logger`] | `logger_file` | Writes to a file using a background thread |
//! | [`buffer::Logger`] | `logger_buffer` | Stores log messages in memory (for testing) |
//! | [`log::Logger`] | `logger_log` | Bridges to the `log` crate ecosystem |
//! | [`tracing::Logger`] | `logger_tracing` | Bridges to the `tracing` crate ecosystem |
//!
//! See [`iceoryx2_log`] for usage examples and the complete logging API.

#[cfg(feature = "logger_buffer")]
pub mod buffer;
#[cfg(feature = "logger_console")]
pub mod console;
#[cfg(feature = "logger_file")]
pub mod file;
#[cfg(feature = "logger_log")]
pub mod log;
#[cfg(feature = "logger_tracing")]
pub mod tracing;

extern crate alloc;

// /// Sets the [`console::Logger`] as default logger
// #[cfg(feature = "logger_console")]
// pub fn use_console_logger() -> bool {
//     // LazyLock is only available in 'std' but since static values are never dropped in Rust,
//     // we can also use Box::leak
//     let logger = Box::leak(Box::new(console::Logger::new()));
//     crate::set_logger(&*logger)
// }
//
// /// Sets the [`file::Logger`] as default logger
// #[cfg(feature = "logger_file")]
// pub fn use_file_logger(log_file_name: &str) -> bool {
//     // LazyLock is only available in 'std' but since static values are never dropped in Rust,
//     // we can also use Box::leak
//     let logger = Box::leak(Box::new(file::Logger::new(log_file_name)));
//
//     crate::set_logger(logger)
// }
