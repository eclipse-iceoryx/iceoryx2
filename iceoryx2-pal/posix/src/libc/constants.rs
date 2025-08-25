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

use crate::posix::types::*;

pub const CPU_SETSIZE: usize = libc::CPU_SETSIZE as _;
pub const FD_SETSIZE: usize = libc::FD_SETSIZE as _;
pub const NULL_TERMINATOR: c_char = 0;
pub const NAME_MAX: usize = 255;

#[cfg(target_os = "linux")]
pub const USER_NAME_LENGTH: usize = 255;
#[cfg(not(target_os = "linux"))]
pub const USER_NAME_LENGTH: usize = 31;

pub const GROUP_NAME_LENGTH: usize = 31;

pub const O_RDONLY: int = libc::O_RDONLY as _;
pub const O_WRONLY: int = libc::O_WRONLY as _;
pub const O_RDWR: int = libc::O_RDWR as _;
pub const O_SYNC: int = libc::O_SYNC as _;

pub const O_CREAT: int = libc::O_CREAT as _;
pub const O_EXCL: int = libc::O_EXCL as _;
pub const O_NOCTTY: int = libc::O_NOCTTY as _;
pub const O_APPEND: int = libc::O_APPEND as _;
pub const O_NONBLOCK: int = libc::O_NONBLOCK as _;
pub const O_DIRECTORY: int = libc::O_DIRECTORY as _;

pub const F_RDLCK: int = libc::F_RDLCK as _;
pub const F_WRLCK: int = libc::F_WRLCK as _;
pub const F_UNLCK: int = libc::F_UNLCK as _;
pub const F_GETFD: int = libc::F_GETFD as _;
pub const F_GETFL: int = libc::F_GETFL as _;
pub const F_SETFL: int = libc::F_SETFL as _;
pub const F_GETLK: int = libc::F_GETLK as _;
pub const F_SETLK: int = libc::F_SETLK as _;
pub const F_SETLKW: int = libc::F_SETLKW as _;

pub const PROT_NONE: int = libc::PROT_NONE as _;
pub const PROT_READ: int = libc::PROT_READ as _;
pub const PROT_WRITE: int = libc::PROT_WRITE as _;
pub const PROT_EXEC: int = libc::PROT_EXEC as _;
pub const MCL_CURRENT: int = libc::MCL_CURRENT as _;
pub const MCL_FUTURE: int = libc::MCL_FUTURE as _;
pub const MAP_SHARED: int = libc::MAP_SHARED as _;
pub const MAP_FAILED: *mut void = u64::MAX as *mut void;

pub const PTHREAD_BARRIER_SERIAL_THREAD: int = libc::PTHREAD_BARRIER_SERIAL_THREAD as _;
pub const PTHREAD_EXPLICIT_SCHED: int = libc::PTHREAD_EXPLICIT_SCHED as _;
pub const PTHREAD_INHERIT_SCHED: int = libc::PTHREAD_INHERIT_SCHED as _;

pub const MAX_SIGNAL_VALUE: usize = 32;

pub const SO_PASSCRED: int = libc::SO_PASSCRED as _;
pub const SO_PEERCRED: int = libc::SO_PEERCRED as _;
pub const SCM_CREDENTIALS: int = 0x02;

pub const PTHREAD_MUTEX_NORMAL: int = libc::PTHREAD_MUTEX_NORMAL as _;
pub const PTHREAD_MUTEX_RECURSIVE: int = libc::PTHREAD_MUTEX_RECURSIVE as _;
pub const PTHREAD_MUTEX_ERRORCHECK: int = libc::PTHREAD_MUTEX_ERRORCHECK as _;
pub const PTHREAD_MUTEX_STALLED: int = libc::PTHREAD_MUTEX_STALLED as _;
pub const PTHREAD_MUTEX_ROBUST: int = libc::PTHREAD_MUTEX_ROBUST as _;

