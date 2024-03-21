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

//! A POSIX inter-process capable [`ReadWriteMutex`] where either multiple readers can acquire
//! multiple read-locks or one writer can acquire a write-lock.
//! It is built by the [`ReadWriteMutexBuilder`].
//!
//! # Example
//!
//! ```ignore
//! use iceoryx2_bb_posix::read_write_mutex::*;
//! use std::thread;
//! use std::time::Duration;
//! use iceoryx2_bb_posix::clock::ClockType;
//!
//! let rw_handle = ReadWriteMutexHandle::new();
//! let rw_mutex = ReadWriteMutexBuilder::new()
//!                         .clock_type(ClockType::Monotonic)
//!                         .is_interprocess_capable(true)
//!                         .mutex_priority(ReadWriteMutexPriority::PreferReader)
//!                         .create(123, &rw_handle)
//!                         .expect("failed to create rw mutex");
//!
//! thread::scope(|s| {
//!     s.spawn(|| {
//!         match rw_mutex.read_timed_lock(Duration::from_millis(100))
//!                       .expect("failed to read_lock") {
//!             None => println!("Timeout while acquiring read lock."),
//!             Some(guard) => println!("The mutex value is: {}", *guard),
//!
//!         }
//!     });
//!
//!     s.spawn(|| {
//!         match rw_mutex.write_timed_lock(Duration::from_millis(100))
//!                       .expect("failed to write_lock") {
//!             None => println!("Timeout while acquiring write lock."),
//!             Some(mut guard) => {
//!                 println!("The old value is: {}", *guard);
//!                 *guard = 456;
//!                 println!("The new value is: {}", *guard);
//!             }
//!         }
//!     });
//! });
//! ```
pub use crate::ipc_capable::{Handle, IpcCapable};

use crate::ipc_capable::internal::{Capability, HandleStorage, IpcConstructible};
use crate::ipc_capable::HandleState;
use crate::{clock::AsTimespec, handle_errno};
use iceoryx2_bb_elementary::{enum_gen, scope_guard::ScopeGuardBuilder};
use iceoryx2_bb_log::{fail, fatal_panic, warn};
use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::posix::Struct;
use iceoryx2_pal_posix::*;
use std::{
    cell::UnsafeCell,
    fmt::Debug,
    ops::{Deref, DerefMut},
    time::Duration,
};

