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

//! The default [`Logger`] implementation.

use alloc::string::String;
use alloc::string::ToString;

use iceoryx2_bb_print::IsTerminal;
use iceoryx2_bb_print::cerrln;
use iceoryx2_bb_print::stderr;
use iceoryx2_log_types::Log;
use iceoryx2_log_types::LogLevel;
use iceoryx2_pal_concurrency_sync::atomic::AtomicU8;
use iceoryx2_pal_concurrency_sync::atomic::AtomicU64;
use iceoryx2_pal_concurrency_sync::atomic::Ordering;
use iceoryx2_pal_concurrency_sync::cell::UnsafeCell;
use iceoryx2_pal_posix::posix;

const STATE_UNINITIALIZED: u8 = 0;
const STATE_LOCKED: u8 = 1;
const STATE_INITIALIZED: u8 = 2;

#[derive(Default)]
#[allow(dead_code)]
pub enum ConsoleLogOrder {
    Time,
    #[default]
    Counter,
}

#[derive(Default)]
#[allow(dead_code)]
pub enum OriginMode {
    None,
    #[default]
    Simple,
    Full,
}

#[derive(Default)]
pub struct Config {
    pub ordering_mode: ConsoleLogOrder,
    pub show_process_details: bool,
    pub origin_mode: OriginMode,
}

pub struct Logger {
    counter: AtomicU64,
    config: Config,
    pid: UnsafeCell<posix::pid_t>,
    tid: UnsafeCell<posix::pthread_t>,
    executable: UnsafeCell<String>,
    state: AtomicU8,
}

unsafe impl Send for Logger {}
unsafe impl Sync for Logger {}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

fn duration_since_epoch() -> core::time::Duration {
    #[cfg(feature = "std")]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
    }
    #[cfg(all(not(feature = "std"), any(target_os = "linux", target_os = "nto",)))]
    {
        use core::time::Duration;
        use iceoryx2_pal_posix::*;

        let mut current_time = posix::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };

        let result =
            unsafe { posix::clock_gettime(posix::CLOCK_MONOTONIC as _, &mut current_time) };
        if result == 0 {
            return Duration::from_secs(current_time.tv_sec as u64)
                + Duration::from_nanos(current_time.tv_nsec as u64);
        }
        Duration::from_secs(0)
    }
    #[cfg(all(
        not(feature = "std"),
        not(any(target_os = "linux", target_os = "nto",))
    ))]
    {
        Duration::from_secs(0)
    }
}

fn is_terminal() -> bool {
    stderr().is_terminal()
}

struct ProcessDetails {
    process_id: posix::pid_t,
    thread_id: posix::pthread_t,
    executable: &'static str,
}

impl Logger {
    pub const fn new() -> Self {
        Self::from_config(Config {
            ordering_mode: ConsoleLogOrder::Counter,
            show_process_details: true,
            origin_mode: OriginMode::Simple,
        })
    }

    const fn from_config(config: Config) -> Self {
        Self {
            counter: AtomicU64::new(0),
            config,
            pid: UnsafeCell::new(0),
            tid: UnsafeCell::new(0),
            executable: UnsafeCell::new(String::new()),
            state: AtomicU8::new(STATE_UNINITIALIZED),
        }
    }

