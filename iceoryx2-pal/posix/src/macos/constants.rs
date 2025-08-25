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

pub const CPU_SETSIZE: usize = 16;
pub const MAX_NUMBER_OF_THREADS: usize = 1024;
pub const FD_SETSIZE: usize = crate::internal::FD_SETSIZE as _;
pub const THREAD_NAME_LENGTH: usize = 16;
pub const NULL_TERMINATOR: c_char = 0;
pub const USER_NAME_LENGTH: usize = 31;
pub const GROUP_NAME_LENGTH: usize = 31;
pub const NAME_MAX: usize = crate::internal::NAME_MAX as _;

pub const O_RDONLY: int = crate::internal::O_RDONLY as _;
pub const O_WRONLY: int = crate::internal::O_WRONLY as _;
pub const O_RDWR: int = crate::internal::O_RDWR as _;
pub const O_SYNC: int = crate::internal::O_SYNC as _;

pub const O_CREAT: int = crate::internal::O_CREAT as _;
pub const O_EXCL: int = crate::internal::O_EXCL as _;
pub const O_NOCTTY: int = crate::internal::O_NOCTTY as _;
pub const O_APPEND: int = crate::internal::O_APPEND as _;
pub const O_NONBLOCK: int = crate::internal::O_NONBLOCK as _;
pub const O_DIRECTORY: int = crate::internal::O_DIRECTORY as _;

pub const F_RDLCK: int = crate::internal::F_RDLCK as _;
pub const F_WRLCK: int = crate::internal::F_WRLCK as _;
pub const F_UNLCK: int = crate::internal::F_UNLCK as _;
pub const F_GETFD: int = crate::internal::F_GETFD as _;
pub const F_GETFL: int = crate::internal::F_GETFL as _;
pub const F_SETFL: int = crate::internal::F_SETFL as _;
pub const F_GETLK: int = crate::internal::F_GETLK as _;
pub const F_SETLK: int = crate::internal::F_SETLK as _;
pub const F_SETLKW: int = crate::internal::F_SETLKW as _;

pub const PROT_NONE: int = crate::internal::PROT_NONE as _;
pub const PROT_READ: int = crate::internal::PROT_READ as _;
pub const PROT_WRITE: int = crate::internal::PROT_WRITE as _;
pub const PROT_EXEC: int = crate::internal::PROT_EXEC as _;
pub const MCL_CURRENT: int = crate::internal::MCL_CURRENT as _;
pub const MCL_FUTURE: int = crate::internal::MCL_FUTURE as _;
pub const MAP_SHARED: int = crate::internal::MAP_SHARED as _;
pub const MAP_FAILED: *mut void = u64::MAX as *mut void;

pub const PTHREAD_BARRIER_SERIAL_THREAD: int = int::MAX;
pub const PTHREAD_EXPLICIT_SCHED: int = crate::internal::PTHREAD_EXPLICIT_SCHED as _;
pub const PTHREAD_INHERIT_SCHED: int = crate::internal::PTHREAD_INHERIT_SCHED as _;

pub const MAX_SIGNAL_VALUE: usize = 34;

pub const SO_PASSCRED: int = crate::internal::LOCAL_PEERCRED as _;
pub const SO_PEERCRED: int = crate::internal::LOCAL_PEERCRED as _;
pub const SCM_CREDENTIALS: int = crate::internal::SCM_CREDS as _;

pub const PTHREAD_PREFER_READER_NP: int = 0;
pub const PTHREAD_PREFER_WRITER_NP: int = 1;
pub const PTHREAD_PREFER_WRITER_NONRECURSIVE_NP: int = 2;

pub const PTHREAD_MUTEX_STALLED: int = 32;
pub const PTHREAD_MUTEX_ROBUST: int = 64;
pub const PTHREAD_MUTEX_NORMAL: int = 128;
pub const PTHREAD_MUTEX_RECURSIVE: int = 256;
pub const PTHREAD_MUTEX_ERRORCHECK: int = 512;

