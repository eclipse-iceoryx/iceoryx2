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

#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

//! The Logging API for iceoryx2. It has 6 [`LogLevel`]s which can be set via
//! [`set_log_level()`] and read via [`get_log_level()`].
//!
//! The API includes convinience macros to combine error/panic handling
//! directly with a logger selected from the `iceoryx2_bb_loggers` crate.
//! The [`fail!`] macro can return when the function which was called return an
//! error containing result.
//! The [`fatal_panic!`] macro calls [`panic!`].
//!
//! # Example
//!
//! ## Logging
//! ```
//! use iceoryx2_log::{debug, error, info, trace, warn};
//!
//! #[derive(Debug)]
//! struct MyDataType {
//!     value: u64
//! }
//!
//! impl MyDataType {
//!     fn log_stuff(&self) {
//!         trace!("trace message");
//!         trace!(from self, "trace message");
//!         trace!(from "Custom::Origin", "trace message");
//!
//!         debug!("hello {} {}", 123, 456);
//!         debug!(from self, "hello {}", 123);
//!         debug!(from "Another::Origin", "hello {}", 123);
//!
//!         info!("world");
//!         info!(from self, "world");
//!         info!(from "hello", "world");
//!
//!         warn!("warn message");
//!         warn!(from self, "warning");
//!         warn!(from "Somewhere::Else", "warning!");
//!
//!         error!("bla {}", 1);
//!         error!(from self, "bla {}", 1);
//!         error!(from "error origin", "bla {}", 1);
//!     }
//!}
//! ```
//!
//! ## Error Handling
//! ```
//! use iceoryx2_log::fail;
//!
//! #[derive(Debug)]
//! struct MyDataType {
//!     value: u64
//! }
//!
//! impl MyDataType {
//!     fn doStuff(&self, value: u64) -> Result<(), ()> {
//!         if value == 0 { Err(()) } else { Ok(()) }
//!     }
//!
//!     fn doMoreStuff(&self) -> Result<(), u64> {
//!         // fail when doStuff.is_err() and return the error 1234
//!         fail!(from self, when self.doStuff(0),
//!                 with 1234, "Failed while calling doStuff");
//!         Ok(())
//!     }
//!
//!     fn doMore(&self) -> Result<(), u64> {
//!         if self.value == 0 {
//!             // without condition, return error 4567
//!             fail!(from self, with 4567, "Value is zero");
//!         }
//!
//!         Ok(())
//!     }
//!
//!     fn evenMore(&self) -> Result<(), u64> {
//!         // forward error when it is compatible or convertable
//!         fail!(from self, when self.doMore(), "doMore failed");
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ## Panic Handling
//! ```
//! use iceoryx2_log::fatal_panic;
//!
//! #[derive(Debug)]
//! struct MyDataType {
//!     value: u64
//! }
//!
//! impl MyDataType {
//!     fn doStuff(&self, value: u64) {
//!         if value == 0 {
//!             fatal_panic!(from self, "value is {}", value);
//!         }
//!     }
//!
//!     fn moreStuff(&self) -> Result<(), ()> {
//!         if self.value == 0 { Err(()) } else { Ok(()) }
//!     }
//!
//!     fn doIt(&self) {
//!         fatal_panic!(from self, when self.moreStuff(), "moreStuff failed");
//!     }
//! }
//! ```

#[cfg(feature = "std")]
pub use from_env::{set_log_level_from_env_or, set_log_level_from_env_or_default};

// Re-export so library crates need only depend on this crate
pub use iceoryx2_log_types::{Log, LogLevel};

use iceoryx2_pal_concurrency_sync::atomic::AtomicU8;
use iceoryx2_pal_concurrency_sync::atomic::Ordering;
use iceoryx2_pal_concurrency_sync::once::Once;

mod fail;
mod log;

const DEFAULT_LOG_LEVEL: LogLevel = LogLevel::Info;

static mut LOGGER: Option<&'static dyn Log> = None;

