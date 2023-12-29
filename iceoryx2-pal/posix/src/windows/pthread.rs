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

#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]
#![allow(unused_variables)]

use core::panic;
use std::{
    cell::UnsafeCell,
    os::windows::prelude::OsStringExt,
    sync::atomic::{AtomicU32, Ordering},
    time::SystemTime,
    time::UNIX_EPOCH,
};

use iceoryx2_pal_concurrency_sync::rwlock::*;
use iceoryx2_pal_concurrency_sync::{barrier::Barrier, mutex::Mutex};
use iceoryx2_pal_concurrency_sync::{WaitAction, WaitResult};
use windows_sys::Win32::{
    Foundation::{CloseHandle, ERROR_TIMEOUT, STILL_ACTIVE, WAIT_FAILED},
    System::{
        Memory::LocalFree,
        Threading::{
            CreateThread, ExitThread, GetCurrentThread, GetCurrentThreadId, GetExitCodeThread,
            GetThreadDescription, GetThreadId, SetThreadAffinityMask, SetThreadDescription,
            SetThreadPriority, TerminateThread, WaitForSingleObject, WaitOnAddress,
            WakeByAddressAll, WakeByAddressSingle, INFINITE, THREAD_PRIORITY_ABOVE_NORMAL,
            THREAD_PRIORITY_BELOW_NORMAL, THREAD_PRIORITY_HIGHEST, THREAD_PRIORITY_IDLE,
            THREAD_PRIORITY_LOWEST, THREAD_PRIORITY_NORMAL, THREAD_PRIORITY_TIME_CRITICAL,
        },
    },
};

pub use crate::posix::Struct;
use crate::{
    posix::Errno,
    posix::{
        CLOCK_REALTIME, PTHREAD_MUTEX_ERRORCHECK, PTHREAD_MUTEX_RECURSIVE, PTHREAD_MUTEX_ROBUST,
        PTHREAD_PREFER_WRITER_NONRECURSIVE_NP, PTHREAD_PREFER_WRITER_NP, PTHREAD_PRIO_NONE,
    },
    win32call,
};
use crate::{posix::*, win_internal::*};

#[derive(Clone, Copy)]
struct ThreadState {
    id: u32,
    affinity: cpu_set_t,
}

struct ThreadStates {
    states: [UnsafeCell<Option<ThreadState>>; MAX_NUMBER_OF_THREADS],
    mtx: Mutex,
}

unsafe impl Send for ThreadStates {}
unsafe impl Sync for ThreadStates {}

impl ThreadStates {
    const fn new() -> Self {
        #[allow(clippy::declare_interior_mutable_const)]
        const NONE: UnsafeCell<Option<ThreadState>> = UnsafeCell::new(None);
        #[deny(clippy::declare_interior_mutable_const)]
        Self {
            states: [NONE; MAX_NUMBER_OF_THREADS],
            mtx: Mutex::new(),
        }
    }

    fn get_instance() -> &'static ThreadStates {
        static INSTANCE: ThreadStates = ThreadStates::new();

        &INSTANCE
    }

    fn lock(&self) {
        self.mtx.lock(|atomic, value| {
            unsafe {
                win32call! { WaitOnAddress(
                    (atomic as *const AtomicU32).cast(),
                    (value as *const u32).cast(),
                    4,
                    INFINITE,
                ) };
            }
            WaitAction::Continue
        });
    }

    fn unlock(&self) {
        self.mtx.unlock(|atomic| unsafe {
            WakeByAddressSingle((atomic as *const AtomicU32).cast());
        });
    }

    fn add(&self, value: ThreadState) -> usize {
        let mut index = usize::MAX;
        self.lock();
        for i in 0..MAX_NUMBER_OF_THREADS {
            if unsafe { (*self.states[i].get()).is_none() } {
                unsafe { *self.states[i].get() = Some(value) };
                index = i;
                break;
            }
        }
        self.unlock();

        if index == usize::MAX {
            panic!("With this thread the maximum number of supported thread ({}) of the system is exceeded.", MAX_NUMBER_OF_THREADS);
        }
        index
    }

    fn get_index_of(&self, id: u32) -> usize {
        for i in 0..MAX_NUMBER_OF_THREADS {
            if let Some(ref state) = unsafe { *self.states[i].get() } {
                if state.id == id {
                    return i;
                }
            }
        }

        panic!("The thread id {} is not tracked.", id)
    }

    fn get(&self, index: usize) -> ThreadState {
        self.lock();
        let state = unsafe { (*self.states[index].get()).as_ref().unwrap() };
        self.unlock();
        *state
    }

    #[allow(clippy::mut_from_ref)]
    unsafe fn get_mut(&self, index: usize) -> &mut ThreadState {
        self.lock();
        let state = unsafe { (*self.states[index].get()).as_mut().unwrap() };
        self.unlock();
        state
    }
    #[deny(clippy::mut_from_ref)]

    fn remove(&self, index: usize) {
        self.lock();
        unsafe { (*self.states[index].get()) = None };
        self.unlock();
    }
}

