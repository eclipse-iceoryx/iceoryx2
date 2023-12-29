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
//!                         // try to let the thread run on CPU core 0, must be less than
//!                         // number_of_cpu_cores
//!                         .affinity(0)
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

use std::{cell::UnsafeCell, fmt::Debug};

use crate::handle_errno;
use iceoryx2_bb_container::byte_string::FixedSizeByteString;
use iceoryx2_bb_elementary::{enum_gen, scope_guard::ScopeGuardBuilder};
use iceoryx2_bb_log::{fail, fatal_panic, warn};
use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::posix::Struct;
use iceoryx2_pal_posix::*;

use crate::{
    config::{MAX_SUPPORTED_CPUS_IN_SYSTEM, MAX_THREAD_NAME_LENGTH},
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

/// Terminates the callers thread
pub fn thread_exit() {
    unsafe { posix::pthread_exit(std::ptr::null_mut::<posix::void>()) }
}

/// Describes the scope in which a thread is competing with other threads for CPU time.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum ContentionScope {
    /// The threads competes with all threads in the system
    System = posix::PTHREAD_SCOPE_SYSTEM,
    /// The thread competes only with threads from the parent process
    Process = posix::PTHREAD_SCOPE_PROCESS,
}

/// The builder for a [`Thread`] object.
#[derive(Debug)]
pub struct ThreadBuilder {
    guard_size: u64,
    inherit_scheduling_attributes: bool,
    scheduler: Scheduler,
    priority: u8,
    contention_scope: Option<ContentionScope>,
    stack_size: Option<u64>,
    affinity: [bool; posix::CPU_SETSIZE],
    name: ThreadName,
}

