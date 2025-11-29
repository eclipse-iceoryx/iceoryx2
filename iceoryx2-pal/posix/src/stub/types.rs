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

#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]
#![allow(unused_variables)]
#![allow(dead_code)]

use core::unimplemented;

use crate::common::mem_zeroed_struct::MemZeroedStruct;
use crate::posix::*;
use iceoryx2_pal_concurrency_sync::atomic::AtomicU64;
use iceoryx2_pal_concurrency_sync::barrier::Barrier;
use iceoryx2_pal_concurrency_sync::mutex::Mutex;
use iceoryx2_pal_concurrency_sync::rwlock::*;
use iceoryx2_pal_concurrency_sync::semaphore::Semaphore;

use super::settings::MAX_PATH_LENGTH;

pub struct DIR {}

pub type blkcnt_t = u64;
pub type blksize_t = u64;
pub type c_char = core::ffi::c_char;
pub type clockid_t = i32;
pub type dev_t = u64;
pub type gid_t = u32;
pub type ino_t = u64;
pub type int = core::ffi::c_int;
pub type long = core::ffi::c_long;
pub type mode_t = u64;
pub type nlink_t = u64;
pub type off_t = i64;
pub type pid_t = u32;
pub type rlim_t = i64;
pub type __rlim_t = u64;
pub type sa_family_t = u64;
pub type short = core::ffi::c_short;
pub type sighandler_t = size_t;
pub type size_t = usize;
pub type socklen_t = u32;
pub type ssize_t = isize;
pub type suseconds_t = u64;
pub type uchar = core::ffi::c_uchar;
pub type uid_t = u32;
pub type uint = u32;
pub type ushort = u16;
pub type ulong = u64;
pub type void = core::ffi::c_void;

#[derive(Clone, Copy, Debug)]
pub struct sigset_t {}
impl MemZeroedStruct for sigset_t {}

pub struct pthread_barrier_t {
    pub(crate) barrier: Barrier,
}
impl MemZeroedStruct for pthread_barrier_t {}

pub struct pthread_barrierattr_t {}
impl MemZeroedStruct for pthread_barrierattr_t {}

pub struct pthread_attr_t {
    pub(crate) stacksize: size_t,
    pub(crate) priority: int,
    pub(crate) affinity: cpu_set_t,
}
impl MemZeroedStruct for pthread_attr_t {}

#[derive(Debug, Copy, Clone)]
pub struct pthread_t {
    pub(crate) handle: usize,
    pub(crate) index_to_state: usize,
    pub(crate) id: u32,
}
impl MemZeroedStruct for pthread_t {}

pub struct pthread_rwlockattr_t {
    pub(crate) prefer_writer: bool,
}
impl MemZeroedStruct for pthread_rwlockattr_t {}

pub(crate) enum RwLockType {
    PreferReader(RwLockReaderPreference),
    PreferWriter(RwLockWriterPreference),
    None,
}

pub struct pthread_rwlock_t {
    pub(crate) lock: RwLockType,
}
impl MemZeroedStruct for pthread_rwlock_t {}

#[repr(C)]
pub struct pthread_mutex_t {
    pub(crate) mtx: Mutex,
    pub(crate) track_thread_id: bool,
    pub(crate) mtype: int,
    pub(crate) current_owner: AtomicU64,
    pub(crate) inconsistent_state: bool,
    pub(crate) unrecoverable_state: bool,
}
impl MemZeroedStruct for pthread_mutex_t {}

#[repr(C)]
pub struct pthread_mutexattr_t {
    pub(crate) robustness: int,
    pub(crate) mtype: int,
}
impl MemZeroedStruct for pthread_mutexattr_t {}

#[repr(C)]
pub struct sem_t {
    pub(crate) semaphore: Semaphore,
}
impl MemZeroedStruct for sem_t {}

#[repr(C)]
pub struct flock {
    pub l_type: short,
    pub l_whence: short,
    pub l_start: off_t,
    pub l_len: off_t,
    pub l_pid: pid_t,
}
impl MemZeroedStruct for flock {}

