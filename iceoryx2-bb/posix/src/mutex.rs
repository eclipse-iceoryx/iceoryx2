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

//! Provides an inter-process capable POSIX [`Mutex`] which can be created by the [`MutexBuilder`].
//!
//! # Example
//!
//! ```ignore
//! use iceoryx2_bb_posix::mutex::*;
//! use std::time::Duration;
//! use iceoryx2_bb_posix::clock::ClockType;
//!
//! let handle = MutexHandle::<i32>::new();
//! let mutex = MutexBuilder::new()
//!               // is used in [`Mutex::timed_lock()`]
//!               .clock_type(ClockType::Monotonic)
//!               .is_interprocess_capable(false)
//!               .mutex_type(MutexType::WithDeadlockDetection)
//!               .thread_termination_behavior(MutexThreadTerminationBehavior::ReleaseWhenLocked)
//!               .priority_inheritance(MutexPriorityInheritance::Inherit)
//!               .create(1234, &handle)
//!               .expect("failed to create mutex");
//!
//! {
//!     let guard = mutex.lock().expect("failed to lock mutex");
//!     println!("current mutex value is: {}", *guard);
//! }
//!
//! match mutex.try_lock().expect("failed to lock") {
//!     Some(mut guard) => *guard = 123, // set mutex value to 123
//!     None => println!("unable to acquire lock"),
//! };
//!
//! match mutex.timed_lock(Duration::from_secs(1)).expect("failed to lock") {
//!     Some(guard) => println!("New mutex value is: {}", *guard),
//!     None => println!("Timeout occurred while trying to get lock.")
//! };
//! ```
pub use crate::unmovable_ipc_handle::internal::CreateIpcConstruct;
pub use crate::unmovable_ipc_handle::IpcCapable;
use crate::unmovable_ipc_handle::IpcHandleState;

use iceoryx2_bb_elementary::scope_guard::*;
use iceoryx2_bb_log::{fail, fatal_panic, warn};
use iceoryx2_pal_posix::*;
use std::cell::UnsafeCell;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::time::Duration;

use crate::adaptive_wait::*;
use crate::clock::{AsTimespec, ClockType, NanosleepError, Time, TimeError};
use crate::handle_errno;
use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::posix::Struct;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum MutexCreationError {
    InsufficientMemory,
    InsufficientResources,
    InsufficientPermissions,
    NoInterProcessSupport,
    UnableToSetType,
    UnableToSetProtocol,
    UnableToSetThreadTerminationBehavior,
    UnableToSetPriorityCeiling,
    HandleAlreadyInitialized,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum MutexGetPrioCeilingError {
    InsufficientPermissions,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum MutexSetPrioCeilingError {
    ValueOutOfRange,
    InsufficientPermissions,
    UnknownError(i32),
}

#[derive(Debug, PartialEq, Eq)]
pub enum MutexLockError<'mutex, 'handle, T: Sized + Debug> {
    ExceededMaximumNumberOfRecursiveLocks,
    DeadlockDetected,
    LockAcquiredButOwnerDied(MutexGuard<'mutex, 'handle, T>),
    UnrecoverableState,
    UnknownError(i32),
}

impl<T: Sized + Debug> PartialEq for MutexGuard<'_, '_, T> {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.mutex, other.mutex)
    }
}

impl<T: Sized + Debug> Eq for MutexGuard<'_, '_, T> {}

#[derive(Debug, PartialEq, Eq)]
pub enum MutexTimedLockError<'mutex, 'handle, T: Sized + Debug> {
    TimeoutExceedsMaximumSupportedDuration,
    MutexLockError(MutexLockError<'mutex, 'handle, T>),
    NanosleepError(NanosleepError),
    AdaptiveWaitError(AdaptiveWaitError),
    FailureInInternalClockWhileWait(TimeError),
}

impl<'mutex, 'handle, T: Debug> From<TimeError> for MutexTimedLockError<'mutex, 'handle, T> {
    fn from(v: TimeError) -> Self {
        MutexTimedLockError::FailureInInternalClockWhileWait(v)
    }
}

impl<'mutex, 'handle, T: Debug> From<NanosleepError> for MutexTimedLockError<'mutex, 'handle, T> {
    fn from(v: NanosleepError) -> Self {
        MutexTimedLockError::NanosleepError(v)
    }
}

