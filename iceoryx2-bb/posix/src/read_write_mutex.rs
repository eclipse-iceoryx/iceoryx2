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
//! ```no_run
//! use iceoryx2_bb_posix::read_write_mutex::*;
//! use std::thread;
//! use core::time::Duration;
//! use iceoryx2_bb_posix::clock::ClockType;
//!
//! let rw_handle = ReadWriteMutexHandle::new();
//! let rw_mutex = ReadWriteMutexBuilder::new()
//!                         .is_interprocess_capable(true)
//!                         .create(123, &rw_handle)
//!                         .expect("failed to create rw mutex");
//!
//! thread::scope(|s| {
//!     s.spawn(|| {
//!         let guard = rw_mutex.read_blocking_lock()
//!                             .expect("failed to read_lock");
//!         println!("The mutex value is: {}", *guard);
//!     });
//!
//!     s.spawn(|| {
//!         let mut guard = rw_mutex.write_blocking_lock()
//!                                 .expect("failed to write_lock");
//!         println!("The old value is: {}", *guard);
//!         *guard = 456;
//!         println!("The new value is: {}", *guard);
//!     });
//! });
//! ```
pub use crate::ipc_capable::{Handle, IpcCapable};

use crate::handle_errno;
use crate::ipc_capable::internal::{Capability, HandleStorage, IpcConstructible};
use iceoryx2_bb_elementary::{enum_gen, scope_guard::ScopeGuardBuilder};
use iceoryx2_bb_log::{fail, fatal_panic, warn};
use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::posix::MemZeroedStruct;
use iceoryx2_pal_posix::*;

use core::marker::PhantomData;
use core::{
    cell::UnsafeCell,
    fmt::Debug,
    ops::{Deref, DerefMut},
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
    /// The ReadWriteMutexError enum is a generalization when one doesn't require the fine-grained error
    /// handling enums. One can forward ReadWriteMutexError as more generic return value when a method
    /// returns a ReadWriteMutex***Error.
    /// On a higher level it is again convertable to [`crate::Error`].
    ReadWriteMutexError
  generalization:
    FailedToLock <= ReadWriteMutexWriteLockError; ReadWriteMutexReadLockError,
    FailedToCreate <= ReadWriteMutexCreationError
}

/// The builder for the [`ReadWriteMutex`].
#[derive(Debug)]
pub struct ReadWriteMutexBuilder {
    is_interprocess_capable: bool,
}

impl Default for ReadWriteMutexBuilder {
    fn default() -> Self {
        ReadWriteMutexBuilder {
            is_interprocess_capable: true,
        }
    }
}

impl ReadWriteMutexBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Defines if the [`ReadWriteMutex`] is inter-process capable or not.
    pub fn is_interprocess_capable(mut self, value: bool) -> Self {
        self.is_interprocess_capable = value;
        self
    }

    fn initialize_rw_mutex(
        &self,
        mtx: *mut posix::pthread_rwlock_t,
    ) -> Result<Capability, ReadWriteMutexCreationError> {
        let msg = "Failed to create mutex";
        let origin = format!("{self:?}");

        let mut attributes = ScopeGuardBuilder::new(posix::pthread_rwlockattr_t::new_zeroed()).on_init(|attr| {
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
    ) -> Result<ReadWriteMutex<'_, '_, T>, ReadWriteMutexCreationError> {
        unsafe {
            handle
                .handle
                .initialize(|mtx| self.initialize_rw_mutex(mtx))?
        };

        unsafe { *handle.value.get() = Some(t) };

        Ok(ReadWriteMutex::new(handle))
    }
}

/// A guard which provides read access to the underlying value of a [`ReadWriteMutex`].
///
/// Is returned by [`ReadWriteMutex::read_blocking_lock()`] and [`ReadWriteMutex::read_try_lock()`].
#[derive(Debug)]
pub struct MutexReadGuard<'handle, T: Debug> {
    handle: &'handle ReadWriteMutexHandle<T>,
}