pub const _SC_UIO_MAXIOV: int = int::MAX;
pub const _SC_IOV_MAX: int = int::MAX - 1;
pub const _SC_AVPHYS_PAGES: int = int::MAX - 2;
pub const _SC_PASS_MAX: int = int::MAX - 3;
pub const _SC_XOPEN_XPG2: int = int::MAX - 4;
pub const _SC_XOPEN_XPG3: int = int::MAX - 5;
pub const _SC_XOPEN_XPG4: int = int::MAX - 6;
pub const _SC_NZERO: int = int::MAX - 7;
pub const _SC_XBS5_ILP32_OFF32: int = int::MAX - 8;
pub const _SC_XBS5_ILP32_OFFBIG: int = int::MAX - 9;
pub const _SC_XBS5_LP64_OFF64: int = int::MAX - 10;
pub const _SC_XBS5_LPBIG_OFFBIG: int = int::MAX - 11;
pub const _SC_STREAMS: int = int::MAX - 12;
pub const _SC_V7_ILP32_OFF32: int = int::MAX - 13;
pub const _SC_V7_ILP32_OFFBIG: int = int::MAX - 14;
pub const _SC_V7_LP64_OFF64: int = int::MAX - 15;
pub const _SC_V7_LPBIG_OFFBIG: int = int::MAX - 16;
pub const _SC_SS_REPL_MAX: int = int::MAX - 17;
pub const _SC_TRACE_EVENT_NAME_MAX: int = int::MAX - 18;
pub const _SC_TRACE_NAME_MAX: int = int::MAX - 19;
pub const _SC_TRACE_SYS_MAX: int = int::MAX - 20;
pub const _SC_TRACE_USER_EVENT_MAX: int = int::MAX - 21;
pub const _SC_THREAD_ROBUST_PRIO_INHERIT: int = int::MAX - 22;
pub const _SC_THREAD_ROBUST_PRIO_PROTECT: int = int::MAX - 23;
pub const _PC_SOCK_MAXBUF: int = int::MAX - 24;
pub const _PC_2_SYMLINKS: int = int::MAX - 25;

pub const PTHREAD_PROCESS_PRIVATE: int = crate::internal::PTHREAD_PROCESS_PRIVATE as _;
pub const PTHREAD_PROCESS_SHARED: int = crate::internal::PTHREAD_PROCESS_SHARED as _;
pub const PTHREAD_PRIO_NONE: int = crate::internal::PTHREAD_PRIO_NONE as _;
pub const PTHREAD_PRIO_INHERIT: int = crate::internal::PTHREAD_PRIO_INHERIT as _;
pub const PTHREAD_PRIO_PROTECT: int = crate::internal::PTHREAD_PRIO_PROTECT as _;

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

pub const SCHED_OTHER: int = crate::internal::SCHED_OTHER as _;
pub const SCHED_FIFO: int = crate::internal::SCHED_FIFO as _;
pub const SCHED_RR: int = crate::internal::SCHED_RR as _;

pub const SEEK_SET: int = crate::internal::SEEK_SET as _;
pub const SEEK_CUR: int = crate::internal::SEEK_CUR as _;
pub const SEEK_END: int = crate::internal::SEEK_END as _;

pub const SEM_FAILED: *mut sem_t = 0 as *mut sem_t;