unsafe fn barrier_wait(barrier: &Barrier) {
    barrier.wait(
        |atomic, value| {
            win32call! { WaitOnAddress(
                (atomic as *const AtomicU32).cast(),
                (value as *const u32).cast(),
                4,
                INFINITE,
            ) };
        },
        |atomic| {
            win32call! { WakeByAddressAll((atomic as *const AtomicU32).cast()) };
        },
    );
}

pub unsafe fn pthread_barrier_wait(barrier: *mut pthread_barrier_t) -> int {
    barrier_wait(&(*barrier).barrier);
    0
}

pub unsafe fn pthread_barrier_init(
    barrier: *mut pthread_barrier_t,
    attr: *const pthread_barrierattr_t,
    count: uint,
) -> int {
    (*barrier).barrier = Barrier::new(count as _);
    0
}

pub unsafe fn pthread_barrier_destroy(barrier: *mut pthread_barrier_t) -> int {
    0
}

pub unsafe fn pthread_barrierattr_destroy(attr: *mut pthread_barrierattr_t) -> int {
    0
}

pub unsafe fn pthread_barrierattr_init(attr: *mut pthread_barrierattr_t) -> int {
    0
}

pub unsafe fn pthread_barrierattr_setpshared(
    attr: *mut pthread_barrierattr_t,
    pshared: int,
) -> int {
    // is always ipc capable
    0
}

pub unsafe fn pthread_attr_init(attr: *mut pthread_attr_t) -> int {
    attr.write(pthread_attr_t::new());
    Errno::ESUCCES as int
}

pub unsafe fn pthread_attr_destroy(attr: *mut pthread_attr_t) -> int {
    Errno::ESUCCES as int
}

pub unsafe fn pthread_attr_setguardsize(attr: *mut pthread_attr_t, guardsize: size_t) -> int {
    Errno::ESUCCES as int
}

pub unsafe fn pthread_attr_setinheritsched(attr: *mut pthread_attr_t, inheritsched: int) -> int {
    Errno::ESUCCES as int
}

pub unsafe fn pthread_attr_setschedpolicy(attr: *mut pthread_attr_t, policy: int) -> int {
    Errno::ESUCCES as int
}

pub unsafe fn pthread_attr_setscope(attr: *mut pthread_attr_t, scope: int) -> int {
    Errno::ESUCCES as int
}

pub unsafe fn pthread_attr_setschedparam(
    attr: *mut pthread_attr_t,
    param: *const sched_param,
) -> int {
    (*attr).priority = (*param)
        .sched_priority
        .clamp(sched_get_priority_min(0), sched_get_priority_max(0));

    Errno::ESUCCES as int
}

pub unsafe fn pthread_attr_setstacksize(attr: *mut pthread_attr_t, stacksize: size_t) -> int {
    (*attr).stacksize = stacksize;
    Errno::ESUCCES as int
}

pub unsafe fn pthread_attr_setstack(
    attr: *mut pthread_attr_t,
    stackaddr: *mut void,
    stacksize: size_t,
) -> int {
    Errno::ENOTSUP as int
}

pub unsafe fn pthread_attr_setaffinity_np(
    attr: *mut pthread_attr_t,
    cpusetsize: size_t,
    cpuset: *const cpu_set_t,
) -> int {
    (*attr).affinity = *cpuset;
    Errno::ESUCCES as int
}