    fn process_details(&self) -> ProcessDetails {
        loop {
            match self.state.compare_exchange(
                STATE_UNINITIALIZED,
                STATE_LOCKED,
                Ordering::Relaxed,
                ///////////////////////////////////
                // SYNC POINT: self.pid, self.executable read
                ///////////////////////////////////
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    let pid = unsafe { posix::getpid() };
                    let tid = unsafe { posix::pthread_self() };
                    let mut buffer = [0u8; 1024];
                    let path_len = unsafe {
                        posix::proc_pidpath(pid, buffer.as_mut_ptr().cast(), buffer.len())
                    };

                    let mut executable = if path_len < 0 {
                        "unknown".to_string()
                    } else {
                        String::from_utf8_lossy(&buffer[..(path_len as usize)]).to_string()
                    };

                    executable = executable
                        .rsplit(iceoryx2_pal_configuration::PATH_SEPARATOR as char)
                        .next()
                        .unwrap_or_default()
                        .to_string();

                    *unsafe { &mut *self.pid.get() } = pid;
                    *unsafe { &mut *self.tid.get() } = tid;
                    *unsafe { &mut *self.executable.get() } = executable;

                    ///////////////////////////////////
                    // SYNC POINT: self.pid, self.executable write
                    ///////////////////////////////////
                    self.state.store(STATE_INITIALIZED, Ordering::Release);
                }
                Err(STATE_INITIALIZED) => {
                    return ProcessDetails {
                        process_id: unsafe { *self.pid.get() },
                        thread_id: unsafe { *self.tid.get() },
                        executable: unsafe { &*self.executable.get() },
                    };
                }
                Err(_) => {}
            }
        }
    }

    fn log_level_string(log_level: LogLevel) -> &'static str {
        if is_terminal() {
            match log_level {
                LogLevel::Trace => "\x1b[0;90m[T]",
                LogLevel::Debug => "\x1b[0;93m[D]",
                LogLevel::Info => "\x1b[0;92m[I]",
                LogLevel::Warn => "\x1b[0;33m[W]",
                LogLevel::Error => "\x1b[0;31m[E]",
                LogLevel::Fatal => "\x1b[1;4;91m[F]",
            }
        } else {
            match log_level {
                LogLevel::Trace => "[T]",
                LogLevel::Debug => "[D]",
                LogLevel::Info => "[I]",
                LogLevel::Warn => "[W]",
                LogLevel::Error => "[E]",
                LogLevel::Fatal => "[F]",
            }
        }
    }

    fn message_color(log_level: LogLevel) -> &'static str {
        if is_terminal() {
            match log_level {
                LogLevel::Trace => "\x1b[1;90m",
                LogLevel::Debug => "\x1b[1;90m",
                LogLevel::Info => "\x1b[1;37m",
                LogLevel::Warn => "\x1b[1;93m",
                LogLevel::Error => "\x1b[1;91m",
                LogLevel::Fatal => "\x1b[1;4;91m",
            }
        } else {
            ""
        }
    }

    fn counter_color(_log_level: LogLevel) -> &'static str {
        if is_terminal() { "\x1b[0;90m" } else { "" }
    }

    fn origin_color(log_level: LogLevel) -> &'static str {
        if is_terminal() {
            match log_level {
                LogLevel::Trace => "\x1b[0;90m",
                LogLevel::Debug => "\x1b[0;90m",
                LogLevel::Info => "\x1b[0;37m",
                LogLevel::Warn => "\x1b[0;37m",
                LogLevel::Error => "\x1b[0;37m",
                LogLevel::Fatal => "\x1b[0;4;91m",
            }
        } else {
            ""
        }
    }

    fn format_entry(
        &self,
        log_level: LogLevel,
        prefix: &str,
        origin: &str,
        message: &str,
    ) -> String {
        let line_end = if is_terminal() { "\x1b[0m" } else { " " };

        let origin = match self.config.origin_mode {
            OriginMode::None => String::new(),
            OriginMode::Full => origin.to_string(),
            OriginMode::Simple => origin
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .to_string(),
        };

        const BOLD_GREY: &str = "\x1b[1;90m";
        const GREY: &str = "\x1b[0;90m";
        const LIGHT_GREEN: &str = "\x1b[0;92m";

        let process = if self.config.show_process_details {
            let details = self.process_details();
            alloc::format!(
                "{BOLD_GREY}[{GREY}pid={LIGHT_GREEN}{}{GREY}, tid={LIGHT_GREEN}{}{GREY}, exec={LIGHT_GREEN}{}{BOLD_GREY}]",
                details.process_id,
                details.thread_id,
                details.executable
            )
        } else {
            String::new()
        };

        if origin.is_empty() {
            alloc::format!(
                "{prefix}{} {process} {}{message}{line_end}",
                Logger::log_level_string(log_level),
                Logger::message_color(log_level),
            )
        } else {
            alloc::format!(
                "{prefix}{} {process} {}{origin}{line_end}\n| {}{message}{line_end}",
                Logger::log_level_string(log_level),
                Logger::origin_color(log_level),
                Logger::message_color(log_level),
            )
        }
    }
}

impl Log for Logger {
    fn log(
        &self,
        log_level: LogLevel,
        origin: core::fmt::Arguments,
        formatted_message: core::fmt::Arguments,
    ) {
        let counter = self.counter.fetch_add(1, Ordering::Relaxed);

        let origin_str = origin.to_string();
        let msg_str = formatted_message.to_string();

        let prefix = match self.config.ordering_mode {
            ConsoleLogOrder::Time => {
                let time = duration_since_epoch();
                alloc::format!(
                    "{}{}.{:0>9} ",
                    Logger::counter_color(log_level),
                    time.as_secs(),
                    time.subsec_nanos(),
                )
            }
            ConsoleLogOrder::Counter => {
                alloc::format!("{}{} ", Logger::counter_color(log_level), counter)
            }
        };

        cerrln!(
            "{}",
            self.format_entry(log_level, &prefix, &origin_str, &msg_str)
        );
    }
}
