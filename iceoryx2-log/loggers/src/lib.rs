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
