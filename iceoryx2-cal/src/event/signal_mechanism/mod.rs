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

use std::time::Duration;

use super::{ListenerCreateError, ListenerWaitError, NotifierNotifyError};

pub mod semaphore;

pub trait SignalMechanism: Send + Sync {
    /// Creates a [`SignalMechanism`]. It cannot be used until
    /// [`SignalMechanism::init()`] was called.
    fn new() -> Self;

    /// Initializes the [`SignalMechanism`] in place.
    ///
    /// # Safety
    ///   * [`SignalMechanism::init()`] was not called before
    unsafe fn init(&mut self) -> Result<(), ListenerCreateError>;

    /// Notifies the counter-part of the [`SignalMechanism`] meaning
    /// that a [`SignalMechanism::try_wait()`], [`SignalMechanism::timed_wait()`]
    /// or [`SignalMechanism::blocking_wait()`] call wakes up and returns true.
    ///
    /// # Safety
    ///   * [`SignalMechanism::init()`] was called before
    ///   * Is not dropped while in use from another process
    unsafe fn notify(&self) -> Result<(), NotifierNotifyError>;

    /// When a signal was received it returns true, otherwise false.
    ///
    /// # Safety
    ///   * [`SignalMechanism::init()`] was called before
    ///   * Is not dropped while in use from another process
    unsafe fn try_wait(&self) -> Result<bool, ListenerWaitError>;

    /// When a signal was received it returns true, otherwise it blocks until
    /// the timeout has passed. If in the meantime a signal arrives it wakes
    /// up and returns true. If no signal was received it returns false.
    ///
    /// This call can be woken up by an operating system SIGINT signal. In this case
    /// it returns [`ListenerWaitError::InterruptSignal`].
    ///
    /// # Safety
    ///   * [`SignalMechanism::init()`] was called before
    ///   * Is not dropped while in use from another process
    unsafe fn timed_wait(&self, timeout: Duration) -> Result<bool, ListenerWaitError>;

    /// When a signal was received it returns true, otherwise it blocks until
    /// a signal was received.
    /// This call can be woken up by an operating system SIGINT signal. In this case
    /// it returns [`ListenerWaitError::InterruptSignal`].
    ///
    /// # Safety
    ///   * [`SignalMechanism::init()`] was called before
    ///   * Is not dropped while in use from another process
    unsafe fn blocking_wait(&self) -> Result<(), ListenerWaitError>;
}
