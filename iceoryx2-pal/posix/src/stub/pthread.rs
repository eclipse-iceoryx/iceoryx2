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

use core::unimplemented;

use crate::posix::*;

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
    unimplemented!("pthread_mutex_init")
}

pub unsafe fn pthread_mutex_destroy(mtx: *mut pthread_mutex_t) -> int {
    unimplemented!("pthread_mutex_destroy")
}

pub unsafe fn pthread_mutex_lock(mtx: *mut pthread_mutex_t) -> int {
    unimplemented!("pthread_mutex_lock")
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
    unimplemented!("pthread_mutex_unlock")
}

pub unsafe fn pthread_mutex_consistent(mtx: *mut pthread_mutex_t) -> int {
    unimplemented!("pthread_mutex_consistent")
}

pub unsafe fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> int {
    unimplemented!("pthread_mutexattr_init")
}

pub unsafe fn pthread_mutexattr_destroy(attr: *mut pthread_mutexattr_t) -> int {
    unimplemented!("pthread_mutexattr_destroy")
}

pub unsafe fn pthread_mutexattr_setprotocol(attr: *mut pthread_mutexattr_t, protocol: int) -> int {
    unimplemented!("pthread_mutexattr_setprotocol")
}

pub unsafe fn pthread_mutexattr_setpshared(attr: *mut pthread_mutexattr_t, pshared: int) -> int {
    unimplemented!("pthread_mutexattr_setpshared")
}

pub unsafe fn pthread_mutexattr_setrobust(attr: *mut pthread_mutexattr_t, robustness: int) -> int {
    unimplemented!("pthread_mutexattr_setrobust")
}

pub unsafe fn pthread_mutexattr_settype(attr: *mut pthread_mutexattr_t, mtype: int) -> int {
    unimplemented!("pthread_mutexattr_settype")
}