unsafe impl<T: Send + Debug> Send for MutexReadGuard<'_, T> {}
unsafe impl<T: Send + Sync + Debug> Sync for MutexReadGuard<'_, T> {}

impl<T: Debug> Deref for MutexReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { (*self.handle.value.get()).as_ref().unwrap() }
    }
}

impl<T: Debug> Drop for MutexReadGuard<'_, T> {
    fn drop(&mut self) {
        if ReadWriteMutex::release(self.handle).is_err() {
            fatal_panic!(from self, "This should never happen! Failed to release read lock.");
        }
    }
}

/// A guard which provides read and write access to the underlying value of a [`ReadWriteMutex`].
///
/// Is returned by [`ReadWriteMutex::write_blocking_lock()`] and [`ReadWriteMutex::write_try_lock()`].
#[derive(Debug)]
pub struct MutexWriteGuard<'handle, T: Debug> {
    handle: &'handle ReadWriteMutexHandle<T>,
}

unsafe impl<T: Send + Debug> Send for MutexWriteGuard<'_, T> {}
unsafe impl<T: Send + Sync + Debug> Sync for MutexWriteGuard<'_, T> {}

impl<T: Debug> Deref for MutexWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { (*self.handle.value.get()).as_ref().unwrap() }
    }
}

impl<T: Debug> DerefMut for MutexWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { (*self.handle.value.get()).as_mut().unwrap() }
    }
}

