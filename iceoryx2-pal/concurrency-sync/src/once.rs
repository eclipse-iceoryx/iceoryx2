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

use crate::atomic::AtomicU8;
use crate::atomic::Ordering;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Incomplete = 0,
    InProgress = 1,
    Complete = 2,
}

impl From<u8> for State {
    fn from(value: u8) -> Self {
        match value {
            0 => State::Incomplete,
            1 => State::InProgress,
            2 => State::Complete,
            _ => unreachable!("Invalid state"),
        }
    }
}

/// A spin-based one-time initialization primitive.
///
/// This provides similar functionality to `std::sync::Once` but uses only
/// spin-waiting instead of OS-level blocking primitives.
///
/// # Examples
///
/// ```
/// use iceoryx2_pal_concurrency_sync::once::Once;
///
/// static INIT: Once = Once::new();
///
/// fn initialize() {
///     INIT.call_once(|| {
///         // This will only run once, even if called from multiple threads
///         println!("Initializing...");
///     });
/// }
/// ```
pub struct Once {
    state: AtomicU8,
}

impl Default for Once {
    fn default() -> Self {
        Self::new()
    }
}

impl Once {
    /// Creates a new `Once` in the uninitialized state.
    #[cfg(not(all(test, loom, feature = "std")))]
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(State::Incomplete as u8),
        }
    }

    /// Creates a new `Once` in the uninitialized state.
    #[cfg(all(test, loom, feature = "std"))]
    pub fn new() -> Self {
        Self {
            state: AtomicU8::new(State::Incomplete as u8),
        }
    }

    /// Executes the given closure if this is the first call to `call_once`.
    ///
    /// If multiple threads call this simultaneously, only one will execute
    /// the closure. Other threads will spin-wait until initialization completes.
    ///
    /// # Panics
    ///
    /// If the closure `f` panics, the panic will propagate and the `Once`
    /// will be left in an inconsistent state.
    pub fn call_once<F: FnOnce()>(&self, f: F) {
        if self.load_state() == State::Complete {
            return;
        }

        if self.try_start_init() {
            f();
            self.state.store(State::Complete as u8, Ordering::Release);
        } else {
            while self.load_state() != State::Complete {
                core::hint::spin_loop();
            }
        }
    }

    /// Returns `true` if the initialization has been completed.
    ///
    /// This method can be used to check whether the closure passed to
    /// `call_once` has already been executed without re-triggering initialization.
    pub fn is_completed(&self) -> bool {
        self.load_state() == State::Complete
    }

    #[inline]
    fn load_state(&self) -> State {
        self.state.load(Ordering::Acquire).into()
    }

    #[inline]
    fn try_start_init(&self) -> bool {
        self.state
            .compare_exchange(
                State::Incomplete as u8,
                State::InProgress as u8,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_ok()
    }
}
