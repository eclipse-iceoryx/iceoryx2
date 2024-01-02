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

pub unsafe fn pthread_rwlockattr_setkind_np(_attr: *mut pthread_rwlockattr_t, _pref: int) -> int {
    0
}

pub unsafe fn pthread_barrier_wait(barrier: *mut pthread_barrier_t) -> int {
    crate::internal::pthread_barrier_wait(barrier)
}

pub unsafe fn pthread_barrier_init(
    barrier: *mut pthread_barrier_t,
    attr: *const pthread_barrierattr_t,
    count: uint,
) -> int {
    crate::internal::pthread_barrier_init(barrier, attr, count)
}

pub unsafe fn pthread_barrier_destroy(barrier: *mut pthread_barrier_t) -> int {
    crate::internal::pthread_barrier_destroy(barrier)
}

pub unsafe fn pthread_barrierattr_destroy(attr: *mut pthread_barrierattr_t) -> int {
    crate::internal::pthread_barrierattr_destroy(attr)
}

pub unsafe fn pthread_barrierattr_init(attr: *mut pthread_barrierattr_t) -> int {
    crate::internal::pthread_barrierattr_init(attr)
}

pub unsafe fn pthread_barrierattr_setpshared(
    attr: *mut pthread_barrierattr_t,
    pshared: int,
) -> int {
    crate::internal::pthread_barrierattr_setpshared(attr, pshared)
}

pub unsafe fn pthread_attr_init(attr: *mut pthread_attr_t) -> int {
    crate::internal::pthread_attr_init(attr)
}

pub unsafe fn pthread_attr_destroy(attr: *mut pthread_attr_t) -> int {
    crate::internal::pthread_attr_destroy(attr)
}

pub unsafe fn pthread_attr_setguardsize(attr: *mut pthread_attr_t, guardsize: size_t) -> int {
    crate::internal::pthread_attr_setguardsize(attr, guardsize)
}

pub unsafe fn pthread_attr_setinheritsched(attr: *mut pthread_attr_t, inheritsched: int) -> int {
    crate::internal::pthread_attr_setinheritsched(attr, inheritsched)
}

pub unsafe fn pthread_attr_setschedpolicy(attr: *mut pthread_attr_t, policy: int) -> int {
    crate::internal::pthread_attr_setschedpolicy(attr, policy)
}

pub unsafe fn pthread_attr_setscope(attr: *mut pthread_attr_t, scope: int) -> int {
    crate::internal::pthread_attr_setscope(attr, scope)
}

pub unsafe fn pthread_attr_setschedparam(
    attr: *mut pthread_attr_t,
    param: *const sched_param,
) -> int {
    crate::internal::pthread_attr_setschedparam(attr, param)
}

pub unsafe fn pthread_attr_setstacksize(attr: *mut pthread_attr_t, stacksize: size_t) -> int {
    crate::internal::pthread_attr_setstacksize(attr, stacksize)
}

pub unsafe fn pthread_attr_setstack(
    attr: *mut pthread_attr_t,
    stackaddr: *mut void,
    stacksize: size_t,
) -> int {
    crate::internal::pthread_attr_setstack(attr, stackaddr, stacksize)
}

pub unsafe fn pthread_attr_setaffinity_np(
    attr: *mut pthread_attr_t,
    cpusetsize: size_t,
    cpuset: *const cpu_set_t,
) -> int {
    internal::pthread_attr_setaffinity_np(attr, cpusetsize, cpuset)
}

pub unsafe fn pthread_create(
    thread: *mut pthread_t,
    attr: *const pthread_attr_t,
    start_routine: Option<unsafe extern "C" fn(*mut void) -> *mut void>,
    arg: *mut void,
) -> int {
    crate::internal::pthread_create(thread, attr, start_routine, arg)
}

pub unsafe fn pthread_join(thread: pthread_t, retval: *mut *mut void) -> int {
    crate::internal::pthread_join(thread, retval)
}

pub unsafe fn pthread_self() -> pthread_t {
    crate::internal::pthread_self()
}

pub unsafe fn pthread_setname_np(thread: pthread_t, name: *const c_char) -> int {
    internal::pthread_setname_np(thread, name)
}

pub unsafe fn pthread_getname_np(thread: pthread_t, name: *mut c_char, len: size_t) -> int {
    internal::pthread_getname_np(thread, name, len)
}

pub unsafe fn pthread_cancel(thread: pthread_t) -> int {
    crate::internal::pthread_cancel(thread)
}