pub const _SC_UIO_MAXIOV: int = libc::_SC_UIO_MAXIOV as _;
pub const _SC_IOV_MAX: int = libc::_SC_IOV_MAX as _;
pub const _SC_AVPHYS_PAGES: int = libc::_SC_AVPHYS_PAGES as _;
pub const _SC_PASS_MAX: int = libc::_SC_PASS_MAX as _;
pub const _SC_XOPEN_XPG2: int = libc::_SC_XOPEN_XPG2 as _;
pub const _SC_XOPEN_XPG3: int = libc::_SC_XOPEN_XPG3 as _;
pub const _SC_XOPEN_XPG4: int = libc::_SC_XOPEN_XPG4 as _;
pub const _SC_NZERO: int = libc::_SC_NZERO as _;
pub const _SC_XBS5_ILP32_OFF32: int = libc::_SC_XBS5_ILP32_OFF32 as _;
pub const _SC_XBS5_ILP32_OFFBIG: int = libc::_SC_XBS5_ILP32_OFFBIG as _;
pub const _SC_XBS5_LP64_OFF64: int = libc::_SC_XBS5_LP64_OFF64 as _;
pub const _SC_XBS5_LPBIG_OFFBIG: int = libc::_SC_XBS5_LPBIG_OFFBIG as _;
pub const _SC_STREAMS: int = libc::_SC_STREAMS as _;
pub const _SC_V7_ILP32_OFF32: int = libc::_SC_V7_ILP32_OFF32 as _;
pub const _SC_V7_ILP32_OFFBIG: int = libc::_SC_V7_ILP32_OFFBIG as _;
pub const _SC_V7_LP64_OFF64: int = libc::_SC_V7_LP64_OFF64 as _;
pub const _SC_V7_LPBIG_OFFBIG: int = libc::_SC_V7_LPBIG_OFFBIG as _;
pub const _SC_SS_REPL_MAX: int = libc::_SC_SS_REPL_MAX as _;
pub const _SC_TRACE_EVENT_NAME_MAX: int = libc::_SC_TRACE_EVENT_NAME_MAX as _;
pub const _SC_TRACE_NAME_MAX: int = libc::_SC_TRACE_NAME_MAX as _;
pub const _SC_TRACE_SYS_MAX: int = libc::_SC_TRACE_SYS_MAX as _;
pub const _SC_THREAD_ROBUST_PRIO_INHERIT: int = libc::_SC_THREAD_ROBUST_PRIO_INHERIT as _;
pub const _SC_THREAD_ROBUST_PRIO_PROTECT: int = libc::_SC_THREAD_ROBUST_PRIO_PROTECT as _;
pub const _PC_SOCK_MAXBUF: int = libc::_PC_SOCK_MAXBUF as _;
pub const _PC_2_SYMLINKS: int = libc::_PC_2_SYMLINKS as _;
pub const _SC_TRACE_USER_EVENT_MAX: int = libc::_SC_TRACE_USER_EVENT_MAX as _;

pub const PTHREAD_PROCESS_PRIVATE: int = libc::PTHREAD_PROCESS_PRIVATE as _;
pub const PTHREAD_PROCESS_SHARED: int = libc::PTHREAD_PROCESS_SHARED as _;
pub const PTHREAD_PRIO_NONE: int = libc::PTHREAD_PRIO_NONE as _;
pub const PTHREAD_PRIO_INHERIT: int = libc::PTHREAD_PRIO_INHERIT as _;
pub const PTHREAD_PRIO_PROTECT: int = libc::PTHREAD_PRIO_PROTECT as _;

pub const RLIMIT_CPU: __rlim_t = 0;
pub const RLIMIT_FSIZE: __rlim_t = 1;
pub const RLIMIT_DATA: __rlim_t = 2;
pub const RLIMIT_STACK: __rlim_t = 3;
pub const RLIMIT_CORE: __rlim_t = 4;
pub const RLIMIT_RSS: __rlim_t = 5;
pub const RLIMIT_NPROC: __rlim_t = 6;
pub const RLIMIT_NOFILE: __rlim_t = 7;
pub const RLIMIT_MEMLOCK: __rlim_t = 8;
pub const RLIMIT_AS: __rlim_t = 9;
pub const RLIMIT_LOCKS: __rlim_t = 10;
pub const RLIMIT_SIGPENDING: __rlim_t = 11;
pub const RLIMIT_MSGQUEUE: __rlim_t = 12;
pub const RLIMIT_NICE: __rlim_t = 13;
pub const RLIMIT_RTPRIO: __rlim_t = 14;
pub const RLIMIT_RTTIME: __rlim_t = 15;
pub const RLIMIT_NLIMITS: __rlim_t = 16;
pub const RLIMIT_INFINITY: __rlim_t = __rlim_t::MAX;

