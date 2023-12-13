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

//! Contains several variants of POSIX condition variables. A mutex is always included, either as
//! reference or the object itself.
//!
//! Furthermore, we distinct between
//! * [`MultiConditionVariable`] - waiters can wait on multiple conditions with the draw back that
//!                                notify_one is not available
//! * [`ConditionVariable`] - the condition has to be provided on construction but notify_one is
//!                           available

use crate::clock::{AsTimespec, Time, TimeError};
use crate::handle_errno;
use crate::{clock::ClockType, mutex::*};
use iceoryx2_bb_elementary::{enum_gen, scope_guard::*};
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::posix::Struct;
use iceoryx2_pal_posix::*;
use std::ops::{Deref, DerefMut};
use std::time::Duration;
use std::{cell::UnsafeCell, fmt::Debug};
use tiny_fn::tiny_fn;

pub use crate::mutex::MutexHandle;

enum_gen! { ConditionVariableCreationError
  mapping:
    MutexCreationError,
    ConditionVariableWithSegregatedMutexCreationError
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum ConditionVariableWithSegregatedMutexCreationError {
    InsufficientMemory,
    InsufficientResources,
    NoInterProcessSupport,
    UnsupportedClockType,
    MutexAndMultiConditionVariableClockTypeDiffer,
    InternalMemoryAlreadyInitializedWithMultiConditionVariable,
    UnknownError(i32),
}

#[derive(Debug)]
pub enum ConditionVariableWaitError<'mutex, 'handle, T: Debug> {
    MutexLockError(MutexLockError<'mutex, 'handle, T>),
    UnknownError(i32),
}

#[derive(Debug)]
pub enum ConditionVariableTimedWaitError<'mutex, 'handle, T: Debug> {
    MutexLockError(MutexLockError<'mutex, 'handle, T>),
    UnableToGetCurrentTime(TimeError),
    UnknownError(i32),
}

/// The [`ConditionVariableError`] enum is a generalization when one doesn't require the fine-grained
/// error handling enums. One can forward forward [`ConditionVariableError`] as more generic return
/// value when a method returns a Condition***Error.
/// On a higher level it is again convertable into [`crate::Error`].
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum ConditionVariableError {
    CreationFailed,
    WaitFailed,
    TriggerOrLockFailed,
}

impl<'mutex, 'handle, T: Debug> From<TimeError>
    for ConditionVariableTimedWaitError<'mutex, 'handle, T>
{
    fn from(v: TimeError) -> Self {
        ConditionVariableTimedWaitError::UnableToGetCurrentTime(v)
    }
}

impl From<ConditionVariableCreationError> for ConditionVariableError {
    fn from(_: ConditionVariableCreationError) -> Self {
        ConditionVariableError::CreationFailed
    }
}

impl From<ConditionVariableWithSegregatedMutexCreationError> for ConditionVariableError {
    fn from(_: ConditionVariableWithSegregatedMutexCreationError) -> Self {
        ConditionVariableError::CreationFailed
    }
}

impl<'mutex, 'handle, T: Debug> From<ConditionVariableWaitError<'mutex, 'handle, T>>
    for ConditionVariableError
{
    fn from(_: ConditionVariableWaitError<'mutex, 'handle, T>) -> Self {
        ConditionVariableError::WaitFailed
    }
}

impl<'mutex, 'handle, T: Debug> From<ConditionVariableTimedWaitError<'mutex, 'handle, T>>
    for ConditionVariableError
{
    fn from(_: ConditionVariableTimedWaitError<'mutex, 'handle, T>) -> Self {
        ConditionVariableError::WaitFailed
    }
}

impl<'mutex, 'handle, T: Debug> From<MutexLockError<'mutex, 'handle, T>>
    for ConditionVariableError
{
    fn from(_: MutexLockError<'mutex, 'handle, T>) -> Self {
        ConditionVariableError::TriggerOrLockFailed
    }
}

impl<'mutex, 'handle, T: Debug> From<MutexLockError<'mutex, 'handle, T>>
    for ConditionVariableWaitError<'mutex, 'handle, T>
{
    fn from(v: MutexLockError<'mutex, 'handle, T>) -> Self {
        ConditionVariableWaitError::MutexLockError(v)
    }
}

impl<'mutex, 'handle, T: Debug> From<MutexLockError<'mutex, 'handle, T>>
    for ConditionVariableTimedWaitError<'mutex, 'handle, T>
{
    fn from(v: MutexLockError<'mutex, 'handle, T>) -> Self {
        ConditionVariableTimedWaitError::MutexLockError(v)
    }
}

/// Builder for the [`MultiConditionVariable`] and the [`ConditionVariable`]. It creates a condition
/// variable which already includes a mutex. The design idea originated from the fact that waiting
/// on the same condition variable with two distinct mutexes is undefined behavior according to the
/// posix standard. An integrated mutex avoids this possible pitfall.
///
/// # Defaults
///
/// As default the condition variable is
/// [is_interprocess_capable](ConditionVariableBuilder::is_interprocess_capable()).
/// For the internal mutex the defaults from [`MutexBuilder`] are used.
///
/// # Example
/// ```
/// use iceoryx2_bb_posix::condition_variable::*;
/// use iceoryx2_bb_posix::clock::*;
/// use iceoryx2_bb_posix::mutex::*;
///
///
/// // create a condition variable which allows multiple predicates in wait_while nad
/// // timed_wait_while
/// let mtx_handle = MutexHandle::<i32>::new();
/// let condvar = ConditionVariableBuilder::new()
///                     .is_interprocess_capable(true)
///                     // this is applied to the mutex and the condition variable
///                     .clock_type(ClockType::Monotonic)
///                     // begin: mutex settings
///                     .mutex_type(MutexType::WithDeadlockDetection)
///                     .thread_termination_behavior(MutexThreadTerminationBehavior::ReleaseWhenLocked)
///                     .priority_inheritance(MutexPriorityInheritance::None)
///                     // end: mutex settings
///                     .create_multi_condition_variable(123, &mtx_handle);
///
/// // create a condition variable which requires a predicate on creation but enables methods like
/// // notify_one, trigger_one ...
/// let mtx_handle = MutexHandle::<ConditionVariableData<i32>>::new();
/// let condvar = ConditionVariableBuilder::new()
///                     // do not configure anything and let's use the default settings
///                     .create_condition_variable(0, |v| *v >= 100, &mtx_handle);
/// ```
pub struct ConditionVariableBuilder {
    is_interprocess_capable: bool,
    clock_type: ClockType,
    mutex: MutexBuilder,
}

impl Default for ConditionVariableBuilder {
    fn default() -> Self {
        Self {
            is_interprocess_capable: false,
            clock_type: ClockType::default(),
            mutex: MutexBuilder::new(),
        }
    }
}

impl ConditionVariableBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable interprocess capabilities for the condition variable and the underlying mutex
    pub fn is_interprocess_capable(mut self, value: bool) -> Self {
        self.is_interprocess_capable = value;
        self.mutex.is_interprocess_capable = value;
        self
    }

    /// Defines the type of clock which should be used for timed_wait and timed_wait_while.
    pub fn clock_type(mut self, value: ClockType) -> Self {
        self.clock_type = value;
        self.mutex.clock_type = value;
        self
    }

    /// Defines the mutex type of the underlying mutex
    pub fn mutex_type(mut self, value: MutexType) -> Self {
        self.mutex.mutex_type = value;
        self
    }

    /// Defines the thread termination behavior of the underlying mutex
    pub fn thread_termination_behavior(mut self, value: MutexThreadTerminationBehavior) -> Self {
        self.mutex.thread_termination_behavior = value;
        self
    }

    /// Defines the priority inheritance of the underlying mutex
    pub fn priority_inheritance(mut self, value: MutexPriorityInheritance) -> Self {
        self.mutex.priority_inheritance = value;
        self
    }

    /// Defines the priority ceiling of the underlying mutex
    pub fn priority_ceiling(mut self, value: Option<i32>) -> Self {
        self.mutex.priority_ceiling = value;
        self
    }

    /// Creates a [`MultiConditionVariable`] encapsulating the value of type T which can be
    /// modified and used for triggering.
    ///
    /// The condition variable can use
    /// multiple conditions in [`MultiConditionVariable::wait_while()`] and
    /// [`MultiConditionVariable::timed_wait_while()`] but is only able to trigger all waiters.
    pub fn create_multi_condition_variable<T: Debug>(
        self,
        t: T,
        handle: &MutexHandle<T>,
    ) -> Result<MultiConditionVariable<T>, ConditionVariableCreationError> {
        MultiConditionVariable::new(t, self, handle)
    }

    /// Creates a [`ConditionVariable`] encapsulating the value of type T which can be modified and
    /// used for triggering.
    ///
    /// The condition variable has one fixed
    /// condition which has to be provided on construction. The methods
    /// [`ConditionVariable::wait_while()`] and
    /// [`ConditionVariable::timed_wait_while()`] will wait on that preset condition until
    /// it is satisfied.
    /// The restriction to a preset fixed condition comes with the feature to signal single waiters
    /// with [`ConditionVariable::trigger_one()`] for instance.
    pub fn create_condition_variable<T: Debug, F: Fn(&T) -> bool + 'static>(
        self,
        t: T,
        predicate: F,
        handle: &MutexHandle<ConditionVariableData<T>>,
    ) -> Result<ConditionVariable<T>, ConditionVariableCreationError> {
        ConditionVariable::new(t, predicate, self, handle)
    }
}

