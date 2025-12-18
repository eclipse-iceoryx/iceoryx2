// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

#![allow(non_camel_case_types, non_snake_case)]
#![allow(clippy::missing_safety_doc)]
#![allow(unused_variables)]

use core::{sync::atomic::Ordering, unimplemented};

use iceoryx2_pal_concurrency_sync::cell::UnsafeCell;
use iceoryx2_pal_concurrency_sync::mutex::Mutex;
use iceoryx2_pal_concurrency_sync::WaitAction;

use crate::posix::*;

#[allow(dead_code)]
#[derive(Clone, Copy)]
struct ThreadState {
    id: u64,
    affinity: cpu_set_t,
    name: [u8; THREAD_NAME_LENGTH],
}

#[allow(dead_code)]
impl ThreadState {
    fn new(pthread: pthread_t) -> Self {
        Self {
            id: pthread.id as u64,
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

#[allow(dead_code)]
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
        let t = pthread.id as u64;
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
        let t = pthread.id as u64;
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
        let t = pthread.id as u64;
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

unsafe fn get_thread_id() -> u32 {
    let id = 0u64;
    id as _
}

unsafe fn owner_and_ref_count(v: u64) -> (u32, u32) {
    ((v >> 32) as u32, ((v << 32) >> 32) as u32)
}

pub unsafe fn pthread_rwlockattr_setkind_np(_attr: *mut pthread_rwlockattr_t, _pref: int) -> int {
    unimplemented!("pthread_rwlockattr_setkind_np")
}

pub unsafe fn pthread_barrier_wait(barrier: *mut pthread_barrier_t) -> int {
    unimplemented!("pthread_barrier_wait")
}

pub unsafe fn pthread_barrier_init(
    barrier: *mut pthread_barrier_t,
    attr: *const pthread_barrierattr_t,
    count: uint,
) -> int {
    unimplemented!("pthread_barrier_init")
}

pub unsafe fn pthread_barrier_destroy(barrier: *mut pthread_barrier_t) -> int {
    unimplemented!("pthread_barrier_destroy")
}

pub unsafe fn pthread_barrierattr_destroy(attr: *mut pthread_barrierattr_t) -> int {
    unimplemented!("pthread_barrierattr_destroy")
}

pub unsafe fn pthread_barrierattr_init(attr: *mut pthread_barrierattr_t) -> int {
    unimplemented!("pthread_barrierattr_init")
}

pub unsafe fn pthread_barrierattr_setpshared(
    attr: *mut pthread_barrierattr_t,
    pshared: int,
) -> int {
    unimplemented!("pthread_barrierattr_setpshared")
}

pub unsafe fn pthread_attr_init(attr: *mut pthread_attr_t) -> int {
    unimplemented!("pthread_attr_init")
}

pub unsafe fn pthread_attr_destroy(attr: *mut pthread_attr_t) -> int {
    unimplemented!("pthread_attr_destroy")
}

pub unsafe fn pthread_attr_setguardsize(attr: *mut pthread_attr_t, guardsize: size_t) -> int {
    unimplemented!("pthread_attr_setguardsize")
}

pub unsafe fn pthread_attr_setinheritsched(attr: *mut pthread_attr_t, inheritsched: int) -> int {
    unimplemented!("pthread_attr_setinheritsched")
}

pub unsafe fn pthread_attr_setschedpolicy(attr: *mut pthread_attr_t, policy: int) -> int {
    unimplemented!("pthread_attr_setschedpolicy")
}

pub unsafe fn pthread_attr_setschedparam(
    attr: *mut pthread_attr_t,
    param: *const sched_param,
) -> int {
    unimplemented!("pthread_attr_setschedparam")
}

pub unsafe fn pthread_attr_setstacksize(attr: *mut pthread_attr_t, stacksize: size_t) -> int {
    unimplemented!("pthread_attr_setstacksize")
}

pub unsafe fn pthread_attr_setaffinity_np(
    attr: *mut pthread_attr_t,
    cpusetsize: size_t,
    cpuset: *const cpu_set_t,
) -> int {
    unimplemented!("pthread_attr_setaffinity_np")
}

pub unsafe fn pthread_create(
    thread: *mut pthread_t,
    attr: *const pthread_attr_t,
    start_routine: unsafe extern "C" fn(*mut void) -> *mut void,
    arg: *mut void,
) -> int {
    unimplemented!("pthread_create")
}

pub unsafe fn pthread_join(thread: pthread_t, retval: *mut *mut void) -> int {
    unimplemented!("pthread_join")
}

pub unsafe fn pthread_self() -> pthread_t {
    unimplemented!("pthread_self")
}

pub unsafe fn pthread_setname_np(thread: pthread_t, name: *const c_char) -> int {
    unimplemented!("pthread_setname_np")
}

pub unsafe fn pthread_getname_np(thread: pthread_t, name: *mut c_char, len: size_t) -> int {
    unimplemented!("pthread_getname_np")
}

pub unsafe fn pthread_kill(thread: pthread_t, sig: int) -> int {
    unimplemented!("pthread_kill")
}

pub unsafe fn pthread_setaffinity_np(
    thread: pthread_t,
    cpusetsize: size_t,
    cpuset: *const cpu_set_t,
) -> int {
    unimplemented!("pthread_setaffinity_np")
}

pub unsafe fn pthread_getaffinity_np(
    thread: pthread_t,
    cpusetsize: size_t,
    cpuset: *mut cpu_set_t,
) -> int {
    unimplemented!("pthread_getaffinity_np")
}

pub unsafe fn pthread_rwlockattr_init(attr: *mut pthread_rwlockattr_t) -> int {
    unimplemented!("pthread_rwlockattr_init")
}

pub unsafe fn pthread_rwlockattr_destroy(attr: *mut pthread_rwlockattr_t) -> int {
    unimplemented!("pthread_rwlockattr_destroy")
}

pub unsafe fn pthread_rwlockattr_setpshared(attr: *mut pthread_rwlockattr_t, pshared: int) -> int {
    unimplemented!("pthread_rwlockattr_setpshared")
}

pub unsafe fn pthread_rwlock_init(
    lock: *mut pthread_rwlock_t,
    attr: *const pthread_rwlockattr_t,
) -> int {
    unimplemented!("pthread_rwlock_init")
}

pub unsafe fn pthread_rwlock_destroy(lock: *mut pthread_rwlock_t) -> int {
    unimplemented!("pthread_rwlock_destroy")
}

pub unsafe fn pthread_rwlock_rdlock(lock: *mut pthread_rwlock_t) -> int {
    unimplemented!("pthread_rwlock_rdlock")
}

pub unsafe fn pthread_rwlock_tryrdlock(lock: *mut pthread_rwlock_t) -> int {
    unimplemented!("pthread_rwlock_tryrdlock")
}

pub unsafe fn pthread_rwlock_unlock(lock: *mut pthread_rwlock_t) -> int {
    unimplemented!("pthread_rwlock_unlock")
}

pub unsafe fn pthread_rwlock_wrlock(lock: *mut pthread_rwlock_t) -> int {
    unimplemented!("pthread_rwlock_wrlock")
}

pub unsafe fn pthread_rwlock_trywrlock(lock: *mut pthread_rwlock_t) -> int {
    unimplemented!("pthread_rwlock_trywrlock")
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

pub unsafe fn pthread_mutex_destroy(mtx: *mut pthread_mutex_t) -> int {
    0
}

pub unsafe fn pthread_mutex_lock(mtx: *mut pthread_mutex_t) -> int {
    (*mtx).mtx.lock(|atomic, value| WaitAction::Continue);
    0
}

pub unsafe fn pthread_mutex_timedlock(
    mtx: *mut pthread_mutex_t,
    abs_timeout: *const timespec,
) -> int {
    unimplemented!("pthread_mutex_timedlock")
}

pub unsafe fn pthread_mutex_trylock(mtx: *mut pthread_mutex_t) -> int {
    unimplemented!("pthread_mutex_trylock")
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
        (*mtx).mtx.unlock(|_| {});
    }

    Errno::ESUCCES as _
}

pub unsafe fn pthread_mutex_consistent(mtx: *mut pthread_mutex_t) -> int {
    unimplemented!("pthread_mutex_consistent")
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
    unimplemented!("pthread_mutexattr_setpshared")
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
