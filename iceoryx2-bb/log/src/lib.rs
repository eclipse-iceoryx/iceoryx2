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

#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

//! Simplistic logger. It has 6 [`LogLevel`]s which can be set via [`set_log_level()`] and read via
//! [`get_log_level()`].
//!
//! The logger provides convinience macros to combine error/panic handling directly with the
//! logger.
//! The [`fail!`] macro can return when the function which was called return an error containing
//! result.
//! The [`fatal_panic!`] macro calls [`panic!`].
//!
//! # Example
//!
//! ## Logging
//! ```
//! use iceoryx2_bb_log::{debug, error, info, trace, warn};
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
//! use iceoryx2_bb_log::fail;
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
//! use iceoryx2_bb_log::fatal_panic;
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
//! ## Setting custom logger on application startup
//!
//! In this example we use the [`crate::logger::buffer::Logger`], that stores every log
//! message in an internal buffer, and use it as the default logger.
//!
//! ```
//! use iceoryx2_bb_log::{set_logger, info};
//!
//! static LOGGER: iceoryx2_bb_log::logger::buffer::Logger =
//!     iceoryx2_bb_log::logger::buffer::Logger::new();
//!
//! assert!(set_logger(&LOGGER));
//! info!("hello world");
//! let log_content = LOGGER.content();
//!
//! for entry in log_content {
//!     println!("{:?} {} {}", entry.log_level, entry.origin, entry.message);
//! }
//! ```

#[macro_use]
pub mod log;
#[macro_use]
pub mod fail;
pub mod logger;

use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU8;

use core::{fmt::Arguments, sync::atomic::Ordering};
use std::sync::Once;

use std::env;

#[cfg(feature = "logger_tracing")]
static DEFAULT_LOGGER: logger::tracing::Logger = logger::tracing::Logger::new();

#[cfg(feature = "logger_log")]
static DEFAULT_LOGGER: logger::log::Logger = logger::log::Logger::new();

#[cfg(not(any(feature = "logger_log", feature = "logger_tracing")))]
static DEFAULT_LOGGER: logger::console::Logger = logger::console::Logger::new();

const DEFAULT_LOG_LEVEL: LogLevel = LogLevel::Info;

static mut LOGGER: Option<&'static dyn Log> = None;
static LOG_LEVEL: IoxAtomicU8 = IoxAtomicU8::new(DEFAULT_LOG_LEVEL as u8);
static INIT: Once = Once::new();

pub trait Log: Send + Sync {
    /// logs a message
    fn log(&self, log_level: LogLevel, origin: Arguments, formatted_message: Arguments);
}

/// Describes the log level.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

impl LogLevel {
    fn from_str_fuzzy(log_level_string: &str, log_level_fallback: LogLevel) -> LogLevel {
        match log_level_string.to_lowercase().as_str() {
            "trace" => LogLevel::Trace,
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            "fatal" => LogLevel::Fatal,
            _ => {
                println!(
                    "Invalid value for 'IOX2_LOG_LEVEL' environment variable!\
                \nFound: {:?}\
                \nAllowed is one of: fatal, error, warn, info, debug, trace\
                \nSetting log level as : {:?}",
                    log_level_string, log_level_fallback
                );
                log_level_fallback
            }
        }
    }
}

/// Sets the log level by reading environment variable "IOX2_LOG_LEVEL" or default it wiht LogLevel::INFO
pub fn set_log_level_from_env_or_default() {
    set_log_level_from_env_or(DEFAULT_LOG_LEVEL);
}

/// Sets the log level by reading environment variable "IOX2_LOG_LEVEL", and if the environment variable
/// doesn't exits it sets it with a user-defined logging level
pub fn set_log_level_from_env_or(v: LogLevel) {
    let log_level = env::var("IOX2_LOG_LEVEL")
        .ok()
        .map(|s| LogLevel::from_str_fuzzy(&s, v))
        .unwrap_or(v);
    set_log_level(log_level);
}

/// Sets the current log level. This is ignored for external frameworks like `log` or `tracing`.
/// Here you have to use the log-level settings of that framework.
pub fn set_log_level(v: LogLevel) {
    LOG_LEVEL.store(v as u8, Ordering::Relaxed);
}

/// Returns the current log level
pub fn get_log_level() -> u8 {
    LOG_LEVEL.load(Ordering::Relaxed)
}

/// Sets the [`Log`]ger. Can be only called once at the beginning of the program. If the
/// [`Log`]ger is already set it returns false and does not update it.
pub fn set_logger<T: Log + 'static>(value: &'static T) -> bool {
    let mut set_logger_success = false;
    INIT.call_once(|| {
        unsafe { LOGGER = Some(value) };
        set_logger_success = true;
    });
    set_logger_success
}

/// Returns a reference to the [`Log`]ger.
pub fn get_logger() -> &'static dyn Log {
    INIT.call_once(|| {
        unsafe { LOGGER = Some(&DEFAULT_LOGGER) };
    });

    // # From The Compiler
    //
    // shared references to mutable statics are dangerous; it's undefined behavior
    //   1. if the static is mutated or
    //   2. if a mutable reference is created for it while the shared reference lives
    //
    // # Safety
    //
    // 1. The logger is always an immutable threadsafe object with only interior mutability.
    // 2. [`std::sync::Once`] is used to ensure it can only mutated on initialization and the
    //    lifetime is `'static`.
    #[allow(static_mut_refs)]
    unsafe {
        *LOGGER.as_ref().unwrap()
    }
}

#[doc(hidden)]
pub fn __internal_print_log_msg(log_level: LogLevel, origin: Arguments, args: Arguments) {
    if get_log_level() <= log_level as u8 {
        get_logger().log(log_level, origin, args)
    }
}