pub(super) mod internal {
    use super::*;

    pub enum WaitState {
        Success,
        Timeout,
    }

    #[derive(Debug)]
    pub struct ConditionVariableInternal {
        pub(super) handle: UnsafeCell<posix::pthread_cond_t>,
        pub(super) clock_type: ClockType,
    }

    impl ConditionVariableInternal {
        pub(super) fn new(
            clock_type: ClockType,
            is_interprocess_capable: bool,
        ) -> Result<Self, ConditionVariableWithSegregatedMutexCreationError> {
            let new_cv = Self {
                handle: UnsafeCell::new(posix::pthread_cond_t::new()),
                clock_type,
            };

            let msg = "Unable to create condition variable";
            let mut attributes = ScopeGuardBuilder::new(
                posix::pthread_condattr_t::new()
            )
            .on_init(|attr| {
                handle_errno!(ConditionVariableWithSegregatedMutexCreationError, from "ConditionVariableInternal",
                    errno_source unsafe{ posix::pthread_condattr_init(attr)}.into(),
                    success Errno::ESUCCES => (),
                    Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory to create condition variable attributes.", msg),
                    v => (UnknownError(v as i32), "{} since an unknown error occurred while creating condition variable attributes ({}).", msg, v)

                );}
            )
            .on_drop(|attr| match unsafe { posix::pthread_condattr_destroy(attr) } {
                0 => (),
                _ => {
                    fatal_panic!(
                        "MultiConditionVariableInternal, unable to destroy condition variable attributes - possible leak?");
                }
            })
            .create()?;

            if unsafe {
                posix::pthread_condattr_setpshared(
                    attributes.get_mut(),
                    match is_interprocess_capable {
                        true => posix::PTHREAD_PROCESS_SHARED,
                        false => posix::PTHREAD_PROCESS_PRIVATE,
                    },
                )
            } != 0
            {
                fail!(from new_cv, with ConditionVariableWithSegregatedMutexCreationError::NoInterProcessSupport,
                    "{} due to an failure while setting inter process flag in condition variable attributes.", msg);
            }

            if unsafe { posix::pthread_condattr_setclock(attributes.get_mut(), clock_type as _) }
                != 0
            {
                fail!(from new_cv, with ConditionVariableWithSegregatedMutexCreationError::UnsupportedClockType,
                    "{} due to an failure while setting clock to CLOCK_MONOTONIC in condition variable attributes.", msg);
            }
            handle_errno!(ConditionVariableWithSegregatedMutexCreationError, from new_cv,
                errno_source unsafe { posix::pthread_cond_init(new_cv.handle.get(), attributes.get_mut()) }.into(),
                success Errno::ESUCCES => new_cv,
                Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
                Errno::EBUSY => (InternalMemoryAlreadyInitializedWithMultiConditionVariable, "{} since it seems that the memory of the condition variable is already initialized with a condition variable. This can be a sign of an internal logic error or corrupted memory.", msg),
                Errno::EAGAIN => (InsufficientResources, "{} due to insufficient resources.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            );
        }

        pub(super) fn pthread_wait<T: Debug>(
            &self,
            mutex: &Mutex<T>,
        ) -> Result<(), ConditionVariableWaitError<'_, '_, T>> {
            handle_errno!(ConditionVariableWaitError, from self,
                errno_source unsafe { posix::pthread_cond_wait(self.handle.get(), mutex.handle.as_ptr()) }.into(),
                success Errno::ESUCCES => (),
                v => (UnknownError(v as i32), "An unknown error occurred in wait ({}).", v)
            );
        }

        pub(super) fn pthread_timedwait<T: Debug>(
            &self,
            dur: Duration,
            mutex: &Mutex<T>,
        ) -> Result<WaitState, ConditionVariableTimedWaitError<'_, '_, T>> {
            let wait_time = dur
                + fail!(from self, when Time::now_with_clock(self.clock_type), "Failed to get current time in timed_wait.")
                    .as_duration();
            handle_errno!(ConditionVariableTimedWaitError, from self,
                errno_source unsafe { posix::pthread_cond_timedwait(self.handle.get(), mutex.handle.as_ptr(), &wait_time.as_timespec()) }.into(),
                success Errno::ESUCCES => WaitState::Success;
                success Errno::ETIMEDOUT => WaitState::Timeout,
                v => (UnknownError(v as i32), "An unknown error occured in timed_wait ({}).", v)
            );
        }
    }

    impl Drop for ConditionVariableInternal {
        fn drop(&mut self) {
            if unsafe { posix::pthread_cond_destroy(self.handle.get_mut()) } != 0 {
                fatal_panic!(from self, "This should never happen! Unable to destroy condition variable.");
            }
        }
    }

    unsafe impl Send for ConditionVariableInternal {}
    unsafe impl Sync for ConditionVariableInternal {}

    pub trait ConditionVariableHandle<T: Debug> {
        type Type: Debug;
        fn handle(&self) -> &ConditionVariableInternal;
        fn mutex(&self) -> &Mutex<Self::Type>;
    }
}

/// Is returned by the [`MultiConditionVariable`] in
/// [`MultiConditionVariable::notify_all()`]. It enables the user to change the guarded content
/// before the notification is send.
/// The trigger is signaled when the guard goes out of scope.
pub struct MultiConditionVariableGuard<'cv, 'mtx, 'handle, T: Debug> {
    pub(super) condvar: &'cv MultiConditionVariable<'handle, T>,
    pub(super) guard: MutexGuard<'mtx, 'handle, T>,
}

unsafe impl<T: Debug> Send for MultiConditionVariableGuard<'_, '_, '_, T> where T: Send {}
unsafe impl<T: Debug> Sync for MultiConditionVariableGuard<'_, '_, '_, T> where T: Send + Sync {}

impl<T: Debug> Deref for MultiConditionVariableGuard<'_, '_, '_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

impl<T: Debug> DerefMut for MultiConditionVariableGuard<'_, '_, '_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut()
    }
}