pub const SIGABRT: int = crate::internal::SIGABRT as _;
pub const SIGALRM: int = crate::internal::SIGALRM as _;
pub const SIGBUS: int = crate::internal::SIGBUS as _;
pub const SIGCHLD: int = crate::internal::SIGCHLD as _;
pub const SIGCONT: int = crate::internal::SIGCONT as _;
pub const SIGFPE: int = crate::internal::SIGFPE as _;
pub const SIGHUP: int = crate::internal::SIGHUP as _;
pub const SIGILL: int = crate::internal::SIGILL as _;
pub const SIGINT: int = crate::internal::SIGINT as _;
pub const SIGKILL: int = crate::internal::SIGKILL as _;
pub const SIGPIPE: int = crate::internal::SIGPIPE as _;
pub const SIGQUIT: int = crate::internal::SIGQUIT as _;
pub const SIGSEGV: int = crate::internal::SIGSEGV as _;
pub const SIGSTOP: int = crate::internal::SIGSTOP as _;
pub const SIGTERM: int = crate::internal::SIGTERM as _;
pub const SIGTSTP: int = crate::internal::SIGTSTP as _;
pub const SIGTTIN: int = crate::internal::SIGTTIN as _;
pub const SIGTTOU: int = crate::internal::SIGTTOU as _;
pub const SIGUSR1: int = crate::internal::SIGUSR1 as _;
pub const SIGUSR2: int = crate::internal::SIGUSR2 as _;
pub const SIGPROF: int = crate::internal::SIGPROF as _;
pub const SIGSYS: int = crate::internal::SIGSYS as _;
pub const SIGTRAP: int = crate::internal::SIGTRAP as _;
pub const SIGURG: int = crate::internal::SIGURG as _;
pub const SIGVTALRM: int = crate::internal::SIGVTALRM as _;
pub const SIGXCPU: int = crate::internal::SIGXCPU as _;
pub const SIGXFSZ: int = crate::internal::SIGXFSZ as _;
pub const SIG_ERR: sighandler_t = sighandler_t::MAX;
pub const SIG_DFL: int = 0;
pub const SIG_IGN: int = 1;
pub const SA_RESTART: int = crate::internal::SA_RESTART as _;

pub const AF_LOCAL: sa_family_t = crate::internal::AF_UNIX as _;
pub const AF_UNIX: sa_family_t = crate::internal::AF_UNIX as _;
pub const AF_INET: sa_family_t = crate::internal::AF_INET as _;
pub const PF_INET: sa_family_t = crate::internal::PF_INET as _;
pub const PF_LOCAL: sa_family_t = crate::internal::AF_UNIX as _;
pub const PF_UNIX: sa_family_t = crate::internal::AF_UNIX as _;
pub const INADDR_ANY: in_addr_t = 0;
pub const SO_SNDBUF: int = crate::internal::SO_SNDBUF as _;
pub const SO_RCVBUF: int = crate::internal::SO_RCVBUF as _;
pub const SO_RCVTIMEO: int = crate::internal::SO_RCVTIMEO as _;
pub const SO_SNDTIMEO: int = crate::internal::SO_SNDTIMEO as _;
pub const SOCK_STREAM: int = crate::internal::SOCK_STREAM as _;
pub const SOCK_DGRAM: int = crate::internal::SOCK_DGRAM as _;
pub const IPPROTO_UDP: int = crate::internal::IPPROTO_UDP as _;
pub const SOCK_NONBLOCK: int = O_NONBLOCK;
pub const MSG_PEEK: int = crate::internal::MSG_PEEK as _;
pub const SCM_MAX_FD: u32 = 253;
pub const SCM_RIGHTS: int = crate::internal::SCM_RIGHTS as _;
pub const SOL_SOCKET: int = crate::internal::SOL_SOCKET as _;
pub const SUN_PATH_LEN: usize = 108;
pub const SA_DATA_LEN: usize = 14;

pub const S_IFMT: mode_t = crate::internal::S_IFMT as _;
pub const S_IFSOCK: mode_t = crate::internal::S_IFSOCK as _;
pub const S_IFLNK: mode_t = crate::internal::S_IFLNK as _;
pub const S_IFREG: mode_t = crate::internal::S_IFREG as _;
pub const S_IFBLK: mode_t = crate::internal::S_IFBLK as _;
pub const S_IFDIR: mode_t = crate::internal::S_IFDIR as _;
pub const S_IFCHR: mode_t = crate::internal::S_IFCHR as _;
pub const S_IFIFO: mode_t = crate::internal::S_IFIFO as _;
pub const S_IRWXU: mode_t = crate::internal::S_IRWXU as _;
pub const S_IXUSR: mode_t = crate::internal::S_IXUSR as _;
pub const S_IWUSR: mode_t = crate::internal::S_IWUSR as _;
pub const S_IRUSR: mode_t = crate::internal::S_IRUSR as _;
pub const S_IRWXG: mode_t = crate::internal::S_IRWXG as _;
pub const S_IXGRP: mode_t = crate::internal::S_IXGRP as _;
pub const S_IWGRP: mode_t = crate::internal::S_IWGRP as _;
pub const S_IRGRP: mode_t = crate::internal::S_IRGRP as _;
pub const S_IRWXO: mode_t = crate::internal::S_IRWXO as _;
pub const S_IXOTH: mode_t = crate::internal::S_IXOTH as _;
pub const S_IWOTH: mode_t = crate::internal::S_IWOTH as _;
pub const S_IROTH: mode_t = crate::internal::S_IROTH as _;
pub const S_ISUID: mode_t = crate::internal::S_ISUID as _;
pub const S_ISGID: mode_t = crate::internal::S_ISGID as _;
pub const S_ISVTX: mode_t = crate::internal::S_ISVTX as _;