use crate::{
    adaptive_wait::*,
    clock::{ClockType, Time, TimeError},
};

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum ReadWriteMutexCreationError {
    InsufficientMemory,
    InsufficientResources,
    InsufficientPermissions,
    NoInterProcessSupport,
    NoMutexKindSupport,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum ReadWriteMutexReadLockError {
    MaximumAmountOfReadLocksAcquired,
    DeadlockConditionDetected,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum ReadWriteMutexUnlockError {
    OwnedByDifferentEntity,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum ReadWriteMutexWriteLockError {
    DeadlockConditionDetected,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum ReadWriteMutexOpenIpcHandleError {
    IsNotInterProcessCapable,
    Uninitialized,
}

enum_gen! {
    ReadWriteMutexReadTimedLockError
  entry:
    TimeoutExceedsMaximumSupportedDuration
  mapping:
    ReadWriteMutexReadLockError,
    AdaptiveWaitError,
    TimeError
}

enum_gen! {
    ReadWriteMutexWriteTimedLockError
  entry:
    TimeoutExceedsMaximumSupportedDuration
  mapping:
    ReadWriteMutexWriteLockError,
    AdaptiveWaitError,
    TimeError
}

enum_gen! {
    /// The ReadWriteMutexError enum is a generalization when one doesn't require the fine-grained error
    /// handling enums. One can forward ReadWriteMutexError as more generic return value when a method
    /// returns a ReadWriteMutex***Error.
    /// On a higher level it is again convertable to [`crate::Error`].
    ReadWriteMutexError
  generalization:
    FailedToLock <= ReadWriteMutexWriteTimedLockError; ReadWriteMutexReadTimedLockError; ReadWriteMutexWriteLockError; ReadWriteMutexReadLockError,
    FailedToCreate <= ReadWriteMutexCreationError
}

/// Defines if the reader or the writer should prioritized in the [`ReadWriteMutex`].
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum ReadWriteMutexPriority {
    PreferReader = posix::PTHREAD_PREFER_READER_NP,
    PreferWriter = posix::PTHREAD_PREFER_WRITER_NP,
    PreferWriterNonRecursive = posix::PTHREAD_PREFER_WRITER_NONRECURSIVE_NP,
}

/// The builder for the [`ReadWriteMutex`].
#[derive(Debug)]
pub struct ReadWriteMutexBuilder {
    clock_type: ClockType,
    mutex_priority: ReadWriteMutexPriority,
    is_interprocess_capable: bool,
}

impl Default for ReadWriteMutexBuilder {
    fn default() -> Self {
        ReadWriteMutexBuilder {
            clock_type: ClockType::default(),
            mutex_priority: ReadWriteMutexPriority::PreferReader,
            is_interprocess_capable: true,
        }
    }
}

impl ReadWriteMutexBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the type of clock which will be used in [`ReadWriteMutex::read_timed_lock()`]
    /// and [`ReadWriteMutex::write_timed_lock()`].
    pub fn clock_type(mut self, value: ClockType) -> Self {
        self.clock_type = value;
        self
    }

    /// Defines if the [`ReadWriteMutex`] is inter-process capable or not.
    pub fn is_interprocess_capable(mut self, value: bool) -> Self {
        self.is_interprocess_capable = value;
        self
    }

    /// Defines if the reader or the writer should be prioritized.
    pub fn mutex_priority(mut self, value: ReadWriteMutexPriority) -> Self {
        self.mutex_priority = value;
        self
    }

    fn initialize_rw_mutex(
        &self,
        mtx: *mut posix::pthread_rwlock_t,
    ) -> Result<Capability, ReadWriteMutexCreationError> {
        let msg = "Failed to create mutex";
        let origin = format!("{:?}", self);

        let mut attributes = ScopeGuardBuilder::new(posix::pthread_rwlockattr_t::new()).on_init(|attr| {
            handle_errno!(ReadWriteMutexCreationError, from self,
                errno_source unsafe { posix::pthread_rwlockattr_init( attr).into() },
                success Errno::ESUCCES => (),
                Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory while creating rwlock attributes.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred while creating rwlock attributes.", msg)
            );
        }).on_drop(|attr| {
            match unsafe {posix::pthread_rwlockattr_destroy(attr) } {
                0 => (),
                v => {
                    fatal_panic!(from origin, "This should never happen! Failed to release rwlock attributes ({}).", v);
                }
            }
        }).create()?;

        match unsafe { posix::pthread_rwlockattr_setpshared(attributes.get_mut(), 0) } {
            0 => (),
            v => {
                fail!(from origin, with ReadWriteMutexCreationError::NoInterProcessSupport,
                        "{} due to an unknown error while setting interprocess capabilities ({}).", msg,v );
            }
        }

        match unsafe {
            posix::pthread_rwlockattr_setkind_np(attributes.get_mut(), self.mutex_priority as i32)
        } {
            0 => (),
            v => {
                fail!(from origin, with ReadWriteMutexCreationError::NoMutexKindSupport,
                    "{} due to an unknown error while setting the mutex kind ({}).", msg, v);
            }
        }

        match unsafe { posix::pthread_rwlock_init(mtx, attributes.get()).into() } {
            Errno::ESUCCES => (),
            Errno::EAGAIN => {
                fail!(from origin, with ReadWriteMutexCreationError::InsufficientResources, "{} due to insufficient resources.", msg);
            }
            Errno::ENOMEM => {
                fail!(from origin, with ReadWriteMutexCreationError::InsufficientResources, "{} due to insufficient memory.", msg);
            }
            Errno::EPERM => {
                fail!(from origin, with ReadWriteMutexCreationError::InsufficientPermissions, "{} due to insufficient permissions.", msg);
            }
            v => {
                fail!(from origin, with ReadWriteMutexCreationError::UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v);
            }
        };

        match self.is_interprocess_capable {
            true => Ok(Capability::InterProcess),
            false => Ok(Capability::ProcessLocal),
        }
    }

    /// Creates a new [`ReadWriteMutex`]
    pub fn create<T: Debug>(
        self,
        t: T,
        handle: &ReadWriteMutexHandle<T>,
    ) -> Result<ReadWriteMutex<'_, T>, ReadWriteMutexCreationError> {
        unsafe {
            handle
                .handle
                .initialize(|mtx| self.initialize_rw_mutex(mtx))?
        };

        unsafe { *handle.clock_type.get() = self.clock_type };
        unsafe { *handle.value.get() = Some(t) };

        Ok(ReadWriteMutex::new(handle))
    }
}

/// A guard which provides read access to the underlying value of a [`ReadWriteMutex`].
///
/// Is returned by [`ReadWriteMutex::read_lock()`], [`ReadWriteMutex::read_try_lock()`] and
/// [`ReadWriteMutex::read_timed_lock()`].
#[derive(Debug)]
pub struct MutexReadGuard<'a, 'b, T: Debug> {
    mutex: &'a ReadWriteMutex<'b, T>,
}

unsafe impl<T: Send + Debug> Send for MutexReadGuard<'_, '_, T> {}
unsafe impl<T: Send + Sync + Debug> Sync for MutexReadGuard<'_, '_, T> {}

impl<T: Debug> Deref for MutexReadGuard<'_, '_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { (*self.mutex.handle.value.get()).as_ref().unwrap() }
    }
}

