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

use core::fmt::Write;

use alloc::string::ToString;

use iceoryx2_bb_print::is_terminal;
use iceoryx2_bb_print::writer::stderr;
use iceoryx2_log_types::Log;
use iceoryx2_log_types::LogLevel;
use iceoryx2_pal_concurrency_sync::atomic::AtomicU64;
use iceoryx2_pal_concurrency_sync::atomic::Ordering;

#[allow(dead_code)]
enum ConsoleLogOrder {
    Time,
    Counter,
}

pub struct Logger {
    counter: AtomicU64,
    ordering_mode: ConsoleLogOrder,
}

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
    #[cfg(feature = "posix")]
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
    #[cfg(not(any(feature = "std", feature = "posix")))]
    {
        Duration::from_secs(0)
    }
}

impl Logger {
    pub const fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
            ordering_mode: ConsoleLogOrder::Counter,
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
        if is_terminal() {
            "\x1b[0;90m"
        } else {
            ""
        }
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

    fn print(separator: &str, color: &str, output: &str) {
        if is_terminal() {
            let _ = core::write!(stderr(), "{color}");
        }

        let _ = core::write!(stderr(), "{separator}{output}");

        if is_terminal() {
            let _ = core::writeln!(stderr(), "\x1b[0m");
        } else {
            let _ = core::writeln!(stderr(), " ");
        }
    }

    fn print_message(log_level: LogLevel, formatted_message: &str) {
        Self::print("", Self::message_color(log_level), formatted_message);
    }

    fn print_origin(log_level: LogLevel, origin: &str) {
        let _ = core::write!(stderr(), "{} ", Logger::log_level_string(log_level));
        Self::print("", Logger::origin_color(log_level), origin);
        let _ = core::write!(stderr(), "| ");
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

        match self.ordering_mode {
            ConsoleLogOrder::Time => {
                let time = duration_since_epoch();

                match origin_str.is_empty() {
                    false => {
                        let _ = core::write!(
                            stderr(),
                            "{}{}.{:0>9} ",
                            Logger::counter_color(log_level),
                            time.as_secs(),
                            time.subsec_nanos(),
                        );
                        Self::print_origin(log_level, &origin_str);
                    }
                    true => {
                        let _ = core::writeln!(
                            stderr(),
                            "{}{}.{:0>9} {} ",
                            Logger::counter_color(log_level),
                            time.as_secs(),
                            time.subsec_nanos(),
                            Logger::log_level_string(log_level),
                        );
                    }
                }
            }
            ConsoleLogOrder::Counter => match origin.to_string().is_empty() {
                false => {
                    let _ =
                        core::write!(stderr(), "{}{} ", Logger::counter_color(log_level), counter);
                    Self::print_origin(log_level, &origin_str);
                }
                true => {
                    let _ = core::write!(
                        stderr(),
                        "{}{} {} ",
                        Logger::counter_color(log_level),
                        counter,
                        Logger::log_level_string(log_level),
                    );
                }
            },
        }

        Self::print_message(log_level, &msg_str);
    }
}