fn to_win_priority(prio: int) -> int {
    if prio <= -3 {
        THREAD_PRIORITY_IDLE
    } else if prio == -2 {
        THREAD_PRIORITY_LOWEST
    } else if prio == -1 {
        THREAD_PRIORITY_BELOW_NORMAL
    } else if prio == 0 {
        THREAD_PRIORITY_NORMAL
    } else if prio == 1 {
        THREAD_PRIORITY_ABOVE_NORMAL
    } else if prio == 2 {
        THREAD_PRIORITY_HIGHEST
    } else {
        THREAD_PRIORITY_TIME_CRITICAL
    }
}

struct CallbackArguments {
    startup_barrier: Barrier,
    start_routine: Option<unsafe extern "C" fn(*mut void) -> *mut void>,
    arg: *mut void,
}

unsafe extern "system" fn thread_callback(args: *mut void) -> u32 {
    let thread = args as *mut CallbackArguments;

    let start_routine = (*thread).start_routine;
    let arg = (*thread).arg;

    barrier_wait(&(*thread).startup_barrier);

    if let Some(c) = (*thread).start_routine {
        c((*thread).arg);
    }

    0
}

pub unsafe fn pthread_create(
    thread: *mut pthread_t,
    attributes: *const pthread_attr_t,
    start_routine: Option<unsafe extern "C" fn(*mut void) -> *mut void>,
    arg: *mut void,
) -> int {
    thread.write(pthread_t::new());

    let mut thread_args = CallbackArguments {
        startup_barrier: Barrier::new(2),
        start_routine,
        arg,
    };

    let handle = win32call! {CreateThread(
        core::ptr::null(),
        (*attributes).stacksize,
        Some(thread_callback),
        (&mut thread_args as *mut CallbackArguments) as *mut void,
        0,
        core::ptr::null_mut(),
    )};

    win32call! { SetThreadPriority(handle, to_win_priority((*attributes).priority)) };
    win32call! { SetThreadAffinityMask(handle, core::mem::transmute::<[u8; CPU_SETSIZE/ 8], usize>((*attributes).affinity.__bits) )};

    let thread_id = GetThreadId(handle);
    (*thread).handle = handle;
    (*thread).id = thread_id;

    let index_to_state = ThreadStates::get_instance().add(ThreadState {
        affinity: (*attributes).affinity,
        id: thread_id,
    });
    (*thread).index_to_state = index_to_state;

    barrier_wait(&thread_args.startup_barrier);

    Errno::ESUCCES as int
}

pub unsafe fn pthread_join(thread: pthread_t, retval: *mut *mut void) -> int {
    let mut ret_val = Errno::ESUCCES;
    if win32call! { WaitForSingleObject(thread.handle, INFINITE) } == WAIT_FAILED {
        ret_val = Errno::EINVAL;
    }
    if win32call! { CloseHandle(thread.handle) } == 0 {
        ret_val = Errno::EINVAL;
    }

    ThreadStates::get_instance().remove(thread.index_to_state);
    ret_val as int
}

pub unsafe fn pthread_self() -> pthread_t {
    let mut thread = pthread_t::new();
    thread.handle = win32call! { GetCurrentThread() };
    thread.id = win32call! { GetCurrentThreadId() };
    thread.index_to_state = ThreadStates::get_instance().get_index_of(thread.id);
    thread
}

pub unsafe fn pthread_setname_np(thread: pthread_t, name: *const char) -> int {
    let wide_name = c_string_to_wide_string(name);
    win32call! { SetThreadDescription(thread.handle, wide_name.as_ptr()) };

    Errno::ESUCCES as int
}

pub unsafe fn pthread_getname_np(thread: pthread_t, name: *mut char, len: size_t) -> int {
    let mut wide_name: *mut u16 = core::ptr::null_mut();

    win32call! { GetThreadDescription(thread.handle, &mut wide_name) };
    let name_str = std::ffi::OsString::from_wide(core::slice::from_raw_parts(
        wide_name,
        c_wide_string_length(wide_name),
    ))
    .into_string()
    .expect("thread name contains invalid utf-8 characters");

    for i in 0..len {
        *name.add(i) = 0;
    }

    for i in 0..core::cmp::min(len, name_str.len()) {
        *name.add(i) = name_str.as_bytes()[i] as i8;
    }

    win32call! { LocalFree(wide_name as isize) };

    Errno::ESUCCES as int
}

pub unsafe fn pthread_cancel(thread: pthread_t) -> int {
    TerminateThread(thread.handle, 0);
    Errno::ESUCCES as int
}