pub unsafe fn pthread_exit(value_ptr: *mut void) {
    crate::internal::pthread_exit(value_ptr)
}

pub unsafe fn pthread_kill(thread: pthread_t, sig: int) -> int {
    internal::pthread_kill(thread, sig)
}

pub unsafe fn pthread_setaffinity_np(
    thread: pthread_t,
    cpusetsize: size_t,
    cpuset: *const cpu_set_t,
) -> int {
    internal::pthread_setaffinity_np(thread, cpusetsize, cpuset)
}

pub unsafe fn pthread_getaffinity_np(
    thread: pthread_t,
    cpusetsize: size_t,
    cpuset: *mut cpu_set_t,
) -> int {
    internal::pthread_getaffinity_np(thread, cpusetsize, cpuset)
}

pub unsafe fn pthread_rwlockattr_init(attr: *mut pthread_rwlockattr_t) -> int {
    crate::internal::pthread_rwlockattr_init(attr)
}

pub unsafe fn pthread_rwlockattr_destroy(attr: *mut pthread_rwlockattr_t) -> int {
    crate::internal::pthread_rwlockattr_destroy(attr)
}

pub unsafe fn pthread_rwlockattr_setpshared(attr: *mut pthread_rwlockattr_t, pshared: int) -> int {
    crate::internal::pthread_rwlockattr_setpshared(attr, pshared)
}

pub unsafe fn pthread_rwlock_init(
    lock: *mut pthread_rwlock_t,
    attr: *const pthread_rwlockattr_t,
) -> int {
    crate::internal::pthread_rwlock_init(lock, attr)
}

pub unsafe fn pthread_rwlock_destroy(lock: *mut pthread_rwlock_t) -> int {
    crate::internal::pthread_rwlock_destroy(lock)
}

pub unsafe fn pthread_rwlock_rdlock(lock: *mut pthread_rwlock_t) -> int {
    crate::internal::pthread_rwlock_rdlock(lock)
}

pub unsafe fn pthread_rwlock_tryrdlock(lock: *mut pthread_rwlock_t) -> int {
    crate::internal::pthread_rwlock_tryrdlock(lock)
}

pub unsafe fn pthread_rwlock_unlock(lock: *mut pthread_rwlock_t) -> int {
    crate::internal::pthread_rwlock_unlock(lock)
}

pub unsafe fn pthread_rwlock_wrlock(lock: *mut pthread_rwlock_t) -> int {
    crate::internal::pthread_rwlock_wrlock(lock)
}

pub unsafe fn pthread_rwlock_trywrlock(lock: *mut pthread_rwlock_t) -> int {
    crate::internal::pthread_rwlock_trywrlock(lock)
}

pub unsafe fn pthread_rwlock_timedwrlock(
    lock: *mut pthread_rwlock_t,
    abs_timeout: *const timespec,
) -> int {
    crate::internal::pthread_rwlock_timedwrlock(lock, abs_timeout)
}

pub unsafe fn pthread_rwlock_timedrdlock(
    lock: *mut pthread_rwlock_t,
    abs_timeout: *const timespec,
) -> int {
    crate::internal::pthread_rwlock_timedrdlock(lock, abs_timeout)
}

pub unsafe fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> int {
    crate::internal::pthread_cond_broadcast(cond)
}

pub unsafe fn pthread_cond_signal(cond: *mut pthread_cond_t) -> int {
    crate::internal::pthread_cond_signal(cond)
}

pub unsafe fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> int {
    crate::internal::pthread_cond_destroy(cond)
}

pub unsafe fn pthread_cond_init(cond: *mut pthread_cond_t, attr: *const pthread_condattr_t) -> int {
    crate::internal::pthread_cond_init(cond, attr)
}

pub unsafe fn pthread_cond_wait(cond: *mut pthread_cond_t, mutex: *mut pthread_mutex_t) -> int {
    crate::internal::pthread_cond_wait(cond, mutex)
}

pub unsafe fn pthread_cond_timedwait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
    abstime: *const timespec,
) -> int {
    crate::internal::pthread_cond_timedwait(cond, mutex, abstime)
}

pub unsafe fn pthread_condattr_init(attr: *mut pthread_condattr_t) -> int {
    crate::internal::pthread_condattr_init(attr)
}

pub unsafe fn pthread_condattr_destroy(attr: *mut pthread_condattr_t) -> int {
    crate::internal::pthread_condattr_destroy(attr)
}

pub unsafe fn pthread_condattr_setclock(attr: *mut pthread_condattr_t, clock_id: clockid_t) -> int {
    crate::internal::pthread_condattr_setclock(attr, clock_id)
}

