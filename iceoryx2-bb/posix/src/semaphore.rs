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

//! Provides the [`NamedSemaphore`] and the [`UnnamedSemaphore`]. Both can be used in an
//! inter-process context to signal events between processes.

pub use crate::ipc_capable::{Handle, IpcCapable};

use core::cell::UnsafeCell;
use core::fmt::Debug;

use crate::ipc_capable::internal::{Capability, HandleStorage, IpcConstructible};
use iceoryx2_bb_container::semantic_string::*;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_log::{debug, fail, fatal_panic, warn};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::*;
use iceoryx2_bb_system_types::path::*;
use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::posix::MemZeroedStruct;
use iceoryx2_pal_posix::*;

use crate::{
    adaptive_wait::*,
    clock::{AsTimespec, Time, TimeError},
    config::MAX_INITIAL_SEMAPHORE_VALUE,
    handle_errno,
    system_configuration::*,
};
use core::time::Duration;

pub use crate::clock::ClockType;
pub use crate::creation_mode::CreationMode;
pub use crate::permission::Permission;

enum_gen! { NamedSemaphoreCreationError
  entry:
    InsufficientPermissions,
    InitialValueTooLarge,
    PerProcessFileHandleLimitReached,
    SystemWideFileHandleLimitReached,
    AlreadyExists,
    MaxFilePathLengthExceeded,
    Interrupt,
    NotSupportForGivenName,
    DoesNotExist,
    NoSpaceLeft,
    UnknownError(i32)
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum UnnamedSemaphoreCreationError {
    InitialValueTooLarge,
    ExceedsMaximumNumberOfSemaphores,
    InsufficientPermissions,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum SemaphorePostError {
    Overflow,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum SemaphoreWaitError {
    NotSupported,
    DeadlockConditionDetected,
    Interrupt,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum UnnamedSemaphoreOpenIpcHandleError {
    IsNotInterProcessCapable,
    Uninitialized,
}

enum_gen! {
    SemaphoreTimedWaitError
  entry:
    WaitingTimeExceedsSystemLimits
  mapping:
    SemaphoreWaitError,
    AdaptiveWaitError,
    TimeError
}

enum_gen! {
    /// The SemaphoreError enum is a generalization when one doesn't require the fine-grained error
    /// handling enums. One can forward SemaphoreError as more generic return value when a method
    /// returns a Semaphore***Error.
    /// On a higher level it is again convertable to [`crate::Error`].
    SemaphoreError
  generalization:
    FailedToCreate <= NamedSemaphoreCreationError; UnnamedSemaphoreCreationError,
    FailedToPost <= SemaphorePostError,
    FailedToWait <= SemaphoreWaitError; SemaphoreTimedWaitError
}

#[derive(PartialEq, Eq)]
enum UnlinkMode {
    IgnoreNonExistingSemaphore,
    FailWhenSemaphoreDoesNotExist,
}

#[derive(PartialEq, Eq)]
enum InitMode {
    Create,
    Open,
    TryOpen,
}

mod internal {
    use super::*;

    #[doc(hidden)]
    pub trait SemaphoreHandle {
        fn handle(&self) -> *mut posix::sem_t;
        fn get_clock_type(&self) -> ClockType;
    }
}

/// Defines the interface of a [`NamedSemaphore`] and an [`UnnamedSemaphore`].
pub trait SemaphoreInterface: internal::SemaphoreHandle + Debug {
    /// Increments the semaphore by one. If the semaphore already holds the maximum supported value
    /// another post call will lead to [`SemaphorePostError::Overflow`].
    fn post(&self) -> Result<(), SemaphorePostError> {
        if unsafe { posix::sem_post(self.handle()) } == 0 {
            return Ok(());
        }

        let msg = "Unable to post semaphore";
        handle_errno!(SemaphorePostError, from self,
            fatal Errno::EINVAL => ("This should never happen! {} since an invalid handle was provided.", msg),
            Errno::EOVERFLOW => (Overflow, "{} since the operation would cause an overflow.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    /// Decrements the semaphore by one. If the semaphore is zero it waits until a
    /// [`SemaphoreInterface::post()`] call incremented the semaphore by one. A semaphores internal
    /// value is always greater or equal to zero.
    fn blocking_wait(&self) -> Result<(), SemaphoreWaitError> {
        if unsafe { posix::sem_wait(self.handle()) } == 0 {
            return Ok(());
        }

        let msg = "Unable to wait on semaphore";
        handle_errno!(SemaphoreWaitError, from self,
            fatal Errno::EINVAL => ("This should never happen! {} since an invalid handle was provided!", msg),
            Errno::ENOSYS => (NotSupported, "{} since sem_wait is not supported by the system.", msg),
            Errno::EDEADLK => (DeadlockConditionDetected, "{} since a deadlock condition was detected.", msg),
            Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    /// Tries to decrement the semaphore by one if it is greater zero and returns true. If the semaphores
    /// internal value is zero it returns false and does not decrement the semaphore.
    fn try_wait(&self) -> Result<bool, SemaphoreWaitError> {
        if unsafe { posix::sem_trywait(self.handle()) } == 0 {
            return Ok(true);
        }

        let msg = "Unable to wait on semaphore";
        handle_errno!(SemaphoreWaitError, from self,
            success Errno::EAGAIN => false,
            fatal Errno::EINVAL => ("This should never happen! {} since an invalid handle was provided!", msg),
            Errno::ENOSYS => (NotSupported, "{} since sem_wait is not supported by the system.", msg),
            Errno::EDEADLK => (DeadlockConditionDetected, "{} since a deadlock condition was detected.", msg),
            Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    /// Tries to decrement the semaphore until the decrement was successful and returns true
    /// or the timeout has passed and then returns false.
    fn timed_wait(&self, timeout: Duration) -> Result<bool, SemaphoreTimedWaitError> {
        let msg = "Unable to timed wait on semaphore";
        match self.clock_type() {
            ClockType::Monotonic => {
                let mut adaptive_wait = fail!(from self, when AdaptiveWaitBuilder::new()
                    .clock_type(self.clock_type())
                    .create(), "{} since the adaptive wait could not be created.", msg);

                match adaptive_wait.timed_wait_while(
                    || -> Result<bool, SemaphoreWaitError> { Ok(!self.try_wait()?) },
                    timeout,
                ) {
                    Ok(v) => Ok(v),
                    Err(AdaptiveTimedWaitWhileError::PredicateFailure(v)) => {
                        fail!(from self, with SemaphoreTimedWaitError::from(v),
                            "{} since try_wait() failed with ({:?}).", msg, v);
                    }
                    Err(AdaptiveTimedWaitWhileError::AdaptiveWaitError(v)) => {
                        fail!(from self, with SemaphoreTimedWaitError::from(v),
                             "{} since adaptive wait failed with ({:?}).", msg, v);
                    }
                }
            }
            ClockType::Realtime => {
                let wait_time = timeout
                    + fail!(from self, when Time::now_with_clock(self.clock_type()),
                    "{} due to a failure while acquiring the current system time.", msg)
                    .as_duration();
                if unsafe { posix::sem_timedwait(self.handle(), &wait_time.as_timespec()) } == 0 {
                    return Ok(true);
                }

                let msg = "Failed to perform timedwait";
                handle_errno!(SemaphoreTimedWaitError, from self,
                    success Errno::ETIMEDOUT => false,
                    Errno::EINVAL => (WaitingTimeExceedsSystemLimits, "{} since the provided duration {:?} exceeds the maximum supported limit.", msg, timeout),
                    Errno::EDEADLK => (SemaphoreWaitError(SemaphoreWaitError::DeadlockConditionDetected), "{} since a deadlock condition was detected.", msg),
                    Errno::EINTR => (SemaphoreWaitError(SemaphoreWaitError::Interrupt), "{} since an interrupt signal occurred.", msg),
                    v => (SemaphoreWaitError(SemaphoreWaitError::UnknownError(v as i32)), "{} since an unknown error occurred ({}).", msg, v)
                )
            }
        }
    }

    fn clock_type(&self) -> ClockType {
        self.get_clock_type()
    }
}

/// Builder for the [`NamedSemaphore`].
///
/// # Example
///
/// ## Create new named semaphore
///
/// ```ignore
/// use iceoryx2_bb_posix::semaphore::*;
/// use iceoryx2_bb_system_types::file_name::FileName;
/// use iceoryx2_bb_container::semantic_string::*;
///
/// let name = FileName::new(b"mySemaphoreName").unwrap();
/// let semaphore = NamedSemaphoreBuilder::new(&name)
///     // defines the clock which is used in [`SemaphoreInterface::timed_wait()`]
///                     .clock_type(ClockType::Monotonic)
///     // the semaphore is created, if there already exists a semaphore it is deleted
///                     .creation_mode(CreationMode::PurgeAndCreate)
///                     .initial_value(5)
///                     .permission(Permission::OWNER_ALL | Permission::GROUP_ALL)
///                     .create()
///                     .expect("failed to create semaphore");
/// ```
///
/// ## Open existing semaphore
///
/// ```no_run
/// use iceoryx2_bb_posix::semaphore::*;
/// use iceoryx2_bb_system_types::file_name::FileName;
/// use iceoryx2_bb_container::semantic_string::*;
///
/// let name = FileName::new(b"mySemaphoreName").unwrap();
/// let semaphore = NamedSemaphoreBuilder::new(&name)
///                     .clock_type(ClockType::Monotonic)
///                     .open_existing()
///                     .expect("failed to open semaphore");
/// ```
#[derive(Debug)]
pub struct NamedSemaphoreBuilder {
    name: FileName,
    initial_value: u32,
    permission: Permission,
    clock_type: ClockType,
    creation_mode: Option<CreationMode>,
}

impl NamedSemaphoreBuilder {
    pub fn new(name: &FileName) -> Self {
        Self {
            creation_mode: None,
            name: name.clone(),
            initial_value: 0,
            permission: Permission::OWNER_ALL,
            clock_type: ClockType::default(),
        }
    }

    /// Sets the type of clock which will be used in [`SemaphoreInterface::timed_wait()`]. Be
    /// aware a clock like [`ClockType::Realtime`] is depending on the systems local time. If this
    /// time changes while waiting it can cause extrem long waits or no wait at all.
    pub fn clock_type(mut self, value: ClockType) -> Self {
        self.clock_type = value;
        self
    }

    /// Opens an already existing [`NamedSemaphore`].
    pub fn open_existing(self) -> Result<NamedSemaphore, NamedSemaphoreCreationError> {
        NamedSemaphore::new(self)
    }

    /// Defines how the semaphore will be created and returns the [`NamedSemaphoreCreationBuilder`]
    /// which provides further means of configuration only available when a semaphore is created.
    pub fn creation_mode(mut self, creation_mode: CreationMode) -> NamedSemaphoreCreationBuilder {
        self.creation_mode = Some(creation_mode);
        NamedSemaphoreCreationBuilder { config: self }
    }
}

/// Provides additional settings which are only available for newly created semaphores. Is
/// returned by [`NamedSemaphoreBuilder::creation_mode()`].
///
/// For an example see [`NamedSemaphoreBuilder`]
pub struct NamedSemaphoreCreationBuilder {
    config: NamedSemaphoreBuilder,
}

impl NamedSemaphoreCreationBuilder {
    /// Sets the initial value of the semaphore. Must be less than [`MAX_INITIAL_SEMAPHORE_VALUE`].
    pub fn initial_value(mut self, value: u32) -> Self {
        self.config.initial_value = value;
        self
    }

    /// Sets the permission of the newly created semaphore.
    pub fn permission(mut self, value: Permission) -> Self {
        self.config.permission = value;
        self
    }

    /// Creates a [`NamedSemaphore`].
    pub fn create(self) -> Result<NamedSemaphore, NamedSemaphoreCreationError> {
        NamedSemaphore::new(self.config)
    }
}

/// Represents a POSIX named semaphore - a semaphore with a corresponding file handle which can
/// be opened by other processes. The filename corresponds to the semaphore name. A named semaphore
/// is created by the [`NamedSemaphoreBuilder`].
///
/// # Example
///
/// ## In process 1
/// ```no_run
/// use iceoryx2_bb_posix::semaphore::*;
/// use iceoryx2_bb_posix::clock::*;
/// use core::time::Duration;
/// use iceoryx2_bb_system_types::file_name::FileName;
/// use iceoryx2_bb_container::semantic_string::*;
///
/// let name = FileName::new(b"mySemaphoreName").unwrap();
/// let semaphore = NamedSemaphoreBuilder::new(&name)
///                     .creation_mode(CreationMode::PurgeAndCreate)
///                     .permission(Permission::OWNER_ALL)
///                     .create()
///                     .expect("failed to create semaphore");
///
/// loop {
///     nanosleep(Duration::from_secs(1));
///     println!("trigger process 2");
///     semaphore.post().expect("failed to trigger semaphore");
/// }
/// ```
///
/// ## In process 2
/// ```no_run
/// use iceoryx2_bb_posix::semaphore::*;
/// use iceoryx2_bb_system_types::file_name::FileName;
/// use iceoryx2_bb_container::semantic_string::*;
///
/// let name = FileName::new(b"mySemaphoreName").unwrap();
/// let semaphore = NamedSemaphoreBuilder::new(&name)
///                     .open_existing()
///                     .expect("failed to open semaphore");
///
/// loop {
///     semaphore.blocking_wait().expect("failed to wait on semaphore");
///     println!("process 1 has triggered me");
/// }
/// ```
///
/// ## Output
///
/// When both processes are running in two separate terminals one can observe that process 1 triggers
/// process 2 every second.
#[derive(Debug)]
pub struct NamedSemaphore {
    name: FileName,
    handle: *mut posix::sem_t,
    has_ownership: bool,
    clock_type: ClockType,
}

unsafe impl Send for NamedSemaphore {}
unsafe impl Sync for NamedSemaphore {}

impl Drop for NamedSemaphore {
    fn drop(&mut self) {
        if core::ptr::eq(self.handle, posix::SEM_FAILED) {
            return;
        }

        if unsafe { posix::sem_close(self.handle) } != 0 {
            fatal_panic!(from self, "This should never happen! The semaphore handle is invalid and cannot be closed.");
        }

        if self.has_ownership
            && self
                .unlink(UnlinkMode::FailWhenSemaphoreDoesNotExist)
                .is_err()
        {
            fatal_panic!(from self, "Failed to cleanup semaphore. Something else removed a managed semaphore which should never happen!");
        }
    }
}

impl NamedSemaphore {
    fn new(config: NamedSemaphoreBuilder) -> Result<NamedSemaphore, NamedSemaphoreCreationError> {
        let mut new_sem = NamedSemaphore {
            name: config.name,
            handle: posix::SEM_FAILED,
            has_ownership: false,
            clock_type: config.clock_type,
        };

        match config.creation_mode {
            None => {
                new_sem.open(Permission::none(), InitMode::Open, 0)?;
            }
            Some(CreationMode::PurgeAndCreate) => {
                new_sem.has_ownership = true;
                fail!(from new_sem, when new_sem.unlink(UnlinkMode::IgnoreNonExistingSemaphore), "Failed to remove semaphore before creating a new one.");
                new_sem.open(config.permission, InitMode::Create, config.initial_value)?;
            }
            Some(CreationMode::CreateExclusive) => {
                new_sem.has_ownership = true;
                new_sem.open(config.permission, InitMode::Create, config.initial_value)?;
            }
            Some(CreationMode::OpenOrCreate) => {
                match new_sem.open(Permission::none(), InitMode::TryOpen, 0) {
                    Ok(()) => (),
                    Err(NamedSemaphoreCreationError::DoesNotExist) => {
                        new_sem.has_ownership = true;
                        new_sem.open(config.permission, InitMode::Create, config.initial_value)?;
                    }
                    Err(v) => return Err(v),
                }
            }
        };

        Ok(new_sem)
    }

    fn unlink(&mut self, mode: UnlinkMode) -> Result<(), NamedSemaphoreCreationError> {
        let file_path =
            FilePath::from_path_and_file(&Path::new(b"/").unwrap(), &self.name).unwrap();
        if unsafe { posix::sem_unlink(file_path.as_c_str()) } == 0 {
            debug!(from self, "semaphore removed.");
            return Ok(());
        }

        let msg = "Unable to unlink semaphore";
        let ignore_non_existing_semaphore = mode == UnlinkMode::IgnoreNonExistingSemaphore;
        handle_errno!(NamedSemaphoreCreationError, from self,
            success_when ignore_non_existing_semaphore,
                Errno::ENOENT => ((), AlreadyExists, "{} since no semaphore with the given name exists.", msg),
            Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            Errno::ENAMETOOLONG => (MaxFilePathLengthExceeded, "{} since the name exceeds the maximum supported length.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        )
    }

    fn open(
        &mut self,
        permission: Permission,
        mode: InitMode,
        initial_value: u32,
    ) -> Result<(), NamedSemaphoreCreationError> {
        let msg;
        if initial_value > MAX_INITIAL_SEMAPHORE_VALUE {
            fail!(from self, with NamedSemaphoreCreationError::InitialValueTooLarge,
                "Unable to create semaphore since the initial semaphore value {} is greater than the maximum supported value of {}.", initial_value, MAX_INITIAL_SEMAPHORE_VALUE);
        }

        let file_path =
            FilePath::from_path_and_file(&Path::new(b"/").unwrap(), &self.name).unwrap();
        Errno::reset();
        self.handle = match mode {
            InitMode::Create => unsafe {
                msg = "Unable to create semaphore";
                posix::sem_create(
                    file_path.as_c_str(),
                    posix::O_CREAT | posix::O_EXCL,
                    permission.as_mode(),
                    initial_value,
                )
            },
            InitMode::Open | InitMode::TryOpen => unsafe {
                msg = "Unable to open semaphore";
                posix::sem_open(file_path.as_c_str(), 0)
            },
        };

        if !core::ptr::eq(self.handle, posix::SEM_FAILED) {
            match mode {
                InitMode::Create => debug!(from self, "semaphore created."),
                _ => debug!(from self, "semaphore opened."),
            }
            return Ok(());
        }

        let has_try_open_mode = mode == InitMode::TryOpen;
        handle_errno!(NamedSemaphoreCreationError, from self,
            success_when has_try_open_mode,
                Errno::ENOENT => ((), DoesNotExist, "{} since the semaphore does not exist." ,msg),
            Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            Errno::EEXIST => (AlreadyExists, "{} since the semaphore already exists.", msg),
            Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
            Errno::EINVAL => (NotSupportForGivenName, "{} since the operation is not supported for the given name.", msg),
            Errno::EMFILE => (PerProcessFileHandleLimitReached, "{} since the current process already holds the maximum amount of semaphore or file descriptos.", msg),
            Errno::ENAMETOOLONG => (MaxFilePathLengthExceeded, "{} since the name exceeds the maximum supported length.", msg),
            Errno::ENFILE => (SystemWideFileHandleLimitReached, "{} since the system-wide semaphore or file-handle limit is reached.", msg),
            Errno::ENOSPC => (NoSpaceLeft, "{} due to insufficient space on the target.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg,v)
        );
    }

    /// Returns the name of the named semaphore
    pub fn name(&self) -> &FileName {
        &self.name
    }
}

impl internal::SemaphoreHandle for NamedSemaphore {
    fn handle(&self) -> *mut posix::sem_t {
        self.handle
    }

    fn get_clock_type(&self) -> ClockType {
        self.clock_type
    }
}

impl SemaphoreInterface for NamedSemaphore {}

/// Creates an [`UnnamedSemaphore`] which can be either used process locally or can be stored in a
/// shared memory segment and then used during inter-process communication.
///
/// # Example
///
/// ```
/// use iceoryx2_bb_posix::semaphore::*;
///
/// let semaphore_handle = UnnamedSemaphoreHandle::new();
/// let semaphore = UnnamedSemaphoreBuilder::new().initial_value(5)
///                                               .is_interprocess_capable(false)
///                                               .clock_type(ClockType::Monotonic)
///                                               .create(&semaphore_handle)
///                                               .expect("failed to create unnamed semaphore");
/// ```
#[derive(Debug)]
pub struct UnnamedSemaphoreBuilder {
    clock_type: ClockType,
    is_interprocess_capable: bool,
    initial_value: u32,
}

impl Default for UnnamedSemaphoreBuilder {
    fn default() -> Self {
        UnnamedSemaphoreBuilder {
            clock_type: ClockType::default(),
            is_interprocess_capable: true,
            initial_value: 0,
        }
    }
}

impl UnnamedSemaphoreBuilder {
    pub fn new() -> UnnamedSemaphoreBuilder {
        Self::default()
    }

    /// Sets the initial value of the semaphore. Must be less than [`MAX_INITIAL_SEMAPHORE_VALUE`].
    pub fn initial_value(mut self, value: u32) -> Self {
        self.initial_value = value;
        self
    }

    /// Defines if the [`UnnamedSemaphore`] can be used in an inter-process communication context.
    pub fn is_interprocess_capable(mut self, value: bool) -> Self {
        self.is_interprocess_capable = value;
        self
    }

    /// Sets the type of clock which will be used in [`SemaphoreInterface::timed_wait()`]. Be
    /// aware a clock like [`ClockType::Realtime`] is depending on the systems local time. If this
    /// time changes while waiting it can cause extrem long waits or no wait at all.
    pub fn clock_type(mut self, value: ClockType) -> Self {
        self.clock_type = value;
        self
    }

    fn initialize_semaphore(
        &self,
        sem: *mut posix::sem_t,
    ) -> Result<Capability, UnnamedSemaphoreCreationError> {
        let msg = "Unable to create semaphore";

        if self.initial_value > MAX_INITIAL_SEMAPHORE_VALUE {
            fail!(from self, with UnnamedSemaphoreCreationError::InitialValueTooLarge,
                "{} since the initial value {} is too large.", msg, self.initial_value);
        }

        if unsafe {
            posix::sem_init(
                sem,
                if self.is_interprocess_capable { 1 } else { 0 },
                self.initial_value,
            )
        } == -1
        {
            handle_errno!(UnnamedSemaphoreCreationError, from self,
                Errno::EINVAL => (InitialValueTooLarge, "{} since the initial value {} is too large. Please verify posix configuration!", msg, self.initial_value),
                Errno::ENOSPC => (ExceedsMaximumNumberOfSemaphores, "{} since it exceeds the maximum amount of semaphores {}.", msg, Limit::MaxNumberOfSemaphores.value()),
                Errno::EPERM => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            );
        }

        match self.is_interprocess_capable {
            true => Ok(Capability::InterProcess),
            false => Ok(Capability::ProcessLocal),
        }
    }

    /// Creates an [`UnnamedSemaphore`].
    pub fn create(
        self,
        handle: &UnnamedSemaphoreHandle,
    ) -> Result<UnnamedSemaphore<'_>, UnnamedSemaphoreCreationError> {
        unsafe {
            handle
                .handle
                .initialize(|sem| self.initialize_semaphore(sem))?;
        }

        unsafe { *handle.clock_type.get() = self.clock_type };

        Ok(UnnamedSemaphore::new(handle))
    }
}

#[derive(Debug)]
pub struct UnnamedSemaphoreHandle {
    handle: HandleStorage<posix::sem_t>,
    clock_type: UnsafeCell<ClockType>,
}

unsafe impl Send for UnnamedSemaphoreHandle {}
unsafe impl Sync for UnnamedSemaphoreHandle {}

impl Handle for UnnamedSemaphoreHandle {
    fn new() -> Self {
        Self {
            handle: HandleStorage::new(posix::sem_t::new_zeroed()),
            clock_type: UnsafeCell::new(ClockType::default()),
        }
    }

    fn is_inter_process_capable(&self) -> bool {
        self.handle.is_inter_process_capable()
    }

    fn is_initialized(&self) -> bool {
        self.handle.is_initialized()
    }
}

impl Drop for UnnamedSemaphoreHandle {
    fn drop(&mut self) {
        if self.handle.is_initialized() {
            unsafe {
                self.handle.cleanup(|sem| {
                    if posix::sem_destroy(sem) != 0 {
                        warn!(from self,
                            "Unable to destroy unnamed semaphore. Was it already destroyed by another instance in another process?");
                    }
                });
            };
        }
    }
}

/// An unnamed semaphore which can be used process locally or for inter-process triggers.
///
/// # Example
///
/// ```no_run
/// use iceoryx2_bb_posix::semaphore::*;
/// use std::thread;
/// use iceoryx2_bb_posix::clock::*;
/// use core::time::Duration;
///
/// let semaphore_handle = UnnamedSemaphoreHandle::new();
/// let semaphore = UnnamedSemaphoreBuilder::new().create(&semaphore_handle)
///     .expect("failed to create semaphore");
///
/// thread::scope(|s| {
///     s.spawn(|| {
///         loop {
///             semaphore.blocking_wait().expect("failed to wait on semaphore");
///             println!("the thread was triggered");
///         }
///     });
///
///     loop {
///         nanosleep(Duration::from_secs(1));
///         println!("trigger thread");
///         semaphore.post().expect("failed to trigger semaphore");
///     }
/// });
/// ```
#[derive(Debug)]
pub struct UnnamedSemaphore<'a> {
    handle: &'a UnnamedSemaphoreHandle,
}

unsafe impl Send for UnnamedSemaphore<'_> {}
unsafe impl Sync for UnnamedSemaphore<'_> {}

impl<'a> IpcConstructible<'a, UnnamedSemaphoreHandle> for UnnamedSemaphore<'a> {
    fn new(handle: &'a UnnamedSemaphoreHandle) -> Self {
        Self { handle }
    }
}

impl<'a> IpcCapable<'a, UnnamedSemaphoreHandle> for UnnamedSemaphore<'a> {
    fn is_interprocess_capable(&self) -> bool {
        self.handle.is_inter_process_capable()
    }
}

impl internal::SemaphoreHandle for UnnamedSemaphore<'_> {
    fn handle(&self) -> *mut posix::sem_t {
        unsafe { self.handle.handle.get() }
    }

    fn get_clock_type(&self) -> ClockType {
        unsafe { *self.handle.clock_type.get() }
    }
}

impl SemaphoreInterface for UnnamedSemaphore<'_> {}