impl Default for ThreadBuilder {
    fn default() -> Self {
        Self {
            guard_size: 0,
            inherit_scheduling_attributes: true,
            priority: 0,
            scheduler: Scheduler::default(),
            contention_scope: None,
            affinity: [true; posix::CPU_SETSIZE],
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

    /// Sets the threads CPU affinity. The CPU core must exists otherwise it has no effect.
    /// The maximum number of supported of CPU cores is defined in
    /// [`crate::config::MAX_SUPPORTED_CPUS_IN_SYSTEM`] and the systems number of CPU cores can
    /// be acquired with:
    /// ```
    /// use iceoryx2_bb_posix::system_configuration::*;
    ///
    /// let number_of_cores = SystemInfo::NumberOfCpuCores.value();
    /// ```
    pub fn affinity(mut self, value: usize) -> Self {
        let number_of_cores = SystemInfo::NumberOfCpuCores.value();
        if value >= number_of_cores {
            warn!(from self, "The system has cpu cores in the range [0, {}]. Setting affinity to cpu core {} will have no effect.", number_of_cores - 1, value);
        }
        if value > MAX_SUPPORTED_CPUS_IN_SYSTEM {
            warn!(from self, "Maximum range of supported CPUs is [0, {}]. Unable to set affinity to cpu core {}.", number_of_cores - 1, value);
            return self;
        }

        self.affinity = [false; posix::CPU_SETSIZE];
        self.affinity[value] = true;
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

    /// Defines the scope in which the thread competes with other threads for CPU time.
    /// * [`ContentionScope::System`] the thread competes system wide
    /// * [`ContentionScope::Process`] the thread competes only with other threads from the same
    ///    process.
    ///
    /// With defining this value a thread with guarded stack is created. See
    /// [`ThreadGuardedStackBuilder`].
    pub fn contention_scope(mut self, value: ContentionScope) -> ThreadGuardedStackBuilder {
        self.contention_scope = Some(value);
        ThreadGuardedStackBuilder { config: self }
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

    /// Provides memory which must have at least a size of the minimum required thread stack size
    /// which is used as the threads stack.
    pub fn stack(self, value: &mut [u8]) -> ThreadCustomStackBuilder {
        ThreadCustomStackBuilder {
            config: self,
            stack: value,
        }
    }

    pub fn spawn<'a, T: Debug, F>(self, f: F) -> Result<Thread<'a>, ThreadSpawnError>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        self.spawn_impl(f, None)
    }

    /// Creates a new thread with the provided callable `f`.
    fn spawn_impl<'a, T: Debug, F>(
        self,
        f: F,
        stack: Option<&'a mut [u8]>,
    ) -> Result<Thread<'a>, ThreadSpawnError>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let mut attributes = ScopeGuardBuilder::new( posix::pthread_attr_t::new())
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

        if self.contention_scope.is_some() {
            let msg = "Failed to set contention scope";
            handle_errno!(ThreadSpawnError, from self,
                errno_source unsafe { posix::pthread_attr_setscope(attributes.get_mut(), self.contention_scope.unwrap() as i32).into() },
                continue_on_success,
                success Errno::ESUCCES => (),
                Errno::ENOSYS => (ContentionScopeNotSupported, "{} since it is not supported by the system.", msg),
                Errno::ENOTSUP => (ContentionScopeNotSupported, "{} since it is not supported by the system.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            );
        }

        let mut param = posix::sched_param::new();
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

        let mut cpuset = posix::cpu_set_t::new();
        for i in 0..std::cmp::min(posix::CPU_SETSIZE, SystemInfo::NumberOfCpuCores.value()) {
            if self.affinity[i] {
                cpuset.set(i);
            }
        }

        let msg = "Unable to set cpu affinity for thread";
        handle_errno!(ThreadSpawnError, from self,
            errno_source unsafe { posix::pthread_attr_setaffinity_np(attributes.get_mut(), std::mem::size_of::<posix::cpu_set_t>(), &cpuset)
                                            .into()},
            continue_on_success,
            success Errno::ESUCCES => (),
            Errno::EINVAL => (CpuCoreOutsideOfSupportedCpuRangeForAffinity, "{} since it contains cores greater than the maximum supported number of CPU cores of the system.", msg),
            Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg,v)
        );

        if stack.is_some() {
            let msg = "Failed to set threads stack";
            let stack_size = stack.as_ref().unwrap().len();
            let min_stack_size = Limit::MinStackSizeOfThread.value();

            if stack_size < min_stack_size as usize {
                fail!(from self, with ThreadSpawnError::ProvidedStackSizeMemoryTooSmall,
                        "{} since the size {} of the provided stack is smaller than the minimum required stack size of {}.",
                                    msg, stack_size, min_stack_size );
            }

            handle_errno!(ThreadSpawnError, from self,
               errno_source unsafe { posix::pthread_attr_setstack(attributes.get_mut(), stack.as_ref().unwrap().as_ref().as_ptr() as *mut posix::void, stack_size)
                                               .into()},
               continue_on_success,
               success Errno::ESUCCES => (),
               Errno::EACCES => (ProvidedStackMemoryIsNotReadAndWritable, "{} since the provided memory is not read- and writable by the thread.", msg),
               v => (UnknownError(v as i32), "{} since an unknown error has occurred ({}).", msg,v)
            );
        }

        extern "C" fn start_routine<FF, TT>(args: *mut posix::void) -> *mut posix::void
        where
            TT: Send + Debug + 'static,
            FF: FnOnce() -> TT + Send + 'static,
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
            std::ptr::null_mut::<posix::void>()
        }

        let startup_args = unsafe {
            posix::malloc(core::mem::size_of::<ThreadStartupArgs<T, F>>())
                as *mut ThreadStartupArgs<T, F>
        };

        unsafe {
            startup_args.write(ThreadStartupArgs {
                callback: f,
                name: self.name,
            });
        }

        let mut handle = posix::pthread_t::new();

        let msg = "Unable to create thread";
        match unsafe {
            posix::pthread_create(
                &mut handle,
                attributes.get(),
                Some(start_routine::<F, T>),
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

        Ok(Thread::<'a>::new(
            ThreadHandle {
                handle,
                name: UnsafeCell::new(self.name),
            },
            stack,
        ))
    }
}

/// Creates a thread with a custom sized and guarded stack. This means that a region at the end of
/// the threads stack is defined which is later used to discover possible memory issues.
///
/// Is being created when calling [stack_size](ThreadBuilder::stack_size()),
/// [guard_size](ThreadBuilder::guard_size) or [contention_scope](ThreadBuilder::contention_scope())
/// inside the [`ThreadBuilder`].
///
/// For an example take a look at [`ThreadBuilder`]s last example.
pub struct ThreadGuardedStackBuilder {
    config: ThreadBuilder,
}

impl ThreadGuardedStackBuilder {
    /// See: [`ThreadBuilder::contention_scope()`]
    pub fn contention_scope(mut self, value: ContentionScope) -> Self {
        self.config.contention_scope = Some(value);
        self
    }

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
    pub fn spawn<'a, T: Debug, F>(self, f: F) -> Result<Thread<'a>, ThreadSpawnError>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        self.config.spawn_impl(f, None)
    }
}

/// Creates a thread with a user provided stack. For an example take a look at the second example
/// in [`ThreadBuilder`]
#[derive(Debug)]
pub struct ThreadCustomStackBuilder<'a> {
    config: ThreadBuilder,
    stack: &'a mut [u8],
}

impl<'a> ThreadCustomStackBuilder<'a> {
    /// See: [`ThreadBuilder::spawn()`]
    pub fn spawn<T: Debug, F>(self, f: F) -> Result<Thread<'a>, ThreadSpawnError>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        self.config.spawn_impl(f, Some(self.stack))
    }
}