pub unsafe fn pthread_exit(value_ptr: *mut void) {
    ExitThread(0);
}

pub unsafe fn pthread_kill(thread: pthread_t, sig: int) -> int {
    TerminateThread(thread.handle, 0);
    Errno::ESUCCES as int
}

pub unsafe fn pthread_setaffinity_np(
    thread: pthread_t,
    cpusetsize: size_t,
    cpuset: *const cpu_set_t,
) -> int {
    if cpusetsize != CPU_SETSIZE / 8 {
        return Errno::EINVAL as int;
    }

    if win32call! { SetThreadAffinityMask(thread.handle, core::mem::transmute::<[u8; CPU_SETSIZE/ 8], usize>((*cpuset).__bits) )}
        == 0
    {
        return Errno::EINVAL as int;
    }

    ThreadStates::get_instance()
        .get_mut(thread.index_to_state)
        .affinity = *cpuset;

    Errno::ESUCCES as int
}

pub unsafe fn pthread_getaffinity_np(
    thread: pthread_t,
    cpusetsize: size_t,
    cpuset: *mut cpu_set_t,
) -> int {
    if cpusetsize != CPU_SETSIZE / 8 {
        return Errno::EINVAL as int;
    }

    *cpuset = ThreadStates::get_instance()
        .get(thread.index_to_state)
        .affinity;

    0
}

pub unsafe fn pthread_rwlockattr_setkind_np(attr: *mut pthread_rwlockattr_t, pref: int) -> int {
    (*attr).prefer_writer =
        (pref == PTHREAD_PREFER_WRITER_NP) || (pref == PTHREAD_PREFER_WRITER_NONRECURSIVE_NP);
    Errno::ESUCCES as _
}

pub unsafe fn pthread_rwlockattr_init(attr: *mut pthread_rwlockattr_t) -> int {
    Errno::ESUCCES as _
}

pub unsafe fn pthread_rwlockattr_destroy(attr: *mut pthread_rwlockattr_t) -> int {
    Errno::ESUCCES as _
}

pub unsafe fn pthread_rwlockattr_setpshared(attr: *mut pthread_rwlockattr_t, pshared: int) -> int {
    // is always IPC capable
    Errno::ESUCCES as _
}

pub unsafe fn pthread_rwlock_init(
    lock: *mut pthread_rwlock_t,
    attr: *const pthread_rwlockattr_t,
) -> int {
    match (*attr).prefer_writer {
        true => (*lock).lock = RwLockType::PreferWriter(RwLockWriterPreference::new()),
        false => (*lock).lock = RwLockType::PreferReader(RwLockReaderPreference::new()),
    }

    Errno::ESUCCES as _
}

pub unsafe fn pthread_rwlock_destroy(lock: *mut pthread_rwlock_t) -> int {
    (*lock).lock = RwLockType::None;
    Errno::ESUCCES as _
}

pub unsafe fn pthread_rwlock_rdlock(lock: *mut pthread_rwlock_t) -> int {
    match (*lock).lock {
        RwLockType::PreferReader(ref l) => l.read_lock(|atomic, value| {
            win32call! { WaitOnAddress(
                (atomic as *const AtomicU32).cast(),
                (value as *const u32).cast(),
                4,
                INFINITE,
            )};
            WaitAction::Continue
        }),
        RwLockType::PreferWriter(ref l) => l.read_lock(|atomic, value| {
            win32call! {WaitOnAddress(
                (atomic as *const AtomicU32).cast(),
                (value as *const u32).cast(),
                4,
                INFINITE,
            )};
            WaitAction::Continue
        }),
        _ => {
            return Errno::EINVAL as _;
        }
    };

    Errno::ESUCCES as _
}

pub unsafe fn pthread_rwlock_tryrdlock(lock: *mut pthread_rwlock_t) -> int {
    let wait_result = match (*lock).lock {
        RwLockType::PreferReader(ref l) => l.try_read_lock(),
        RwLockType::PreferWriter(ref l) => l.try_read_lock(),
        _ => {
            return Errno::EINVAL as _;
        }
    };

    if wait_result == WaitResult::Success {
        Errno::ESUCCES as _
    } else {
        Errno::EBUSY as _
    }
}

