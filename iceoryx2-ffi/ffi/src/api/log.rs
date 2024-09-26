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

// BEGIN type definition

use iceoryx2_bb_log::{get_log_level, set_log_level, LogLevel};

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

// END type definition

// BEGIN C API

#[no_mangle]
pub unsafe extern "C" fn iox2_set_log_level(v: iox2_log_level_e) {
    set_log_level(v.into());
}

#[no_mangle]
pub unsafe extern "C" fn iox2_get_log_level() -> iox2_log_level_e {
    get_log_level().into()
}

// END C API
