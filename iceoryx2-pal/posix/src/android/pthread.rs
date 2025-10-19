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

use crate::posix::*;

pub unsafe fn pthread_rwlockattr_setkind_np(attr: *mut pthread_rwlockattr_t, pref: int) -> int {
    libc::pthread_rwlockattr_setkind_np(attr, pref)
}

pub unsafe fn pthread_barrier_wait(barrier: *mut pthread_barrier_t) -> int {
    libc::pthread_barrier_wait(barrier)
}

pub unsafe fn pthread_barrier_init(
    barrier: *mut pthread_barrier_t,
    attr: *const pthread_barrierattr_t,
    count: uint,
) -> int {
    libc::pthread_barrier_init(barrier, attr, count)
}

pub unsafe fn pthread_barrier_destroy(barrier: *mut pthread_barrier_t) -> int {
    libc::pthread_barrier_destroy(barrier)
}

pub unsafe fn pthread_barrierattr_destroy(attr: *mut pthread_barrierattr_t) -> int {
    libc::pthread_barrierattr_destroy(attr)
}

pub unsafe fn pthread_barrierattr_init(attr: *mut pthread_barrierattr_t) -> int {
    libc::pthread_barrierattr_init(attr)
}

pub unsafe fn pthread_barrierattr_setpshared(
    attr: *mut pthread_barrierattr_t,
    pshared: int,
) -> int {
    libc::pthread_barrierattr_setpshared(attr, pshared)
}

pub unsafe fn pthread_attr_init(attr: *mut pthread_attr_t) -> int {
    libc::pthread_attr_init(attr)
}

pub unsafe fn pthread_attr_destroy(attr: *mut pthread_attr_t) -> int {
    libc::pthread_attr_destroy(attr)
}

pub unsafe fn pthread_attr_setguardsize(attr: *mut pthread_attr_t, guardsize: size_t) -> int {
    libc::pthread_attr_setguardsize(attr, guardsize)
}

pub unsafe fn pthread_attr_setinheritsched(attr: *mut pthread_attr_t, inheritsched: int) -> int {
    libc::pthread_attr_setinheritsched(attr, inheritsched)
}

pub unsafe fn pthread_attr_setschedpolicy(attr: *mut pthread_attr_t, policy: int) -> int {
    libc::pthread_attr_setschedpolicy(attr, policy)
}

pub unsafe fn pthread_attr_setschedparam(
    attr: *mut pthread_attr_t,
    param: *const sched_param,
) -> int {
    libc::pthread_attr_setschedparam(attr, param)
}

pub unsafe fn pthread_attr_setstacksize(attr: *mut pthread_attr_t, stacksize: size_t) -> int {
    libc::pthread_attr_setstacksize(attr, stacksize)
}

pub unsafe fn pthread_attr_setaffinity_np(
    attr: *mut pthread_attr_t,
    cpusetsize: size_t,
    cpuset: *const cpu_set_t,
) -> int {
    let cpuset = core::mem::transmute::<cpu_set_t, native_cpu_set_t>(*cpuset);

    libc::pthread_attr_setaffinity_np(attr, cpusetsize, &cpuset)
}

pub unsafe fn pthread_create(
    thread: *mut pthread_t,
    attr: *const pthread_attr_t,
    start_routine: extern "C" fn(*mut void) -> *mut void,
    arg: *mut void,
) -> int {
    libc::pthread_create(thread, attr, start_routine, arg)
}

pub unsafe fn pthread_join(thread: pthread_t, retval: *mut *mut void) -> int {
    libc::pthread_join(thread, retval)
}

pub unsafe fn pthread_self() -> pthread_t {
    libc::pthread_self()
}

pub unsafe fn pthread_setname_np(thread: pthread_t, name: *const c_char) -> int {
    libc::pthread_setname_np(thread, name)
}

pub unsafe fn pthread_getname_np(thread: pthread_t, name: *mut c_char, len: size_t) -> int {
    libc::pthread_getname_np(thread, name, len)
}

pub unsafe fn pthread_kill(thread: pthread_t, sig: int) -> int {
    libc::pthread_kill(thread, sig)
}

pub unsafe fn pthread_setaffinity_np(
    thread: pthread_t,
    cpusetsize: size_t,
    cpuset: *const cpu_set_t,
) -> int {
    let cpuset = core::mem::transmute::<cpu_set_t, native_cpu_set_t>(*cpuset);

    libc::pthread_setaffinity_np(thread, cpusetsize, &cpuset)
}