pub unsafe fn pthread_rwlock_unlock(lock: *mut pthread_rwlock_t) -> int {
    match (*lock).lock {
        RwLockType::PreferReader(ref l) => l.unlock(|atomic| {
            WakeByAddressSingle((atomic as *const AtomicU32).cast());
        }),
        RwLockType::PreferWriter(ref l) => l.unlock(
            |atomic| {
                WakeByAddressSingle((atomic as *const AtomicU32).cast());
            },
            |atomic| {
                WakeByAddressAll((atomic as *const AtomicU32).cast());
            },
        ),
        _ => {
            return Errno::EINVAL as _;
        }
    };

    Errno::ESUCCES as _
}

pub unsafe fn pthread_rwlock_wrlock(lock: *mut pthread_rwlock_t) -> int {
    match (*lock).lock {
        RwLockType::PreferReader(ref l) => l.write_lock(|atomic, value| {
            win32call! {WaitOnAddress(
                (atomic as *const AtomicU32).cast(),
                (value as *const u32).cast(),
                4,
                INFINITE,
            )};
            WaitAction::Continue
        }),
        RwLockType::PreferWriter(ref l) => l.write_lock(
            |atomic, value| {
                win32call! {WaitOnAddress(
                    (atomic as *const AtomicU32).cast(),
                    (value as *const u32).cast(),
                    4,
                    INFINITE,
                )};
                WaitAction::Continue
            },
            |atomic| {
                win32call! { WakeByAddressSingle((atomic as *const AtomicU32).cast()) };
            },
            |atomic| {
                WakeByAddressAll((atomic as *const AtomicU32).cast());
            },
        ),
        _ => {
            return Errno::EINVAL as _;
        }
    };

    Errno::ESUCCES as _
}

pub unsafe fn pthread_rwlock_trywrlock(lock: *mut pthread_rwlock_t) -> int {
    let wait_result = match (*lock).lock {
        RwLockType::PreferReader(ref l) => l.try_write_lock(),
        RwLockType::PreferWriter(ref l) => l.try_write_lock(),
        _ => {
            return Errno::EINVAL as _;
        }
    };

    if wait_result == WaitResult::Success {
        Errno::ESUCCES as _
    } else {
        Errno::EBUSY as _
    }
}

pub unsafe fn pthread_rwlock_timedwrlock(
    lock: *mut pthread_rwlock_t,
    abs_timeout: *const timespec,
) -> int {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let timeout = std::cmp::max(
        0,
        (*abs_timeout).tv_sec * 1000 + (*abs_timeout).tv_nsec as i64 / 1000000
            - now.as_millis() as i64,
    );

    let wait_result = match (*lock).lock {
        RwLockType::PreferReader(ref l) => l.write_lock(|atomic, value| {
            win32call! { WaitOnAddress(
                (atomic as *const AtomicU32).cast(),
                (value as *const u32).cast(),
                4,
                timeout as _,
            ), ignore ERROR_TIMEOUT };
            WaitAction::Continue
        }),
        RwLockType::PreferWriter(ref l) => l.write_lock(
            |atomic, value| {
                win32call! { WaitOnAddress(
                    (atomic as *const AtomicU32).cast(),
                    (value as *const u32).cast(),
                    4,
                    timeout as _,
                ), ignore ERROR_TIMEOUT};
                WaitAction::Continue
            },
            |atomic| {
                WakeByAddressSingle((atomic as *const AtomicU32).cast());
            },
            |atomic| {
                WakeByAddressAll((atomic as *const AtomicU32).cast());
            },
        ),
        _ => {
            return Errno::EINVAL as _;
        }
    };

    if wait_result == WaitResult::Success {
        Errno::ESUCCES as _
    } else {
        Errno::ETIMEDOUT as _
    }
}

pub unsafe fn pthread_rwlock_timedrdlock(
    lock: *mut pthread_rwlock_t,
    abs_timeout: *const timespec,
) -> int {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let timeout = std::cmp::max(
        0,
        (*abs_timeout).tv_sec * 1000 + (*abs_timeout).tv_nsec as i64 / 1000000
            - now.as_millis() as i64,
    );

    let wait_result = match (*lock).lock {
        RwLockType::PreferReader(ref l) => l.read_lock(|atomic, value| {
            win32call! { WaitOnAddress(
                (atomic as *const AtomicU32).cast(),
                (value as *const u32).cast(),
                4,
                timeout as _,
            ), ignore ERROR_TIMEOUT };
            WaitAction::Continue
        }),
        RwLockType::PreferWriter(ref l) => l.read_lock(|atomic, value| {
            win32call! {WaitOnAddress(
                (atomic as *const AtomicU32).cast(),
                (value as *const u32).cast(),
                4,
                timeout as _,
            ), ignore ERROR_TIMEOUT};
            WaitAction::Continue
        }),
        _ => {
            return Errno::EINVAL as _;
        }
    };

    if wait_result == WaitResult::Success {
        Errno::ESUCCES as _
    } else {
        Errno::ETIMEDOUT as _
    }
}