pub trait ThreadProperties {
    /// Returns the name of the thread.
    fn get_name(&self) -> Result<&ThreadName, ThreadGetNameError>;

    /// Returns a vector of numbers which represents the CPU cores in which the
    /// thread may run.
    fn get_affinity(&self) -> Result<Vec<usize>, ThreadSetAffinityError>;

    /// Sets the threads affinity to a single CPU core. If the core does not exist it has no
    /// effect.
    fn set_affinity(&mut self, cpu: usize) -> Result<(), ThreadSetAffinityError>;

    /// Sets the threads affinity to multiple CPU cores. If one of the CPU cores does not exist
    /// the core will be ignored.
    fn set_affinity_to_cores(&mut self, cores: &[usize]) -> Result<(), ThreadSetAffinityError>;
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

        let mut name: [posix::char; MAX_THREAD_NAME_LENGTH] = [0; MAX_THREAD_NAME_LENGTH];

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
                        msg, self.name.get().as_ref().unwrap().capacity());

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
        let mut cpuset = posix::cpu_set_t::new();
        let msg = "Unable to acquire threads CPU affinity";
        handle_errno!(ThreadSetAffinityError, from self,
            errno_source unsafe { posix::pthread_getaffinity_np(self.handle, std::mem::size_of::<posix::cpu_set_t>(), &mut cpuset).into()},
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

    fn set_affinity(&mut self, cpu: usize) -> Result<(), ThreadSetAffinityError> {
        let msg = "Unable to set cpu affinity to core";
        if cpu >= posix::CPU_SETSIZE {
            fail!(from self, with ThreadSetAffinityError::InvalidCpuCores,
                "{} {} since some cpu cores were invalid (maybe exceeded maximum supported CPU core number of the system).", msg, cpu);
        }

        let cores = vec![cpu];
        self.set_affinity_to_cores(&cores)
    }

    fn set_affinity_to_cores(&mut self, cores: &[usize]) -> Result<(), ThreadSetAffinityError> {
        let mut cpuset = posix::cpu_set_t::new();

        for core in cores {
            cpuset.set(*core);
        }

        let msg = "Unable to set cpu affinity";
        handle_errno!(ThreadSetAffinityError, from self,
            errno_source unsafe { posix::pthread_setaffinity_np(self.handle, std::mem::size_of::<posix::cpu_set_t>(), &cpuset).into() },
            success Errno::ESUCCES => (),
            Errno::EINVAL => (InvalidCpuCores, "{} since some cpu cores were invalid (maybe exceeded maximum supported CPU core number of the system).", msg),
            v=> (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }
}
struct ThreadStartupArgs<T: Send + Debug + 'static, F: FnOnce() -> T + Send + 'static> {
    callback: F,
    name: ThreadName,
}

/// A POSIX thread which can be build with the [`ThreadBuilder`].
///
/// # Examples
///
/// See the [`ThreadBuilder`] for advanced construction examples.
///
/// ```
/// use iceoryx2_bb_posix::thread::*;
///
/// fn some_func() {
///     let handle = ThreadHandle::from_self();
///     println!("Hello from: {:?}", handle);
/// }
///
/// let thread = ThreadBuilder::new()
///                          .name(&ThreadName::from(b"some-name"))
///                          .spawn(some_func)
///                          .expect("Failed to create thread");
///
/// println!("Created thread: {:?}", thread);
/// ```
pub struct Thread<'a> {
    handle: ThreadHandle,
    _stack: Option<&'a mut [u8]>,
}

impl<'a> Debug for Thread<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Thread {{ handle: {:?} }}", self.handle)
    }
}

impl Drop for Thread<'_> {
    fn drop(&mut self) {
        let msg = "Unable to join thread";
        match unsafe {
            posix::pthread_join(self.handle.handle, std::ptr::null_mut::<*mut posix::void>()).into()
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

impl<'a> Thread<'a> {
    fn new(handle: ThreadHandle, _stack: Option<&'a mut [u8]>) -> Self {
        Self { handle, _stack }
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

    /// Terminates the current thread.
    pub fn cancel(&mut self) -> bool {
        unsafe { posix::pthread_cancel(self.handle.handle) != 0 }
    }
}

impl<'a> ThreadProperties for Thread<'a> {
    fn get_name(&self) -> Result<&ThreadName, ThreadGetNameError> {
        self.handle.get_name()
    }

    fn get_affinity(&self) -> Result<Vec<usize>, ThreadSetAffinityError> {
        self.handle.get_affinity()
    }

    fn set_affinity(&mut self, cpu: usize) -> Result<(), ThreadSetAffinityError> {
        self.handle.set_affinity(cpu)
    }

    fn set_affinity_to_cores(&mut self, cores: &[usize]) -> Result<(), ThreadSetAffinityError> {
        self.handle.set_affinity_to_cores(cores)
    }
}
