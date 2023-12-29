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

use crate::posix::{SockAddrIn, Struct};

pub type ulong = crate::internal::u_long;
pub type kinfo_file = crate::internal::kinfo_file;

#[repr(C)]
pub struct ucred {
    pub pid: pid_t,
    pub uid: uid_t,
    pub euid: uid_t,
    pub gid: gid_t,
}

impl Struct for ucred {}

pub type DIR = crate::internal::DIR;

pub type blkcnt_t = crate::internal::blkcnt_t;
pub type blksize_t = crate::internal::blksize_t;
pub type char = core::ffi::c_char;
pub type clockid_t = crate::internal::clockid_t;
pub type dev_t = crate::internal::dev_t;
pub type gid_t = crate::internal::gid_t;
pub type ino_t = crate::internal::ino_t;
pub type int = core::ffi::c_int;
pub type in_port_t = u16;
pub type in_addr_t = u32;
pub type long = core::ffi::c_long;
pub type mode_t = crate::internal::mode_t;
pub type mqd_t = crate::internal::mqd_t;
pub type nlink_t = crate::internal::nlink_t;
pub type off_t = crate::internal::off_t;
pub type pid_t = crate::internal::pid_t;
pub type rlim_t = crate::internal::rlim_t;
pub type __rlim_t = crate::internal::__rlim_t;
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
impl Struct for sigset_t {}

pub type pthread_barrier_t = crate::internal::pthread_barrier_t;
impl Struct for pthread_barrier_t {}

pub type pthread_barrierattr_t = crate::internal::pthread_barrierattr_t;
impl Struct for pthread_barrierattr_t {}

pub type pthread_attr_t = crate::internal::pthread_attr_t;
impl Struct for pthread_attr_t {}

pub type pthread_t = crate::internal::pthread_t;
impl Struct for pthread_t {}

pub type pthread_rwlockattr_t = crate::internal::pthread_rwlockattr_t;
impl Struct for pthread_rwlockattr_t {}

pub type pthread_rwlock_t = crate::internal::pthread_rwlock_t;
impl Struct for pthread_rwlock_t {}

pub type pthread_cond_t = crate::internal::pthread_cond_t;
impl Struct for pthread_cond_t {}

pub type pthread_condattr_t = crate::internal::pthread_condattr_t;
impl Struct for pthread_condattr_t {}

pub type pthread_mutex_t = crate::internal::pthread_mutex_t;
impl Struct for pthread_mutex_t {}

pub type pthread_mutexattr_t = crate::internal::pthread_mutexattr_t;
impl Struct for pthread_mutexattr_t {}

pub type sem_t = crate::internal::sem_t;
impl Struct for sem_t {}

pub type flock = crate::internal::flock;
impl Struct for flock {}

pub type mq_attr = crate::internal::mq_attr;
impl Struct for mq_attr {}

pub type rlimit = crate::internal::rlimit;
impl Struct for rlimit {}

pub type sched_param = crate::internal::sched_param;
impl Struct for sched_param {}

#[repr(C)]
pub struct sigaction_t {
    pub sa_handler: sighandler_t,
    pub sa_mask: sigset_t,
    pub sa_flags: int,
    pub sa_restorer: Option<extern "C" fn()>,
}
impl Struct for sigaction_t {
    fn new() -> Self {
        Self {
            sa_handler: 0,
            sa_mask: sigset_t::new(),
            sa_flags: 0,
            sa_restorer: None,
        }
    }
}

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
impl From<crate::internal::stat> for stat_t {
    fn from(value: crate::internal::stat) -> Self {
        stat_t {
            st_dev: value.st_dev,
            st_ino: value.st_ino,
            st_nlink: value.st_nlink,
            st_mode: value.st_mode,
            st_uid: value.st_uid,
            st_gid: value.st_gid,
            st_rdev: value.st_rdev,
            st_size: value.st_size,
            st_atime: value.st_atim.tv_sec,
            st_mtime: value.st_mtim.tv_sec,
            st_ctime: value.st_ctim.tv_sec,
            st_blksize: value.st_blksize,
            st_blocks: value.st_blocks,
        }
    }
}
impl Struct for stat_t {}
impl Struct for crate::internal::stat {}

pub type timespec = crate::internal::timespec;
impl Struct for timespec {}

pub type timeval = crate::internal::timeval;
impl Struct for timeval {}

pub type fd_set = crate::internal::fd_set;
impl Struct for fd_set {}

pub type dirent = crate::internal::dirent;
impl Struct for dirent {}

pub type msghdr = crate::internal::msghdr;
impl Struct for msghdr {}

pub type cmsghdr = crate::internal::cmsghdr;
impl Struct for cmsghdr {}

pub type iovec = crate::internal::iovec;
impl Struct for iovec {}

pub type sockaddr = crate::internal::sockaddr;
impl Struct for sockaddr {}

pub type sockaddr_un = crate::internal::sockaddr_un;
impl Struct for sockaddr_un {}

pub type sockaddr_in = crate::internal::sockaddr_in;
impl Struct for sockaddr_in {}

impl SockAddrIn for sockaddr_in {
    fn set_s_addr(&mut self, value: u32) {
        self.sin_addr.s_addr = value;
    }

    fn get_s_addr(&self) -> u32 {
        self.sin_addr.s_addr
    }
}

pub type passwd = crate::internal::passwd;
impl Struct for passwd {}

pub type group = crate::internal::group;
impl Struct for group {}
