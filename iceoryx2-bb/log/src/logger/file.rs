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

//! # Example
//!
//! Using the file logger.
//!
//! ```no_run
//! use iceoryx2_bb_log::{info, set_logger, set_log_level, LogLevel, logger::file};
//! use std::sync::LazyLock;
//!
//! const LOG_FILE: &str = "fuu.log";
//! static FILE_LOGGER: LazyLock<file::Logger> = LazyLock::new(|| file::Logger::new(LOG_FILE));
//! set_logger(&*FILE_LOGGER);
//! set_log_level(LogLevel::Trace);
//!
//! // written into log file "fuu.log"
//! info!("hello world");
//! ```

// TODO: [Reminder to my future self]
// In the long-term the file logger may be required to be based on the same
// iceoryx2_pal_posix platform. In this case, the logger needs to use the low-level calls directly
// to avoid a circular dependency with iceoryx2_bb_posix.
use std::{
    collections::VecDeque,
    fmt::Debug,
    fs::OpenOptions,
    io::Write,
    sync::{Arc, Condvar, Mutex},
    thread::JoinHandle,
    time::{Duration, Instant, SystemTime},
};

use crate::{get_log_level, LogLevel};

struct Entry {
    timestamp: Duration,
    elapsed_time: Duration,
    log_level: LogLevel,
    origin: String,
    message: String,
}

impl Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "timestamp: {:?}, elapsed_time: {:?}, log_level: {:?}, origin: {}, message: {}",
            self.timestamp, self.elapsed_time, self.log_level, self.origin, self.message
        )
    }
}

struct State {
    buffer: VecDeque<Entry>,
    keep_running: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            buffer: VecDeque::new(),
            keep_running: true,
        }
    }
}

/// A logger that logs all messages into a file. It implements an active object pattern. A
/// background thread waits on a queue of log messages and whenever a new message is added.
pub struct Logger {
    state: Arc<Mutex<State>>,
    trigger: Arc<Condvar>,
    start_time: Instant,
    _background_thread: Arc<Option<JoinHandle<()>>>,
}

impl Logger {
    /// Creates a new file logger.
    pub fn new(file_name: &str) -> Self {
        let state = Arc::new(Mutex::new(State::default()));
        let trigger = Arc::new(Condvar::new());
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_name)
            .expect("Open log file for writing.");

        let self_state = state.clone();
        let self_trigger = trigger.clone();

        let write_buffer_to_file = move || loop {
            let lock = state.lock().expect("Acquire internal state mutex.");

            let lock = trigger
                .wait_while(lock, |state: &mut State| {
                    state.buffer.is_empty() || !state.keep_running
                })
                .expect("Wait for internal state trigger.");
            drop(lock);

            loop {
                // acquiring the lock only to get the next buffer to minimize contention as much
                // as possible.
                let mut lock = state.lock().expect("Acquire internal state mutex.");
                let buffer = lock.buffer.pop_front();
                drop(lock);

                match buffer {
                    Some(entry) => file
                        .write_all(format!("{:?}\n", entry).as_bytes())
                        .expect("Writing log message into log file."),
                    None => break,
                }
            }
            file.sync_all().expect("Sync log file with disc.");

            let lock = state.lock().expect("Acquire internal state mutex.");
            if !lock.keep_running {
                break;
            }
        };

        Self {
            state: self_state,
            trigger: self_trigger,
            _background_thread: Arc::new(Some(std::thread::spawn(write_buffer_to_file))),
            start_time: Instant::now(),
        }
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        let mut state = self.state.lock().expect("Acquire internal state mutex.");
        state.keep_running = false;
        self.trigger.notify_one();
    }
}

impl crate::Logger for Logger {
    fn log(
        &self,
        log_level: LogLevel,
        origin: std::fmt::Arguments,
        formatted_message: std::fmt::Arguments,
    ) {
        if get_log_level() > log_level as u8 {
            return;
        }

        let mut state = self.state.lock().expect("Acquire internal state mutex.");
        state.buffer.push_back(Entry {
            log_level,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Acquire current system time."),
            elapsed_time: self.start_time.elapsed(),
            origin: origin.to_string(),
            message: formatted_message.to_string(),
        });
        self.trigger.notify_one();
    }
}
