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

//! Defines configuration options for the posix module. They can be adapted when working on
//! a different system.

use core::time::Duration;

use iceoryx2_bb_system_types::{file_name::FileName, path::Path, user_name::UserName};

use crate::{scheduler::Scheduler, system_configuration::*};
use iceoryx2_bb_container::semantic_string::SemanticString;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockMode {
    Performance,
    Safety,
}

/// Defines how often a call is repeated when it was interrupted by SIGINT
pub const EINTR_REPETITIONS: u64 = 10;

pub const MAX_THREAD_NAME_LENGTH: usize = 16;
pub const DEFAULT_SCHEDULER: Scheduler = Scheduler::Other;

pub const ADAPTIVE_WAIT_YIELD_REPETITIONS: u64 = 10000;
pub const ADAPTIVE_WAIT_INITIAL_REPETITIONS: u64 = 100 + ADAPTIVE_WAIT_YIELD_REPETITIONS;
pub const ADAPTIVE_WAIT_INITIAL_WAITING_TIME: Duration = Duration::from_micros(100);
pub const ADAPTIVE_WAIT_FINAL_WAITING_TIME: Duration = Duration::from_millis(10);

// directories
pub fn temp_directory() -> Path {
    unsafe { Path::new_unchecked(iceoryx2_pal_configuration::TEMP_DIRECTORY) }
}

pub fn test_directory() -> Path {
    unsafe { Path::new_unchecked(iceoryx2_pal_configuration::TEST_DIRECTORY) }
}

pub fn shared_memory_directory() -> Path {
    unsafe { Path::new_unchecked(iceoryx2_pal_configuration::SHARED_MEMORY_DIRECTORY) }
}

// TODO unable to verify?
pub const ACL_LIST_CAPACITY: u32 = 25;

pub const UNIX_DOMAIN_SOCKET_PATH_LENGTH: usize = 108;
pub const PASSWD_BUFFER_SIZE: usize = 1024;
pub const GROUP_BUFFER_SIZE: usize = 1024;

pub const MAX_INITIAL_SEMAPHORE_VALUE: u32 = i16::MAX as u32;

pub const MIN_REQUIRED_SYSTEM: [(SystemInfo, usize); 1] = [(SystemInfo::PosixVersion, 200809)];
pub const MAX_REQUIRED_SYSTEM: [(SystemInfo, usize); 1] = [(SystemInfo::NumberOfCpuCores, 128)];

pub const REQUIRED_FEATURES: [Feature; 11] = [
    Feature::Barriers,
    Feature::ClockSelection,
    Feature::MappedFiles,
    Feature::MonotonicClock,
    Feature::ReaderWriterLocks,
    Feature::RealtimeSignals,
    Feature::Semaphores,
    Feature::SharedMemoryObjects,
    Feature::Threads,
    Feature::ThreadSafeFunctions,
    Feature::Timeouts,
];

pub const MIN_REQUIRED_LIMITS: [(Limit, u64); 8] = [
    (Limit::MaxSemaphoreValue, MAX_INITIAL_SEMAPHORE_VALUE as _),
    (Limit::MaxLengthOfLoginName, UserName::max_len() as u64),
    (Limit::MaxNumberOfSemaphores, 1024),
    (Limit::MaxNumberOfThreads, 1024),
    (Limit::MaxNumberOfOpenFiles, 1024),
    (Limit::MaxPathLength, Path::max_len() as _),
    (Limit::MaxFileNameLength, FileName::max_len() as _),
    (
        Limit::MaxUnixDomainSocketNameLength,
        UNIX_DOMAIN_SOCKET_PATH_LENGTH as _,
    ),
];

pub const MAX_REQUIRED_LIMITS: [(Limit, u64); 2] = [
    (Limit::MaxSizeOfGroupBuffer, GROUP_BUFFER_SIZE as _),
    (Limit::MaxSizeOfPasswordBuffer, PASSWD_BUFFER_SIZE as _),
];

/// Defines how the posix compliance check [`does_system_satisfy_posix_requirements()`]
/// is called, either with console output or not.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum ComplianceCheckMode {
    Verbose,
    Silent,
}