impl<T: Debug> Drop for MutexReadGuard<'_, '_, T> {
    fn drop(&mut self) {
        if self.mutex.release().is_err() {
            fatal_panic!(from self.mutex, "This should never happen! Failed to release read lock.");
        }
    }
}

/// A guard which provides read and write access to the underlying value of a [`ReadWriteMutex`].
///
/// Is returned by [`ReadWriteMutex::write_lock()`], [`ReadWriteMutex::write_try_lock()`] and
/// [`ReadWriteMutex::write_timed_lock()`].
#[derive(Debug)]
pub struct MutexWriteGuard<'a, 'b, T: Debug> {
    mutex: &'a ReadWriteMutex<'b, T>,
}

unsafe impl<T: Send + Debug> Send for MutexWriteGuard<'_, '_, T> {}
unsafe impl<T: Send + Sync + Debug> Sync for MutexWriteGuard<'_, '_, T> {}

impl<T: Debug> Deref for MutexWriteGuard<'_, '_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { (*self.mutex.handle.value.get()).as_ref().unwrap() }
    }
}

impl<T: Debug> DerefMut for MutexWriteGuard<'_, '_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { (*self.mutex.handle.value.get()).as_mut().unwrap() }
    }
}

impl<T: Debug> Drop for MutexWriteGuard<'_, '_, T> {
    fn drop(&mut self) {
        if self.mutex.release().is_err() {
            fatal_panic!(from self.mutex, "This should never happen! Failed to release write lock.");
        }
    }
}

/// The underlying memory of a [`ReadWriteMutex`] is not allowed to be moved. This issue is solved
/// by storing the underlying posix handle inside [`ReadWriteMutexHandle`]. When a [`ReadWriteMutex`]
/// is initialized it stores a const reference to the [`ReadWriteMutexHandle`] and makes it by
/// that inmovable.
#[derive(Debug)]
pub struct ReadWriteMutexHandle<T: Sized + Debug> {
    handle: HandleStorage<posix::pthread_rwlock_t>,
    clock_type: UnsafeCell<ClockType>,
    value: UnsafeCell<Option<T>>,
}

unsafe impl<T: Sized + Debug> Send for ReadWriteMutexHandle<T> {}
unsafe impl<T: Sized + Debug> Sync for ReadWriteMutexHandle<T> {}