#[cfg(not(all(test, loom, feature = "std")))]
static LOG_LEVEL: AtomicU8 = AtomicU8::new(DEFAULT_LOG_LEVEL as u8);
#[cfg(all(test, loom, feature = "std"))]
static LOG_LEVEL: std::sync::LazyLock<IoxAtomicU8> = std::sync::LazyLock::new(|| {
    unimplemented!("loom does not provide const-initialization for atomic variables.")
});

#[cfg(not(all(test, loom, feature = "std")))]
static INIT: Once = Once::new();
#[cfg(all(test, loom, feature = "std"))]
static INIT: std::sync::LazyLock<Once> = std::sync::LazyLock::new(|| {
    unimplemented!("loom does not provide const-initialization for atomic variables.")
});

/// Sets the current log level. This is ignored for external frameworks like `log` or `tracing`.
/// Here you have to use the log-level settings of that framework.
pub fn set_log_level(v: LogLevel) {
    LOG_LEVEL.store(v as u8, Ordering::Relaxed);
}

/// Returns the current log level
pub fn get_log_level() -> u8 {
    LOG_LEVEL.load(Ordering::Relaxed)
}

/// Get a reference to the current logger
///
/// This initializes the logger to NULL_LOGGER if it hasn't been set yet.
fn get_logger() -> &'static dyn Log {
    INIT.call_once(|| unsafe {
        #[allow(static_mut_refs)]
        if LOGGER.is_none() {
            LOGGER = Some(__internal_default_logger());
        }
    });

    // # Safety
    // 1. The logger is always an immutable threadsafe object with only interior mutability.
    // 2. Once::call_once ensures LOGGER can only be mutated during initialization
    //    and the lifetime is 'static.
    // 3. After INIT.call_once returns, LOGGER is guaranteed to be Some(_)
    #[allow(static_mut_refs)]
    unsafe {
        LOGGER.unwrap()
    }
}

/// Sets the [`Log`]ger. Can be only called once at the beginning of the program. If the
/// [`Log`]ger is already set it returns false and does not update it.
pub fn set_logger(logger: &'static dyn Log) -> bool {
    let mut set_logger_success = false;
    INIT.call_once(|| {
        unsafe { LOGGER = Some(logger) };
        set_logger_success = true;
    });
    set_logger_success
}

#[cfg(feature = "std")]
mod from_env {
    use super::{set_log_level, LogLevel, DEFAULT_LOG_LEVEL};
    use std::env;

    fn get_log_level_from_str_fuzzy(
        log_level_string: &str,
        log_level_fallback: LogLevel,
    ) -> LogLevel {
        match log_level_string.to_lowercase().as_str() {
            "trace" => LogLevel::Trace,
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            "fatal" => LogLevel::Fatal,
            _ => {
                eprintln!(
                    "Invalid value for 'IOX2_LOG_LEVEL' environment variable!\
                    \nFound: {log_level_string:?}\
                    \nAllowed is one of: fatal, error, warn, info, debug, trace\
                    \nSetting log level as : {log_level_fallback:?}"
                );
                log_level_fallback
            }
        }
    }

    /// Sets the log level by reading environment variable "IOX2_LOG_LEVEL" or default it with LogLevel::INFO
    pub fn set_log_level_from_env_or_default() {
        set_log_level_from_env_or(DEFAULT_LOG_LEVEL);
    }

    /// Sets the log level by reading environment variable "IOX2_LOG_LEVEL", and if the environment variable
    /// doesn't exit it sets it with a user-defined logging level
    pub fn set_log_level_from_env_or(v: LogLevel) {
        let log_level = env::var("IOX2_LOG_LEVEL")
            .ok()
            .map(|s| get_log_level_from_str_fuzzy(&s, v))
            .unwrap_or(v);
        set_log_level(log_level);
    }
}

#[doc(hidden)]
pub fn __internal_print_log_msg(
    log_level: LogLevel,
    origin: core::fmt::Arguments,
    args: core::fmt::Arguments,
) {
    if get_log_level() <= log_level as u8 {
        get_logger().log(log_level, origin, args)
    }
}

extern "Rust" {
    fn __internal_default_logger() -> &'static dyn Log;
}