/// Checks if the current system satisfies all requirements to the POSIX subsystem.
/// With [`ComplianceCheckMode::Verbose`] it prints the results to the console otherwise it is
/// silent. It returns true when the system fulfills the requirements otherwise false.
pub fn does_system_satisfy_posix_requirements(mode: ComplianceCheckMode) -> bool {
    const HEADER_COLOR: &str = "\x1b[4;92m";
    const VALUE_COLOR: &str = "\x1b[0;94m";
    const ENTRY_COLOR: &str = "\x1b[0;37m";
    const COLOR_RESET: &str = "\x1b[0m";
    const FAILED_COLOR: &str = "\x1b[1;91m";

    let mut is_compliant: bool = true;

    if mode == ComplianceCheckMode::Verbose {
        println!("{HEADER_COLOR}system requirements check{COLOR_RESET}");
        println!();
        println!(" {HEADER_COLOR}minimum system{COLOR_RESET}");
    }
    for i in MIN_REQUIRED_SYSTEM.iter() {
        let supported_value = i.0.value();
        let required_value = i.1;

        let (entry_color, support_color) = if supported_value < required_value {
            is_compliant = false;
            (FAILED_COLOR, FAILED_COLOR)
        } else {
            (ENTRY_COLOR, VALUE_COLOR)
        };

        if mode == ComplianceCheckMode::Verbose {
            println!(
                "  {}{:<40}{} minimum:  {}{:<15}{} current:   {}{:<15}{}",
                entry_color,
                format!("{:?}", i.0),
                COLOR_RESET,
                VALUE_COLOR,
                required_value,
                COLOR_RESET,
                support_color,
                supported_value,
                COLOR_RESET
            );
        }
    }

    if mode == ComplianceCheckMode::Verbose {
        println!();
        println!(" {HEADER_COLOR}maximum system{COLOR_RESET}");
    }
    for i in MAX_REQUIRED_SYSTEM.iter() {
        let supported_value = i.0.value();
        let required_value = i.1;

        let (entry_color, support_color) = if supported_value > required_value {
            is_compliant = false;
            (FAILED_COLOR, FAILED_COLOR)
        } else {
            (ENTRY_COLOR, VALUE_COLOR)
        };

        if mode == ComplianceCheckMode::Verbose {
            println!(
                "  {}{:<40}{} maximum:  {}{:<15}{} current:   {}{:<15}{}",
                entry_color,
                format!("{:?}", i.0),
                COLOR_RESET,
                VALUE_COLOR,
                required_value,
                COLOR_RESET,
                support_color,
                supported_value,
                COLOR_RESET
            );
        }
    }

    if mode == ComplianceCheckMode::Verbose {
        println!();
        println!(" {HEADER_COLOR}minimum limits{COLOR_RESET}");
    }
    for i in MIN_REQUIRED_LIMITS.iter() {
        let supported_value = i.0.value();
        let required_value = i.1;

        let (entry_color, support_color) =
            if supported_value < required_value && supported_value != 0 {
                is_compliant = false;
                (FAILED_COLOR, FAILED_COLOR)
            } else {
                (ENTRY_COLOR, VALUE_COLOR)
            };

        let supported_value_str = if supported_value == 0 {
            "[ unlimited ]".to_string()
        } else {
            supported_value.to_string()
        };

        if mode == ComplianceCheckMode::Verbose {
            println!(
                "  {}{:<40}{} minimum:  {}{:<15}{} current:   {}{:<15}{}",
                entry_color,
                format!("{:?}", i.0),
                COLOR_RESET,
                VALUE_COLOR,
                required_value,
                COLOR_RESET,
                support_color,
                supported_value_str,
                COLOR_RESET
            );
        }
    }

    if mode == ComplianceCheckMode::Verbose {
        println!();
        println!(" {HEADER_COLOR}maximum limits{COLOR_RESET}");
    }
    for i in MAX_REQUIRED_LIMITS.iter() {
        let supported_value = i.0.value();
        let required_value = i.1;

        let (entry_color, support_color) =
            if supported_value < required_value && supported_value != 0 {
                is_compliant = false;
                (FAILED_COLOR, FAILED_COLOR)
            } else {
                (ENTRY_COLOR, VALUE_COLOR)
            };

        let supported_value_str = if supported_value == 0 {
            "[ unlimited ]".to_string()
        } else {
            supported_value.to_string()
        };

        if mode == ComplianceCheckMode::Verbose {
            println!(
                "  {}{:<40}{} maximum:  {}{:<15}{} current:   {}{:<15}{}",
                entry_color,
                format!("{:?}", i.0),
                COLOR_RESET,
                VALUE_COLOR,
                required_value,
                COLOR_RESET,
                support_color,
                supported_value_str,
                COLOR_RESET
            );
        }
    }

    if mode == ComplianceCheckMode::Verbose {
        println!();
        println!(" {HEADER_COLOR}features{COLOR_RESET}");
    }
    for i in REQUIRED_FEATURES.iter() {
        let is_supported = i.is_available();

        let (entry_color, support_color) = if !is_supported {
            is_compliant = false;
            (FAILED_COLOR, FAILED_COLOR)
        } else {
            (ENTRY_COLOR, VALUE_COLOR)
        };

        if mode == ComplianceCheckMode::Verbose {
            println!(
                "  {}{:<40}{} required: {}{:<15}{} supported: {}{:<15}{}",
                entry_color,
                format!("{:?}", i),
                COLOR_RESET,
                VALUE_COLOR,
                "true",
                COLOR_RESET,
                support_color,
                is_supported,
                COLOR_RESET
            );
        }
    }

    if mode == ComplianceCheckMode::Verbose {
        println!();
        match !is_compliant {
            true => println!("  {FAILED_COLOR}[ system non-compliant ]{COLOR_RESET}"),
            false => println!("  [ system compliant ]"),
        }
    }

    is_compliant
}
