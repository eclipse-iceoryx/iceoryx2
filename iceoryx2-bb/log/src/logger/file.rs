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

use std::{
    collections::VecDeque,
    fmt::Debug,
    fs::File,
    io::Write,
    sync::{Arc, Condvar, Mutex},
    thread::JoinHandle,
    time::{Duration, Instant, SystemTime},
};

use crate::LogLevel;

use super::Logger;

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

pub struct FileLogger {
    state: Arc<Mutex<State>>,
    trigger: Arc<Condvar>,
    start_time: Instant,
    _background_thread: Arc<Option<JoinHandle<()>>>,
}

impl FileLogger {
    pub fn new(file_name: &str) -> Self {
        let state = Arc::new(Mutex::new(State::default()));
        let trigger = Arc::new(Condvar::new());
        let mut file = File::create(file_name).unwrap();

        let self_state = state.clone();
        let self_trigger = trigger.clone();

        let write_buffer_to_file = move || loop {
            println!("write buffer to file");
            let mut lock = state.lock().unwrap();

            lock = trigger
                .wait_while(lock, |state: &mut State| state.buffer.is_empty())
                .unwrap();

            while let Some(entry) = lock.buffer.pop_front() {
                println!("write_entry");
                file.write_all(format!("{:?}\n", entry).as_bytes()).unwrap();
            }
            file.sync_all().unwrap();

            if !lock.keep_running {
                println!("buffer len {}", lock.buffer.len());
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

impl Drop for FileLogger {
    fn drop(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.keep_running = false;
        self.trigger.notify_one();
    }
}

impl Logger for FileLogger {
    fn log(
        &self,
        log_level: LogLevel,
        origin: std::fmt::Arguments,
        formatted_message: std::fmt::Arguments,
    ) {
        let mut state = self.state.lock().unwrap();
        state.buffer.push_back(Entry {
            log_level,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap(),
            elapsed_time: self.start_time.elapsed(),
            origin: origin.to_string(),
            message: formatted_message.to_string(),
        });
        self.trigger.notify_one();
    }
}