impl<'mutex, 'handle, T: Debug> From<AdaptiveWaitError>
    for MutexTimedLockError<'mutex, 'handle, T>
{
    fn from(v: AdaptiveWaitError) -> Self {
        MutexTimedLockError::AdaptiveWaitError(v)
    }
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum MutexUnlockError {
    OwnedByDifferentEntity,
    UnknownError(i32),
}

/// The MutexError enum is a generalization when one doesn't require the fine-grained error
/// handling enums. One can forward MutexError as more generic return value when a method
/// returns a Mutex***Error.
/// On a higher level it is again convertable to [`crate::Error`].
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum MutexError {
    CreationFailed,
    LockFailed,
    UnlockFailed,
}

impl<'mutex, 'handle, T: Debug> From<MutexLockError<'mutex, 'handle, T>> for MutexError {
    fn from(_: MutexLockError<'mutex, 'handle, T>) -> Self {
        MutexError::LockFailed
    }
}

impl<'mutex, 'handle, T: Debug> From<MutexTimedLockError<'mutex, 'handle, T>> for MutexError {
    fn from(_: MutexTimedLockError<'mutex, 'handle, T>) -> Self {
        MutexError::LockFailed
    }
}

impl From<MutexUnlockError> for MutexError {
    fn from(_: MutexUnlockError) -> Self {
        MutexError::UnlockFailed
    }
}

impl From<MutexCreationError> for MutexError {
    fn from(_: MutexCreationError) -> Self {
        MutexError::CreationFailed
    }
}

/// A guard which allows the modification of a value guarded by a [`Mutex`]. It is returned in
/// [`Mutex::lock()`], [`Mutex::try_lock()`] and [`Mutex::timed_lock()`].
///
/// # Example
///
/// ```
/// use iceoryx2_bb_posix::mutex::*;
///
/// let handle = MutexHandle::<i32>::new();
/// let mutex = MutexBuilder::new().create(123, &handle)
///                                .expect("failed to create mutex");
///
/// {
///     let mut guard = mutex.lock().expect("failed to lock");
///     println!("Old value is {}", *guard);
///     *guard = 456;
/// }
/// ```
#[derive(Debug)]
pub struct MutexGuard<'mutex, 'handle, T: Debug> {
    mutex: &'mutex Mutex<'handle, T>,
}

unsafe impl<T: Send + Debug> Send for MutexGuard<'_, '_, T> {}
unsafe impl<T: Send + Sync + Debug> Sync for MutexGuard<'_, '_, T> {}

impl<T: Debug> Deref for MutexGuard<'_, '_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { (*self.mutex.handle.value.get()).as_ref().unwrap() }
    }
}

impl<T: Debug> DerefMut for MutexGuard<'_, '_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { (*self.mutex.handle.value.get()).as_mut().unwrap() }
    }
}

impl<T: Debug> Drop for MutexGuard<'_, '_, T> {
    fn drop(&mut self) {
        if self.mutex.release().is_err() {
            fatal_panic!(from self.mutex, "This should never happen! The MutexGuard is unable to release the mutex.");
        }
    }
}

/// The type of a mutex defines its behavior.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum MutexType {
    /// default behavior
    Normal = posix::PTHREAD_MUTEX_NORMAL,
    /// the mutex can be locked multiple times by the same thread
    Recursive = posix::PTHREAD_MUTEX_RECURSIVE,
    /// if the call [`Mutex::lock()`] would cause a deadlock it returns an error instead of causing
    /// an actual deadlock
    WithDeadlockDetection = posix::PTHREAD_MUTEX_ERRORCHECK,
}

/// Defines how the priority of a mutex owning thread changes when another thread
/// with an higher priority would like to acquire the mutex.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum MutexPriorityInheritance {
    /// No change in priority
    None = posix::PTHREAD_PRIO_NONE,
    /// The priority of a thread holding the mutex is always promoted to the priority set up
    /// in [`MutexBuilder::priority_ceiling()`].
    Inherit = posix::PTHREAD_PRIO_INHERIT,
    /// The priority of a thread holding the mutex is promoted to the priority of the
    /// highest priority thread waiting for the lock.
    Protect = posix::PTHREAD_PRIO_PROTECT,
}

