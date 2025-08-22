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
use iceoryx2_pal_concurrency_sync::barrier::Barrier;
use iceoryx2_pal_concurrency_sync::condition_variable::*;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU64;
use iceoryx2_pal_concurrency_sync::rwlock::*;
use iceoryx2_pal_concurrency_sync::semaphore::*;

pub type ulong = usize;

#[repr(C)]
pub struct ucred {
    pub pid: pid_t,
    pub uid: uid_t,
    pub gid: gid_t,
}
impl MemZeroedStruct for ucred {}

pub type DIR = crate::internal::DIR;

pub type blkcnt_t = crate::internal::blkcnt_t;
pub type blksize_t = crate::internal::blksize_t;
pub type c_char = core::ffi::c_char;
pub type clockid_t = crate::internal::clockid_t;
pub type dev_t = crate::internal::dev_t;
pub type gid_t = crate::internal::gid_t;
pub type ino_t = crate::internal::ino_t;
pub type int = core::ffi::c_int;
pub type in_port_t = u16;
pub type in_addr_t = u32;
pub type long = core::ffi::c_long;
pub type mode_t = crate::internal::mode_t;
pub type nlink_t = crate::internal::nlink_t;
pub type off_t = crate::internal::off_t;
pub type pid_t = crate::internal::pid_t;
pub type rlim_t = crate::internal::rlim_t;
pub type __rlim_t = usize;
pub type sa_family_t = crate::internal::sa_family_t;
pub type short = core::ffi::c_short;
pub type sighandler_t = size_t;
pub type size_t = usize;
pub type socklen_t = crate::internal::socklen_t;
pub type ssize_t = isize;
pub type suseconds_t = crate::internal::suseconds_t;
pub type time_t = crate::internal::time_t;
pub type uchar = core::ffi::c_uchar;
pub type uid_t = crate::internal::uid_t;
pub type uint = crate::internal::uint;
pub type ushort = crate::internal::ushort;
pub type void = core::ffi::c_void;

pub type sigset_t = crate::internal::sigset_t;
impl MemZeroedStruct for sigset_t {}

pub struct pthread_barrier_t {
    pub(crate) barrier: Barrier,
}

impl MemZeroedStruct for pthread_barrier_t {
    fn new_zeroed() -> Self {
        Self {
            barrier: Barrier::new(0),
        }
    }
}

pub struct pthread_barrierattr_t {}
impl MemZeroedStruct for pthread_barrierattr_t {}

pub struct pthread_attr_t {
    pub(crate) affinity: cpu_set_t,
    pub(crate) attr: crate::internal::pthread_attr_t,
}
impl MemZeroedStruct for pthread_attr_t {
    fn new_zeroed() -> Self {
        Self {
            affinity: cpu_set_t::new_allow_all(),
            attr: crate::internal::pthread_attr_t::new_zeroed(),
        }
    }
}

impl MemZeroedStruct for crate::internal::pthread_attr_t {}

pub type pthread_t = crate::internal::pthread_t;
impl MemZeroedStruct for pthread_t {}

pub struct pthread_rwlockattr_t {
    pub(crate) prefer_writer: bool,
}
impl MemZeroedStruct for pthread_rwlockattr_t {
    fn new_zeroed() -> Self {
        Self {
            prefer_writer: false,
        }
    }
}

pub(crate) enum RwLockType {
    PreferReader(RwLockReaderPreference),
    PreferWriter(RwLockWriterPreference),
    None,
}

pub struct pthread_rwlock_t {
    pub(crate) lock: RwLockType,
}
impl MemZeroedStruct for pthread_rwlock_t {
    fn new_zeroed() -> Self {
        Self {
            lock: RwLockType::None,
        }
    }
}

#[repr(C)]
pub struct pthread_mutex_t {
    pub(crate) mtx: Mutex,
    pub(crate) track_thread_id: bool,
    pub(crate) mtype: int,
    pub(crate) current_owner: IoxAtomicU64,
    pub(crate) thread_handle: pthread_t,
    pub(crate) inconsistent_state: bool,
    pub(crate) unrecoverable_state: bool,
}
impl MemZeroedStruct for pthread_mutex_t {
    fn new_zeroed() -> Self {
        Self {
            mtx: Mutex::new(),
            mtype: PTHREAD_MUTEX_NORMAL,
            current_owner: IoxAtomicU64::new(0),
            thread_handle: unsafe { crate::internal::pthread_self() },
            track_thread_id: false,
            inconsistent_state: false,
            unrecoverable_state: false,
        }
    }
}