impl<T: Debug> Drop for MultiConditionVariableGuard<'_, '_, '_, T> {
    fn drop(&mut self) {
        self.condvar.trigger_all();
    }
}

pub trait BasicConditionVariableInterface<T: Debug>:
    Sized + internal::ConditionVariableHandle<T> + Debug
{
    /// Waits until the condition variable was signaled by notify_*, modify_notify_* or trigger_*.
    fn wait(
        &self,
    ) -> Result<MutexGuard<'_, '_, Self::Type>, ConditionVariableWaitError<'_, '_, Self::Type>>
    {
        let guard = fail!(from self, when self.mutex().lock(), "Failed to lock mutex in wait");

        self.handle().pthread_wait(self.mutex())?;
        Ok(guard)
    }

    /// Waits until the condition variable was signaled by notify_*, modify_notify_* or trigger_*
    /// or the provided timeout has elapsed.
    fn timed_wait(
        &self,
        timeout: Duration,
    ) -> Result<
        Option<MutexGuard<'_, '_, Self::Type>>,
        ConditionVariableTimedWaitError<'_, '_, Self::Type>,
    > {
        let guard =
            fail!(from self, when self.mutex().lock(), "failed to lock mutex in timed_wait");

        match self.handle().pthread_timedwait(timeout, self.mutex())? {
            internal::WaitState::Success => Ok(Some(guard)),
            internal::WaitState::Timeout => Ok(None),
        }
    }

    /// Notifies all waiters.
    fn trigger_all(&self) {
        if unsafe { posix::pthread_cond_broadcast(self.handle().handle.get()) } != 0 {
            fatal_panic!( from self,
                "This should never happen! Failed to send broadcast on condition variable."
            );
        }
    }

    /// Returns a [`MutexGuard`] which can be used to read/write the underlying value of
    /// the condition variable.
    fn lock(&self) -> Result<MutexGuard<'_, '_, Self::Type>, MutexLockError<'_, '_, Self::Type>> {
        self.mutex().lock()
    }

    /// Tries to return a [`MutexGuard`] which can be used to read/write the underlying value of
    /// the condition variable. If someone else already holds the lock of the underlying mutex
    /// [`None`] is returned.
    fn try_lock(
        &self,
    ) -> Result<Option<MutexGuard<'_, '_, Self::Type>>, MutexLockError<'_, '_, Self::Type>> {
        self.mutex().try_lock()
    }

    /// Tries to return a [`MutexGuard`] which can be used to read/write the underlying value of
    /// the condition variable. When the timeout has passed it returns [`None`].
    fn timed_lock(
        &self,
        timeout: Duration,
    ) -> Result<Option<MutexGuard<'_, '_, Self::Type>>, MutexTimedLockError<'_, '_, Self::Type>>
    {
        self.mutex().timed_lock(timeout)
    }
}