pub unsafe fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> int {
    (*cond).cv.notify_all(|atomic| {
        WakeByAddressAll((atomic as *const AtomicU32).cast());
    });
    Errno::ESUCCES as _
}

pub unsafe fn pthread_cond_signal(cond: *mut pthread_cond_t) -> int {
    (*cond).cv.notify_one(|atomic| {
        WakeByAddressSingle((atomic as *const AtomicU32).cast());
    });
    Errno::ESUCCES as _
}

pub unsafe fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> int {
    Errno::ESUCCES as _
}

pub unsafe fn pthread_cond_init(cond: *mut pthread_cond_t, attr: *const pthread_condattr_t) -> int {
    cond.write(pthread_cond_t::new());
    Errno::ESUCCES as _
}

pub unsafe fn pthread_cond_wait(cond: *mut pthread_cond_t, mutex: *mut pthread_mutex_t) -> int {
    (*cond).cv.wait(
        &(*mutex).mtx,
        |atomic| {
            WakeByAddressSingle((atomic as *const AtomicU32).cast());
        },
        |atomic, value| {
            win32call! { WaitOnAddress(
                (atomic as *const AtomicU32).cast(),
                (value as *const u32).cast(),
                4,
                INFINITE,
            )};
            WaitAction::Continue
        },
        |atomic, value| {
            win32call! {WaitOnAddress(
                (atomic as *const AtomicU32).cast(),
                (value as *const u32).cast(),
                4,
                INFINITE,
            )};
            WaitAction::Continue
        },
    );

    Errno::ESUCCES as _
}

pub unsafe fn pthread_cond_timedwait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
    abstime: *const timespec,
) -> int {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let timeout = std::cmp::max(
        0,
        (*abstime).tv_sec * 1000 + (*abstime).tv_nsec as i64 / 1000000 - now.as_millis() as i64,
    );

    match (*cond).cv.wait(
        &(*mutex).mtx,
        |atomic| {
            win32call! { WakeByAddressSingle((atomic as *const AtomicU32).cast()) };
        },
        |atomic, value| {
            win32call! { WaitOnAddress(
                (atomic as *const AtomicU32).cast(),
                (value as *const u32).cast(),
                4,
                timeout as _,
            ), ignore ERROR_TIMEOUT };
            WaitAction::Abort
        },
        |atomic, value| {
            win32call! { WaitOnAddress(
                (atomic as *const AtomicU32).cast(),
                (value as *const u32).cast(),
                4,
                timeout as _,
            ), ignore ERROR_TIMEOUT };
            WaitAction::Abort
        },
    ) {
        WaitResult::Success => Errno::ESUCCES as _,
        WaitResult::Interrupted => Errno::ETIMEDOUT as _,
    }
}

pub unsafe fn pthread_condattr_init(attr: *mut pthread_condattr_t) -> int {
    Errno::ESUCCES as _
}

pub unsafe fn pthread_condattr_destroy(attr: *mut pthread_condattr_t) -> int {
    Errno::ESUCCES as _
}

pub unsafe fn pthread_condattr_setclock(attr: *mut pthread_condattr_t, clock_id: clockid_t) -> int {
    if clock_id != CLOCK_REALTIME {
        return Errno::EINVAL as _;
    }
    Errno::ESUCCES as _
}

pub unsafe fn pthread_condattr_setpshared(attr: *mut pthread_condattr_t, pshared: int) -> int {
    // is always IPC capable
    Errno::ESUCCES as _
}

pub unsafe fn pthread_mutex_init(
    mtx: *mut pthread_mutex_t,
    attr: *const pthread_mutexattr_t,
) -> int {
    mtx.write(pthread_mutex_t::new());
    (*mtx).mtype = (*attr).mtype | (*attr).robustness;
    (*mtx).track_thread_id = (*attr).mtype == PTHREAD_MUTEX_ERRORCHECK
        || (*attr).mtype == PTHREAD_MUTEX_RECURSIVE
        || (*attr).robustness == PTHREAD_MUTEX_ROBUST;

    0
}