/// Defines the behavior when a mutex owning thread is terminated
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum MutexThreadTerminationBehavior {
    /// The mutex stays locked, is unlockable and no longer usable. This can also lead to a mutex
    /// leak in the drop.
    StallWhenLocked = posix::PTHREAD_MUTEX_STALLED,

    /// It implies the same behavior as [`MutexType::WithDeadlockDetection`]. Additionally, when a
    /// mutex owning
    /// thread/process dies the mutex is put into an inconsistent state which can be recovered with
    /// [`Mutex::make_consistent()`]. The inconsistent state is detected by the next instance which
    /// calls [`Mutex::lock()`], [`Mutex::try_lock()`] or [`Mutex::timed_lock()`].
    ///
    /// This is also known as robust mutex.
    ReleaseWhenLocked = posix::PTHREAD_MUTEX_ROBUST,
}

/// Creates a [`Mutex`].
#[derive(Debug)]
pub struct MutexBuilder {
    pub(crate) is_interprocess_capable: bool,
    pub(crate) mutex_type: MutexType,
    pub(crate) thread_termination_behavior: MutexThreadTerminationBehavior,
    pub(crate) priority_inheritance: MutexPriorityInheritance,
    pub(crate) priority_ceiling: Option<i32>,
    pub(crate) clock_type: ClockType,
}

impl Default for MutexBuilder {
    fn default() -> Self {
        Self {
            is_interprocess_capable: true,
            mutex_type: MutexType::Normal,
            priority_inheritance: MutexPriorityInheritance::None,
            priority_ceiling: None,
            thread_termination_behavior: MutexThreadTerminationBehavior::StallWhenLocked,
            clock_type: ClockType::default(),
        }
    }
}