pub const SCHED_OTHER: int = libc::SCHED_OTHER as _;
pub const SCHED_FIFO: int = libc::SCHED_FIFO as _;
pub const SCHED_RR: int = libc::SCHED_RR as _;

pub const SEEK_SET: int = libc::SEEK_SET as _;
pub const SEEK_CUR: int = libc::SEEK_CUR as _;
pub const SEEK_END: int = libc::SEEK_END as _;

pub const SEM_FAILED: *mut sem_t = 0 as *mut sem_t;

pub const SIGABRT: int = libc::SIGABRT as _;
pub const SIGALRM: int = libc::SIGALRM as _;
pub const SIGBUS: int = libc::SIGBUS as _;
pub const SIGCHLD: int = libc::SIGCHLD as _;
pub const SIGCONT: int = libc::SIGCONT as _;
pub const SIGFPE: int = libc::SIGFPE as _;
pub const SIGHUP: int = libc::SIGHUP as _;
pub const SIGILL: int = libc::SIGILL as _;
pub const SIGINT: int = libc::SIGINT as _;
pub const SIGKILL: int = libc::SIGKILL as _;
pub const SIGPIPE: int = libc::SIGPIPE as _;
pub const SIGQUIT: int = libc::SIGQUIT as _;
pub const SIGSEGV: int = libc::SIGSEGV as _;
pub const SIGSTOP: int = libc::SIGSTOP as _;
pub const SIGTERM: int = libc::SIGTERM as _;
pub const SIGTSTP: int = libc::SIGTSTP as _;
pub const SIGTTIN: int = libc::SIGTTIN as _;
pub const SIGTTOU: int = libc::SIGTTOU as _;
pub const SIGUSR1: int = libc::SIGUSR1 as _;
pub const SIGUSR2: int = libc::SIGUSR2 as _;
pub const SIGPROF: int = libc::SIGPROF as _;
pub const SIGSYS: int = libc::SIGSYS as _;
pub const SIGTRAP: int = libc::SIGTRAP as _;
pub const SIGURG: int = libc::SIGURG as _;
pub const SIGVTALRM: int = libc::SIGVTALRM as _;
pub const SIGXCPU: int = libc::SIGXCPU as _;
pub const SIGXFSZ: int = libc::SIGXFSZ as _;
pub const SIG_ERR: sighandler_t = sighandler_t::MAX;
pub const SIG_DFL: int = 0;
pub const SIG_IGN: int = 1;
pub const SA_RESTART: int = libc::SA_RESTART as _;

pub const AF_LOCAL: sa_family_t = libc::AF_UNIX as _;
pub const AF_UNIX: sa_family_t = libc::AF_UNIX as _;
pub const AF_INET: sa_family_t = libc::AF_INET as _;
pub const PF_INET: sa_family_t = libc::PF_INET as _;
pub const PF_LOCAL: sa_family_t = libc::AF_UNIX as _;
pub const PF_UNIX: sa_family_t = libc::AF_UNIX as _;
pub const INADDR_ANY: in_addr_t = 0;
pub const SO_SNDBUF: int = libc::SO_SNDBUF as _;
pub const SO_RCVBUF: int = libc::SO_RCVBUF as _;
pub const SO_RCVTIMEO: int = libc::SO_RCVTIMEO as _;
pub const SO_SNDTIMEO: int = libc::SO_SNDTIMEO as _;
pub const SOCK_STREAM: int = libc::SOCK_STREAM as _;
pub const SOCK_DGRAM: int = libc::SOCK_DGRAM as _;
pub const IPPROTO_UDP: int = libc::IPPROTO_UDP as _;
pub const SOCK_NONBLOCK: int = O_NONBLOCK;
pub const MSG_PEEK: int = libc::MSG_PEEK as _;
pub const SCM_MAX_FD: u32 = 253;
pub const SCM_RIGHTS: int = libc::SCM_RIGHTS as _;
pub const SOL_SOCKET: int = libc::SOL_SOCKET as _;
pub const SUN_PATH_LEN: usize = 108;
pub const SA_DATA_LEN: usize = 14;