impl<T: Debug> Drop for MutexWriteGuard<'_, T> {
    fn drop(&mut self) {
        if ReadWriteMutex::release(self.handle).is_err() {
            fatal_panic!(from self, "This should never happen! Failed to release write lock.");
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
    value: UnsafeCell<Option<T>>,
}

unsafe impl<T: Sized + Debug> Send for ReadWriteMutexHandle<T> {}
unsafe impl<T: Sized + Debug> Sync for ReadWriteMutexHandle<T> {}

impl<T: Sized + Debug> Handle for ReadWriteMutexHandle<T> {
    fn new() -> Self {
        Self {
            handle: HandleStorage::new(posix::pthread_rwlock_t::new_zeroed()),
            value: UnsafeCell::new(None),
        }
    }

    fn is_initialized(&self) -> bool {
        self.handle.is_initialized()
    }

    fn is_inter_process_capable(&self) -> bool {
        self.handle.is_inter_process_capable()
    }
}

impl<T: Sized + Debug> Drop for ReadWriteMutexHandle<T> {
    fn drop(&mut self) {
        if self.handle.is_initialized() {
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
pub struct ReadWriteMutex<'this, 'handle: 'this, T: Sized + Debug> {
    handle: &'handle ReadWriteMutexHandle<T>,
    _lifetime: PhantomData<&'this ()>,
}

unsafe impl<T: Send + Debug> Send for ReadWriteMutex<'_, '_, T> {}
unsafe impl<T: Sync + Debug> Sync for ReadWriteMutex<'_, '_, T> {}

impl<'handle, T: Sized + Debug> IpcConstructible<'handle, ReadWriteMutexHandle<T>>
    for ReadWriteMutex<'_, 'handle, T>
{
    fn new(handle: &'handle ReadWriteMutexHandle<T>) -> Self {
        Self {
            handle,
            _lifetime: PhantomData,
        }
    }
}

impl<'handle, T: Sized + Debug> IpcCapable<'handle, ReadWriteMutexHandle<T>>
    for ReadWriteMutex<'_, 'handle, T>
{
    fn is_interprocess_capable(&self) -> bool {
        self.handle.is_inter_process_capable()
    }
}

impl<'this, 'handle: 'this, T: Sized + Debug> ReadWriteMutex<'this, 'handle, T> {
    /// Instantiates a [`ReadWriteMutex`] from an already initialized [`ReadWriteMutexHandle`].
    /// Useful for inter-process usage where the [`ReadWriteMutexHandle`] was created by
    /// [`ReadWriteMutexBuilder`] in another process.
    ///
    /// # Safety
    ///
    /// * `handle` must have been successfully initialized by the [`ReadWriteMutexBuilder`].
    pub fn from_handle(
        handle: &'handle ReadWriteMutexHandle<T>,
    ) -> ReadWriteMutex<'this, 'handle, T> {
        Self::new(handle)
    }

    fn new(handle: &'handle ReadWriteMutexHandle<T>) -> Self {
        Self {
            handle,
            _lifetime: PhantomData,
        }
    }

    pub fn read_blocking_lock(
        &'this self,
    ) -> Result<MutexReadGuard<'handle, T>, ReadWriteMutexReadLockError> {
        let msg = "Failed to acquire read-lock";
        handle_errno!(ReadWriteMutexReadLockError, from self,
            errno_source unsafe { posix::pthread_rwlock_rdlock(self.handle.handle.get()).into() },
            success Errno::ESUCCES => MutexReadGuard { handle: self.handle },
            Errno::EAGAIN => (MaximumAmountOfReadLocksAcquired, "{} since the maximum amount of read-locks is already acquired.", msg),
            Errno::EDEADLK => (DeadlockConditionDetected, "{} since a deadlock condition was detected.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    /// Tries to acquire a read-lock. If a write-locks was already acquired it returns [`None`]
    /// otherwise a [`MutexReadGuard`].
    pub fn read_try_lock(
        &'this self,
    ) -> Result<Option<MutexReadGuard<'handle, T>>, ReadWriteMutexReadLockError> {
        let msg = "Failed to try to acquire read-lock";
        handle_errno!(ReadWriteMutexReadLockError, from self,
            errno_source unsafe { posix::pthread_rwlock_tryrdlock(self.handle.handle.get()).into() },
            success Errno::ESUCCES => Some(MutexReadGuard { handle: self.handle });
            success Errno::EBUSY => None;
            success Errno::EDEADLK => None,
            Errno::EAGAIN => (MaximumAmountOfReadLocksAcquired, "{} since the maximum amount of read-locks is already acquired.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    /// Blocks until a write-lock could be acquired and returns a [`MutexWriteGuard`] to provide
    /// read-write access to the underlying value.
    pub fn write_blocking_lock(
        &'this self,
    ) -> Result<MutexWriteGuard<'handle, T>, ReadWriteMutexWriteLockError> {
        let msg = "Failed to acquire write-lock";
        handle_errno!(ReadWriteMutexWriteLockError, from self,
            errno_source unsafe { posix::pthread_rwlock_wrlock(self.handle.handle.get()).into() },
            success Errno::ESUCCES => MutexWriteGuard { handle: self.handle },
            Errno::EDEADLK => (DeadlockConditionDetected, "{} since a deadlock condition was detected.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    /// Tries to acquire a read-lock. If a read-locks was already acquired it returns [`None`]
    /// otherwise a [`MutexWriteGuard`].
    pub fn write_try_lock(
        &'this self,
    ) -> Result<Option<MutexWriteGuard<'handle, T>>, ReadWriteMutexWriteLockError> {
        let msg = "Failed to try to acquire write-lock";
        handle_errno!(ReadWriteMutexWriteLockError, from self,
            errno_source unsafe { posix::pthread_rwlock_trywrlock(self.handle.handle.get()).into() },
            success Errno::ESUCCES => Some(MutexWriteGuard { handle: self.handle });
            success Errno::EBUSY => None;
            success Errno::EDEADLK => None,
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    fn release(handle: &ReadWriteMutexHandle<T>) -> Result<(), ReadWriteMutexUnlockError> {
        let msg = "Unable to release lock";
        match unsafe { posix::pthread_rwlock_unlock(handle.handle.get()).into() } {
            Errno::ESUCCES => Ok(()),
            Errno::EPERM => {
                fail!(from handle, with ReadWriteMutexUnlockError::OwnedByDifferentEntity,
                    "{} since it is not owned by the current thread.", msg);
            }
            v => {
                fail!(from handle, with ReadWriteMutexUnlockError::UnknownError(v as i32),
                    "{} since an unknown error occurred ({}).", msg, v);
            }
        }
    }
}
