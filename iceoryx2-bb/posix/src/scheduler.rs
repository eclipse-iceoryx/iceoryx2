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

//! Provides a POSIX [`Scheduler`] abstraction.

use iceoryx2_bb_log::{fail, fatal_panic, warn};
use iceoryx2_pal_posix::*;

use crate::config::DEFAULT_SCHEDULER;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum SchedulerConversionError {
    UnknownScheduler,
}

/// Represents the scheduler in a POSIX system.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum Scheduler {
    Fifo = posix::SCHED_FIFO,
    RoundRobin = posix::SCHED_RR,
    Other = posix::SCHED_OTHER,
}

impl Default for Scheduler {
    fn default() -> Self {
        DEFAULT_SCHEDULER
    }
}

/// Signals the scheduler to put the running thread at the end of the execution queue.
pub fn yield_now() {
    if unsafe { posix::sched_yield() } != 0 {
        warn!(from "yield_now()", "Failed to yield the scheduler.");
    }
}

impl Scheduler {
    /// Returns the priority granularity of the scheduler. It is defined as the distance
    /// between the minimum priority and maximum priority. The [`crate::process::Process`]
    /// creates an uniform priority range from 0..255.
    /// A granularity of 50 for instance requires a difference of at least 5 to introduce
    /// a change in the actual priority in [`crate::process::Process::set_priority()`]
    pub fn priority_granularity(&self) -> u8 {
        (self.max_priority() - self.min_priority()).unsigned_abs() as u8
    }

    fn min_priority(&self) -> i32 {
        match unsafe { posix::sched_get_priority_min(*self as i32) } {
            -1 => {
                fatal_panic!("This should never happen! Unable to acquire minimum priority for scheduler {:#?}.", self);
            }
            v => v,
        }
    }

    fn max_priority(&self) -> i32 {
        match unsafe { posix::sched_get_priority_max(*self as i32) } {
            -1 => {
                fatal_panic!("This should never happen! Unable to acquire maximum priority for scheduler {:#?}.", self);
            }
            v => v,
        }
    }

    pub(crate) fn from_int(value: posix::int) -> Result<Scheduler, SchedulerConversionError> {
        match value {
            posix::SCHED_FIFO => Ok(Scheduler::Fifo),
            posix::SCHED_RR => Ok(Scheduler::RoundRobin),
            posix::SCHED_OTHER => Ok(Scheduler::Other),
            v => {
                fail!(from "Scheduler::from_int", with SchedulerConversionError::UnknownScheduler,
                    "Unable to extract scheduler from int value {}. Maybe the scheduler is not supported by the implementation.", v);
            }
        }
    }

    pub(crate) fn get_priority_from(&self, parameter: &posix::sched_param) -> u8 {
        let priority = parameter.sched_priority;

        let max_priority = self.max_priority();
        let min_priority = self.min_priority();

        let relative_prio: f32 = match max_priority > min_priority {
            true => ((priority - min_priority) as f32 / (max_priority - min_priority) as f32)
                .clamp(0.0, 1.0),
            false => (1.0
                - (priority - max_priority) as f32 / (min_priority - max_priority) as f32)
                .clamp(0.0, 1.0),
        };

        (u8::MAX as f32 * relative_prio) as u8
    }

    pub(crate) fn policy_specific_priority(&self, priority: u8) -> i32 {
        let max_priority = self.max_priority();
        let min_priority = self.min_priority();

        let range = match max_priority > min_priority {
            true => max_priority - min_priority,
            false => min_priority - max_priority,
        };

        let relative_prio: f32 = (priority as f32 / u8::MAX as f32).clamp(0.0, 1.0);

        match max_priority > min_priority {
            true => (range as f32 * relative_prio) as i32 + min_priority,
            false => (range as f32 - range as f32 * relative_prio) as i32 + max_priority,
        }
    }
}
