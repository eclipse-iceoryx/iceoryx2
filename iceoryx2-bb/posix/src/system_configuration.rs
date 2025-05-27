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

//! Provides information about the POSIX [`SystemInfo`], [`Limit`]s, available [`SysOption`] and
//! [`Feature`]s.

use enum_iterator::Sequence;
use iceoryx2_bb_container::semantic_string::*;
use iceoryx2_bb_log::{fatal_panic, warn};
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_pal_posix::{posix::MemZeroedStruct, *};

/// The global config path of the system, where all config files shall be stored.
pub fn get_global_config_path() -> Path {
    fatal_panic!(from "get_global_config_path",
        when Path::new(iceoryx2_pal_configuration::GLOBAL_CONFIG_PATH),
        "This should never happen! The underlying platform GLOBAL_CONFIG_PATH variable contains a path with invalid characters.")
}

pub fn get_user_config_path() -> Path {
    fatal_panic!(from "get_user_config_path",
        when Path::new(iceoryx2_pal_configuration::USER_CONFIG_PATH),
        "This should never happen! The underlying platform USER_CONFIG_PATH variable contains a path with invalid characters.")
}

/// Generic information about the POSIX system.
/// ```
/// use iceoryx2_bb_posix::system_configuration::*;
///
/// println!("{}", SystemInfo::PageSize.value());
/// ```
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Sequence)]
#[repr(i32)]
pub enum SystemInfo {
    PosixVersion = posix::_SC_VERSION,
    PageSize = posix::_SC_PAGESIZE,
    NumberOfClockTicksPerSecond = posix::_SC_CLK_TCK,
    NumberOfCpuCores = posix::_SC_NPROCESSORS_CONF,
}

impl SystemInfo {
    /// Returns the system specific value to the SystemInfo field
    pub fn value(&self) -> usize {
        let result = unsafe { posix::sysconf(*self as i32) };
        result.clamp(0, posix::long::MAX) as usize
    }
}

/// The minimum and maximum limits of the POSIXs constructs.
/// ```
/// use iceoryx2_bb_posix::system_configuration::*;
///
/// println!("{}", Limit::MaxLengthOfHostname.value());
/// ```
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Sequence)]
#[repr(i32)]
pub enum Limit {
    MaxLengthOfHostname = posix::_SC_HOST_NAME_MAX,
    MaxLengthOfLoginName = posix::_SC_LOGIN_NAME_MAX,
    MaxNumberOfSupplementaryGroupIds = posix::_SC_NGROUPS_MAX,
    MaxNumberOfOpenFiles = posix::_SC_OPEN_MAX,
    MaxNumberOfSimultaneousProcessesPerUser = posix::_SC_CHILD_MAX,
    MaxLengthOfArguments = posix::_SC_ARG_MAX,
    MaxNumberOfOpenStreams = posix::_SC_STREAM_MAX,
    MaxNumberOfSymbolicLinks = posix::_SC_SYMLOOP_MAX,
    MaxLengthOfTerminalDeviceName = posix::_SC_TTY_NAME_MAX,
    MaxNumberOfBytesInATimezone = posix::_SC_TZNAME_MAX,
    MaxNumberOfSemaphores = posix::_SC_SEM_NSEMS_MAX,
    MaxSemaphoreValue = posix::_SC_SEM_VALUE_MAX,
    MaxNumberOfOpenMessageQueues = posix::_SC_MQ_OPEN_MAX,
    MaxMessageQueuePriority = posix::_SC_MQ_PRIO_MAX,
    MaxNumberOfThreads = posix::_SC_THREAD_THREADS_MAX,
    MaxSizeOfPasswordBuffer = posix::_SC_GETPW_R_SIZE_MAX,
    MaxSizeOfGroupBuffer = posix::_SC_GETGR_R_SIZE_MAX,
    MaxPathLength = -posix::_PC_PATH_MAX,
    MaxFileNameLength = -posix::_PC_NAME_MAX,
    MaxUnixDomainSocketNameLength = i32::MIN + 1,
    MinStackSizeOfThread = posix::_SC_THREAD_STACK_MIN,
}

impl Limit {
    /// Returns the system specific limit
    pub fn value(&self) -> u64 {
        match self {
            Limit::MaxPathLength | Limit::MaxFileNameLength => {
                let result = unsafe {
                    posix::pathconf("/".as_ptr() as *const posix::c_char, -(*self as i32))
                };
                result.clamp(0, posix::long::MAX) as u64
            }
            Limit::MaxUnixDomainSocketNameLength => {
                let s = posix::sockaddr_un::new_zeroed();
                s.sun_path.len() as u64
            }
            v => {
                let result = unsafe { posix::sysconf(*v as i32) };
                result.clamp(0, posix::long::MAX) as u64
            }
        }
    }
}

