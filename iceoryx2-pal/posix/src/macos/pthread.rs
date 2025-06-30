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

use crate::posix::*;

use core::cell::UnsafeCell;
use core::sync::atomic::Ordering;
use iceoryx2_pal_concurrency_sync::barrier::Barrier;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU32;
use iceoryx2_pal_concurrency_sync::mutex::Mutex;
use iceoryx2_pal_concurrency_sync::{rwlock::*, WaitAction, WaitResult};

#[derive(Clone, Copy)]
struct ThreadState {
    id: u64,
    affinity: cpu_set_t,
    name: [u8; THREAD_NAME_LENGTH],
}

impl ThreadState {
    fn new(pthread: pthread_t) -> Self {
        Self {
            id: pthread as u64,
            affinity: cpu_set_t::new_allow_all(),
            name: [0u8; THREAD_NAME_LENGTH],
        }
    }
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
        self.mtx.lock(|_, _| WaitAction::Continue);
    }

    fn unlock(&self) {
        self.mtx.unlock(|_| {});
    }

    fn add(&self, pthread: pthread_t) -> usize {
        let mut index = usize::MAX;
        self.lock();
        for i in 0..MAX_NUMBER_OF_THREADS {
            if unsafe { (*self.states[i].get()).is_none() } {
                unsafe { *self.states[i].get() = Some(ThreadState::new(pthread)) };
                index = i;
                break;
            }
        }
        self.unlock();

        if index == usize::MAX {
            panic!("With this thread the maximum number of supported thread ({MAX_NUMBER_OF_THREADS}) of the system is exceeded.");
        }
        index
    }

    fn get(&self, pthread: pthread_t) -> ThreadState {
        let t = pthread as u64;
        let mut thread_state = core::ptr::null::<ThreadState>();
        self.lock();
        for state in &self.states {
            unsafe {
                if let Some(ref v) = *state.get() {
                    if v.id == t {
                        thread_state = v;
                        break;
                    }
                }
            }
        }

        self.unlock();
        unsafe { *thread_state }
    }

    #[allow(clippy::mut_from_ref)]
    unsafe fn get_mut(&self, pthread: pthread_t) -> &mut ThreadState {
        let t = pthread as u64;
        let mut thread_state = core::ptr::null_mut::<ThreadState>();
        self.lock();
        for state in &self.states {
            unsafe {
                if let Some(ref mut v) = *state.get() {
                    if v.id == t {
                        thread_state = v;
                        break;
                    }
                }
            }
        }

        self.unlock();
        unsafe { &mut *thread_state }
    }

    fn remove(&self, pthread: pthread_t) {
        let t = pthread as u64;
        self.lock();
        for state in &self.states {
            unsafe {
                if let Some(ref v) = *state.get() {
                    if v.id == t {
                        *state.get() = None;
                        break;
                    }
                }
            }
        }
        self.unlock();
    }
}

#[link(name = "c++")]
extern "C" {
    #[link_name = "_ZNSt3__123__libcpp_atomic_monitorEPVKv"]
    fn __libcpp_atomic_monitor(ptr: *const void) -> i64;

    #[link_name = "_ZNSt3__120__libcpp_atomic_waitEPVKvx"]
    fn __libcpp_atomic_wait(ptr: *const void, monitor: i64);

    #[link_name = "_ZNSt3__123__cxx_atomic_notify_oneEPVKv"]
    fn __cxx_atomic_notify_one(ptr: *const void);

    #[link_name = "_ZNSt3__123__cxx_atomic_notify_allEPVKv"]
    fn __cxx_atomic_notify_all(ptr: *const void);
}

pub fn wait(atomic: &IoxAtomicU32, expected: &u32) {
    let ptr = (atomic as *const IoxAtomicU32) as *const void;
    let monitor = unsafe { __libcpp_atomic_monitor(ptr) };
    if atomic.load(Ordering::Relaxed) != *expected {
        return;
    }
    unsafe { __libcpp_atomic_wait(ptr, monitor) };
}

pub fn timed_wait(atomic: &IoxAtomicU32, expected: &u32, timeout: timespec) {
    let sleep_time = timespec {
        tv_sec: 0,
        tv_nsec: 1000000,
    };
    let mut now = timespec::new_zeroed();
    loop {
        if atomic.load(Ordering::Relaxed) != *expected {
            return;
        }

        unsafe { clock_gettime(CLOCK_REALTIME, &mut now) };
        if now.tv_sec > timeout.tv_sec
            || (now.tv_sec == timeout.tv_sec && now.tv_nsec > timeout.tv_nsec)
        {
            return;
        }

        unsafe {
            clock_nanosleep(
                CLOCK_REALTIME,
                0,
                &sleep_time,
                core::ptr::null_mut::<timespec>().cast(),
            )
        };
    }
}