#[repr(C)]
pub struct rlimit {
    pub rlim_cur: rlim_t,
    pub rlim_max: rlim_t,
}
impl MemZeroedStruct for rlimit {}

#[repr(C)]
pub struct sched_param {
    pub sched_priority: int,
}
impl MemZeroedStruct for sched_param {}

pub type time_t = i64;

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
impl MemZeroedStruct for stat_t {}

#[repr(C)]
pub struct timespec {
    pub tv_sec: time_t,
    pub tv_nsec: long,
}
impl MemZeroedStruct for timespec {}

#[repr(C)]
pub struct timeval {
    pub tv_sec: time_t,
    pub tv_usec: suseconds_t,
}
impl MemZeroedStruct for timeval {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct fd_set {}
impl MemZeroedStruct for fd_set {}

#[repr(C)]
pub struct dirent {
    pub d_ino: ino_t,
    pub d_off: off_t,
    pub d_reclen: ushort,
    pub d_type: uchar,
    pub d_name: [c_char; MAX_PATH_LENGTH],
}
impl MemZeroedStruct for dirent {}

#[repr(C)]
pub struct ucred {
    pub pid: pid_t,
    pub uid: uid_t,
    pub gid: gid_t,
}
impl MemZeroedStruct for ucred {}

pub struct msghdr {
    pub msg_name: *mut void,
    pub msg_namelen: socklen_t,
    pub msg_iov: *mut iovec,
    pub msg_iovlen: int,
    pub msg_control: *mut void,
    pub msg_controllen: socklen_t,
    pub msg_flags: int,
}
impl MemZeroedStruct for msghdr {}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct cmsghdr {
    pub cmsg_len: size_t,
    pub cmsg_level: int,
    pub cmsg_type: int,
}
impl MemZeroedStruct for cmsghdr {}

pub struct iovec {
    pub iov_base: *mut void,
    pub iov_len: size_t,
}
impl MemZeroedStruct for iovec {}

impl iovec {
    pub fn set_base(&mut self, base: *mut crate::posix::void) {
        self.iov_base = base;
    }
    pub fn set_len(&mut self, len: usize) {
        self.iov_len = len;
    }
    pub fn as_mut_ptr(&mut self) -> *mut iovec {
        self
    }
}

#[repr(C)]
pub struct sockaddr {}
impl MemZeroedStruct for sockaddr {}

#[repr(C)]
pub struct sockaddr_un {
    pub sun_family: sa_family_t,
    pub sun_path: [c_char; SUN_PATH_LEN],
}
impl MemZeroedStruct for sockaddr_un {}

pub struct passwd {
    pub pw_name: *mut c_char,
    pub pw_uid: uid_t,
    pub pw_gid: gid_t,
    pub pw_dir: *mut c_char,
    pub pw_shell: *mut c_char,
    pub pw_gecos: *mut c_char,
    pub pw_passwd: *mut c_char,
}
impl MemZeroedStruct for passwd {}

pub struct group {
    pub gr_name: *mut c_char,
    pub gr_gid: gid_t,
    pub gr_mem: *mut *mut c_char,
    pub gr_passwd: *mut c_char,
}
impl MemZeroedStruct for group {}

pub type in_port_t = u16;
pub type in_addr_t = u32;

#[repr(C)]
pub struct in_addr {
    pub s_addr: in_addr_t,
}

#[repr(C)]
pub struct sockaddr_in {
    pub sin_family: sa_family_t,
    pub sin_port: in_port_t,
    pub sin_addr: in_addr,
    pub sin_zero: [u8; 8],
}
impl MemZeroedStruct for sockaddr_in {}

impl SockAddrIn for sockaddr_in {
    fn set_s_addr(&mut self, value: u32) {
        unimplemented!("set_s_addr")
    }

    fn get_s_addr(&self) -> u32 {
        unimplemented!("get_s_addr")
    }
}