pub const CLOCK_REALTIME: clockid_t = 0;
pub const CLOCK_MONOTONIC: clockid_t = 1;
pub const CLOCK_TIMER_ABSTIME: int = 1;

pub const F_OK: int = crate::internal::F_OK as _;
pub const R_OK: int = crate::internal::R_OK as _;
pub const W_OK: int = crate::internal::W_OK as _;
pub const X_OK: int = crate::internal::X_OK as _;

pub const _SC_ARG_MAX: int = crate::internal::_SC_ARG_MAX as _;
pub const _SC_CHILD_MAX: int = crate::internal::_SC_CHILD_MAX as _;
pub const _SC_CLK_TCK: int = crate::internal::_SC_CLK_TCK as _;
pub const _SC_NGROUPS_MAX: int = int::MAX;
pub const _SC_OPEN_MAX: int = crate::internal::_SC_OPEN_MAX as _;
pub const _SC_STREAM_MAX: int = crate::internal::_SC_STREAM_MAX as _;
pub const _SC_TZNAME_MAX: int = crate::internal::_SC_TZNAME_MAX as _;
pub const _SC_JOB_CONTROL: int = crate::internal::_SC_JOB_CONTROL as _;
pub const _SC_SAVED_IDS: int = crate::internal::_SC_SAVED_IDS as _;
pub const _SC_REALTIME_SIGNALS: int = crate::internal::_SC_REALTIME_SIGNALS as _;
pub const _SC_PRIORITY_SCHEDULING: int = crate::internal::_SC_PRIORITY_SCHEDULING as _;
pub const _SC_TIMERS: int = crate::internal::_SC_TIMERS as _;
pub const _SC_ASYNCHRONOUS_IO: int = crate::internal::_SC_ASYNCHRONOUS_IO as _;
pub const _SC_PRIORITIZED_IO: int = crate::internal::_SC_PRIORITIZED_IO as _;
pub const _SC_SYNCHRONIZED_IO: int = crate::internal::_SC_SYNCHRONIZED_IO as _;
pub const _SC_FSYNC: int = crate::internal::_SC_FSYNC as _;
pub const _SC_MAPPED_FILES: int = crate::internal::_SC_MAPPED_FILES as _;
pub const _SC_MEMLOCK: int = crate::internal::_SC_MEMLOCK as _;
pub const _SC_MEMLOCK_RANGE: int = crate::internal::_SC_MEMLOCK_RANGE as _;
pub const _SC_MEMORY_PROTECTION: int = crate::internal::_SC_MEMORY_PROTECTION as _;
pub const _SC_MESSAGE_PASSING: int = crate::internal::_SC_MESSAGE_PASSING as _;
pub const _SC_SEMAPHORES: int = crate::internal::_SC_SEMAPHORES as _;
pub const _SC_SHARED_MEMORY_OBJECTS: int = crate::internal::_SC_SHARED_MEMORY_OBJECTS as _;
pub const _SC_AIO_LISTIO_MAX: int = crate::internal::_SC_AIO_LISTIO_MAX as _;
pub const _SC_AIO_MAX: int = crate::internal::_SC_AIO_MAX as _;
pub const _SC_AIO_PRIO_DELTA_MAX: int = crate::internal::_SC_AIO_PRIO_DELTA_MAX as _;
pub const _SC_DELAYTIMER_MAX: int = crate::internal::_SC_DELAYTIMER_MAX as _;
pub const _SC_MQ_OPEN_MAX: int = crate::internal::_SC_MQ_OPEN_MAX as _;
pub const _SC_MQ_PRIO_MAX: int = int::MAX - 1;
pub const _SC_VERSION: int = crate::internal::_SC_VERSION as _;
pub const _SC_PAGESIZE: int = crate::internal::_SC_PAGESIZE as _;
pub const _SC_RTSIG_MAX: int = crate::internal::_SC_RTSIG_MAX as _;
pub const _SC_SEM_NSEMS_MAX: int = crate::internal::_SC_SEM_NSEMS_MAX as _;
pub const _SC_SEM_VALUE_MAX: int = crate::internal::_SC_SEM_VALUE_MAX as _;
pub const _SC_SIGQUEUE_MAX: int = crate::internal::_SC_SIGQUEUE_MAX as _;
pub const _SC_TIMER_MAX: int = crate::internal::_SC_TIMER_MAX as _;
pub const _SC_BC_BASE_MAX: int = crate::internal::_SC_BC_BASE_MAX as _;
pub const _SC_BC_DIM_MAX: int = crate::internal::_SC_BC_DIM_MAX as _;
pub const _SC_BC_SCALE_MAX: int = crate::internal::_SC_BC_SCALE_MAX as _;
pub const _SC_BC_STRING_MAX: int = crate::internal::_SC_BC_STRING_MAX as _;
pub const _SC_COLL_WEIGHTS_MAX: int = crate::internal::_SC_COLL_WEIGHTS_MAX as _;
pub const _SC_EXPR_NEST_MAX: int = crate::internal::_SC_EXPR_NEST_MAX as _;
pub const _SC_LINE_MAX: int = crate::internal::_SC_LINE_MAX as _;
pub const _SC_RE_DUP_MAX: int = crate::internal::_SC_RE_DUP_MAX as _;
pub const _SC_2_VERSION: int = crate::internal::_SC_2_VERSION as _;
pub const _SC_2_C_BIND: int = crate::internal::_SC_2_C_BIND as _;
pub const _SC_2_C_DEV: int = crate::internal::_SC_2_C_DEV as _;
pub const _SC_2_FORT_DEV: int = crate::internal::_SC_2_FORT_DEV as _;
pub const _SC_2_FORT_RUN: int = crate::internal::_SC_2_FORT_RUN as _;
pub const _SC_2_SW_DEV: int = crate::internal::_SC_2_SW_DEV as _;
pub const _SC_2_LOCALEDEF: int = crate::internal::_SC_2_LOCALEDEF as _;
pub const _SC_THREADS: int = crate::internal::_SC_THREADS as _;
pub const _SC_THREAD_SAFE_FUNCTIONS: int = crate::internal::_SC_THREAD_SAFE_FUNCTIONS as _;
pub const _SC_GETGR_R_SIZE_MAX: int = crate::internal::_SC_GETGR_R_SIZE_MAX as _;
pub const _SC_GETPW_R_SIZE_MAX: int = crate::internal::_SC_GETPW_R_SIZE_MAX as _;
pub const _SC_LOGIN_NAME_MAX: int = crate::internal::_SC_LOGIN_NAME_MAX as _;
pub const _SC_TTY_NAME_MAX: int = crate::internal::_SC_TTY_NAME_MAX as _;
pub const _SC_THREAD_DESTRUCTOR_ITERATIONS: int =
    crate::internal::_SC_THREAD_DESTRUCTOR_ITERATIONS as _;