/// Condition variable which allows to use multiple conditions in
/// [`MultiConditionVariable::wait_while()`] and
/// [`MultiConditionVariable::timed_wait_while()`] concurrently but with the draw
/// back that only all waiters can be triggered and not one.
/// The reason is when one waits on multiple
/// conditions but triggers only one waiter it is possible that a waiter is triggered which
/// condition is not satisfied while another waiter is not signaled which could have continued.
/// This could lead to some deadlocks.
///
/// The condition variable comes also with an integrated mutex to avoid undefined behavior
/// when multiple waiters try to wait on different mutexes. With an integrated mutex this
/// problem is solved.
///
/// The underlying data can be read/written and waiters can be triggered as soon as the data
/// is written.
/// The condition variable provides the following features:
///  * wait on the condition variable: [wait](BasicConditionVariableInterface::wait()), [timed_wait](BasicConditionVariableInterface::timed_wait())
///  * wait until a defined condition occurs: [wait_while](MultiConditionVariable::wait_while()), [timed_wait_while](MultiConditionVariable::timed_wait_while())
///  * modify condition variable and then notify waiters:
///     [notify_all](MultiConditionVariable::notify_all()), [modify_notify_all](MultiConditionVariable::modify_notify_all())
///  * trigger waiters without changing condition variable: [trigger_all](BasicConditionVariableInterface::trigger_all())
///  * change the underlying value without notifying any waiter: [lock](BasicConditionVariableInterface::lock()),
///     [try_lock](BasicConditionVariableInterface::try_lock()),
///     [timed_lock](BasicConditionVariableInterface::timed_lock())
///
/// # Example
///
/// * a detailed builder example can be found here [`ConditionVariableBuilder`]
///
/// ```
/// use iceoryx2_bb_posix::condition_variable::*;
/// use std::thread;
/// use std::sync::Arc;
/// use std::time::Duration;
///
/// let mtx_handle = MutexHandle::<i32>::new();
/// let cv = ConditionVariableBuilder::new()
///                 .create_multi_condition_variable(1234, &mtx_handle).expect("failed to create cv");
///
/// thread::scope(|s| {
///     let t1 = s.spawn(|| {
///         // wait until value is 5000
///         let guard = cv.wait_while(|t| *t == 5000).expect("failed to wait");
///         println!("cv value changed to 5000");
///     });
///
///     let t2 = s.spawn(|| {
///         // wait for 1 second until value is greater 4000
///         let result = cv.timed_wait_while(Duration::from_secs(1), |t| *t > 4000).expect("failed to wait");
///         match result {
///             None => println!("timeout reached"),
///             Some(v) => println!("value {} > 4000", *v),
///         }
///     });
///
///     {
///         // acquire notification guard
///         let mut guard = cv.notify_all().expect("failed to notify");
///         // change underlying value to 4005
///         *guard = 4005;
///         // the guard goes out of scope and notifies all waiters
///     }
///
///     // apply clojure, set value to 4008 and then trigger waiters
///     cv.modify_notify_all(|value| { *value = 4008 }).expect("failed to notify");
///
///     // acquire guard to the underlying value without triggering the condition variable
///     let mut guard = cv.lock().expect("failed to acquire value");
///     // read the underlying value
///     println!("current value is {}", *guard);
///     // set the underlying value to 5000
///     *guard = 5000;
/// });
///
/// ```
#[derive(Debug)]
pub struct MultiConditionVariable<'mtx_handle, T: Sized + Debug> {
    mutex: Mutex<'mtx_handle, T>,
    condvar: internal::ConditionVariableInternal,
}