pub const S_IFMT: mode_t = libc::S_IFMT as _;
pub const S_IFSOCK: mode_t = libc::S_IFSOCK as _;
pub const S_IFLNK: mode_t = libc::S_IFLNK as _;
pub const S_IFREG: mode_t = libc::S_IFREG as _;
pub const S_IFBLK: mode_t = libc::S_IFBLK as _;
pub const S_IFDIR: mode_t = libc::S_IFDIR as _;
pub const S_IFCHR: mode_t = libc::S_IFCHR as _;
pub const S_IFIFO: mode_t = libc::S_IFIFO as _;
pub const S_IRWXU: mode_t = libc::S_IRWXU as _;
pub const S_IXUSR: mode_t = libc::S_IXUSR as _;
pub const S_IWUSR: mode_t = libc::S_IWUSR as _;
pub const S_IRUSR: mode_t = libc::S_IRUSR as _;
pub const S_IRWXG: mode_t = libc::S_IRWXG as _;
pub const S_IXGRP: mode_t = libc::S_IXGRP as _;
pub const S_IWGRP: mode_t = libc::S_IWGRP as _;
pub const S_IRGRP: mode_t = libc::S_IRGRP as _;
pub const S_IRWXO: mode_t = libc::S_IRWXO as _;
pub const S_IXOTH: mode_t = libc::S_IXOTH as _;
pub const S_IWOTH: mode_t = libc::S_IWOTH as _;
pub const S_IROTH: mode_t = libc::S_IROTH as _;
pub const S_ISUID: mode_t = libc::S_ISUID as _;
pub const S_ISGID: mode_t = libc::S_ISGID as _;
pub const S_ISVTX: mode_t = libc::S_ISVTX as _;

pub const CLOCK_REALTIME: clockid_t = libc::CLOCK_REALTIME as _;
pub const CLOCK_MONOTONIC: clockid_t = libc::CLOCK_MONOTONIC as _;
pub const CLOCK_TIMER_ABSTIME: int = 1;

pub const F_OK: int = libc::F_OK as _;
pub const R_OK: int = libc::R_OK as _;
pub const W_OK: int = libc::W_OK as _;
pub const X_OK: int = libc::X_OK as _;

