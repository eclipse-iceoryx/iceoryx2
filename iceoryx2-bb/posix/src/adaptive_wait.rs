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

//! [`AdaptiveWait`] is a building block which can be integrated into busy loops to make
//! them less CPU consuming.
//!
//! The strategy is that for [`ADAPTIVE_WAIT_YIELD_REPETITIONS`] the
//! wait call will yield and then it will increase its waiting time to
//! [`ADAPTIVE_WAIT_INITIAL_WAITING_TIME`] for the next [`ADAPTIVE_WAIT_INITIAL_REPETITIONS`].
//! After that every further wait will wait [`ADAPTIVE_WAIT_FINAL_WAITING_TIME`]
//!
//! # Examples
//! ```ignore
//! use iceoryx2_bb_posix::adaptive_wait::*;
//! use iceoryx2_bb_posix::clock::*;
//!
//! let mut adaptive_wait = AdaptiveWaitBuilder::new()
//!     .clock_type(ClockType::Monotonic)
//!     .create().expect("Unable to create adaptive wait");
//!
//! for i in 1..1000 {
//!     // loop which waits for some event
//!     adaptive_wait.wait().expect("unable to wait");
//! }
//! ```

use core::fmt::Debug;
use core::time::Duration;

use crate::clock::*;
use crate::config::{
    ADAPTIVE_WAIT_FINAL_WAITING_TIME, ADAPTIVE_WAIT_INITIAL_REPETITIONS,
    ADAPTIVE_WAIT_INITIAL_WAITING_TIME, ADAPTIVE_WAIT_YIELD_REPETITIONS,
};
use crate::scheduler::yield_now;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_log::fail;

/// The AdaptiveWaitBuilder is required to produce an [`AdaptiveWait`] object.
/// The default value for clock is defined in [`ClockType::default()`].
#[derive(Debug, Default)]
pub struct AdaptiveWaitBuilder {
    clock_type: ClockType,
}

impl AdaptiveWaitBuilder {
    pub fn new() -> AdaptiveWaitBuilder {
        Self::default()
    }

    pub fn clock_type(mut self, clock_type: ClockType) -> Self {
        self.clock_type = clock_type;
        self
    }

    pub fn create(self) -> Result<AdaptiveWait, TimeError> {
        AdaptiveWait::new(self)
    }
}

enum_gen! { AdaptiveWaitError
  mapping:
    NanosleepError,
    TimeError
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum AdaptiveTimedWaitWhileError<T: Debug> {
    AdaptiveWaitError(AdaptiveWaitError),
    PredicateFailure(T),
}

impl<T: Debug> From<T> for AdaptiveTimedWaitWhileError<T> {
    fn from(v: T) -> Self {
        AdaptiveTimedWaitWhileError::PredicateFailure(v)
    }
}

/// AdaptiveWait is a building block which can be integrated into busy loops to make
/// them less CPU consuming. The strategy is that for [`ADAPTIVE_WAIT_YIELD_REPETITIONS`] the
/// wait call will yield and then it will increase its waiting time to
/// [`ADAPTIVE_WAIT_INITIAL_WAITING_TIME`] for the next [`ADAPTIVE_WAIT_INITIAL_REPETITIONS`].
/// After that every further wait will wait [`ADAPTIVE_WAIT_FINAL_WAITING_TIME`]
#[derive(Debug)]
pub struct AdaptiveWait {
    yield_count: u64,
    clock_type: ClockType,
    start_time: Time,
}

impl AdaptiveWait {
    fn new(config: AdaptiveWaitBuilder) -> Result<Self, TimeError> {
        Ok(AdaptiveWait {
            yield_count: 0,
            clock_type: config.clock_type,
            start_time: fail!(from config, when Time::now_with_clock(config.clock_type),
                            "Unable to create AdaptiveWait since the Time could not be acquired."),
        })
    }

    /// Returns how main times wait() was called or how often the clojure in wait_while()
    /// was invoked.
    pub fn yield_count(&self) -> u64 {
        self.yield_count
    }

    pub fn clock_type(&self) -> ClockType {
        self.clock_type
    }

    /// Wait in a less busy wait.
    pub fn wait(&mut self) -> Result<Duration, AdaptiveWaitError> {
        let msg = "Failure while waiting";
        self.wait_impl()?;

        Ok(fail!(from self, when self.start_time.elapsed(),
                "{} due to a failure while acquiring the elapsed time.", msg))
    }

    /// Wait until the predicate returns false
    ///
    /// # Examples
    /// ```
    /// use iceoryx2_bb_posix::adaptive_wait::*;
    ///
    /// let mut counter = 0;
    /// AdaptiveWaitBuilder::new()
    ///     .create().unwrap().wait_while(move || -> bool {
    ///         counter += 1;
    ///         counter < 10
    ///     });
    /// ```
    pub fn wait_while<F: FnMut() -> bool>(
        &mut self,
        mut predicate: F,
    ) -> Result<Duration, AdaptiveWaitError> {
        let msg = "Failed to wait with predicate";
        while predicate() {
            fail!(from self, when self.wait_impl(), "{} since the underlying wait failed.", msg);
        }

        Ok(fail!(from self, when self.start_time.elapsed(),
                "{} due to a failure while acquiring the elapsed time.", msg))
    }

    /// Wait until the predicate returns false or the timeout has passed
    ///
    /// # Examples
    ///
    /// ```
    /// use iceoryx2_bb_posix::adaptive_wait::*;
    /// use core::time::Duration;
    ///
    /// AdaptiveWaitBuilder::new()
    ///     .create().unwrap().timed_wait_while(|| -> Result<bool, ()> { Ok(true) },
    ///                                         Duration::from_millis(50));
    /// ```
    pub fn timed_wait_while<T: Debug, F: FnMut() -> Result<bool, T>>(
        &mut self,
        mut predicate: F,
        timeout: Duration,
    ) -> Result<bool, AdaptiveTimedWaitWhileError<T>> {
        let msg = "Failed to wait with predicate and timeout";

        loop {
            if !predicate()? {
                return Ok(true);
            }

            let result = self.wait();
            if result.is_err() {
                fail!(from self, with AdaptiveTimedWaitWhileError::AdaptiveWaitError(result.err().unwrap()),
                    "{} since the underlying wait failed.", msg);
            }

            if result.unwrap() > timeout {
                break;
            }
        }

        Ok(false)
    }

    fn wait_impl(&mut self) -> Result<(), AdaptiveWaitError> {
        let msg = "Failure while waiting";
        self.yield_count += 1;

        if self.yield_count <= ADAPTIVE_WAIT_YIELD_REPETITIONS {
            yield_now();
        } else {
            let waiting_time = if self.yield_count <= ADAPTIVE_WAIT_INITIAL_REPETITIONS {
                ADAPTIVE_WAIT_INITIAL_WAITING_TIME
            } else {
                ADAPTIVE_WAIT_FINAL_WAITING_TIME
            };
            fail!(from self, when nanosleep_with_clock(waiting_time, self.clock_type),
                "{} due to a failure while sleeping.", msg);
        }

        Ok(())
    }
}