/// Can be used to verify if a POSIX system option is available at the system.
/// ```
/// use iceoryx2_bb_posix::system_configuration::*;
///
/// let option = SysOption::Ipv6;
/// println!("is available: {}, details: {}", option.is_available(), option.value());
/// ```
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Sequence)]
#[repr(i32)]
pub enum SysOption {
    AdvisoryInfo = posix::_SC_ADVISORY_INFO,
    CpuTime = posix::_SC_CPUTIME,
    Fsync = posix::_SC_FSYNC,
    Ipv6 = posix::_SC_IPV6,
    MemLock = posix::_SC_MEMLOCK,
    MemLockRange = posix::_SC_MEMLOCK_RANGE,
    MessagePassing = posix::_SC_MESSAGE_PASSING,
    PrioritizedIo = posix::_SC_PRIORITIZED_IO,
    PriorityScheduling = posix::_SC_PRIORITY_SCHEDULING,
    RegularExpressions = posix::_SC_REGEXP,
    Spawn = posix::_SC_SPAWN,
    ProcessSporadicServer = posix::_SC_SPORADIC_SERVER,
    SynchronizedIo = posix::_SC_SYNCHRONIZED_IO,
    ThreadStackAddress = posix::_SC_THREAD_ATTR_STACKADDR,
    ThreadStackSize = posix::_SC_THREAD_ATTR_STACKSIZE,
    ThreadCpuTimeClock = posix::_SC_THREAD_CPUTIME,
    ThreadExecutionScheduling = posix::_SC_THREAD_PRIORITY_SCHEDULING,
    ThreadProcessSharedSynchronization = posix::_SC_THREAD_PROCESS_SHARED,
    MutexPriorityInheritance = posix::_SC_THREAD_PRIO_INHERIT,
    MutexPriorityProtection = posix::_SC_THREAD_PRIO_PROTECT,
    RobustMutexPriorityInhertiance = posix::_SC_THREAD_ROBUST_PRIO_INHERIT,
    RobustMutexPriorityProtection = posix::_SC_THREAD_ROBUST_PRIO_PROTECT,
    ThreadSporadicServer = posix::_SC_THREAD_SPORADIC_SERVER,
    Trace = posix::_SC_TRACE,
    TraceEventFilter = posix::_SC_TRACE_EVENT_FILTER,
    TraceInherit = posix::_SC_TRACE_INHERIT,
    TraceLog = posix::_SC_TRACE_LOG,
    TypedMemoryObjects = posix::_SC_TYPED_MEMORY_OBJECTS,
}

impl SysOption {
    /// Returns true if the system option is available on the system, otherwise false.
    pub fn is_available(&self) -> bool {
        self.value() > 0
    }

    /// Returns the value of that system option. Most likely it is the POSIX version to which the
    /// option is compliant.
    pub fn value(&self) -> u64 {
        let result = unsafe { posix::sysconf(*self as i32) };
        result.clamp(0, posix::long::MAX) as u64
    }
}

/// Can be used to verify if a POSIX feature is available at the system.
/// ```
/// use iceoryx2_bb_posix::system_configuration::*;
///
/// let feature = Feature::Threads;
/// println!("is available: {}, details: {}", feature.is_available(), feature.value());
/// ```
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Sequence)]
#[repr(i32)]
pub enum Feature {
    Barriers = posix::_SC_BARRIERS,
    AsynchronousIo = posix::_SC_ASYNCHRONOUS_IO,
    ClockSelection = posix::_SC_CLOCK_SELECTION,
    JobControl = posix::_SC_JOB_CONTROL,
    MappedFiles = posix::_SC_MAPPED_FILES,
    MemoryProtection = posix::_SC_MEMORY_PROTECTION,
    MonotonicClock = posix::_SC_MONOTONIC_CLOCK,
    RawSockets = posix::_SC_RAW_SOCKETS,
    ReaderWriterLocks = posix::_SC_READER_WRITER_LOCKS,
    RealtimeSignals = posix::_SC_REALTIME_SIGNALS,
    SavedUserAndGroupIds = posix::_SC_SAVED_IDS,
    Semaphores = posix::_SC_SEMAPHORES,
    SharedMemoryObjects = posix::_SC_SHARED_MEMORY_OBJECTS,
    Shell = posix::_SC_SHELL,
    SpinLocks = posix::_SC_SPIN_LOCKS,
    ThreadSafeFunctions = posix::_SC_THREAD_SAFE_FUNCTIONS,
    Threads = posix::_SC_THREADS,
    Timeouts = posix::_SC_TIMEOUTS,
    Timers = posix::_SC_TIMERS,
    OpenRealtimeOptionGroup = posix::_SC_XOPEN_REALTIME,
    OpenRealtimeThreadsOptionGroup = posix::_SC_XOPEN_REALTIME_THREADS,
}