pub const _SC_ARG_MAX: int = libc::_SC_ARG_MAX as _;
pub const _SC_CHILD_MAX: int = libc::_SC_CHILD_MAX as _;
pub const _SC_CLK_TCK: int = libc::_SC_CLK_TCK as _;
pub const _SC_NGROUPS_MAX: int = libc::_SC_NGROUPS_MAX as _;
pub const _SC_OPEN_MAX: int = libc::_SC_OPEN_MAX as _;
pub const _SC_STREAM_MAX: int = libc::_SC_STREAM_MAX as _;
pub const _SC_TZNAME_MAX: int = libc::_SC_TZNAME_MAX as _;
pub const _SC_JOB_CONTROL: int = libc::_SC_JOB_CONTROL as _;
pub const _SC_SAVED_IDS: int = libc::_SC_SAVED_IDS as _;
pub const _SC_REALTIME_SIGNALS: int = libc::_SC_REALTIME_SIGNALS as _;
pub const _SC_PRIORITY_SCHEDULING: int = libc::_SC_PRIORITY_SCHEDULING as _;
pub const _SC_TIMERS: int = libc::_SC_TIMERS as _;
pub const _SC_ASYNCHRONOUS_IO: int = libc::_SC_ASYNCHRONOUS_IO as _;
pub const _SC_PRIORITIZED_IO: int = libc::_SC_PRIORITIZED_IO as _;
pub const _SC_SYNCHRONIZED_IO: int = libc::_SC_SYNCHRONIZED_IO as _;
pub const _SC_FSYNC: int = libc::_SC_FSYNC as _;
pub const _SC_MAPPED_FILES: int = libc::_SC_MAPPED_FILES as _;
pub const _SC_MEMLOCK: int = libc::_SC_MEMLOCK as _;
pub const _SC_MEMLOCK_RANGE: int = libc::_SC_MEMLOCK_RANGE as _;
pub const _SC_MEMORY_PROTECTION: int = libc::_SC_MEMORY_PROTECTION as _;
pub const _SC_MESSAGE_PASSING: int = libc::_SC_MESSAGE_PASSING as _;
pub const _SC_SEMAPHORES: int = libc::_SC_SEMAPHORES as _;
pub const _SC_SHARED_MEMORY_OBJECTS: int = libc::_SC_SHARED_MEMORY_OBJECTS as _;
pub const _SC_AIO_LISTIO_MAX: int = libc::_SC_AIO_LISTIO_MAX as _;
pub const _SC_AIO_MAX: int = libc::_SC_AIO_MAX as _;
pub const _SC_AIO_PRIO_DELTA_MAX: int = libc::_SC_AIO_PRIO_DELTA_MAX as _;
pub const _SC_DELAYTIMER_MAX: int = libc::_SC_DELAYTIMER_MAX as _;
pub const _SC_MQ_OPEN_MAX: int = libc::_SC_MQ_OPEN_MAX as _;
pub const _SC_MQ_PRIO_MAX: int = libc::_SC_MQ_PRIO_MAX as _;
pub const _SC_VERSION: int = libc::_SC_VERSION as _;
pub const _SC_PAGESIZE: int = libc::_SC_PAGESIZE as _;
pub const _SC_RTSIG_MAX: int = libc::_SC_RTSIG_MAX as _;
pub const _SC_SEM_NSEMS_MAX: int = libc::_SC_SEM_NSEMS_MAX as _;
pub const _SC_SEM_VALUE_MAX: int = libc::_SC_SEM_VALUE_MAX as _;
pub const _SC_SIGQUEUE_MAX: int = libc::_SC_SIGQUEUE_MAX as _;
pub const _SC_TIMER_MAX: int = libc::_SC_TIMER_MAX as _;
pub const _SC_BC_BASE_MAX: int = libc::_SC_BC_BASE_MAX as _;
pub const _SC_BC_DIM_MAX: int = libc::_SC_BC_DIM_MAX as _;
pub const _SC_BC_SCALE_MAX: int = libc::_SC_BC_SCALE_MAX as _;
pub const _SC_BC_STRING_MAX: int = libc::_SC_BC_STRING_MAX as _;
pub const _SC_COLL_WEIGHTS_MAX: int = libc::_SC_COLL_WEIGHTS_MAX as _;
pub const _SC_EXPR_NEST_MAX: int = libc::_SC_EXPR_NEST_MAX as _;
pub const _SC_LINE_MAX: int = libc::_SC_LINE_MAX as _;
pub const _SC_RE_DUP_MAX: int = libc::_SC_RE_DUP_MAX as _;
pub const _SC_2_VERSION: int = libc::_SC_2_VERSION as _;
pub const _SC_2_C_BIND: int = libc::_SC_2_C_BIND as _;
pub const _SC_2_C_DEV: int = libc::_SC_2_C_DEV as _;
pub const _SC_2_FORT_DEV: int = libc::_SC_2_FORT_DEV as _;
pub const _SC_2_FORT_RUN: int = libc::_SC_2_FORT_RUN as _;
pub const _SC_2_SW_DEV: int = libc::_SC_2_SW_DEV as _;
pub const _SC_2_LOCALEDEF: int = libc::_SC_2_LOCALEDEF as _;
pub const _SC_THREADS: int = libc::_SC_THREADS as _;
pub const _SC_THREAD_SAFE_FUNCTIONS: int = libc::_SC_THREAD_SAFE_FUNCTIONS as _;
pub const _SC_GETGR_R_SIZE_MAX: int = libc::_SC_GETGR_R_SIZE_MAX as _;
pub const _SC_GETPW_R_SIZE_MAX: int = libc::_SC_GETPW_R_SIZE_MAX as _;
pub const _SC_LOGIN_NAME_MAX: int = libc::_SC_LOGIN_NAME_MAX as _;
pub const _SC_TTY_NAME_MAX: int = libc::_SC_TTY_NAME_MAX as _;
pub const _SC_THREAD_DESTRUCTOR_ITERATIONS: int = libc::_SC_THREAD_DESTRUCTOR_ITERATIONS as _;
pub const _SC_THREAD_KEYS_MAX: int = libc::_SC_THREAD_KEYS_MAX as _;
pub const _SC_THREAD_STACK_MIN: int = libc::_SC_THREAD_STACK_MIN as _;
pub const _SC_THREAD_THREADS_MAX: int = libc::_SC_THREAD_THREADS_MAX as _;
pub const _SC_THREAD_ATTR_STACKADDR: int = libc::_SC_THREAD_ATTR_STACKADDR as _;
pub const _SC_THREAD_ATTR_STACKSIZE: int = libc::_SC_THREAD_ATTR_STACKSIZE as _;
pub const _SC_THREAD_PRIORITY_SCHEDULING: int = libc::_SC_THREAD_PRIORITY_SCHEDULING as _;
pub const _SC_THREAD_PRIO_INHERIT: int = libc::_SC_THREAD_PRIO_INHERIT as _;
pub const _SC_THREAD_PRIO_PROTECT: int = libc::_SC_THREAD_PRIO_PROTECT as _;
pub const _SC_THREAD_PROCESS_SHARED: int = libc::_SC_THREAD_PROCESS_SHARED as _;
pub const _SC_NPROCESSORS_CONF: int = libc::_SC_NPROCESSORS_CONF as _;
pub const _SC_NPROCESSORS_ONLN: int = libc::_SC_NPROCESSORS_ONLN as _;
pub const _SC_PHYS_PAGES: int = libc::_SC_PHYS_PAGES as _;
pub const _SC_ATEXIT_MAX: int = libc::_SC_ATEXIT_MAX as _;
pub const _SC_XOPEN_VERSION: int = libc::_SC_XOPEN_VERSION as _;
pub const _SC_XOPEN_XCU_VERSION: int = libc::_SC_XOPEN_XCU_VERSION as _;
pub const _SC_XOPEN_UNIX: int = libc::_SC_XOPEN_UNIX as _;
pub const _SC_XOPEN_CRYPT: int = libc::_SC_XOPEN_CRYPT as _;
pub const _SC_XOPEN_ENH_I18N: int = libc::_SC_XOPEN_ENH_I18N as _;
pub const _SC_XOPEN_SHM: int = libc::_SC_XOPEN_SHM as _;
pub const _SC_2_CHAR_TERM: int = libc::_SC_2_CHAR_TERM as _;
pub const _SC_2_UPE: int = libc::_SC_2_UPE as _;
pub const _SC_XOPEN_LEGACY: int = libc::_SC_XOPEN_LEGACY as _;
pub const _SC_XOPEN_REALTIME: int = libc::_SC_XOPEN_REALTIME as _;
pub const _SC_XOPEN_REALTIME_THREADS: int = libc::_SC_XOPEN_REALTIME_THREADS as _;
pub const _SC_ADVISORY_INFO: int = libc::_SC_ADVISORY_INFO as _;
pub const _SC_BARRIERS: int = libc::_SC_BARRIERS as _;
pub const _SC_CLOCK_SELECTION: int = libc::_SC_CLOCK_SELECTION as _;
pub const _SC_CPUTIME: int = libc::_SC_CPUTIME as _;
pub const _SC_THREAD_CPUTIME: int = libc::_SC_THREAD_CPUTIME as _;
pub const _SC_MONOTONIC_CLOCK: int = libc::_SC_MONOTONIC_CLOCK as _;
pub const _SC_READER_WRITER_LOCKS: int = libc::_SC_READER_WRITER_LOCKS as _;
pub const _SC_SPIN_LOCKS: int = libc::_SC_SPIN_LOCKS as _;
pub const _SC_REGEXP: int = libc::_SC_REGEXP as _;
pub const _SC_SHELL: int = libc::_SC_SHELL as _;
pub const _SC_SPAWN: int = libc::_SC_SPAWN as _;
pub const _SC_SPORADIC_SERVER: int = libc::_SC_SPORADIC_SERVER as _;
pub const _SC_THREAD_SPORADIC_SERVER: int = libc::_SC_THREAD_SPORADIC_SERVER as _;
pub const _SC_TIMEOUTS: int = libc::_SC_TIMEOUTS as _;
pub const _SC_TYPED_MEMORY_OBJECTS: int = libc::_SC_TYPED_MEMORY_OBJECTS as _;
pub const _SC_2_PBS: int = libc::_SC_2_PBS as _;
pub const _SC_2_PBS_ACCOUNTING: int = libc::_SC_2_PBS_ACCOUNTING as _;
pub const _SC_2_PBS_LOCATE: int = libc::_SC_2_PBS_LOCATE as _;
pub const _SC_2_PBS_MESSAGE: int = libc::_SC_2_PBS_MESSAGE as _;
pub const _SC_2_PBS_TRACK: int = libc::_SC_2_PBS_TRACK as _;
pub const _SC_SYMLOOP_MAX: int = libc::_SC_SYMLOOP_MAX as _;
pub const _SC_2_PBS_CHECKPOINT: int = libc::_SC_2_PBS_CHECKPOINT as _;
pub const _SC_V6_ILP32_OFF32: int = libc::_SC_V6_ILP32_OFF32 as _;
pub const _SC_V6_ILP32_OFFBIG: int = libc::_SC_V6_ILP32_OFFBIG as _;
pub const _SC_V6_LP64_OFF64: int = libc::_SC_V6_LP64_OFF64 as _;
pub const _SC_V6_LPBIG_OFFBIG: int = libc::_SC_V6_LPBIG_OFFBIG as _;
pub const _SC_HOST_NAME_MAX: int = libc::_SC_HOST_NAME_MAX as _;
pub const _SC_TRACE: int = libc::_SC_TRACE as _;
pub const _SC_TRACE_EVENT_FILTER: int = libc::_SC_TRACE_EVENT_FILTER as _;
pub const _SC_TRACE_INHERIT: int = libc::_SC_TRACE_INHERIT as _;
pub const _SC_TRACE_LOG: int = libc::_SC_TRACE_LOG as _;
pub const _SC_IPV6: int = libc::_SC_IPV6 as _;
pub const _SC_RAW_SOCKETS: int = libc::_SC_RAW_SOCKETS as _;
pub const _SC_XOPEN_STREAMS: int = libc::_SC_XOPEN_STREAMS as _;

