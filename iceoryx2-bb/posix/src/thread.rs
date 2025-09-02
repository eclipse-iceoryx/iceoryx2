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

//! An abstraction of a POSIX [`Thread`] with a rich feature set.
//!
//! # Examples
//!
//! ## Create a simple thread
//!
//! ```
//! use iceoryx2_bb_posix::thread::*;
//!
//! fn some_func() {}
//!
//! let thread = ThreadBuilder::new()
//!                          .spawn(some_func)
//!                          .expect("Failed to create thread");
//!
//! println!("The thread {:?} was created.", thread);
//! ```
//!
//! ## Create a thread with user provided stack memory
//!
//! ```ignore
//! use iceoryx2_bb_posix::thread::*;
//!
//! fn some_func() {}
//! // the threads stack will reside in here
//! let mut memory: [u8; 1024 * 1024] = [0; 1024 * 1024];
//!
//! let thread = ThreadBuilder::new()
//!                          .name(&ThreadName::from(b"stackThread"))
//!                          .stack(memory.as_mut_slice())
//!                          .spawn(some_func)
//!                          .expect("Failed to create thread");
//!
//! println!("The thread {:?} was created.", thread);
//! ```
//!
//! ## Create a highly customized thread with guarded stack
//!
//! ```ignore
//! use iceoryx2_bb_posix::thread::*;
//! use iceoryx2_bb_posix::scheduler::*;
//! use iceoryx2_bb_posix::system_configuration::*;
//!
//! fn some_func() {}
//!
//! // when creating highly specialized threads check the system parameters first
//! // check how many cpu cores we have available to set the CPU affinity correctly. The cores are
//! // enumerated from 0..number_of_cpu_cores-1
//! let number_of_cpu_cores = SystemInfo::NumberOfCpuCores.value();
//! // the stack size must have at least this size otherwise we are unable to create a thread
//! let minimum_stack_size = Limit::MinStackSizeOfThread.value();
//!
//! let thread = ThreadBuilder::new()
//!                         .name(&ThreadName::from(b"myFunkyThread"))
//!                         // try to let the thread run on CPU core 0 and 4, must be less than
//!                         // number_of_cpu_cores
//!                         .affinity([0, 4])
//!                         .priority(255) // it is important, run on highest priority
//!                         .scheduler(Scheduler::Fifo)
//!                         // from here one we create a guarded based thread
//!                         .contention_scope(ContentionScope::System)
//!                         // uses the last 1024 bytes of the system for a guard to detect memory
//!                         // bugs
//!                         .guard_size(1024)
//!                         // 1mb stack size, must be greater than minimum_stack_size
//!                         .stack_size(1024 * 1024)
//!                         .spawn(some_func)
//!                         .expect("Failed to create thread");
//!
//! println!("The thread {:?} was created.", thread);
//! ```

use core::{cell::UnsafeCell, fmt::Debug, marker::PhantomData};

use crate::handle_errno;
use iceoryx2_bb_container::byte_string::FixedSizeByteString;
use iceoryx2_bb_elementary::{enum_gen, scope_guard::ScopeGuardBuilder};
use iceoryx2_bb_log::{fail, fatal_panic, warn};
use iceoryx2_pal_posix::posix::CPU_SETSIZE;
use iceoryx2_pal_posix::posix::{errno::Errno, MemZeroedStruct};
use iceoryx2_pal_posix::*;

use crate::{
    config::MAX_THREAD_NAME_LENGTH,
    scheduler::Scheduler,
    signal::Signal,
    system_configuration::{Limit, SystemInfo},
};

/// Stores the name of a thread
pub type ThreadName = FixedSizeByteString<15>;