impl MutexBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Defines the [`ClockType`] which should be used in [`Mutex::timed_lock()`].
    pub fn clock_type(mut self, clock_type: ClockType) -> Self {
        self.clock_type = clock_type;
        self
    }

    /// Can the same mutex be used from multiple processes.
    pub fn is_interprocess_capable(mut self, value: bool) -> Self {
        self.is_interprocess_capable = value;
        self
    }

    /// [`MutexType`] defines the behavior of the mutex.
    pub fn mutex_type(mut self, value: MutexType) -> Self {
        self.mutex_type = value;
        self
    }

    /// Defines the [`MutexThreadTerminationBehavior`].
    pub fn thread_termination_behavior(mut self, value: MutexThreadTerminationBehavior) -> Self {
        self.thread_termination_behavior = value;
        self
    }

    /// Defines the [`MutexPriorityInheritance`] mode.
    pub fn priority_inheritance(mut self, value: MutexPriorityInheritance) -> Self {
        self.priority_inheritance = value;
        self
    }

    /// Defines to priority to which a thread is promoted when another thread is waiting on the
    /// lock and [`MutexPriorityInheritance::Inherit`] is set.
    pub fn priority_ceiling(mut self, value: i32) -> Self {
        self.priority_ceiling = Some(value);
        self
    }

    /// Creates a new mutex with a guarded value.
    pub fn create<T: Debug>(
        self,
        t: T,
        handle: &MutexHandle<T>,
    ) -> Result<Mutex<T>, MutexCreationError> {
        let msg = "Unable to create mutex";
        if handle
            .reference_counter
            .compare_exchange(
                IpcHandleState::Uninitialized as _,
                IpcHandleState::PerformingInitialization as _,
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_err()
        {
            fail!(from self, with MutexCreationError::HandleAlreadyInitialized,
                "{} since the handle is already initialized with another mutex.", msg);
        }

        unsafe { *handle.clock_type.get() = self.clock_type };
        unsafe { *handle.value.get() = Some(t) };
        handle
            .is_interprocess_capable
            .store(self.is_interprocess_capable, Ordering::Relaxed);

        let mut mutex_attributes = ScopeGuardBuilder::new(posix::pthread_mutexattr_t::new())
            .on_init(
                |attr| match unsafe { posix::pthread_mutexattr_init(attr) } {
                    0 => Ok(()),
                    _ => {
                        fail!(from self, with MutexCreationError::InsufficientMemory,
                        "{} since the mutex attribute initialization failed.", msg);
                    }
                },
            )
            .on_drop(
                |attr| match unsafe { posix::pthread_mutexattr_destroy(attr) } {
                    0 => (),
                    _ => {
                        fatal_panic!(
                            "Mutex<{}>, failed to destroy mutex attributes - possible leak?",
                            std::any::type_name::<T>()
                        );
                    }
                },
            )
            .create()?;

        if self.is_interprocess_capable
            && unsafe {
                posix::pthread_mutexattr_setpshared(
                    mutex_attributes.get_mut(),
                    posix::PTHREAD_PROCESS_SHARED,
                )
            } != 0
        {
            fail!(from self, with MutexCreationError::NoInterProcessSupport,
                "{} due to a failure while setting the inter process flag in mutex attributes.", msg);
        }

        if unsafe {
            posix::pthread_mutexattr_settype(mutex_attributes.get_mut(), self.mutex_type as i32)
        } != 0
        {
            fail!(from self, with MutexCreationError::UnableToSetType,
                "{} due to a failure while setting the mutex type in mutex attributes.", msg);
        }

        if unsafe {
            posix::pthread_mutexattr_setprotocol(
                mutex_attributes.get_mut(),
                self.priority_inheritance as i32,
            )
        } != 0
        {
            fail!(from self, with MutexCreationError::UnableToSetProtocol,
                "{} due to a failure while setting the mutex protocol in mutex attributes.", msg);
        }

        if unsafe {
            posix::pthread_mutexattr_setrobust(
                mutex_attributes.get_mut(),
                self.thread_termination_behavior as i32,
            )
        } != 0
        {
            fail!(from self, with MutexCreationError::UnableToSetThreadTerminationBehavior,
                "{} due to a failure while setting the mutex thread termination behavior in mutex attributes.", msg);
        }

        if self.priority_ceiling.is_some() {
            let msg = "Failed to set the mutex priority ceiling";
            handle_errno!(MutexCreationError, from self,
                errno_source unsafe {
                    posix::pthread_mutexattr_setprioceiling(
                    mutex_attributes
                        .get_mut(), self.priority_ceiling.unwrap())
                    }.into(),
                continue_on_success,
                success Errno::ESUCCES => (),
                Errno::EPERM => (UnableToSetPriorityCeiling, "{} since the user does not have enough permissions to set them.", msg),
                v => (UnableToSetPriorityCeiling, "{} since an unknown error has occurred ({}).",msg, v)
            );
        }

        match unsafe { posix::pthread_mutex_init(handle.as_ptr(), mutex_attributes.get()) }.into() {
            Errno::ESUCCES => (),
            Errno::ENOMEM => {
                fail!(from self, with MutexCreationError::InsufficientMemory, "{} due to insufficient memory.", msg);
            }
            Errno::EAGAIN => {
                fail!(from self, with MutexCreationError::InsufficientResources,
                "{} due to insufficient resources.",
                msg);
            }
            Errno::EPERM => {
                fail!(from self, with MutexCreationError::InsufficientPermissions,
                    "{} due to insufficient permissions.", msg
                );
            }
            v => {
                fail!(from self, with MutexCreationError::UnknownError(v as i32),
                "{} since an unknown error occurred ({})", msg, v);
            }
        };

        handle
            .reference_counter
            .store(IpcHandleState::Initialized as _, Ordering::Release);

        Ok(Mutex::new(handle))
    }
}

#[derive(Debug)]
pub struct MutexHandle<T: Sized + Debug> {
    handle: UnsafeCell<posix::pthread_mutex_t>,
    clock_type: UnsafeCell<ClockType>,
    is_interprocess_capable: AtomicBool,
    value: UnsafeCell<Option<T>>,
    reference_counter: AtomicI64,
}

unsafe impl<T: Sized + Debug> Send for MutexHandle<T> {}
unsafe impl<T: Sized + Debug> Sync for MutexHandle<T> {}