pub fn wake_one(atomic: &IoxAtomicU32) {
    let ptr = (atomic as *const IoxAtomicU32) as *const void;
    unsafe { __cxx_atomic_notify_one(ptr) };
}

pub fn wake_all(atomic: &IoxAtomicU32) {
    let ptr = (atomic as *const IoxAtomicU32) as *const void;
    unsafe { __cxx_atomic_notify_all(ptr) };
}

pub unsafe fn pthread_barrier_wait(barrier: *mut pthread_barrier_t) -> int {
    (*barrier).barrier.wait(wait, wake_all);
    0
}

pub unsafe fn pthread_barrier_init(
    barrier: *mut pthread_barrier_t,
    _attr: *const pthread_barrierattr_t,
    count: uint,
) -> int {
    (*barrier).barrier = Barrier::new(count as _);
    0
}

pub unsafe fn pthread_barrier_destroy(_barrier: *mut pthread_barrier_t) -> int {
    0
}

pub unsafe fn pthread_barrierattr_destroy(_attr: *mut pthread_barrierattr_t) -> int {
    0
}

pub unsafe fn pthread_barrierattr_init(_attr: *mut pthread_barrierattr_t) -> int {
    0
}

pub unsafe fn pthread_barrierattr_setpshared(
    _attr: *mut pthread_barrierattr_t,
    _pshared: int,
) -> int {
    // is always ipc capable
    0
}

pub unsafe fn pthread_attr_init(attr: *mut pthread_attr_t) -> int {
    (*attr) = pthread_attr_t::new_zeroed();
    crate::internal::pthread_attr_init(&mut (*attr).attr)
}

pub unsafe fn pthread_attr_destroy(attr: *mut pthread_attr_t) -> int {
    crate::internal::pthread_attr_destroy(&mut (*attr).attr)
}

pub unsafe fn pthread_attr_setguardsize(attr: *mut pthread_attr_t, guardsize: size_t) -> int {
    crate::internal::pthread_attr_setguardsize(&mut (*attr).attr, guardsize)
}

pub unsafe fn pthread_attr_setinheritsched(attr: *mut pthread_attr_t, inheritsched: int) -> int {
    crate::internal::pthread_attr_setinheritsched(&mut (*attr).attr, inheritsched)
}

pub unsafe fn pthread_attr_setschedpolicy(attr: *mut pthread_attr_t, policy: int) -> int {
    crate::internal::pthread_attr_setschedpolicy(&mut (*attr).attr, policy)
}

pub unsafe fn pthread_attr_setschedparam(
    attr: *mut pthread_attr_t,
    param: *const sched_param,
) -> int {
    crate::internal::pthread_attr_setschedparam(&mut (*attr).attr, param)
}

pub unsafe fn pthread_attr_setstacksize(attr: *mut pthread_attr_t, stacksize: size_t) -> int {
    crate::internal::pthread_attr_setstacksize(&mut (*attr).attr, stacksize)
}

pub unsafe fn pthread_attr_setaffinity_np(
    attr: *mut pthread_attr_t,
    cpusetsize: size_t,
    cpuset: *const cpu_set_t,
) -> int {
    if cpusetsize != CPU_SETSIZE / 8 {
        return Errno::EINVAL as int;
    }

    (*attr).affinity = *cpuset;
    Errno::ESUCCES as int
}

struct CallbackArguments {
    startup_barrier: Barrier,
    start_routine: unsafe extern "C" fn(*mut void) -> *mut void,
    arg: *mut void,
}

unsafe extern "C" fn thread_callback(args: *mut void) -> *mut void {
    let thread = args as *mut CallbackArguments;

    let start_routine = (*thread).start_routine;
    let arg = (*thread).arg;
    let startup_barrier = &(*thread).startup_barrier;
    startup_barrier.wait(wait, wake_all);

    start_routine(arg);

    core::ptr::null_mut()
}

pub unsafe fn pthread_create(
    thread: *mut pthread_t,
    attr: *const pthread_attr_t,
    start_routine: unsafe extern "C" fn(*mut void) -> *mut void,
    arg: *mut void,
) -> int {
    let mut thread_args = CallbackArguments {
        startup_barrier: Barrier::new(2),
        start_routine,
        arg,
    };

    let result = crate::internal::pthread_create(
        thread,
        &(*attr).attr,
        Some(thread_callback),
        (&mut thread_args as *mut CallbackArguments).cast(),
    );
    if result == 0 {
        ThreadStates::get_instance().add(*thread);
    }
    ThreadStates::get_instance().get_mut(*thread).affinity = (*attr).affinity;

    thread_args.startup_barrier.wait(wait, wake_all);
    result
}

