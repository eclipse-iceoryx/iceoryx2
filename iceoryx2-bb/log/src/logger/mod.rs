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
pub mod file;
#[cfg(feature = "logger_log")]
pub mod log;
#[cfg(feature = "logger_tracing")]
pub mod tracing;

/// Sets the [`console::Logger`] as default logger
pub fn use_console_logger() -> bool {
    // LazyLock is only available in 'std' but since static values are never dropped in Rust,
    // we can also use Box::leak
    let logger = Box::leak(Box::new(console::Logger::new()));
    crate::set_logger(&*logger)
}

/// Sets the [`file::Logger`] as default logger
pub fn use_file_logger(log_file_name: &str) -> bool {
    // LazyLock is only available in 'std' but since static values are never dropped in Rust,
    // we can also use Box::leak
    let logger = Box::leak(Box::new(file::Logger::new(log_file_name)));

    crate::set_logger(logger)
}