impl<'mtx_handle, T: Debug> MultiConditionVariable<'mtx_handle, T> {
    fn new(
        t: T,
        config: ConditionVariableBuilder,
        handle: &'mtx_handle MutexHandle<T>,
    ) -> Result<Self, ConditionVariableCreationError> {
        Ok(Self {
            mutex: config.mutex.create(t, handle)?,
            condvar: internal::ConditionVariableInternal::new(
                config.clock_type,
                config.is_interprocess_capable,
            )?,
        })
    }

    /// Waits until the condition variable was signaled by
    /// [`MultiConditionVariable::notify_all()`],
    /// [`MultiConditionVariable::modify_notify_all()`] or
    /// [`BasicConditionVariableInterface::trigger_all()`]
    /// and the provided predicate returns true.
    pub fn wait_while<P: FnMut(&mut T) -> bool>(
        &self,
        mut predicate: P,
    ) -> Result<MutexGuard<'_, '_, T>, ConditionVariableWaitError<'_, '_, T>> {
        let mut guard =
            fail!(from self, when self.mutex.lock(), "Failed to lock mutex in wait_while.");

        while !(predicate)(&mut *guard) {
            self.condvar.pthread_wait(&self.mutex)?;
        }

        Ok(guard)
    }

    /// Waits until the condition variable was signaled by
    /// [`MultiConditionVariable::notify_all()`],
    /// [`MultiConditionVariable::modify_notify_all()`] or
    /// [`BasicConditionVariableInterface::trigger_all()`]
    /// and the provided predicate returns true or the provided timeout has elapsed.
    pub fn timed_wait_while<P: FnMut(&mut T) -> bool>(
        &self,
        timeout: Duration,
        mut predicate: P,
    ) -> Result<Option<MutexGuard<'_, '_, T>>, ConditionVariableTimedWaitError<'_, '_, T>> {
        let msg = "Failure in timed_wait_while";
        let mut guard = fail!(from self, when self.mutex.lock(), "{} due to a failure while locking mutex.", msg);

        let mut remaining_duration = timeout;
        while !(predicate)(&mut *guard) {
            let start = fail!(from self, when  Time::now_with_clock(self.condvar.clock_type),
                               "{} due to a failure while getting current time.", msg);
            if matches!(
                self.condvar
                    .pthread_timedwait(remaining_duration, &self.mutex)?,
                internal::WaitState::Timeout
            ) {
                return Ok(None);
            }

            let elapsed = fail!(from self, when start.elapsed(), "{} due to a failure in acquiring elasped time.", msg);

            if elapsed >= remaining_duration {
                return Ok(None);
            }
            remaining_duration -= elapsed;
        }

        Ok(Some(guard))
    }