pub unsafe fn pthread_join(thread: pthread_t, retval: *mut *mut void) -> int {
    let result = crate::internal::pthread_join(thread, retval);
    if result == 0 {
        ThreadStates::get_instance().remove(thread);
    }
    result
}

pub unsafe fn pthread_self() -> pthread_t {
    crate::internal::pthread_self()
}

pub unsafe fn pthread_setname_np(thread: pthread_t, name: *const c_char) -> int {
    let state = ThreadStates::get_instance().get_mut(thread);
    for i in 0..THREAD_NAME_LENGTH {
        state.name[i] = *name.add(i) as _;

        if *name.add(i) == 0 {
            break;
        }
    }

    crate::internal::pthread_setname_np(name)
}

pub unsafe fn pthread_getname_np(thread: pthread_t, name: *mut c_char, len: size_t) -> int {
    if len < 15 {
        return Errno::ERANGE as _;
    }

    let state = ThreadStates::get_instance().get(thread);

    for i in 0..15 {
        *name.add(i) = state.name[i] as _;

        if state.name[i] == 0 {
            break;
        }
    }

    Errno::ESUCCES as _
}

pub unsafe fn pthread_kill(_thread: pthread_t, _sig: int) -> int {
    todo!()
}

pub unsafe fn pthread_setaffinity_np(
    thread: pthread_t,
    cpusetsize: size_t,
    cpuset: *const cpu_set_t,
) -> int {
    if cpusetsize != CPU_SETSIZE / 8 {
        return Errno::EINVAL as int;
    }

    ThreadStates::get_instance().get_mut(thread).affinity = *cpuset;
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

    *cpuset = ThreadStates::get_instance().get_mut(thread).affinity;
    Errno::ESUCCES as int
}

pub unsafe fn pthread_rwlockattr_setkind_np(attr: *mut pthread_rwlockattr_t, pref: int) -> int {
    (*attr).prefer_writer =
        (pref == PTHREAD_PREFER_WRITER_NP) || (pref == PTHREAD_PREFER_WRITER_NONRECURSIVE_NP);
    Errno::ESUCCES as _
}

pub unsafe fn pthread_rwlockattr_init(_attr: *mut pthread_rwlockattr_t) -> int {
    Errno::ESUCCES as _
}

pub unsafe fn pthread_rwlockattr_destroy(_attr: *mut pthread_rwlockattr_t) -> int {
    Errno::ESUCCES as _
}

pub unsafe fn pthread_rwlockattr_setpshared(
    _attr: *mut pthread_rwlockattr_t,
    _pshared: int,
) -> int {
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
            wait(atomic, value);
            WaitAction::Continue
        }),
        RwLockType::PreferWriter(ref l) => l.read_lock(|atomic, value| {
            wait(atomic, value);
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
        RwLockType::PreferReader(ref l) => l.unlock(wake_one),
        RwLockType::PreferWriter(ref l) => l.unlock(wake_one, wake_all),
        _ => {
            return Errno::EINVAL as _;
        }
    };

    Errno::ESUCCES as _
}