impl<T: Sized + Debug> Handle for ReadWriteMutexHandle<T> {
    fn new() -> Self {
        Self {
            handle: HandleStorage::new(posix::pthread_rwlock_t::new()),
            clock_type: UnsafeCell::new(ClockType::default()),
            value: UnsafeCell::new(None),
        }
    }

    fn state(&self) -> HandleState {
        self.handle.state()
    }

    fn is_inter_process_capable(&self) -> bool {
        self.handle.is_inter_process_capable()
    }
}

impl<T: Sized + Debug> Drop for ReadWriteMutexHandle<T> {
    fn drop(&mut self) {
        if self.handle.state() == HandleState::Initialized {
            unsafe {
                self.handle.cleanup(|mtx|{
                    if posix::pthread_rwlock_destroy(mtx) != 0 {
                        warn!(from self,
                            "Unable to destroy read write mutex. Was it already destroyed by another instance in another process?");
                    }
                });
            }
        }
    }
}

/// A POSIX read write mutex where either multiple readers can acquire multiple read-locks or one
/// writer can acquire a write-lock.
/// It is built by the [`ReadWriteMutexBuilder`].
#[derive(Debug)]
pub struct ReadWriteMutex<'a, T: Sized + Debug> {
    handle: &'a ReadWriteMutexHandle<T>,
}

unsafe impl<'a, T: Send + Debug> Send for ReadWriteMutex<'a, T> {}
unsafe impl<'a, T: Sync + Debug> Sync for ReadWriteMutex<'a, T> {}

impl<'a, T: Sized + Debug> IpcConstructible<'a, ReadWriteMutexHandle<T>> for ReadWriteMutex<'a, T> {
    fn new(handle: &'a ReadWriteMutexHandle<T>) -> Self {
        Self { handle }
    }
}

impl<'a, T: Sized + Debug> IpcCapable<'a, ReadWriteMutexHandle<T>> for ReadWriteMutex<'a, T> {
    fn is_interprocess_capable(&self) -> bool {
        self.handle.is_inter_process_capable()
    }
}

impl<'a, T: Sized + Debug> ReadWriteMutex<'a, T> {
    fn new(handle: &'a ReadWriteMutexHandle<T>) -> Self {
        Self { handle }
    }

    /// Returns the used clock type of the mutex
    pub fn clock_type(&self) -> ClockType {
        unsafe { *self.handle.clock_type.get() }
    }

