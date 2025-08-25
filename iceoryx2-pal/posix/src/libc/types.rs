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

use crate::common::mem_zeroed_struct::MemZeroedStruct;
use crate::posix::SockAddrIn;
pub type ulong = libc::c_ulong;

#[repr(C)]
pub struct ucred {
    pub pid: pid_t,
    pub uid: uid_t,
    pub gid: gid_t,
}

impl MemZeroedStruct for ucred {}

pub type DIR = libc::DIR;

pub type blkcnt_t = libc::blkcnt_t;
pub type blksize_t = libc::blksize_t;
pub type c_char = core::ffi::c_char;
pub type clockid_t = libc::clockid_t;
pub type dev_t = libc::dev_t;
pub type gid_t = libc::gid_t;
pub type ino_t = libc::ino_t;
pub type int = core::ffi::c_int;
pub type in_port_t = u16;
pub type in_addr_t = u32;
pub type long = core::ffi::c_long;
pub type mode_t = libc::mode_t;
pub type nlink_t = libc::nlink_t;
pub type off_t = libc::off_t;
pub type pid_t = libc::pid_t;
pub type rlim_t = libc::rlim_t;
pub type __rlim_t = libc::rlim_t;
pub type sa_family_t = libc::sa_family_t;
pub type short = core::ffi::c_short;
pub type sighandler_t = size_t;
pub type size_t = usize;
pub type socklen_t = libc::socklen_t;
pub type ssize_t = isize;
pub type suseconds_t = libc::suseconds_t;
pub type time_t = libc::time_t;
pub type uchar = core::ffi::c_uchar;
pub type uid_t = libc::uid_t;
pub type uint = libc::c_uint;
pub type ushort = libc::c_ushort;
pub type void = core::ffi::c_void;

pub(crate) type native_cpu_set_t = libc::cpu_set_t;
impl MemZeroedStruct for native_cpu_set_t {}

pub type sigset_t = libc::sigset_t;
impl MemZeroedStruct for sigset_t {}

pub type pthread_barrier_t = libc::pthread_barrier_t;
impl MemZeroedStruct for pthread_barrier_t {}

pub type pthread_barrierattr_t = libc::pthread_barrierattr_t;
impl MemZeroedStruct for pthread_barrierattr_t {}

pub type pthread_attr_t = libc::pthread_attr_t;
impl MemZeroedStruct for pthread_attr_t {}

pub type pthread_t = libc::pthread_t;
impl MemZeroedStruct for pthread_t {}

pub type pthread_rwlockattr_t = libc::pthread_rwlockattr_t;
impl MemZeroedStruct for pthread_rwlockattr_t {}

pub type pthread_rwlock_t = libc::pthread_rwlock_t;
impl MemZeroedStruct for pthread_rwlock_t {}

pub type pthread_mutex_t = libc::pthread_mutex_t;
impl MemZeroedStruct for pthread_mutex_t {}

pub type pthread_mutexattr_t = libc::pthread_mutexattr_t;
impl MemZeroedStruct for pthread_mutexattr_t {}

pub type sem_t = libc::sem_t;
impl MemZeroedStruct for sem_t {}

pub type flock = libc::flock;
impl MemZeroedStruct for flock {}

pub type rlimit = libc::rlimit;
impl MemZeroedStruct for rlimit {}

pub type sched_param = libc::sched_param;
impl MemZeroedStruct for sched_param {}

pub(crate) type native_stat_t = libc::stat;
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
            st_atime: value.st_atime,
            st_mtime: value.st_mtime,
            st_ctime: value.st_ctime,
            st_blksize: value.st_blksize,
            st_blocks: value.st_blocks,
        }
    }
}
impl MemZeroedStruct for stat_t {}

pub type timespec = libc::timespec;
impl MemZeroedStruct for timespec {}

pub type timeval = libc::timeval;
impl MemZeroedStruct for timeval {}

pub type fd_set = libc::fd_set;
impl MemZeroedStruct for fd_set {}

pub type dirent = libc::dirent;
impl MemZeroedStruct for dirent {}

pub type msghdr = libc::msghdr;
impl MemZeroedStruct for msghdr {}

pub type cmsghdr = libc::cmsghdr;
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

pub type sockaddr = libc::sockaddr;
impl MemZeroedStruct for sockaddr {}

pub type sockaddr_un = libc::sockaddr_un;
impl MemZeroedStruct for sockaddr_un {}

pub type sockaddr_in = libc::sockaddr_in;
impl MemZeroedStruct for sockaddr_in {}

impl SockAddrIn for sockaddr_in {
    fn set_s_addr(&mut self, value: u32) {
        self.sin_addr.s_addr = value;
    }

    fn get_s_addr(&self) -> u32 {
        self.sin_addr.s_addr
    }
}

pub type passwd = libc::passwd;
impl MemZeroedStruct for passwd {}

pub type group = libc::group;
impl MemZeroedStruct for group {}