    /// Returns a [`MultiConditionVariableGuard`] which provides read/write access to the underlying value of
    /// the condition variable, like the [`MutexGuard`]. When it goes out of scope all waiters will
    /// be notified.
    pub fn notify_all(
        &self,
    ) -> Result<MultiConditionVariableGuard<'_, '_, '_, T>, MutexLockError<'_, '_, T>> {
        Ok(MultiConditionVariableGuard {
            condvar: self,
            guard: fail!(from self, when self.mutex.lock(), "failed to lock mutex in notify_all"),
        })
    }

    /// Applies the modifier to the underlying value and notifies all waiters.
    pub fn modify_notify_all<Modifier: FnOnce(&mut T)>(
        &self,
        modifier: Modifier,
    ) -> Result<(), MutexLockError<'_, '_, T>> {
        {
            let mut guard = fail!(from self, when self.mutex.lock(), "failed to lock mutex in modify_notify_all");
            modifier(guard.deref_mut());
        }
        self.trigger_all();
        Ok(())
    }
}

impl<T: Debug> internal::ConditionVariableHandle<T> for MultiConditionVariable<'_, T> {
    type Type = T;

    fn handle(&self) -> &internal::ConditionVariableInternal {
        &self.condvar
    }

    fn mutex(&self) -> &Mutex<T> {
        &self.mutex
    }
}

impl<T: Debug> BasicConditionVariableInterface<T> for MultiConditionVariable<'_, T> {}

/// Provides access to the underlying type and predicate of a [`ConditionVariable`] or
/// with a single static condition.
///
/// Returned in [`BasicConditionVariableInterface::lock()`],
/// [`BasicConditionVariableInterface::try_lock()`] or
/// [`BasicConditionVariableInterface::timed_lock()`].
#[derive(Debug)]
pub struct ConditionVariableData<T> {
    /// The current value of the condition variable
    pub value: T,
    /// The predicate of the condition variable
    pub predicate: Predicate<'static, T>,
}

unsafe impl<T> Send for ConditionVariableData<T> {}

tiny_fn! {
    pub struct Predicate<T> = Fn(value: &T) -> bool;
}

impl<T> Debug for Predicate<'static, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Predicate")
    }
}

/// Is returned by the [`ConditionVariable`] in
/// [`ConditionVariable::notify_all()`] or [`ConditionVariable::notify_one()`].
/// It enables the user to change the guarded content before the notification is send.
/// The trigger is signaled when the guard goes out of scope.
pub struct ConditionVariableGuard<'cv, 'mtx, 'mtx_handle, T: Debug> {
    pub(super) notify_all: bool,
    pub(super) condvar: &'cv ConditionVariable<'mtx_handle, T>,
    pub(super) guard: MutexGuard<'mtx, 'mtx_handle, ConditionVariableData<T>>,
}

unsafe impl<T: Debug> Send for ConditionVariableGuard<'_, '_, '_, T> where T: Send {}
unsafe impl<T: Debug> Sync for ConditionVariableGuard<'_, '_, '_, T> where T: Send + Sync {}

impl<T: Debug> Deref for ConditionVariableGuard<'_, '_, '_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard.value
    }
}

impl<T: Debug> DerefMut for ConditionVariableGuard<'_, '_, '_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard.value
    }
}

impl<T: Debug> Drop for ConditionVariableGuard<'_, '_, '_, T> {
    fn drop(&mut self) {
        match self.notify_all {
            true => self.condvar.trigger_all(),
            false => self.condvar.trigger_one(),
        }
    }
}

