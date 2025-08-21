// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

#![allow(clippy::upper_case_acronyms)]
#![allow(non_camel_case_types)]

// BEGIN type definition

use iceoryx2_bb_log::{
    get_log_level, set_log_level, set_log_level_from_env_or, set_log_level_from_env_or_default,
    set_logger, Log, LogLevel, __internal_print_log_msg,
    logger::{use_console_logger, use_file_logger},
};

use core::ffi::{c_char, CStr};
use std::sync::Once;

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_log_level_e {
    TRACE = 0,
    DEBUG = 1,
    INFO = 2,
    WARN = 3,
    ERROR = 4,
    FATAL = 5,
}

impl From<u8> for iox2_log_level_e {
    fn from(value: u8) -> Self {
        match value {
            0 => iox2_log_level_e::TRACE,
            1 => iox2_log_level_e::DEBUG,
            2 => iox2_log_level_e::INFO,
            3 => iox2_log_level_e::WARN,
            4 => iox2_log_level_e::ERROR,
            5 => iox2_log_level_e::FATAL,
            _ => iox2_log_level_e::TRACE,
        }
    }
}

impl From<iox2_log_level_e> for LogLevel {
    fn from(value: iox2_log_level_e) -> Self {
        match value {
            iox2_log_level_e::TRACE => LogLevel::Trace,
            iox2_log_level_e::DEBUG => LogLevel::Debug,
            iox2_log_level_e::INFO => LogLevel::Info,
            iox2_log_level_e::WARN => LogLevel::Warn,
            iox2_log_level_e::ERROR => LogLevel::Error,
            iox2_log_level_e::FATAL => LogLevel::Fatal,
        }
    }
}

impl From<LogLevel> for iox2_log_level_e {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => iox2_log_level_e::TRACE,
            LogLevel::Debug => iox2_log_level_e::DEBUG,
            LogLevel::Info => iox2_log_level_e::INFO,
            LogLevel::Warn => iox2_log_level_e::WARN,
            LogLevel::Error => iox2_log_level_e::ERROR,
            LogLevel::Fatal => iox2_log_level_e::FATAL,
        }
    }
}

static mut LOGGER: Option<CLogger> = None;
static INIT: Once = Once::new();

struct CLogger {
    callback: iox2_log_callback,
}

impl CLogger {
    const fn new(callback: iox2_log_callback) -> Self {
        Self { callback }
    }
}

impl Log for CLogger {
    fn log(
        &self,
        log_level: LogLevel,
        origin: core::fmt::Arguments,
        formatted_message: core::fmt::Arguments,
    ) {
        let mut origin = origin.to_string();
        origin.push('\0');
        let mut formatted_message = formatted_message.to_string();
        formatted_message.push('\0');

        (self.callback)(
            log_level.into(),
            origin.as_bytes().as_ptr().cast(),
            formatted_message.as_bytes().as_ptr().cast(),
        );
    }
}

/// The custom log callback for [`iox2_set_logger`]
///
/// # Arguments
///
/// 1. The log level of the message
/// 2. The origin of the message
/// 3. The actual log message
pub type iox2_log_callback = extern "C" fn(iox2_log_level_e, *const c_char, *const c_char);

// END type definition

// BEGIN C API
/// Adds a log message to the logger.
///
/// # Safety
///
///  * origin must be either NULL or a valid pointer to a string.
///  * message must be a valid pointer to a string
#[no_mangle]
pub unsafe extern "C" fn iox2_log(
    log_level: iox2_log_level_e,
    origin: *const c_char,
    message: *const c_char,
) {
    debug_assert!(!message.is_null());

    let empty_origin = b"\0";
    let origin = if origin.is_null() {
        CStr::from_bytes_with_nul(empty_origin).unwrap()
    } else {
        CStr::from_ptr(origin)
    };
    let message = CStr::from_ptr(message);

    __internal_print_log_msg(
        log_level.into(),
        format_args!("{}", origin.to_string_lossy()),
        format_args!("{}", message.to_string_lossy()),
    );
}

/// Sets the console logger as default logger. Returns true if the logger was set, otherwise false.
#[no_mangle]
pub extern "C" fn iox2_use_console_logger() -> bool {
    use_console_logger()
}

/// Sets the file logger as default logger. Returns true if the logger was set, otherwise false.
///
/// # Safety
///
///  * log_file must be a valid pointer to a string
#[no_mangle]
pub unsafe extern "C" fn iox2_use_file_logger(log_file: *const c_char) -> bool {
    debug_assert!(!log_file.is_null());

    let log_file = CStr::from_ptr(log_file).to_string_lossy();
    use_file_logger(&log_file)
}

/// Sets the log level from environment variable or defaults it if variable does not exist
#[no_mangle]
pub unsafe extern "C" fn iox2_set_log_level_from_env_or_default() {
    set_log_level_from_env_or_default();
}

/// Sets the log level from environment variable or to a user given value if variable does not exist
#[no_mangle]
pub unsafe extern "C" fn iox2_set_log_level_from_env_or(v: iox2_log_level_e) {
    set_log_level_from_env_or(v.into());
}

/// Sets the log level.
#[no_mangle]
pub unsafe extern "C" fn iox2_set_log_level(v: iox2_log_level_e) {
    set_log_level(v.into());
}

/// Returns the current log level.
#[no_mangle]
pub unsafe extern "C" fn iox2_get_log_level() -> iox2_log_level_e {
    get_log_level().into()
}

/// Sets the logger that shall be used. This function can only be called once and must be called
/// before any log message was created.
/// It returns true if the logger was set, otherwise false.
#[no_mangle]
pub unsafe extern "C" fn iox2_set_logger(logger: iox2_log_callback) -> bool {
    INIT.call_once(|| {
        LOGGER = Some(CLogger::new(logger));
    });

    #[allow(static_mut_refs)] // internally used and the logger is never changed once it was set
    set_logger(LOGGER.as_ref().unwrap())
}

// END C API