pub const _SC_THREAD_KEYS_MAX: int = crate::internal::_SC_THREAD_KEYS_MAX as _;
pub const _SC_THREAD_STACK_MIN: int = crate::internal::_SC_THREAD_STACK_MIN as _;
pub const _SC_THREAD_THREADS_MAX: int = crate::internal::_SC_THREAD_THREADS_MAX as _;
pub const _SC_THREAD_ATTR_STACKADDR: int = crate::internal::_SC_THREAD_ATTR_STACKADDR as _;
pub const _SC_THREAD_ATTR_STACKSIZE: int = crate::internal::_SC_THREAD_ATTR_STACKSIZE as _;
pub const _SC_THREAD_PRIORITY_SCHEDULING: int =
    crate::internal::_SC_THREAD_PRIORITY_SCHEDULING as _;
pub const _SC_THREAD_PRIO_INHERIT: int = crate::internal::_SC_THREAD_PRIO_INHERIT as _;
pub const _SC_THREAD_PRIO_PROTECT: int = crate::internal::_SC_THREAD_PRIO_PROTECT as _;
pub const _SC_THREAD_PROCESS_SHARED: int = crate::internal::_SC_THREAD_PROCESS_SHARED as _;
pub const _SC_NPROCESSORS_CONF: int = crate::internal::_SC_NPROCESSORS_CONF as _;
pub const _SC_NPROCESSORS_ONLN: int = crate::internal::_SC_NPROCESSORS_ONLN as _;
pub const _SC_PHYS_PAGES: int = crate::internal::_SC_PHYS_PAGES as _;
pub const _SC_ATEXIT_MAX: int = crate::internal::_SC_ATEXIT_MAX as _;
pub const _SC_XOPEN_VERSION: int = crate::internal::_SC_XOPEN_VERSION as _;
pub const _SC_XOPEN_XCU_VERSION: int = crate::internal::_SC_XOPEN_XCU_VERSION as _;
pub const _SC_XOPEN_UNIX: int = crate::internal::_SC_XOPEN_UNIX as _;
pub const _SC_XOPEN_CRYPT: int = crate::internal::_SC_XOPEN_CRYPT as _;
pub const _SC_XOPEN_ENH_I18N: int = crate::internal::_SC_XOPEN_ENH_I18N as _;
pub const _SC_XOPEN_SHM: int = crate::internal::_SC_XOPEN_SHM as _;
pub const _SC_2_CHAR_TERM: int = crate::internal::_SC_2_CHAR_TERM as _;
pub const _SC_2_UPE: int = crate::internal::_SC_2_UPE as _;
pub const _SC_XOPEN_LEGACY: int = crate::internal::_SC_XOPEN_LEGACY as _;
pub const _SC_XOPEN_REALTIME: int = crate::internal::_SC_XOPEN_REALTIME as _;
pub const _SC_XOPEN_REALTIME_THREADS: int = crate::internal::_SC_XOPEN_REALTIME_THREADS as _;
pub const _SC_ADVISORY_INFO: int = crate::internal::_SC_ADVISORY_INFO as _;
pub const _SC_BARRIERS: int = crate::internal::_SC_BARRIERS as _;
pub const _SC_CLOCK_SELECTION: int = crate::internal::_SC_CLOCK_SELECTION as _;
pub const _SC_CPUTIME: int = crate::internal::_SC_CPUTIME as _;
pub const _SC_THREAD_CPUTIME: int = crate::internal::_SC_THREAD_CPUTIME as _;
pub const _SC_MONOTONIC_CLOCK: int = crate::internal::_SC_MONOTONIC_CLOCK as _;
pub const _SC_READER_WRITER_LOCKS: int = crate::internal::_SC_READER_WRITER_LOCKS as _;
pub const _SC_SPIN_LOCKS: int = crate::internal::_SC_SPIN_LOCKS as _;
pub const _SC_REGEXP: int = crate::internal::_SC_REGEXP as _;
pub const _SC_SHELL: int = crate::internal::_SC_SHELL as _;
pub const _SC_SPAWN: int = crate::internal::_SC_SPAWN as _;
pub const _SC_SPORADIC_SERVER: int = crate::internal::_SC_SPORADIC_SERVER as _;
pub const _SC_THREAD_SPORADIC_SERVER: int = crate::internal::_SC_THREAD_SPORADIC_SERVER as _;
pub const _SC_TIMEOUTS: int = crate::internal::_SC_TIMEOUTS as _;
pub const _SC_TYPED_MEMORY_OBJECTS: int = crate::internal::_SC_TYPED_MEMORY_OBJECTS as _;
pub const _SC_2_PBS: int = crate::internal::_SC_2_PBS as _;
pub const _SC_2_PBS_ACCOUNTING: int = crate::internal::_SC_2_PBS_ACCOUNTING as _;
pub const _SC_2_PBS_LOCATE: int = crate::internal::_SC_2_PBS_LOCATE as _;
pub const _SC_2_PBS_MESSAGE: int = crate::internal::_SC_2_PBS_MESSAGE as _;
pub const _SC_2_PBS_TRACK: int = crate::internal::_SC_2_PBS_TRACK as _;
pub const _SC_SYMLOOP_MAX: int = crate::internal::_SC_SYMLOOP_MAX as _;
pub const _SC_2_PBS_CHECKPOINT: int = crate::internal::_SC_2_PBS_CHECKPOINT as _;
pub const _SC_V6_ILP32_OFF32: int = crate::internal::_SC_V6_ILP32_OFF32 as _;
pub const _SC_V6_ILP32_OFFBIG: int = crate::internal::_SC_V6_ILP32_OFFBIG as _;
pub const _SC_V6_LP64_OFF64: int = crate::internal::_SC_V6_LP64_OFF64 as _;
pub const _SC_V6_LPBIG_OFFBIG: int = crate::internal::_SC_V6_LPBIG_OFFBIG as _;
pub const _SC_HOST_NAME_MAX: int = crate::internal::_SC_HOST_NAME_MAX as _;
pub const _SC_TRACE: int = crate::internal::_SC_TRACE as _;
pub const _SC_TRACE_EVENT_FILTER: int = crate::internal::_SC_TRACE_EVENT_FILTER as _;
pub const _SC_TRACE_INHERIT: int = crate::internal::_SC_TRACE_INHERIT as _;
pub const _SC_TRACE_LOG: int = crate::internal::_SC_TRACE_LOG as _;
pub const _SC_IPV6: int = crate::internal::_SC_IPV6 as _;
pub const _SC_RAW_SOCKETS: int = crate::internal::_SC_RAW_SOCKETS as _;
pub const _SC_XOPEN_STREAMS: int = crate::internal::_SC_XOPEN_STREAMS as _;