pub unsafe fn pthread_mutex_destroy(mtx: *mut pthread_mutex_t) -> int {
    0
}

unsafe fn owner_and_ref_count(v: u64) -> (u32, u32) {
    ((v >> 32) as u32, ((v << 32) >> 32) as u32)
}

unsafe fn prepare_lock(mtx: *mut pthread_mutex_t) -> (int, u32) {
    let mut thread_id = 0;
    if (*mtx).track_thread_id {
        thread_id = GetCurrentThreadId();
        let mut current_owner = (*mtx).current_owner.load(Ordering::Relaxed);
        let (owner, ref_count) = owner_and_ref_count(current_owner);

        if ((*mtx).mtype & PTHREAD_MUTEX_ROBUST) != 0 {
            if (*mtx).unrecoverable_state {
                return (Errno::ENOTRECOVERABLE as _, thread_id);
            }

            if owner != 0 {
                let mut exit_code = 0;
                let is_still_active = GetExitCodeThread(owner as _, &mut exit_code) == STILL_ACTIVE;

                if !is_still_active {
                    (*mtx)
                        .current_owner
                        .store((thread_id as u64) << 32, Ordering::Relaxed);
                    (*mtx).inconsistent_state = true;
                    return (Errno::EOWNERDEAD as _, thread_id);
                }
            }
        }

        if owner == thread_id {
            if ((*mtx).mtype & PTHREAD_MUTEX_ERRORCHECK) != 0 {
                return (Errno::EDEADLK as _, thread_id);
            }

            loop {
                if ref_count == u32::MAX {
                    return (Errno::EAGAIN as _, thread_id);
                }

                if ((*mtx).mtype & PTHREAD_MUTEX_RECURSIVE) != 0 {
                    match (*mtx).current_owner.compare_exchange(
                        current_owner,
                        current_owner + 1,
                        Ordering::Acquire,
                        Ordering::Relaxed,
                    ) {
                        Err(v) => current_owner = v,
                        Ok(_) => return (Errno::ESUCCES as _, thread_id),
                    }
                }

                let (owner, ref_count) = owner_and_ref_count(current_owner);

                if owner != thread_id {
                    break;
                }
            }
        }
    }

    (-1, thread_id)
}

pub unsafe fn pthread_mutex_lock(mtx: *mut pthread_mutex_t) -> int {
    let thread_id = match prepare_lock(mtx) {
        (-1, id) => id,
        (0, id) => return Errno::ESUCCES as _,
        (e, _) => return e,
    };

    (*mtx).mtx.lock(|atomic, value| {
        win32call! { WaitOnAddress(
            (atomic as *const AtomicU32).cast(),
            (value as *const u32).cast(),
            4,
            INFINITE,
        ) };
        WaitAction::Continue
    });

    if (*mtx).track_thread_id {
        (*mtx)
            .current_owner
            .store((thread_id as u64) << 32, Ordering::Relaxed);
    }

    0
}