enum_gen! { ThreadSpawnError
  entry:
    InsufficientMemory,
    InsufficientResources,
    InvalidSettings,
    InsufficientPermissions,
    InvalidGuardSize,
    ContentionScopeNotSupported,
    SchedulerPolicyNotSupported,
    StackSizeTooSmall,
    ProvidedStackSizeMemoryTooSmall,
    ProvidedStackMemoryIsNotReadAndWritable,
    SchedulerPriorityInheritanceNotSupported,
    ThreadPrioritiesNotSupported,
    CpuCoreOutsideOfSupportedCpuRangeForAffinity,
    UnknownError(i32)
  mapping:
    ThreadSetNameError
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum ThreadSignalError {
    ThreadNoLongerActive,
    UnknownError(i32),
}

enum_gen! { ThreadSetNameError
  entry:
    UnknownError(i32)
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum ThreadSetAffinityError {
    InvalidCpuCores,
    UnknownError(i32),
}

enum_gen! {
    ThreadGetNameError
  entry:
    ThreadNameLongerThanMaxSupportedSize,
    UnknownError(i32)
}

enum_gen! {
    /// The ThreadError enum is a generalization when one doesn't require the fine-grained error
    /// handling enums. One can forward ThreadError as more generic return value when a method
    /// returns a Thread***Error.
    /// On a higher level it is again convertable to [`crate::Error`].
    ThreadError
  generalization:
    FailedToSpawn <= ThreadSpawnError,
    NameSetupFailed <= ThreadSetNameError; ThreadGetNameError,
    FailedToSignal <= ThreadSignalError,
    FailedToSetAffinity <= ThreadSetAffinityError
}

/// The builder for a [`Thread`] object.
#[derive(Debug)]
pub struct ThreadBuilder {
    guard_size: u64,
    inherit_scheduling_attributes: bool,
    scheduler: Scheduler,
    priority: u8,
    stack_size: Option<u64>,
    affinity: [bool; posix::CPU_SETSIZE],
    has_custom_affinity: bool,
    has_invalid_affinity: bool,
    name: ThreadName,
}

impl Default for ThreadBuilder {
    fn default() -> Self {
        Self {
            guard_size: 0,
            inherit_scheduling_attributes: true,
            priority: 0,
            scheduler: Scheduler::default(),
            affinity: [true; posix::CPU_SETSIZE],
            has_custom_affinity: false,
            has_invalid_affinity: false,
            stack_size: None,
            name: ThreadName::new(),
        }
    }
}

impl ThreadBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the name of the thread. It is not allowed to be longer than
    /// [`crate::config::MAX_THREAD_NAME_LENGTH`] and must consist of ASCII characters only.
    pub fn name(mut self, value: &ThreadName) -> Self {
        self.name = *value;
        self
    }

    /// Inherit the scheduling attributes of the calling thread.
    pub fn inherit_scheduling_attributes(mut self, value: bool) -> Self {
        self.inherit_scheduling_attributes = value;
        self
    }

    /// Sets the threads CPU affinity to the provided list of `cpu_core_id`s.
    /// The cpu cores must exist otherwise [`ThreadBuilder::spawn()`] will
    /// fail with [`ThreadSpawnError::CpuCoreOutsideOfSupportedCpuRangeForAffinity`].
    ///
    /// The systems number of CPU cores can be acquired with:
    /// ```
    /// use iceoryx2_bb_posix::system_configuration::*;
    ///
    /// let number_of_cores = SystemInfo::NumberOfCpuCores.value();
    /// ```
    pub fn affinity(mut self, cpu_core_ids: &[usize]) -> Self {
        self.affinity = [false; posix::CPU_SETSIZE];

        for cpu_core_id in cpu_core_ids {
            if *cpu_core_id >= posix::CPU_SETSIZE {
                self.has_invalid_affinity = true;
                return self;
            }

            self.affinity[*cpu_core_id] = true;
        }

        self.has_custom_affinity = true;
        self
    }

    /// Sets the priority of the thread whereby `0` represents the lowest and `255` the highest
    /// priority. Since the underlying scheduler priority varies in range the values are mapped
    /// to the scheduler dependent priority.
    /// For more details about scheduler priority granularity see:
    /// [`Scheduler::priority_granularity()`]
    pub fn priority(mut self, value: u8) -> Self {
        self.priority = value;
        self
    }

    /// Sets the [`Scheduler`] used by the thread.
    pub fn scheduler(mut self, value: Scheduler) -> Self {
        self.scheduler = value;
        self
    }

    /// Defines the size of the memory block which is put at the end of the stack memory to
    /// discover memory related bugs.
    ///
    /// With defining this value a thread with guarded stack is created. See
    /// [`ThreadGuardedStackBuilder`].
    pub fn guard_size(mut self, value: u64) -> ThreadGuardedStackBuilder {
        self.guard_size = value;
        ThreadGuardedStackBuilder { config: self }
    }

    /// Defines the stack size of the thread. It must be greater or equal the minimum thread stack
    /// size required by the system. One can acquire the minimum required thread stack size with:
    /// ```
    /// use iceoryx2_bb_posix::system_configuration::*;
    ///
    /// let min_stack_size = Limit::MinStackSizeOfThread.value();
    /// ```
    ///
    /// With defining this value a thread with guarded stack is created. See
    /// [`ThreadGuardedStackBuilder`].
    pub fn stack_size(mut self, value: u64) -> ThreadGuardedStackBuilder {
        self.stack_size = Some(value);
        ThreadGuardedStackBuilder { config: self }
    }

    pub fn spawn<'thread, T, F>(self, f: F) -> Result<Thread, ThreadSpawnError>
    where
        T: Debug + Send + 'thread,
        F: FnOnce() -> T + Send + 'thread,
    {
        self.spawn_impl(f)
    }

    /// Creates a new thread with the provided callable `f`.
    fn spawn_impl<'thread, T, F>(self, f: F) -> Result<Thread, ThreadSpawnError>
    where
        T: Debug + Send + 'thread,
        F: FnOnce() -> T + Send + 'thread,
    {
        if self.has_invalid_affinity {
            fail!(from self, with ThreadSpawnError::CpuCoreOutsideOfSupportedCpuRangeForAffinity,
                "Unable to set the threads cpu affinity since the provided core value exceeds the cpu affinity set size of {}.",
                CPU_SETSIZE);
        }

        if self.has_custom_affinity {
            let number_of_cores = SystemInfo::NumberOfCpuCores.value();
            for (cpu_core_id, has_affinity) in self.affinity[number_of_cores..].iter().enumerate() {
                if *has_affinity {
                    fail!(from self,
                        with ThreadSpawnError::CpuCoreOutsideOfSupportedCpuRangeForAffinity,
                        "Unable to set the threads affinity since the system has cores from [0, {}] and the cpu core {} was set.",
                    number_of_cores - 1, cpu_core_id);
                }
            }
        }

        let mut attributes = ScopeGuardBuilder::new( posix::pthread_attr_t::new_zeroed())
            .on_init(|attr| {
                let msg = "Failed to initialize thread attributes";
                handle_errno!(ThreadSpawnError, from self,
                    errno_source unsafe {posix::pthread_attr_init(attr).into()},
                    success Errno::ESUCCES => (),
                    Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
                    v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
                );
            }).on_drop(|attr| match unsafe { posix::pthread_attr_destroy(attr)} {
                0 => (),
                v => {
                    fatal_panic!(from self, "This should never happen! Failed to cleanup thread attributes ({}).",v);
                }
            }).create()?;

        let msg = "Failed to set guard size";
        handle_errno!(ThreadSpawnError, from self,
            errno_source unsafe { posix::pthread_attr_setguardsize(attributes.get_mut(), self.guard_size as usize).into()},
            continue_on_success,
            success Errno::ESUCCES => (),
            Errno::EINVAL => (InvalidGuardSize, "{} since the guard size value is invalid.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );

        let msg = "Failed to set scheduler priority inheritance settings";
        handle_errno!(ThreadSpawnError, from self,
            errno_source unsafe { posix::pthread_attr_setinheritsched(attributes.get_mut(), if self.inherit_scheduling_attributes {
                                                posix::PTHREAD_INHERIT_SCHED
                                            } else {
                                                posix::PTHREAD_EXPLICIT_SCHED
                                            }).into()},
            continue_on_success,
            success Errno::ESUCCES => (),
            Errno::ENOSYS => (SchedulerPriorityInheritanceNotSupported, "{} since it is not supported by the system.", msg),
            Errno::ENOTSUP => (SchedulerPriorityInheritanceNotSupported, "{} since it is not supported by the system.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg,v )
        );

        let msg = "Failed to set scheduler policy";
        handle_errno!(ThreadSpawnError, from self,
            errno_source unsafe { posix::pthread_attr_setschedpolicy(attributes.get_mut(), self.scheduler as i32).into()},
            continue_on_success,
            success Errno::ESUCCES => (),
            Errno::ENOSYS => (SchedulerPolicyNotSupported, "{} since it is not supported by the system.", msg),
            Errno::ENOTSUP => (SchedulerPolicyNotSupported, "{} since it is not supported by the system.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );

        let mut param = posix::sched_param::new_zeroed();
        param.sched_priority = self.scheduler.policy_specific_priority(self.priority);
        let msg = "Failed to set thread priority";
        handle_errno!(ThreadSpawnError, from self,
            errno_source unsafe { posix::pthread_attr_setschedparam(attributes.get_mut(), &param).into() },
            continue_on_success,
            success Errno::ESUCCES => (),
            Errno::ENOTSUP => (ThreadPrioritiesNotSupported, "{} since it is not supported by the system.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg,v)
        );

        if self.stack_size.is_some() {
            let msg = "Failed to set the threads stack size";
            let stack_size = *self.stack_size.as_ref().unwrap();
            let min_stack_size = Limit::MinStackSizeOfThread.value();

            if stack_size < min_stack_size {
                fail!(from self, with ThreadSpawnError::StackSizeTooSmall,
                    "{} since the stack size is smaller than the minimum required stack size of {}.", msg, min_stack_size);
            }

            handle_errno!(ThreadSpawnError, from self,
                errno_source unsafe { posix::pthread_attr_setstacksize(attributes.get_mut(), *self.stack_size.as_ref().unwrap() as usize)
                                                .into() },
                continue_on_success,
                success Errno::ESUCCES => (),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg,v)
            );
        }

        let mut cpuset = posix::cpu_set_t::new_zeroed();
        for i in 0..core::cmp::min(posix::CPU_SETSIZE, SystemInfo::NumberOfCpuCores.value()) {
            if self.affinity[i] {
                cpuset.set(i);
            }
        }

        if posix::support::POSIX_SUPPORT_CPU_AFFINITY {
            let msg = "Unable to set cpu affinity for thread";
            handle_errno!(ThreadSpawnError, from self,
                errno_source unsafe { posix::pthread_attr_setaffinity_np(attributes.get_mut(), core::mem::size_of::<posix::cpu_set_t>(), &cpuset)
                                                .into()},
                continue_on_success,
                success Errno::ESUCCES => (),
                Errno::EINVAL => (CpuCoreOutsideOfSupportedCpuRangeForAffinity, "{} since it contains cores greater than the maximum supported number of CPU cores of the system.", msg),
                Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg,v)
            );
        }

        extern "C" fn start_routine<'thread, FF, TT>(args: *mut posix::void) -> *mut posix::void
        where
            TT: Send + Debug + 'thread,
            FF: FnOnce() -> TT + Send + 'thread,
        {
            let t: ThreadStartupArgs<TT, FF> =
                unsafe { core::ptr::read(args as *const ThreadStartupArgs<TT, FF>) };

            if !t.name.is_empty() {
                let handle = unsafe { posix::pthread_self() };
                if unsafe { posix::pthread_setname_np(handle, t.name.as_c_str()) }
                    != Errno::ESUCCES as i32
                {
                    warn!(from "Thread::spawn", "Unable to set the name of the newly spawned thread to \"{}\".", t.name);
                }
            }

            (t.callback)();
            unsafe { posix::free(args) };
            core::ptr::null_mut::<posix::void>()
        }

        let startup_args = unsafe {
            posix::malloc(core::mem::size_of::<ThreadStartupArgs<T, F>>())
                as *mut ThreadStartupArgs<T, F>
        };

        unsafe {
            startup_args.write(ThreadStartupArgs {
                callback: f,
                name: self.name,
                _data: PhantomData,
            });
        }

        let mut handle = posix::pthread_t::new_zeroed();

        let msg = "Unable to create thread";
        match unsafe {
            posix::pthread_create(
                &mut handle,
                attributes.get(),
                start_routine::<F, T>,
                startup_args as *mut posix::void,
            )
            .into()
        } {
            Errno::ESUCCES => (),
            Errno::EAGAIN => {
                fail!(from self, with ThreadSpawnError::InsufficientResources,
                    "{} due to insufficient resources. Maybe the system limit of threads is reached.", msg);
            }
            Errno::EINVAL => {
                fail!(from self, with ThreadSpawnError::InvalidSettings,
                    "{} due to invalid settings for the thread.", msg);
            }
            Errno::EPERM => {
                fail!(from self, with ThreadSpawnError::InsufficientPermissions,
                    "{} due to insufficient permissions to set scheduling settings.", msg);
            }
            v => {
                fail!(from self, with ThreadSpawnError::UnknownError(v as i32),
                    "{} since an unknown error occurred ({}).", msg,v);
            }
        };

        Ok(Thread::new(ThreadHandle {
            handle,
            name: UnsafeCell::new(self.name),
        }))
    }
}

/// Creates a thread with a custom sized and guarded stack. This means that a region at the end of
/// the threads stack is defined which is later used to discover possible memory issues.
///
/// Is being created when calling [stack_size](ThreadBuilder::stack_size()) or
/// [guard_size](ThreadBuilder::guard_size)
/// inside the [`ThreadBuilder`].
///
/// For an example take a look at [`ThreadBuilder`]s last example.
pub struct ThreadGuardedStackBuilder {
    config: ThreadBuilder,
}

impl ThreadGuardedStackBuilder {
    /// See: [`ThreadBuilder::guard_size()`]
    pub fn guard_size(mut self, value: u64) -> Self {
        self.config.guard_size = value;
        self
    }

    /// See: [`ThreadBuilder::stack_size()`]
    pub fn stack_size(mut self, value: u64) -> Self {
        self.config.stack_size = Some(value);
        self
    }

    /// See: [`ThreadBuilder::spawn()`]
    pub fn spawn<'thread, T, F>(self, f: F) -> Result<Thread, ThreadSpawnError>
    where
        T: Debug + Send + 'thread,
        F: FnOnce() -> T + Send + 'thread,
    {
        self.config.spawn_impl(f)
    }
}

pub trait ThreadProperties {
    /// Returns the name of the thread.
    fn get_name(&self) -> Result<&ThreadName, ThreadGetNameError>;

    /// Returns a vector of numbers which represents the CPU cores in which the
    /// thread may run.
    fn get_affinity(&self) -> Result<Vec<usize>, ThreadSetAffinityError>;

    /// Sets the threads affinity to the provided set of cpu core ids. If one of
    /// the cpu core id's does not exist in the system the call will fail.
    fn set_affinity(&mut self, cpu_core_ids: &[usize]) -> Result<(), ThreadSetAffinityError>;
}

/// A thread handle can be used from within the thread to read or modify certain settings like the
/// name or the cpu affinity.
///
/// # Example
///
/// ```ignore
/// use iceoryx2_bb_posix::thread::*;
///
/// let mut handle = ThreadHandle::from_self();
/// println!("I am running in a thread named {}", handle.get_name().expect(""));
/// println!("On the following CPU cores {:?}", handle.get_affinity().expect(""));
///
/// // only run on CPU 0
/// handle.set_affinity(0);
/// ```
#[derive(Debug)]
pub struct ThreadHandle {
    handle: posix::pthread_t,
    name: UnsafeCell<ThreadName>,
}

impl ThreadHandle {
    /// Returns a handle to the callers thread.
    pub fn from_self() -> ThreadHandle {
        ThreadHandle {
            handle: unsafe { posix::pthread_self() },
            name: UnsafeCell::new(ThreadName::new()),
        }
    }
}

impl ThreadProperties for ThreadHandle {
    fn get_name(&self) -> Result<&ThreadName, ThreadGetNameError> {
        if !unsafe { self.name.get().as_ref().unwrap() }.is_empty() {
            return Ok(unsafe { self.name.get().as_ref().unwrap() });
        }

        let mut name: [posix::c_char; MAX_THREAD_NAME_LENGTH] = [0; MAX_THREAD_NAME_LENGTH];

        let msg = "Unable to acquire thread name";
        match unsafe {
            posix::pthread_getname_np(self.handle, name.as_mut_ptr(), MAX_THREAD_NAME_LENGTH)
        }
        .into()
        {
            Errno::ESUCCES => {
                unsafe {
                    let raw_string = fail!(from self, when ThreadName::from_c_str(name.as_mut_ptr()),
                        with ThreadGetNameError::ThreadNameLongerThanMaxSupportedSize,
                        "{} since it require more characters than the maximum supported length of {}.",
                        msg, ThreadName::capacity());

                    *self.name.get() = raw_string
                };
                Ok(unsafe { self.name.get().as_ref().unwrap() })
            }
            Errno::ERANGE => {
                fatal_panic!(from self, "{} since the provided buffer is too small. Increase MAX_THREAD_NAME_LENGTH for this platform.", msg);
            }
            v => {
                fail!(from self, with ThreadGetNameError::UnknownError(v as i32),
                    "{} since an unknown error has occurred ({}).", msg, v);
            }
        }
    }

    fn get_affinity(&self) -> Result<Vec<usize>, ThreadSetAffinityError> {
        let mut cpuset = posix::cpu_set_t::new_zeroed();
        let msg = "Unable to acquire threads CPU affinity";
        handle_errno!(ThreadSetAffinityError, from self,
            errno_source unsafe { posix::pthread_getaffinity_np(self.handle, core::mem::size_of::<posix::cpu_set_t>(), &mut cpuset).into()},
            continue_on_success,
            success Errno::ESUCCES => (),
            Errno::EINVAL => (InvalidCpuCores, "{} since some cpu cores were invalid (maybe exceeded maximum supported CPU core number of the system).", msg ),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );

        let mut cpu_affinity_set = vec![];
        for i in 0..posix::CPU_SETSIZE {
            if cpuset.has(i) {
                cpu_affinity_set.push(i);
            }
        }

        Ok(cpu_affinity_set)
    }

    fn set_affinity(&mut self, cpu_core_ids: &[usize]) -> Result<(), ThreadSetAffinityError> {
        let msg = "Unable to set cpu affinity to core";
        let number_of_cores = SystemInfo::NumberOfCpuCores.value();

        let mut cpuset = posix::cpu_set_t::new_zeroed();
        for cpu_core_id in cpu_core_ids {
            if *cpu_core_id >= posix::CPU_SETSIZE {
                fail!(from self, with ThreadSetAffinityError::InvalidCpuCores,
                    "{} {} since it exceeds the capacity of the thread affinity mask of {}.",
                    msg, cpu_core_id, posix::CPU_SETSIZE);
            }

            if *cpu_core_id > number_of_cores {
                fail!(from self, with ThreadSetAffinityError::InvalidCpuCores,
                    "{} {} since the maximum range of CPUs in the system is [0, {}].",
                    msg, cpu_core_id, number_of_cores - 1);
            }

            cpuset.set(*cpu_core_id);
        }

        let msg = "Unable to set cpu affinity";
        handle_errno!(ThreadSetAffinityError, from self,
            errno_source unsafe { posix::pthread_setaffinity_np(self.handle, core::mem::size_of::<posix::cpu_set_t>(), &cpuset).into() },
            success Errno::ESUCCES => (),
            Errno::EINVAL => (InvalidCpuCores, "{} since some cpu cores were invalid (maybe exceeded maximum supported CPU core number of the system).", msg),
            v=> (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }
}
struct ThreadStartupArgs<'thread, T: Send + Debug + 'thread, F: FnOnce() -> T + Send + 'thread> {
    callback: F,
    name: ThreadName,
    _data: PhantomData<&'thread ()>,
}

/// A POSIX thread which can be build with the [`ThreadBuilder`].
///
/// # Examples
///
/// See the [`ThreadBuilder`] for advanced construction examples.
///
/// ```
/// use iceoryx2_bb_posix::thread::*;
/// use core::sync::atomic::{AtomicBool, Ordering};
///
/// static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);
///
/// fn some_func() {
///     let handle = ThreadHandle::from_self();
///     println!("Hello from: {:?}", handle);
///
///     while KEEP_RUNNING.load(Ordering::Relaxed) {}
/// }
///
/// let thread = ThreadBuilder::new()
///                          .name(&ThreadName::from(b"some-name"))
///                          .spawn(some_func)
///                          .expect("Failed to create thread");
///
/// println!("Created thread: {:?}", thread);
/// KEEP_RUNNING.store(false, Ordering::Relaxed);
/// ```
pub struct Thread {
    handle: ThreadHandle,
}

impl Debug for Thread {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Thread {{ handle: {:?} }}", self.handle)
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        let msg = "Unable to join thread";
        match unsafe {
            posix::pthread_join(
                self.handle.handle,
                core::ptr::null_mut::<*mut posix::void>(),
            )
            .into()
        } {
            Errno::ESUCCES => (),
            Errno::EDEADLK => {
                fatal_panic!(from self, "{} since a deadlock was detected.", msg);
            }
            Errno::EINVAL => {
                fatal_panic!(from self, "{} since someone else is already trying to join this thread.", msg);
            }
            Errno::ESRCH => {
                fatal_panic!(from self, "This should never happen! Unable to join thread since its handle is invalid.");
            }
            v => {
                fatal_panic!(from self, "{} since an unknown error occurred ({}).", msg, v);
            }
        }
    }
}

impl Thread {
    fn new(handle: ThreadHandle) -> Self {
        Self { handle }
    }

    /// Sends a [`Signal`] to the thread.
    pub fn send_signal(&mut self, signal: Signal) -> Result<(), ThreadSignalError> {
        let msg = "Unable to send signal";
        handle_errno!(ThreadSignalError, from self,
            errno_source unsafe { posix::pthread_kill(self.handle.handle, signal as i32)}.into(),
            success Errno::ESUCCES => (),
            Errno::ESRCH  => (ThreadNoLongerActive, "{} {:?} since the thread is no longer active.", msg, signal),
            v => (UnknownError(v as i32), "{} {:?} since an unknown error occurred ({})", msg, signal, v)
        );
    }
}

impl ThreadProperties for Thread {
    fn get_name(&self) -> Result<&ThreadName, ThreadGetNameError> {
        self.handle.get_name()
    }

    fn get_affinity(&self) -> Result<Vec<usize>, ThreadSetAffinityError> {
        self.handle.get_affinity()
    }

    fn set_affinity(&mut self, cpu_core_ids: &[usize]) -> Result<(), ThreadSetAffinityError> {
        self.handle.set_affinity(cpu_core_ids)
    }
}
