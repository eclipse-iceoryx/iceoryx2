// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_log::fatal_panic;
use iceoryx2_pal_posix::posix::{self, Errno, MemZeroedStruct};

use crate::signal::Signal;

/// Represents a posix signal set.
#[derive(Debug)]
pub struct SignalSet {
    signal_set: posix::sigset_t,
}

impl SignalSet {
    /// Creates a new [`SignalSet`] that contains all pending signals, e.g.
    /// signals that are blocked from delivery, for the calling
    /// [`Thread`](crate::posix::thread::Thread).
    pub fn from_pending() -> Self {
        let mut new_self = SignalSet {
            signal_set: MemZeroedStruct::new_zeroed(),
        };

        if unsafe { posix::sigpending(&mut new_self.signal_set) } == -1 {
            fatal_panic!(
                from new_self,
                "This should never happen! Failed to acquire all pending signals ({:?}).",
                Errno::get());
        }

        new_self
    }

    /// Initializes an empty [`SignalSet`].
    pub fn new_empty() -> Self {
        let mut new_self = SignalSet {
            signal_set: MemZeroedStruct::new_zeroed(),
        };

        if unsafe { posix::sigemptyset(&mut new_self.signal_set) } == -1 {
            fatal_panic!(
                from new_self,
                "This should never happen! Failed to initialized empty signal set ({:?}).",
                Errno::get());
        }

        new_self
    }

    /// Initializes a [`SignalSet`] so that every signal is included.
    pub fn new_filled() -> Self {
        let mut new_self = SignalSet {
            signal_set: MemZeroedStruct::new_zeroed(),
        };

        if unsafe { posix::sigfillset(&mut new_self.signal_set) } == -1 {
            fatal_panic!(
                from new_self,
                "This should never happen! Failed to initialized filled signal set ({:?}).",
                Errno::get());
        }

        new_self
    }

    /// Returns [`true`] if the provided [`Signal`] is contained in the
    /// [`SignalSet`], otherwise [`false`].
    pub fn contains(&self, signal: Signal) -> bool {
        let result = unsafe { posix::sigismember(&self.signal_set, signal as i32) };

        if result == -1 {
            fatal_panic!(from self,
                "This should never happen! Unable to determine if the signal {signal:?} is part of the signal set ({:?})",
                Errno::get())
        }

        result == 1
    }

    /// Adds a [`Signal`] to the [`SignalSet`].
    pub fn add(&mut self, signal: Signal) {
        let result = unsafe { posix::sigaddset(&mut self.signal_set, signal as i32) };

        if result == -1 {
            fatal_panic!(from self,
                "This should never happen! Unable to add the signal {signal:?} to the signal set ({:?}).",
                Errno::get());
        }
    }

    /// Removes a [`Signal`] from the [`SignalSet`]
    pub fn remove(&mut self, signal: Signal) {
        let result = unsafe { posix::sigdelset(&mut self.signal_set, signal as i32) };

        if result == -1 {
            fatal_panic!(from self,
                "This should never happen! Unable to remove the signal {signal:?} from the signal set ({:?}).",
                Errno::get());
        }
    }
}