pub const _PC_LINK_MAX: int = crate::internal::_PC_LINK_MAX as _;
pub const _PC_MAX_CANON: int = crate::internal::_PC_MAX_CANON as _;
pub const _PC_MAX_INPUT: int = crate::internal::_PC_MAX_INPUT as _;
pub const _PC_NAME_MAX: int = crate::internal::_PC_NAME_MAX as _;
pub const _PC_PATH_MAX: int = crate::internal::_PC_PATH_MAX as _;
pub const _PC_PIPE_BUF: int = crate::internal::_PC_PIPE_BUF as _;
pub const _PC_CHOWN_RESTRICTED: int = crate::internal::_PC_CHOWN_RESTRICTED as _;
pub const _PC_NO_TRUNC: int = crate::internal::_PC_NO_TRUNC as _;
pub const _PC_VDISABLE: int = crate::internal::_PC_VDISABLE as _;
pub const _PC_SYNC_IO: int = crate::internal::_PC_SYNC_IO as _;
pub const _PC_ASYNC_IO: int = crate::internal::_PC_ASYNC_IO as _;
pub const _PC_PRIO_IO: int = crate::internal::_PC_PRIO_IO as _;
pub const _PC_FILESIZEBITS: int = crate::internal::_PC_FILESIZEBITS as _;
pub const _PC_REC_INCR_XFER_SIZE: int = crate::internal::_PC_REC_INCR_XFER_SIZE as _;
pub const _PC_REC_MAX_XFER_SIZE: int = crate::internal::_PC_REC_MAX_XFER_SIZE as _;
pub const _PC_REC_MIN_XFER_SIZE: int = crate::internal::_PC_REC_MIN_XFER_SIZE as _;
pub const _PC_REC_XFER_ALIGN: int = crate::internal::_PC_REC_XFER_ALIGN as _;
pub const _PC_ALLOC_SIZE_MIN: int = crate::internal::_PC_ALLOC_SIZE_MIN as _;
pub const _PC_SYMLINK_MAX: int = crate::internal::_PC_SYMLINK_MAX as _;