/// Condition variable which requires a fixed predicate on creation which is then used in
/// [`ConditionVariable::wait_while()`] and
/// [`ConditionVariable::timed_wait_while()`] concurrently with the benefit of triggering
/// single waiters.
/// The reason is when one waits on multiple
/// conditions but triggers only one waiter it is possible that a waiter is triggered which
/// condition is not satisfied while another waiter is not signaled which could have continued.
/// This could lead to some deadlocks.
///
/// The condition variable comes also with an integrated mutex to avoid undefined behavior
/// when multiple waiters try to wait on different mutexes. With an integrated mutex this
/// problem is solved.
///
/// The underlying data can be read/written and waiters can be triggered as soon as the data
/// is written.
/// The condition variable provides the following features:
///  * wait on the condition variable: [wait](BasicConditionVariableInterface::wait()), [timed_wait](BasicConditionVariableInterface::timed_wait())
///  * wait until a defined condition occurs: [wait_while](ConditionVariable::wait_while()), [timed_wait_while](ConditionVariable::timed_wait_while())
///  * modify condition variable and then notify waiters:
///     [notify_all](ConditionVariable::notify_all()), [modify_notify_all](ConditionVariable::modify_notify_all()),
///     [notify_one](ConditionVariable::notify_one()), [modify_notify_one](ConditionVariable::modify_notify_one())
///  * trigger waiters without changing condition variable: [trigger_all](BasicConditionVariableInterface::trigger_all()),
///     [trigger_one](ConditionVariable::trigger_one())
///  * change the underlying value without notifying any waiter: [lock](BasicConditionVariableInterface::lock()),
///     [try_lock](BasicConditionVariableInterface::try_lock()),
///     [timed_lock](BasicConditionVariableInterface::timed_lock())
///
/// # Example
///
/// * a detailed builder example can be found here [`ConditionVariableBuilder`]
///
/// ```
/// use iceoryx2_bb_posix::condition_variable::*;
/// use std::thread;
/// use std::sync::Arc;
/// use std::time::Duration;
///
/// let mtx_handle = MutexHandle::<ConditionVariableData<i32>>::new();
/// let cv = ConditionVariableBuilder::new()
///                 .create_condition_variable(1234, |t| *t >= 5000, &mtx_handle)
///                 .expect("failed to create cv");
///
/// thread::scope(|s| {
///     let t1 = s.spawn(|| {
///         // wait until value is 5000
///         let guard = cv.wait_while().expect("failed to wait");
///         println!("cv value is greater or equal 5000");
///     });
///
///     let t2 = s.spawn(|| {
///         // wait for 1 second until value is greater or equal 5000
///         let result = cv.timed_wait_while(Duration::from_secs(1)).expect("failed to wait");
///         match result {
///             None => println!("timeout reached"),
///             Some(v) => println!("value {} >= 5000", v.value),
///         }
///     });
///
///     {
///         // acquire notification guard
///         let mut guard = cv.notify_one().expect("failed to notify");
///         // change underlying value to 5005
///         *guard = 5005;
///         // the guard goes out of scope and notifies all waiters
///     }
/// });
///
/// // apply clojure, set value to 4008 and then trigger waiters
/// cv.modify_notify_one(|value| { *value = 5008 }).expect("failed to notify");
///
/// // acquire guard to the underlying value without triggering the condition variable
/// let mut guard = cv.lock().expect("failed to acquire value");
/// // read the underlying value
/// println!("current value is {}", guard.value);
/// // set the underlying value to 14009
/// guard.value = 14009;
/// //
/// cv.trigger_one();
/// ```
#[derive(Debug)]
pub struct ConditionVariable<'mtx_handle, T: Sized + Debug> {
    mutex: Mutex<'mtx_handle, ConditionVariableData<T>>,
    condvar: internal::ConditionVariableInternal,
}

