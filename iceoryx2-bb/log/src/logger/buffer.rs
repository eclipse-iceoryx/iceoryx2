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

use std::sync::Mutex;

use crate::LogLevel;

#[derive(Debug, Clone)]
pub struct Entry {
    pub log_level: LogLevel,
    pub origin: String,
    pub message: String,
}

pub struct Logger {
    buffer: Mutex<Vec<Entry>>,
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Logger {
    pub const fn new() -> Logger {
        Logger {
            buffer: Mutex::new(vec![]),
        }
    }

    pub fn len(&self) -> usize {
        self.buffer
            .lock()
            .expect("Unable to acquire log buffer length since the lock of the log buffer failed.")
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer
            .lock()
            .expect(
                "Unable to acquire log buffer empty state since the lock of the log buffer failed.",
            )
            .is_empty()
    }

    pub fn contains(&self, log_level: LogLevel) -> bool {
        let guard = self
            .buffer
            .lock()
            .expect("Unable to check log buffer content since the lock of the log buffer failed.");

        for entry in &*guard {
            if entry.log_level == log_level {
                return true;
            }
        }

        false
    }

    pub fn clear(&self) {
        self.buffer
            .lock()
            .expect("Unable to clear log buffer since the lock of the log buffer failed.")
            .clear();
    }

    pub fn content(&self) -> Vec<Entry> {
        self.buffer
            .lock()
            .expect("Unable to copy log buffer content since the lock of the log buffer failed.")
            .clone()
    }
}

impl crate::Log for Logger {
    fn log(
        &self,
        log_level: LogLevel,
        origin: core::fmt::Arguments,
        formatted_message: core::fmt::Arguments,
    ) {
        self.buffer
            .lock()
            .expect("Unable to log message since the lock of the log buffer failed.")
            .push(Entry {
                log_level,
                origin: origin.to_string(),
                message: formatted_message.to_string(),
            });
    }
}