impl<T: Sized + Debug> crate::unmovable_ipc_handle::internal::UnmovableIpcHandle
    for MutexHandle<T>
{
    fn reference_counter(&self) -> &std::sync::atomic::AtomicI64 {
        &self.reference_counter
    }

    fn is_interprocess_capable(&self) -> bool {
        self.is_interprocess_capable.load(Ordering::Relaxed)
    }
}

impl<T: Sized + Debug> Default for MutexHandle<T> {
    fn default() -> Self {
        Self {
            handle: UnsafeCell::new(posix::pthread_mutex_t::new()),
            clock_type: UnsafeCell::new(ClockType::default()),
            is_interprocess_capable: AtomicBool::new(false),
            value: UnsafeCell::new(None),
            reference_counter: AtomicI64::new(IpcHandleState::Uninitialized as _),
        }
    }
}

impl<T: Sized + Debug> MutexHandle<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn as_ptr(&self) -> *mut posix::pthread_mutex_t {
        self.handle.get()
    }

    fn clock_type(&self) -> ClockType {
        unsafe { *self.clock_type.get() }
    }
}

/// Represents a POSIX mutex which can be created by the [`MutexBuilder`].
///
/// # Example
///
/// For a detailed builder example, see [`MutexBuilder`].
///
/// ```
/// use iceoryx2_bb_posix::mutex::*;
/// use std::time::Duration;
///
/// let handle = MutexHandle::<i32>::new();
/// let mutex = MutexBuilder::new().create(5, &handle)
///     .expect("Failed to create mutex");
///
/// {
///     let guard = mutex.lock().expect("failed to lock mutex");
///     println!("current mutex value is: {}", *guard);
/// }
///
/// match mutex.try_lock().expect("failed to lock") {
///     Some(mut guard) => *guard = 123, // set mutex value to 123
///     None => println!("unable to acquire lock"),
/// };
///
/// match mutex.timed_lock(Duration::from_secs(1)).expect("failed to lock") {
///     Some(guard) => println!("New mutex value is: {}", *guard),
///     None => println!("Timeout occurred while trying to get lock.")
/// };
/// ```
#[derive(Debug)]
pub struct Mutex<'a, T: Sized + Debug> {
    pub(crate) handle: &'a MutexHandle<T>,
}

unsafe impl<T: Sized + Send + Debug> Send for Mutex<'_, T> {}
unsafe impl<T: Sized + Send + Debug> Sync for Mutex<'_, T> {}

impl<T: Sized + Debug> Drop for Mutex<'_, T> {
    fn drop(&mut self) {
        if self.handle.reference_counter.fetch_sub(1, Ordering::AcqRel) == 1 {
            if unsafe { posix::pthread_mutex_destroy(self.handle.as_ptr()) } != 0 {
                fatal_panic!(from self, "This should never happen! Failed to destroy mutex handle - possible leak?");
            }

            self.handle.reference_counter.store(-1, Ordering::Release);
        }
    }
}

impl<'a, T: Debug> CreateIpcConstruct<'a, MutexHandle<T>> for Mutex<'a, T> {
    fn new(handle: &'a MutexHandle<T>) -> Self {
        Self { handle }
    }
}

impl<'a, T: Debug> IpcCapable<'a, MutexHandle<T>> for Mutex<'a, T> {}

impl<'a, T: Debug> Mutex<'a, T> {
    /// Returns true if the mutex can be used in an inter-process context, otherwise false
    pub fn is_interprocess_capable(&self) -> bool {
        self.handle.is_interprocess_capable.load(Ordering::Relaxed)
    }