impl<'mtx_handle, T: Debug> ConditionVariable<'mtx_handle, T> {
    fn new<F: Fn(&T) -> bool + 'static>(
        t: T,
        predicate: F,
        config: ConditionVariableBuilder,
        handle: &'mtx_handle MutexHandle<ConditionVariableData<T>>,
    ) -> Result<Self, ConditionVariableCreationError> {
        Ok(Self {
            mutex: config.mutex.create(
                ConditionVariableData {
                    value: t,
                    predicate: Predicate::new(predicate),
                },
                handle,
            )?,
            condvar: internal::ConditionVariableInternal::new(
                config.clock_type,
                config.is_interprocess_capable,
            )?,
        })
    }

    /// Waits until the condition variable was signaled by
    /// [`MultiConditionVariable::notify_all()`],
    /// [`ConditionVariable::notify_one()`],
    /// [`ConditionVariable::modify_notify_all()`],
    /// [`ConditionVariable::modify_notify_one()`],
    /// [`BasicConditionVariableInterface::trigger_all()`] or
    /// [`ConditionVariable::trigger_one()`]
    /// and the provided predicate returns true.
    pub fn wait_while(
        &self,
    ) -> Result<
        MutexGuard<'_, '_, ConditionVariableData<T>>,
        ConditionVariableWaitError<'_, '_, ConditionVariableData<T>>,
    > {
        let guard = fail!(from self, when self.mutex.lock(), "failed to lock mutex in wait_while");

        while !self.call_underlying_predicate(&guard) {
            self.condvar.pthread_wait(&self.mutex)?;
        }

        Ok(guard)
    }

    /// Waits until the condition variable was signaled by
    /// [`MultiConditionVariable::notify_all()`],
    /// [`ConditionVariable::notify_one()`],
    /// [`ConditionVariable::modify_notify_all()`],
    /// [`ConditionVariable::modify_notify_one()`],
    /// [`BasicConditionVariableInterface::trigger_all()`] or
    /// [`ConditionVariable::trigger_one()`]
    /// and the provided predicate returns true or the provided timeout has elapsed.
    pub fn timed_wait_while(
        &self,
        timeout: Duration,
    ) -> Result<
        Option<MutexGuard<'_, '_, ConditionVariableData<T>>>,
        ConditionVariableTimedWaitError<'_, '_, ConditionVariableData<T>>,
    > {
        let msg = "Failure in timed_wait_while";
        let guard = fail!(from self, when self.mutex.lock(), "{} due to a failure while locking mutex.", msg);

        let mut remaining_duration = timeout;
        while !self.call_underlying_predicate(&guard) {
            let start = fail!(from self, when Time::now_with_clock(self.condvar.clock_type),
                                "{} due to a failure while getting current time.", msg);
            if matches!(
                self.condvar
                    .pthread_timedwait(remaining_duration, &self.mutex)?,
                internal::WaitState::Timeout
            ) {
                return Ok(None);
            }

            let elapsed = fail!(from self, when start.elapsed(), "{} due to a failure in acquiring elasped time.", msg);

            if elapsed >= remaining_duration {
                return Ok(None);
            }
            remaining_duration -= elapsed;
        }

        Ok(Some(guard))
    }

    /// Returns a [`ConditionVariableGuard`] which provides read/write access to the underlying value of
    /// the condition variable, like the [`MutexGuard`]. When it goes out of scope all waiters will
    /// be notified.
    pub fn notify_all(
        &self,
    ) -> Result<
        ConditionVariableGuard<'_, '_, '_, T>,
        MutexLockError<'_, '_, ConditionVariableData<T>>,
    > {
        Ok(ConditionVariableGuard {
            notify_all: true,
            condvar: self,
            guard: fail!(from self, when self.mutex.lock(), "failed to lock mutex in notify_all"),
        })
    }

    /// Returns a [`ConditionVariableGuard`] which provides read/write access to the underlying value of
    /// the condition variable, like the [`MutexGuard`]. When it goes out of scope one waiter will
    /// be notified.
    pub fn notify_one(
        &self,
    ) -> Result<
        ConditionVariableGuard<'_, '_, '_, T>,
        MutexLockError<'_, '_, ConditionVariableData<T>>,
    > {
        Ok(ConditionVariableGuard {
            notify_all: false,
            condvar: self,
            guard: fail!(from self, when self.mutex.lock(), "failed to lock mutex in notify_one"),
        })
    }

    /// Applies the modifier to the underlying value and notifies all waiters.
    pub fn modify_notify_all<Modifier: FnOnce(&mut T)>(
        &self,
        modifier: Modifier,
    ) -> Result<(), MutexLockError<'_, '_, ConditionVariableData<T>>> {
        {
            let mut guard = fail!(from self, when self.mutex.lock(), "failed to lock mutex in modify_notify_all");
            let v = Self::underlying_value_mut(&mut guard);
            modifier(v);
        }
        self.trigger_all();
        Ok(())
    }

    /// Applies the modifier to the underlying value and notifies one waiter.
    pub fn modify_notify_one<Modifier: FnOnce(&mut T)>(
        &self,
        modifier: Modifier,
    ) -> Result<(), MutexLockError<'_, '_, ConditionVariableData<T>>> {
        {
            let mut guard = fail!(from self, when self.mutex.lock(), "failed to lock mutex in modify_notify_one");
            let v = Self::underlying_value_mut(&mut guard);
            modifier(v);
        }
        self.trigger_one();
        Ok(())
    }

    /// Notifies one waiter.
    pub fn trigger_one(&self) {
        if unsafe { posix::pthread_cond_signal(self.condvar.handle.get()) } != 0 {
            fatal_panic!(from self, "This should never happen! Failed to send signal on condition variable.");
        }
    }

    fn call_underlying_predicate(
        &self,
        guard: &MutexGuard<'_, '_, ConditionVariableData<T>>,
    ) -> bool {
        guard.predicate.call(&guard.value)
    }

    fn underlying_value_mut<'a>(
        guard: &'a mut MutexGuard<'_, '_, ConditionVariableData<T>>,
    ) -> &'a mut T {
        &mut guard.value
    }
}

impl<T: Debug> internal::ConditionVariableHandle<T> for ConditionVariable<'_, T> {
    type Type = ConditionVariableData<T>;

    fn handle(&self) -> &internal::ConditionVariableInternal {
        &self.condvar
    }

    fn mutex(&self) -> &Mutex<Self::Type> {
        &self.mutex
    }
}

impl<T: Debug> BasicConditionVariableInterface<T> for ConditionVariable<'_, T> {}