impl Feature {
    /// Returns true if the feature is available on the system, otherwise false.
    pub fn is_available(&self) -> bool {
        self.value() > 0
    }

    /// Returns the value of that feature. Most likely it is the POSIX version to which the
    /// feature is compliant.
    pub fn value(&self) -> u64 {
        let result = unsafe { posix::sysconf(*self as i32) };
        result.clamp(0, posix::long::MAX) as u64
    }
}

/// Can be used to get or set the process resource limits.
/// ```
/// use iceoryx2_bb_posix::system_configuration::*;
///
/// let limit = ProcessResourceLimit::MaxStackSize;
/// println!("soft-limit: {}, hard-limit: {}", limit.soft_limit(), limit.hard_limit());
/// ```
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Sequence)]
#[repr(u64)]
pub enum ProcessResourceLimit {
    MaxCoreFileSize = posix::RLIMIT_CORE as _,
    MaxConsumableCpuTime = posix::RLIMIT_CPU as _,
    MaxDataSegmentSize = posix::RLIMIT_DATA as _,
    MaxFileSize = posix::RLIMIT_FSIZE as _,
    MaxNumberOfOpenFileDescriptors = posix::RLIMIT_NOFILE as _,
    MaxStackSize = posix::RLIMIT_STACK as _,
    MaxSizeOfTotalMemory = posix::RLIMIT_AS as _,
}

impl ProcessResourceLimit {
    /// Returns the soft limit or current maximum value of the resource
    pub fn soft_limit(&self) -> u64 {
        let mut result: posix::rlimit = posix::rlimit::new_zeroed();
        if unsafe { posix::getrlimit(*self as i32, &mut result) } == -1 {
            fatal_panic!(from "ProcessResourceLimit::soft_limit",
                "This should never happen! Unable to acquire soft limit for {:?} due to an unknown error ({}).",
                *self, posix::Errno::get());
        }
        result.rlim_cur as _
    }

    /// Returns the maximum value to which the soft limit can be increased
    pub fn hard_limit(&self) -> u64 {
        let mut result: posix::rlimit = posix::rlimit::new_zeroed();
        if unsafe { posix::getrlimit(*self as i32, &mut result) } == -1 {
            fatal_panic!(from "ProcessResourceLimit::hard_limit",
                "This should never happen! Unable to acquire hard limit for {:?} due to an unknown error ({}).",
                *self, posix::Errno::get());
        }
        result.rlim_max as _
    }

    /// Adjusts the soft limit. If the soft limit value is greater than the hard limit it will be
    /// set to the hard limit.
    pub fn set_soft_limit(&self, value: u64) {
        let msg = "Unable to update soft limit to ".to_string()
            + value.to_string().as_str()
            + " for resource ";

        let hard_limit = self.hard_limit();
        if hard_limit < value {
            warn!(from "ProcessResourceLimit::set_soft_limit", "{} {:?} since we would exceed the hard limit of {}. The soft limit will be saturated to the hard limit.",
                msg, *self, hard_limit);
        }

        let new_value = posix::rlimit {
            rlim_cur: core::cmp::min(value, hard_limit) as _,
            rlim_max: self.hard_limit() as _,
        };

        if unsafe { posix::setrlimit(*self as i32, &new_value) } == -1 {
            fatal_panic!(from "ProcessResourceLimit::set_soft_limit", "This should never happen! {} {:?} due to an unknown error({}).", msg, *self, posix::Errno::get());
        }
    }

    /// Adjusts the hard limit. If the hard limit value is smaller than the soft limit it will be
    /// set to the soft limit.
    pub fn set_hard_limit(&self, value: u64) -> bool {
        let msg = "Unable to update hard limit to ".to_string()
            + value.to_string().as_str()
            + " for resource ";

        let soft_limit = self.soft_limit();
        if soft_limit > value {
            warn!(from "ProcessResourceLimit::set_hard_limit", "{} {:?} since it would be smaller than the current soft limit of {}. The hard limit will be saturated to the soft limit.",
                msg, *self, soft_limit);
        }

        let new_value = posix::rlimit {
            rlim_cur: soft_limit as _,
            rlim_max: core::cmp::max(value, soft_limit) as _,
        };

        if unsafe { posix::setrlimit(*self as i32, &new_value) } == -1 {
            match posix::Errno::get() {
                posix::Errno::EPERM => {
                    warn!(from "ProcessResourceLimit::set_hard_limit", "{} {:?} due to insufficient permissions.", msg, *self);
                    return false;
                }
                v => {
                    fatal_panic!(from "ProcessResourceLimit::set_hard_limit", "This should never happen! {} {:?} due to an unknown error({}).", msg, *self, v)
                }
            }
        }

        true
    }
}