pub unsafe fn pthread_getaffinity_np(
    thread: pthread_t,
    cpusetsize: size_t,
    cpuset: *mut cpu_set_t,
) -> int {
    let mut native_cpuset = native_cpu_set_t::new_zeroed();

    let ret_val = libc::pthread_getaffinity_np(thread, cpusetsize, &mut native_cpuset);

    *cpuset = core::mem::transmute::<native_cpu_set_t, cpu_set_t>(native_cpuset);

    ret_val
}

pub unsafe fn pthread_rwlockattr_init(attr: *mut pthread_rwlockattr_t) -> int {
    libc::pthread_rwlockattr_init(attr)
}

pub unsafe fn pthread_rwlockattr_destroy(attr: *mut pthread_rwlockattr_t) -> int {
    libc::pthread_rwlockattr_destroy(attr)
}

pub unsafe fn pthread_rwlockattr_setpshared(attr: *mut pthread_rwlockattr_t, pshared: int) -> int {
    libc::pthread_rwlockattr_setpshared(attr, pshared)
}

pub unsafe fn pthread_rwlock_init(
    lock: *mut pthread_rwlock_t,
    attr: *const pthread_rwlockattr_t,
) -> int {
    libc::pthread_rwlock_init(lock, attr)
}

pub unsafe fn pthread_rwlock_destroy(lock: *mut pthread_rwlock_t) -> int {
    libc::pthread_rwlock_destroy(lock)
}

pub unsafe fn pthread_rwlock_rdlock(lock: *mut pthread_rwlock_t) -> int {
    libc::pthread_rwlock_rdlock(lock)
}

pub unsafe fn pthread_rwlock_tryrdlock(lock: *mut pthread_rwlock_t) -> int {
    libc::pthread_rwlock_tryrdlock(lock)
}

pub unsafe fn pthread_rwlock_unlock(lock: *mut pthread_rwlock_t) -> int {
    libc::pthread_rwlock_unlock(lock)
}

pub unsafe fn pthread_rwlock_wrlock(lock: *mut pthread_rwlock_t) -> int {
    libc::pthread_rwlock_wrlock(lock)
}

pub unsafe fn pthread_rwlock_trywrlock(lock: *mut pthread_rwlock_t) -> int {
    libc::pthread_rwlock_trywrlock(lock)
}

pub unsafe fn pthread_mutex_init(
    mtx: *mut pthread_mutex_t,
    attr: *const pthread_mutexattr_t,
) -> int {
    libc::pthread_mutex_init(mtx, attr)
}

pub unsafe fn pthread_mutex_destroy(mtx: *mut pthread_mutex_t) -> int {
    libc::pthread_mutex_destroy(mtx)
}

pub unsafe fn pthread_mutex_lock(mtx: *mut pthread_mutex_t) -> int {
    libc::pthread_mutex_lock(mtx)
}

pub unsafe fn pthread_mutex_timedlock(
    mtx: *mut pthread_mutex_t,
    abs_timeout: *const timespec,
) -> int {
    libc::pthread_mutex_timedlock(mtx, abs_timeout)
}

pub unsafe fn pthread_mutex_trylock(mtx: *mut pthread_mutex_t) -> int {
    libc::pthread_mutex_trylock(mtx)
}

pub unsafe fn pthread_mutex_unlock(mtx: *mut pthread_mutex_t) -> int {
    libc::pthread_mutex_unlock(mtx)
}

pub unsafe fn pthread_mutex_consistent(mtx: *mut pthread_mutex_t) -> int {
    libc::pthread_mutex_consistent(mtx)
}

pub unsafe fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> int {
    libc::pthread_mutexattr_init(attr)
}

pub unsafe fn pthread_mutexattr_destroy(attr: *mut pthread_mutexattr_t) -> int {
    libc::pthread_mutexattr_destroy(attr)
}

pub unsafe fn pthread_mutexattr_setprotocol(attr: *mut pthread_mutexattr_t, protocol: int) -> int {
    libc::pthread_mutexattr_setprotocol(attr, protocol)
}

pub unsafe fn pthread_mutexattr_setpshared(attr: *mut pthread_mutexattr_t, pshared: int) -> int {
    libc::pthread_mutexattr_setpshared(attr, pshared)
}

pub unsafe fn pthread_mutexattr_setrobust(attr: *mut pthread_mutexattr_t, robustness: int) -> int {
    libc::pthread_mutexattr_setrobust(attr, robustness)
}

pub unsafe fn pthread_mutexattr_settype(attr: *mut pthread_mutexattr_t, mtype: int) -> int {
    libc::pthread_mutexattr_settype(attr, mtype)
}