    /// Blocks until the ownership of the lock could be acquired. If it was successful it returns a
    /// [`MutexGuard`] to allow access to the underlying value.
    /// If the previously owning thread has died and
    /// [`MutexThreadTerminationBehavior::ReleaseWhenLocked`] was set it returns the error
    /// [`MutexLockError::LockAcquiredButOwnerDied`] which contains also the [`MutexGuard`]. The
    /// new owner now has the responsibility to either repair the underlying value of the mutex and
    /// call [`Mutex::make_consistent()`] when it is repaired or to undertake other measures when
    /// it is unrepairable.
    pub fn lock(&self) -> Result<MutexGuard<'_, '_, T>, MutexLockError<'_, '_, T>> {
        let msg = "Failed to lock";
        handle_errno!(MutexLockError, from self,
            errno_source unsafe { posix::pthread_mutex_lock(self.handle.as_ptr()) }.into(),
            success Errno::ESUCCES => MutexGuard { mutex: self },
            Errno::EAGAIN => (ExceededMaximumNumberOfRecursiveLocks, "{} since the maximum number of recursive locks exceeded.", msg),
            Errno::EDEADLK => (DeadlockDetected, "{} since the operation would lead to a deadlock.", msg),
            Errno::EOWNERDEAD => (LockAcquiredButOwnerDied(MutexGuard { mutex: self }), "{} since the thread/process holding the mutex died.", msg),
            Errno::ENOTRECOVERABLE => (UnrecoverableState, "{} since the thread/process holding the mutex died and the next owner did not repair the state with Mutex::make_consistent.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred while acquiring the lock ({})", msg, v)
        );
    }

    /// Tries to acquire the ownership of the lock. If it was successful it returns a
    /// [`MutexGuard`] packed inside an [`Option`], if the lock is already owned by someone else it
    /// returns [`None`].
    /// If the previously owning thread has died and
    /// [`MutexThreadTerminationBehavior::ReleaseWhenLocked`] was set it returns the error
    /// [`MutexLockError::LockAcquiredButOwnerDied`] which contains also the [`MutexGuard`]. The
    /// new owner now has the responsibility to either repair the underlying value of the mutex and
    /// call [`Mutex::make_consistent()`] when it is repaired or to undertake other measures when
    /// it is unrepairable.
    pub fn try_lock(&self) -> Result<Option<MutexGuard<'_, '_, T>>, MutexLockError<'_, '_, T>> {
        let msg = "Try lock failed";
        handle_errno!(MutexLockError, from self,
            errno_source unsafe { posix::pthread_mutex_trylock(self.handle.as_ptr()) }.into(),
            success Errno::ESUCCES => Some(MutexGuard { mutex: self });
            success Errno::EDEADLK => None;
            success Errno::EBUSY => None,
            Errno::EAGAIN => (ExceededMaximumNumberOfRecursiveLocks, "{} since the maximum number of recursive locks exceeded.", msg),
            Errno::EOWNERDEAD => (LockAcquiredButOwnerDied(MutexGuard { mutex: self }), "{} since the thread/process holding the mutex dies.", msg),
            Errno::ENOTRECOVERABLE => (UnrecoverableState, "{} since the thread/process holding the mutex died and the next owner did not repair the state with Mutex::make_consistent.", msg),
            v => (UnknownError(v as i32), "{} since unknown error occurred while acquiring the lock ({})", msg, v)
        );
    }