pub struct pthread_mutexattr_t {
    pub(crate) robustness: int,
    pub(crate) mtype: int,
}
impl MemZeroedStruct for pthread_mutexattr_t {}

pub struct sem_t {
    pub(crate) semaphore: Semaphore,
}
impl MemZeroedStruct for sem_t {
    fn new_zeroed() -> Self {
        Self {
            semaphore: Semaphore::new(0),
        }
    }
}

pub type flock = crate::internal::flock;
impl MemZeroedStruct for flock {}

pub type rlimit = crate::internal::rlimit;
impl MemZeroedStruct for rlimit {}

pub type sched_param = crate::internal::sched_param;
impl MemZeroedStruct for sched_param {}

pub(crate) type native_stat_t = crate::internal::stat;
impl MemZeroedStruct for native_stat_t {}

#[repr(C)]
pub struct stat_t {
    pub st_dev: dev_t,
    pub st_ino: ino_t,
    pub st_nlink: nlink_t,
    pub st_mode: mode_t,
    pub st_uid: uid_t,
    pub st_gid: gid_t,
    pub st_rdev: dev_t,
    pub st_size: off_t,
    pub st_atime: time_t,
    pub st_mtime: time_t,
    pub st_ctime: time_t,
    pub st_blksize: blksize_t,
    pub st_blocks: blkcnt_t,
}
impl From<native_stat_t> for stat_t {
    fn from(value: native_stat_t) -> Self {
        stat_t {
            st_dev: value.st_dev,
            st_ino: value.st_ino,
            st_nlink: value.st_nlink,
            st_mode: value.st_mode,
            st_uid: value.st_uid,
            st_gid: value.st_gid,
            st_rdev: value.st_rdev,
            st_size: value.st_size,
            st_atime: 0,
            st_mtime: 0,
            st_ctime: 0,
            st_blksize: value.st_blksize,
            st_blocks: value.st_blocks,
        }
    }
}
impl MemZeroedStruct for stat_t {}

pub type timespec = crate::internal::timespec;
impl MemZeroedStruct for timespec {}

pub type timeval = crate::internal::timeval;
impl MemZeroedStruct for timeval {}

pub type fd_set = crate::internal::fd_set;
impl MemZeroedStruct for fd_set {}

pub type dirent = crate::internal::dirent;
impl MemZeroedStruct for dirent {}

pub type msghdr = crate::internal::msghdr;
impl MemZeroedStruct for msghdr {}

pub type cmsghdr = crate::internal::cmsghdr;
impl MemZeroedStruct for cmsghdr {}

#[repr(transparent)]
pub struct iovec(crate::internal::iovec);
impl MemZeroedStruct for iovec {}

impl iovec {
    pub fn set_base(&mut self, base: *mut ::core::ffi::c_void) {
        self.0.iov_base = base;
    }
    pub fn set_len(&mut self, len: usize) {
        self.0.iov_len = len as _;
    }
    pub fn as_mut_ptr(&mut self) -> *mut crate::internal::iovec {
        &mut self.0
    }
}

pub type sockaddr = crate::internal::sockaddr;
impl MemZeroedStruct for sockaddr {}

pub type sockaddr_un = crate::internal::sockaddr_un;
impl MemZeroedStruct for sockaddr_un {}

pub type sockaddr_in = crate::internal::sockaddr_in;
impl MemZeroedStruct for sockaddr_in {}

impl SockAddrIn for sockaddr_in {
    fn set_s_addr(&mut self, value: u32) {
        self.sin_addr.s_addr = value;
    }

    fn get_s_addr(&self) -> u32 {
        self.sin_addr.s_addr
    }
}

pub type passwd = crate::internal::passwd;
impl MemZeroedStruct for passwd {}

pub type group = crate::internal::group;
impl MemZeroedStruct for group {}