pub unsafe fn pthread_rwlock_wrlock(lock: *mut pthread_rwlock_t) -> int {
    match (*lock).lock {
        RwLockType::PreferReader(ref l) => l.write_lock(|atomic, value| {
            wait(atomic, value);
            WaitAction::Continue
        }),
        RwLockType::PreferWriter(ref l) => l.write_lock(
            |atomic, value| {
                wait(atomic, value);
                WaitAction::Continue
            },
            wake_one,
            wake_all,
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

pub unsafe fn pthread_mutex_init(
    mtx: *mut pthread_mutex_t,
    attr: *const pthread_mutexattr_t,
) -> int {
    mtx.write(pthread_mutex_t::new_zeroed());
    (*mtx).mtype = (*attr).mtype | (*attr).robustness;
    (*mtx).track_thread_id = (*attr).mtype == PTHREAD_MUTEX_ERRORCHECK
        || (*attr).mtype == PTHREAD_MUTEX_RECURSIVE
        || (*attr).robustness == PTHREAD_MUTEX_ROBUST;

    0
}

pub unsafe fn pthread_mutex_destroy(_mtx: *mut pthread_mutex_t) -> int {
    0
}

unsafe fn owner_and_ref_count(v: u64) -> (u32, u32) {
    ((v >> 32) as u32, ((v << 32) >> 32) as u32)
}

unsafe fn get_thread_id() -> u32 {
    let mut id = 0u64;
    crate::internal::pthread_threadid_np(core::ptr::null_mut(), &mut id);
    id as _
}

unsafe fn prepare_lock(mtx: *mut pthread_mutex_t) -> (int, u32) {
    let mut thread_id = 0;
    if (*mtx).track_thread_id {
        thread_id = get_thread_id();
        let mut current_owner = (*mtx).current_owner.load(Ordering::Relaxed);
        let (owner, ref_count) = owner_and_ref_count(current_owner);

        if ((*mtx).mtype & PTHREAD_MUTEX_ROBUST) != 0 {
            if (*mtx).unrecoverable_state {
                return (Errno::ENOTRECOVERABLE as _, thread_id);
            }

            if owner != 0 {
                let is_still_active = crate::internal::pthread_kill((*mtx).thread_handle, 0) == 0;

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

                let (owner, _ref_count) = owner_and_ref_count(current_owner);

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
        (0, _id) => return Errno::ESUCCES as _,
        (e, _) => return e,
    };

    (*mtx).mtx.lock(|atomic, value| {
        wait(atomic, value);
        WaitAction::Continue
    });

    if (*mtx).track_thread_id {
        (*mtx)
            .current_owner
            .store((thread_id as u64) << 32, Ordering::Relaxed);
        (*mtx).thread_handle = crate::internal::pthread_self();
    }

    0
}

pub unsafe fn pthread_mutex_timedlock(
    mtx: *mut pthread_mutex_t,
    abs_timeout: *const timespec,
) -> int {
    let mut current_time = timespec::new_zeroed();
    let mut wait_time = timespec::new_zeroed();

    loop {
        match pthread_mutex_trylock(mtx).into() {
            Errno::ESUCCES => return Errno::ESUCCES as _,
            Errno::EBUSY | Errno::EDEADLK => (),
            v => return v as _,
        }

        clock_gettime(CLOCK_REALTIME, &mut current_time);

        if (current_time.tv_sec > (*abs_timeout).tv_sec)
            || (current_time.tv_sec == (*abs_timeout).tv_sec
                && current_time.tv_nsec > (*abs_timeout).tv_nsec)
        {
            return Errno::ETIMEDOUT as _;
        }

        current_time.tv_sec = 0;
        current_time.tv_nsec = 1000000;

        crate::internal::nanosleep(&current_time, &mut wait_time);
    }
}

pub unsafe fn pthread_mutex_trylock(mtx: *mut pthread_mutex_t) -> int {
    let thread_id = match prepare_lock(mtx) {
        (-1, id) => id,
        (0, _id) => return Errno::ESUCCES as _,
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
        let thread_id = get_thread_id();
        if thread_id != owner_and_ref_count((*mtx).current_owner.load(Ordering::Relaxed)).0 {
            return Errno::EBUSY as _;
        }
    }

    if (*mtx).inconsistent_state {
        (*mtx).unrecoverable_state = true;
    }

    let mut unlock_thread = false;
    if (*mtx).mtype & PTHREAD_MUTEX_RECURSIVE != 0 {
        let mut current_value = (*mtx).current_owner.load(Ordering::Relaxed);
        loop {
            let (_owner, ref_count) = owner_and_ref_count(current_value);
            let new_value = if ref_count == 0 { 0 } else { current_value - 1 };

            match (*mtx).current_owner.compare_exchange(
                current_value,
                new_value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    if new_value == 0 {
                        unlock_thread = true;
                    }
                    break;
                }
                Err(v) => current_value = v,
            }
        }
    } else {
        (*mtx).current_owner.store(0, Ordering::Relaxed);
        unlock_thread = true;
    }

    if unlock_thread {
        (*mtx).mtx.unlock(wake_one);
        (*mtx).thread_handle = core::ptr::null_mut();
    }

    Errno::ESUCCES as _
}

pub unsafe fn pthread_mutex_consistent(mtx: *mut pthread_mutex_t) -> int {
    (*mtx).inconsistent_state = false;
    Errno::ESUCCES as _
}

pub unsafe fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> int {
    Errno::set(Errno::ESUCCES);
    attr.write(pthread_mutexattr_t::new_zeroed());
    0
}

pub unsafe fn pthread_mutexattr_destroy(attr: *mut pthread_mutexattr_t) -> int {
    Errno::set(Errno::ESUCCES);
    core::ptr::drop_in_place(attr);
    0
}

pub unsafe fn pthread_mutexattr_setprotocol(_attr: *mut pthread_mutexattr_t, protocol: int) -> int {
    if protocol == PTHREAD_PRIO_NONE {
        Errno::set(Errno::ESUCCES);
        0
    } else {
        Errno::set(Errno::ENOSYS);
        -1
    }
}

pub unsafe fn pthread_mutexattr_setpshared(_attr: *mut pthread_mutexattr_t, _pshared: int) -> int {
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