    /// Blocks until a read-lock could be acquired and returns a [`MutexReadGuard`] to provide
    /// read access to the underlying value.
    pub fn read_lock(&self) -> Result<MutexReadGuard<'_, '_, T>, ReadWriteMutexReadLockError> {
        let msg = "Failed to acquire read-lock";
        handle_errno!(ReadWriteMutexReadLockError, from self,
            errno_source unsafe { posix::pthread_rwlock_rdlock(self.handle.handle.get()).into() },
            success Errno::ESUCCES => MutexReadGuard { mutex: self },
            Errno::EAGAIN => (MaximumAmountOfReadLocksAcquired, "{} since the maximum amount of read-locks is already acquired.", msg),
            Errno::EDEADLK => (DeadlockConditionDetected, "{} since a deadlock condition was detected.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    /// Tries to acquire a read-lock. If a write-locks was already acquired it returns [`None`]
    /// otherwise a [`MutexReadGuard`].
    pub fn read_try_lock(
        &self,
    ) -> Result<Option<MutexReadGuard<'_, '_, T>>, ReadWriteMutexReadLockError> {
        let msg = "Failed to try to acquire read-lock";
        handle_errno!(ReadWriteMutexReadLockError, from self,
            errno_source unsafe { posix::pthread_rwlock_tryrdlock(self.handle.handle.get()).into() },
            success Errno::ESUCCES => Some(MutexReadGuard { mutex: self });
            success Errno::EBUSY => None,
            Errno::EAGAIN => (MaximumAmountOfReadLocksAcquired, "{} since the maximum amount of read-locks is already acquired.", msg),
            Errno::EDEADLK => (DeadlockConditionDetected, "{} since a deadlock condition was detected.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    /// Tries to acquire a read-lock until the timeout has passed. If a read-lock could be
    /// acquired it returns a [`MutexReadGuard`], if the timeout has passed it returns [`None`].
    pub fn read_timed_lock(
        &self,
        timeout: Duration,
    ) -> Result<Option<MutexReadGuard<'_, '_, T>>, ReadWriteMutexReadTimedLockError> {
        let msg = "Failted to timed wait for read lock";

        match self.clock_type() {
            ClockType::Realtime => {
                let now = fail!(from self, when Time::now_with_clock(ClockType::Realtime),
                    "{} due to a failure while acquiring current system time.", msg);
                let timeout_adjusted = now.as_duration() + timeout;
                handle_errno!(ReadWriteMutexReadTimedLockError, from self,
                    errno_source unsafe { posix::pthread_rwlock_timedrdlock(self.handle.handle.get(), &timeout_adjusted.as_timespec()) }.into(),
                    success Errno::ESUCCES => Some(MutexReadGuard { mutex: self });
                    success Errno::ETIMEDOUT => None,
                    Errno::EAGAIN => (ReadWriteMutexReadLockError(ReadWriteMutexReadLockError::MaximumAmountOfReadLocksAcquired), "{} since the maximum number of read locks were already acquired.", msg),
                    Errno::EINVAL => (TimeoutExceedsMaximumSupportedDuration, "{} since the timeout of {:?} exceeds the maximum supported duration.", msg, timeout),
                    Errno::EDEADLK => (ReadWriteMutexReadLockError(ReadWriteMutexReadLockError::DeadlockConditionDetected), "{} since the operation would lead to a deadlock.", msg),
                    v => (ReadWriteMutexReadLockError(ReadWriteMutexReadLockError::UnknownError(v as i32)), "{} since unknown error occurred while acquiring the lock ({})", msg, v)
                )
            }
            ClockType::Monotonic => {
                let time = fail!(from self, when  Time::now_with_clock(self.clock_type()),
                        "{} due to a failure while acquiring current system time in.", msg);
                let mut adaptive_wait = fail!(from self, when AdaptiveWaitBuilder::new()
                    .clock_type(self.clock_type())
                    .create(), "{} since the adaptive wait could not be created.", msg);

                loop {
                    match self.read_try_lock() {
                        Ok(Some(v)) => return Ok(Some(v)),
                        Ok(None) => match fail!(from self, when time.elapsed(),
                             "{} due to a failure while acquiring elapsed system time.", msg)
                            < timeout
                        {
                            true => {
                                fail!(from self, when adaptive_wait.wait(), "{} since AdaptiveWait failed.", msg);
                            }
                            false => return Ok(None),
                        },
                        Err(v) => {
                            fail!(from self, with ReadWriteMutexReadTimedLockError::ReadWriteMutexReadLockError(v),
                        "{} since read_try_lock failed.", msg);
                        }
                    }
                }
            }
        }
    }

    /// Blocks until a write-lock could be acquired and returns a [`MutexWriteGuard`] to provide
    /// read-write access to the underlying value.
    pub fn write_lock(&self) -> Result<MutexWriteGuard<'_, '_, T>, ReadWriteMutexWriteLockError> {
        let msg = "Failed to acquire write-lock";
        handle_errno!(ReadWriteMutexWriteLockError, from self,
            errno_source unsafe { posix::pthread_rwlock_wrlock(self.handle.handle.get()).into() },
            success Errno::ESUCCES => MutexWriteGuard { mutex: self },
            Errno::EDEADLK => (DeadlockConditionDetected, "{} since a deadlock condition was detected.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    /// Tries to acquire a read-lock. If a read-locks was already acquired it returns [`None`]
    /// otherwise a [`MutexWriteGuard`].
    pub fn write_try_lock(
        &self,
    ) -> Result<Option<MutexWriteGuard<'_, '_, T>>, ReadWriteMutexWriteLockError> {
        let msg = "Failed to try to acquire write-lock";
        handle_errno!(ReadWriteMutexWriteLockError, from self,
            errno_source unsafe { posix::pthread_rwlock_trywrlock(self.handle.handle.get()).into() },
            success Errno::ESUCCES => Some(MutexWriteGuard { mutex: self });
            success Errno::EBUSY => None,
            Errno::EDEADLK => (DeadlockConditionDetected, "{} since a deadlock condition was detected.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    /// Tries to acquire a write-lock until the timeout has passed. If a write-lock could be
    /// acquired it returns a [`MutexWriteGuard`], if the timeout has passed it returns [`None`].
    pub fn write_timed_lock(
        &self,
        timeout: Duration,
    ) -> Result<Option<MutexWriteGuard<'_, '_, T>>, ReadWriteMutexWriteTimedLockError> {
        let msg = "Failed to timed wait for write lock";

        match self.clock_type() {
            ClockType::Realtime => {
                let now = fail!(from self, when Time::now_with_clock(ClockType::Realtime),
                    "{} due to a failure while acquiring current system time.", msg);
                let timeout_adjusted = now.as_duration() + timeout;
                handle_errno!(ReadWriteMutexWriteTimedLockError, from self,
                    errno_source unsafe { posix::pthread_rwlock_timedwrlock(self.handle.handle.get(), &timeout_adjusted.as_timespec()) }.into(),
                    success Errno::ESUCCES => Some(MutexWriteGuard { mutex: self });
                    success Errno::ETIMEDOUT => None,
                    Errno::EINVAL => (TimeoutExceedsMaximumSupportedDuration, "{} since the timeout of {:?} exceeds the maximum supported duration.", msg, timeout),
                    Errno::EDEADLK => (ReadWriteMutexWriteLockError(ReadWriteMutexWriteLockError::DeadlockConditionDetected), "{} since the operation would lead to a deadlock.", msg),
                    v => (ReadWriteMutexWriteLockError(ReadWriteMutexWriteLockError::UnknownError(v as i32)), "{} since unknown error occurred while acquiring the lock ({})", msg, v)
                )
            }
            ClockType::Monotonic => {
                let time = fail!(from self, when Time::now_with_clock(self.clock_type()),
                        "{} due to a failure while acquiring current system time.", msg);
                let mut adaptive_wait = fail!(from self, when AdaptiveWaitBuilder::new()
            .clock_type(self.clock_type())
            .create(), "{} since the adaptive wait could not be created.", msg);

                loop {
                    match self.write_try_lock() {
                        Ok(Some(v)) => return Ok(Some(v)),
                        Ok(None) => match fail!(from self, when time.elapsed(),
                                "{} due to a failure while acquiring elapsed system time.", msg)
                            < timeout
                        {
                            true => {
                                fail!(from self, when adaptive_wait.wait(), "{} since AdaptiveWait failed.", msg);
                            }
                            false => return Ok(None),
                        },
                        Err(v) => {
                            fail!(from self, with ReadWriteMutexWriteTimedLockError::ReadWriteMutexWriteLockError(v),
                                "{} since write_try_lock failed.", msg);
                        }
                    }
                }
            }
        }
    }

    fn release(&self) -> Result<(), ReadWriteMutexUnlockError> {
        let msg = "Unable to release lock";
        match unsafe { posix::pthread_rwlock_unlock(self.handle.handle.get()).into() } {
            Errno::ESUCCES => Ok(()),
            Errno::EPERM => {
                fail!(from self, with ReadWriteMutexUnlockError::OwnedByDifferentEntity,
                    "{} since it is not owned by the current thread.", msg);
            }
            v => {
                fail!(from self, with ReadWriteMutexUnlockError::UnknownError(v as i32),
                    "{} since an unknown error occurred ({}).", msg, v);
            }
        }
    }
}