    /// Tries to acquire the ownership of the lock until the provided timeout has elapsed. If it was
    /// successful it returns a [`MutexGuard`] packed inside an [`Option`], if the could not be
    /// acquired lock when the timeout passed it returns [`None`].
    /// If the previously owning thread has died and
    /// [`MutexThreadTerminationBehavior::ReleaseWhenLocked`] was set it returns the error
    /// [`MutexTimedLockError::MutexLockError`] which contains also the [`MutexGuard`]. The
    /// new owner now has the responsibility to either repair the underlying value of the mutex and
    /// call [`Mutex::make_consistent()`] when it is repaired or to undertake other measures when
    /// it is unrepairable.
    pub fn timed_lock(
        &self,
        duration: Duration,
    ) -> Result<Option<MutexGuard<'_, '_, T>>, MutexTimedLockError<'_, '_, T>> {
        let msg = "Timed lock failed";

        match self.handle.clock_type() {
            ClockType::Realtime => {
                let now = fail!(from self, when Time::now_with_clock(ClockType::Realtime),
                    "{} due to a failure while acquiring current system time.", msg);
                let timeout = now.as_duration() + duration;
                handle_errno!(MutexTimedLockError, from self,
                    errno_source unsafe { posix::pthread_mutex_timedlock(self.handle.as_ptr(), &timeout.as_timespec()) }.into(),
                    success Errno::ESUCCES => Some(MutexGuard { mutex: self });
                    success Errno::ETIMEDOUT => None,
                    Errno::EAGAIN => (MutexLockError(MutexLockError::ExceededMaximumNumberOfRecursiveLocks), "{} since the maximum number of recursive locks exceeded.", msg),
                    Errno::EINVAL => (TimeoutExceedsMaximumSupportedDuration, "{} since the timeout of {:?} exceeds the maximum supported duration.", msg, duration),
                    Errno::EDEADLK => (MutexLockError(MutexLockError::DeadlockDetected), "{} since the operation would lead to a deadlock.", msg),
                    Errno::ENOTRECOVERABLE => (MutexLockError(MutexLockError::UnrecoverableState), "{} since the thread/process holding the mutex died and the next owner did not repair the state with Mutex::make_consistent.", msg),
                    v => (MutexLockError(MutexLockError::UnknownError(v as i32)), "{} since unknown error occurred while acquiring the lock ({})", msg, v)
                )
            }
            ClockType::Monotonic => {
                let time = fail!(from self, when Time::now_with_clock(ClockType::Monotonic),
                    "{} due to a failure while acquiring current system time.", msg);
                let mut adaptive_wait = fail!(from self, when AdaptiveWaitBuilder::new()
                    .clock_type(self.handle.clock_type())
                    .create(), "{} since the adaptive wait could not be created.", msg);

                loop {
                    match self.try_lock() {
                        Ok(Some(v)) => return Ok(Some(v)),
                        Ok(None) => match fail!(from self, when time.elapsed(),
                    "{} due to a failure while acquiring elapsed system time.", msg)
                            < duration
                        {
                            true => {
                                fail!(from self, when  adaptive_wait.wait(), "{} since AdaptiveWait failed.", msg);
                            }
                            false => return Ok(None),
                        },
                        Err(v) => {
                            fail!(from self, with MutexTimedLockError::MutexLockError(v),
                        "{} since timed lock failed for duration {:?}.", msg, duration);
                        }
                    }
                }
            }
        }
    }

    /// If the previously owning thread has died and
    /// [`MutexThreadTerminationBehavior::ReleaseWhenLocked`] was set it returns the error
    /// [`MutexLockError::LockAcquiredButOwnerDied`] which contains also the [`MutexGuard`]. The
    /// new owner now has the responsibility to either repair the underlying value of the mutex and
    /// call [`Mutex::make_consistent()`] when it is repaired or to undertake other measures when
    /// it is unrepairable.
    pub fn make_consistent(&self) {
        if unsafe { posix::pthread_mutex_consistent(self.handle.as_ptr()) } != 0 {
            warn!(from self, "pthread_mutex_consistent has no effect since either the mutex was not a robust mutex or the mutex was not in an inconsistent state.");
        }
    }

    /// Returns the current priority ceiling of the mutex.
    pub fn priority_ceiling(&self) -> Result<i32, MutexGetPrioCeilingError> {
        let mut value: i32 = 0;
        let msg = "Unable to acquire priority ceiling";
        handle_errno!(MutexGetPrioCeilingError, from self,
            errno_source unsafe { posix::pthread_mutex_getprioceiling(self.handle.as_ptr(), &mut value) }.into(),
            success Errno::ESUCCES => value,
            Errno::EPERM => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    /// Sets a new priority ceiling for the mutex and returns the old value.
    pub fn set_priority_ceiling(&self, value: i32) -> Result<i32, MutexSetPrioCeilingError> {
        let mut previous_value: i32 = 0;
        let msg = "Unable to set priority ceiling";
        handle_errno!(MutexSetPrioCeilingError, from self,
            errno_source unsafe { posix::pthread_mutex_setprioceiling(self.handle.as_ptr(), value, &mut previous_value) }.into(),
            success Errno::ESUCCES => previous_value,
            Errno::EINVAL => (ValueOutOfRange, "{} since the new priority ceiling value {} is out of range.", msg, value),
            Errno::EPERM => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    pub(crate) fn release(&self) -> Result<(), MutexUnlockError> {
        let msg = "Unable to release lock";
        handle_errno!(MutexUnlockError, from self,
            errno_source unsafe { posix::pthread_mutex_unlock(self.handle.as_ptr()) }.into(),
            success Errno::ESUCCES => (),
            Errno::EPERM => (OwnedByDifferentEntity, "{} since the current thread/process does not own the lock", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }
}