pub unsafe fn pthread_condattr_setpshared(attr: *mut pthread_condattr_t, pshared: int) -> int {
    crate::internal::pthread_condattr_setpshared(attr, pshared)
}

pub unsafe fn pthread_mutex_init(
    mtx: *mut pthread_mutex_t,
    attr: *const pthread_mutexattr_t,
) -> int {
    crate::internal::pthread_mutex_init(mtx, attr)
}

pub unsafe fn pthread_mutex_destroy(mtx: *mut pthread_mutex_t) -> int {
    crate::internal::pthread_mutex_destroy(mtx)
}

pub unsafe fn pthread_mutex_lock(mtx: *mut pthread_mutex_t) -> int {
    crate::internal::pthread_mutex_lock(mtx)
}

pub unsafe fn pthread_mutex_timedlock(
    mtx: *mut pthread_mutex_t,
    abs_timeout: *const timespec,
) -> int {
    crate::internal::pthread_mutex_timedlock(mtx, abs_timeout)
}

pub unsafe fn pthread_mutex_trylock(mtx: *mut pthread_mutex_t) -> int {
    crate::internal::pthread_mutex_trylock(mtx)
}

pub unsafe fn pthread_mutex_unlock(mtx: *mut pthread_mutex_t) -> int {
    crate::internal::pthread_mutex_unlock(mtx)
}

pub unsafe fn pthread_mutex_consistent(mtx: *mut pthread_mutex_t) -> int {
    crate::internal::pthread_mutex_consistent(mtx)
}

pub unsafe fn pthread_mutex_setprioceiling(
    mtx: *mut pthread_mutex_t,
    prioceiling: int,
    old_ceiling: *mut int,
) -> int {
    crate::internal::pthread_mutex_setprioceiling(mtx, prioceiling, old_ceiling)
}

pub unsafe fn pthread_mutex_getprioceiling(
    mtx: *mut pthread_mutex_t,
    prioceiling: *mut int,
) -> int {
    crate::internal::pthread_mutex_getprioceiling(mtx, prioceiling)
}

pub unsafe fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> int {
    crate::internal::pthread_mutexattr_init(attr)
}

pub unsafe fn pthread_mutexattr_destroy(attr: *mut pthread_mutexattr_t) -> int {
    crate::internal::pthread_mutexattr_destroy(attr)
}

pub unsafe fn pthread_mutexattr_setprotocol(attr: *mut pthread_mutexattr_t, protocol: int) -> int {
    crate::internal::pthread_mutexattr_setprotocol(attr, protocol)
}

pub unsafe fn pthread_mutexattr_setpshared(attr: *mut pthread_mutexattr_t, pshared: int) -> int {
    crate::internal::pthread_mutexattr_setpshared(attr, pshared)
}

pub unsafe fn pthread_mutexattr_setrobust(attr: *mut pthread_mutexattr_t, robustness: int) -> int {
    crate::internal::pthread_mutexattr_setrobust(attr, robustness)
}

pub unsafe fn pthread_mutexattr_settype(attr: *mut pthread_mutexattr_t, mtype: int) -> int {
    crate::internal::pthread_mutexattr_settype(attr, mtype)
}

pub unsafe fn pthread_mutexattr_setprioceiling(
    attr: *mut pthread_mutexattr_t,
    prioceiling: int,
) -> int {
    crate::internal::pthread_mutexattr_setprioceiling(attr, prioceiling)
}

mod internal {
    use super::*;

    #[cfg_attr(target_os = "freebsd", link(name = "pthread"))]
    extern "C" {
        pub(super) fn pthread_attr_setaffinity_np(
            attr: *mut pthread_attr_t,
            cpusetsize: size_t,
            cpuset: *const cpu_set_t,
        ) -> int;

        pub(super) fn pthread_setname_np(thread: pthread_t, name: *const c_char) -> int;
        pub(super) fn pthread_getname_np(thread: pthread_t, name: *mut c_char, len: size_t) -> int;
        pub(super) fn pthread_kill(thread: pthread_t, sig: int) -> int;
        pub(super) fn pthread_setaffinity_np(
            thread: pthread_t,
            cpusetsize: size_t,
            cpuset: *const cpu_set_t,
        ) -> int;
        pub(super) fn pthread_getaffinity_np(
            thread: pthread_t,
            cpusetsize: size_t,
            cpuset: *mut cpu_set_t,
        ) -> int;
    }
}
