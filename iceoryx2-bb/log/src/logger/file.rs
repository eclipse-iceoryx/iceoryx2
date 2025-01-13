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
use core::{fmt::Debug, time::Duration};

extern crate alloc;
use alloc::sync::Arc;

use std::{
    fs::OpenOptions,
    io::Write,
    sync::mpsc::Sender,
    thread::JoinHandle,
    time::{Instant, SystemTime},
};

use std::sync::mpsc::channel;

use crate::{get_log_level, LogLevel};

enum Message {
    Entry(Entry),
    Stop,
}

struct Entry {
    timestamp: Duration,
    elapsed_time: Duration,
    log_level: LogLevel,
    origin: String,
    message: String,
}

impl Debug for Entry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "timestamp: {:?}, elapsed_time: {:?}, log_level: {:?}, origin: {}, message: {}",
            self.timestamp, self.elapsed_time, self.log_level, self.origin, self.message
        )
    }
}

/// A logger that logs all messages into a file. It implements an active object pattern. A
/// background thread waits on a queue of log messages and whenever a new message is added.
pub struct Logger {
    sender: Arc<Sender<Message>>,
    start_time: Instant,
    _background_thread: Arc<Option<JoinHandle<()>>>,
}

impl Logger {
    /// Creates a new file logger.
    pub fn new(file_name: &str) -> Self {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_name)
            .expect("Open log file for writing.");

        let (sender, receiver) = channel();

        let write_buffer_to_file = move || loop {
            match receiver.recv() {
                Ok(Message::Entry(entry)) => file
                    .write_all(format!("{:?}\n", entry).as_bytes())
                    .expect("Writing log message into log file."),
                Ok(Message::Stop) => break,
                Err(e) => file
                    .write_all(
                        format!("[This should never happen!] File Logger got error: {:?}", e)
                            .as_bytes(),
                    )
                    .expect("Write log message into log file."),
            };
            file.sync_all().expect("Sync log file with disc.");
        };

        Self {
            sender: Arc::new(sender),
            _background_thread: Arc::new(Some(std::thread::spawn(write_buffer_to_file))),
            start_time: Instant::now(),
        }
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.sender
            .send(Message::Stop)
            .expect("Send stop notification to background thread.");
    }
}

impl crate::Log for Logger {
    fn log(
        &self,
        log_level: LogLevel,
        origin: core::fmt::Arguments,
        formatted_message: core::fmt::Arguments,
    ) {
        if get_log_level() > log_level as u8 {
            return;
        }

        self.sender
            .send({
                Message::Entry(Entry {
                    log_level,
                    timestamp: SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .expect("Acquire current system time."),
                    elapsed_time: self.start_time.elapsed(),
                    origin: origin.to_string(),
                    message: formatted_message.to_string(),
                })
            })
            .expect("Send log message to log thread.");
    }
}