pub unsafe fn pthread_mutex_timedlock(
    mtx: *mut pthread_mutex_t,
    abs_timeout: *const timespec,
) -> int {
    const DEADLOCK_CODE: i32 = Errno::EDEADLK as i32;
    let thread_id = match prepare_lock(mtx) {
        (-1, id) => id,
        (0, id) => return Errno::ESUCCES as _,
        // no EDEADLK for timed lock
        (DEADLOCK_CODE, id) => id,
        (e, _) => return e,
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let timeout = std::cmp::max(
        0,
        (*abs_timeout).tv_sec * 1000 + (*abs_timeout).tv_nsec as i64 / 1000000
            - now.as_millis() as i64,
    );

    match (*mtx).mtx.lock(|atomic, value| {
        win32call! { WaitOnAddress(
            (atomic as *const AtomicU32).cast(),
            (value as *const u32).cast(),
            4,
            timeout as _,
        ), ignore ERROR_TIMEOUT };
        WaitAction::Abort
    }) {
        WaitResult::Success => {
            if (*mtx).track_thread_id {
                (*mtx)
                    .current_owner
                    .store((thread_id as u64) << 32, Ordering::Relaxed);
            }

            Errno::ESUCCES as _
        }
        WaitResult::Interrupted => Errno::ETIMEDOUT as _,
    }
}

pub unsafe fn pthread_mutex_trylock(mtx: *mut pthread_mutex_t) -> int {
    let thread_id = match prepare_lock(mtx) {
        (-1, id) => id,
        (0, id) => return Errno::ESUCCES as _,
        (e, _) => return e,
    };

    match (*mtx).mtx.try_lock() {
        WaitResult::Success => {
            if (*mtx).track_thread_id {
                (*mtx)
                    .current_owner
                    .store((thread_id as u64) << 32, Ordering::Relaxed);
            }

            Errno::ESUCCES as _
        }
        WaitResult::Interrupted => Errno::EBUSY as _,
    }
}

pub unsafe fn pthread_mutex_unlock(mtx: *mut pthread_mutex_t) -> int {
    if ((*mtx).mtype & PTHREAD_MUTEX_ERRORCHECK) != 0 {
        let thread_id = GetCurrentThreadId();
        if thread_id != owner_and_ref_count((*mtx).current_owner.load(Ordering::Relaxed)).0 {
            return Errno::EBUSY as _;
        }
    }

    if (*mtx).inconsistent_state {
        (*mtx).unrecoverable_state = true;
    }

    if (*mtx).mtype & PTHREAD_MUTEX_RECURSIVE != 0 {
        let mut current_value = (*mtx).current_owner.load(Ordering::Relaxed);
        loop {
            let (owner, ref_count) = owner_and_ref_count(current_value);
            let new_value = if ref_count == 0 { 0 } else { current_value - 1 };

            match (*mtx).current_owner.compare_exchange(
                current_value,
                new_value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(v) => current_value = v,
            }
        }
    } else {
        (*mtx).current_owner.store(0, Ordering::Relaxed);
    }

    (*mtx).mtx.unlock(|atomic| {
        WakeByAddressSingle((atomic as *const AtomicU32).cast());
    });

    Errno::ESUCCES as _
}

pub unsafe fn pthread_mutex_consistent(mtx: *mut pthread_mutex_t) -> int {
    (*mtx).inconsistent_state = false;
    Errno::ESUCCES as _
}

pub unsafe fn pthread_mutex_setprioceiling(
    mtx: *mut pthread_mutex_t,
    prioceiling: int,
    old_ceiling: *mut int,
) -> int {
    Errno::set(Errno::ENOSYS);
    -1
}

pub unsafe fn pthread_mutex_getprioceiling(
    mtx: *mut pthread_mutex_t,
    prioceiling: *mut int,
) -> int {
    Errno::set(Errno::ENOSYS);
    -1
}

pub unsafe fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> int {
    Errno::set(Errno::ESUCCES);
    attr.write(pthread_mutexattr_t::new());
    0
}

pub unsafe fn pthread_mutexattr_destroy(attr: *mut pthread_mutexattr_t) -> int {
    Errno::set(Errno::ESUCCES);
    std::ptr::drop_in_place(attr);
    0
}

pub unsafe fn pthread_mutexattr_setprotocol(attr: *mut pthread_mutexattr_t, protocol: int) -> int {
    if protocol == PTHREAD_PRIO_NONE {
        Errno::set(Errno::ESUCCES);
        0
    } else {
        Errno::set(Errno::ENOSYS);
        -1
    }
}

pub unsafe fn pthread_mutexattr_setpshared(attr: *mut pthread_mutexattr_t, pshared: int) -> int {
    Errno::set(Errno::ESUCCES);
    // always ipc capable
    0
}

pub unsafe fn pthread_mutexattr_setrobust(attr: *mut pthread_mutexattr_t, robustness: int) -> int {
    Errno::set(Errno::ESUCCES);
    (*attr).robustness = robustness;
    0
}

pub unsafe fn pthread_mutexattr_settype(attr: *mut pthread_mutexattr_t, mtype: int) -> int {
    Errno::set(Errno::ESUCCES);
    (*attr).mtype = mtype;
    0
}

pub unsafe fn pthread_mutexattr_setprioceiling(
    attr: *mut pthread_mutexattr_t,
    prioceiling: int,
) -> int {
    Errno::set(Errno::ENOSYS);
    -1
}