pub const _PC_LINK_MAX: int = libc::_PC_LINK_MAX as _;
pub const _PC_MAX_CANON: int = libc::_PC_MAX_CANON as _;
pub const _PC_MAX_INPUT: int = libc::_PC_MAX_INPUT as _;
pub const _PC_NAME_MAX: int = libc::_PC_NAME_MAX as _;
pub const _PC_PATH_MAX: int = libc::_PC_PATH_MAX as _;
pub const _PC_PIPE_BUF: int = libc::_PC_PIPE_BUF as _;
pub const _PC_CHOWN_RESTRICTED: int = libc::_PC_CHOWN_RESTRICTED as _;
pub const _PC_NO_TRUNC: int = libc::_PC_NO_TRUNC as _;
pub const _PC_VDISABLE: int = libc::_PC_VDISABLE as _;
pub const _PC_SYNC_IO: int = libc::_PC_SYNC_IO as _;
pub const _PC_ASYNC_IO: int = libc::_PC_ASYNC_IO as _;
pub const _PC_PRIO_IO: int = libc::_PC_PRIO_IO as _;
pub const _PC_FILESIZEBITS: int = libc::_PC_FILESIZEBITS as _;
pub const _PC_REC_INCR_XFER_SIZE: int = libc::_PC_REC_INCR_XFER_SIZE as _;
pub const _PC_REC_MAX_XFER_SIZE: int = libc::_PC_REC_MAX_XFER_SIZE as _;
pub const _PC_REC_MIN_XFER_SIZE: int = libc::_PC_REC_MIN_XFER_SIZE as _;
pub const _PC_REC_XFER_ALIGN: int = libc::_PC_REC_XFER_ALIGN as _;
pub const _PC_ALLOC_SIZE_MIN: int = libc::_PC_ALLOC_SIZE_MIN as _;
pub const _PC_SYMLINK_MAX: int = libc::_PC_SYMLINK_MAX as _;
