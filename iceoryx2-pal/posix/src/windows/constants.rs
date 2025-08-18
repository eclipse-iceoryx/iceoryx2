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

pub const CPU_SETSIZE: usize = core::mem::size_of::<usize>() * 8;
pub const MAX_NUMBER_OF_THREADS: usize = 1024;
pub const FD_SETSIZE: usize = windows_sys::Win32::Networking::WinSock::FD_SETSIZE as _;
pub const NULL_TERMINATOR: c_char = 0;
pub const USER_NAME_LENGTH: usize = 255;
pub const GROUP_NAME_LENGTH: usize = 31;

pub const O_RDONLY: int = 1;
pub const O_WRONLY: int = 2;
pub const O_RDWR: int = 4;
pub const O_SYNC: int = 8;

pub const O_CREAT: int = 8;
pub const O_EXCL: int = 16;
pub const O_APPEND: int = 32;
pub const O_NOCTTY: int = 64;
pub const O_NONBLOCK: int = 128;
pub const O_DIRECTORY: int = 256;

pub const F_RDLCK: int = 1;
pub const F_WRLCK: int = 2;
pub const F_UNLCK: int = 4;
pub const F_GETFL: int = 8;
pub const F_SETFL: int = 16;
pub const F_GETLK: int = 32;
pub const F_SETLK: int = 64;
pub const F_SETLKW: int = 128;
pub const F_GETFD: int = 256;

pub const PROT_NONE: int = 1;
pub const PROT_READ: int = 2;
pub const PROT_WRITE: int = 4;
pub const PROT_EXEC: int = 8;
pub const MCL_CURRENT: int = 16;
pub const MCL_FUTURE: int = 32;
pub const MAP_SHARED: int = 64;
pub const MAP_FAILED: *mut void = 0 as *mut void;

pub const PTHREAD_MUTEX_NORMAL: int = 1;
pub const PTHREAD_MUTEX_RECURSIVE: int = 2;
pub const PTHREAD_MUTEX_ERRORCHECK: int = 4;
pub const PTHREAD_MUTEX_STALLED: int = 8;
pub const PTHREAD_MUTEX_ROBUST: int = 16;
pub const PTHREAD_PROCESS_PRIVATE: int = 32;
pub const PTHREAD_PROCESS_SHARED: int = 64;
pub const PTHREAD_PRIO_NONE: int = 128;
pub const PTHREAD_PRIO_INHERIT: int = 256;
pub const PTHREAD_PRIO_PROTECT: int = 512;

pub const PTHREAD_PREFER_READER_NP: int = 4096;
pub const PTHREAD_PREFER_WRITER_NP: int = 8192;
pub const PTHREAD_PREFER_WRITER_NONRECURSIVE_NP: int = 16384;

pub const PTHREAD_BARRIER_SERIAL_THREAD: int = 32768;
pub const PTHREAD_EXPLICIT_SCHED: int = 65536;
pub const PTHREAD_INHERIT_SCHED: int = 131072;

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

pub const SCHED_OTHER: int = 1;
pub const SCHED_FIFO: int = 2;
pub const SCHED_RR: int = 4;

pub const SEEK_SET: int = crate::internal::SEEK_SET as _;
pub const SEEK_CUR: int = crate::internal::SEEK_CUR as _;
pub const SEEK_END: int = crate::internal::SEEK_END as _;

pub const SEM_FAILED: *mut sem_t = 0 as *mut sem_t;

pub const MAX_SIGNAL_VALUE: usize = 27;
pub const SIGABRT: int = 0;
pub const SIGALRM: int = 1;
pub const SIGBUS: int = 2;
pub const SIGCHLD: int = 3;
pub const SIGCONT: int = 4;
pub const SIGFPE: int = 5;
pub const SIGHUP: int = 6;
pub const SIGILL: int = 7;
pub const SIGINT: int = 8;
pub const SIGKILL: int = 9;
pub const SIGPIPE: int = 10;
pub const SIGQUIT: int = 11;
pub const SIGSEGV: int = 12;
pub const SIGSTOP: int = 13;
pub const SIGTERM: int = 14;
pub const SIGTSTP: int = 15;
pub const SIGTTIN: int = 16;
pub const SIGTTOU: int = 17;
pub const SIGUSR1: int = 18;
pub const SIGUSR2: int = 19;
pub const SIGPROF: int = 20;
pub const SIGSYS: int = 21;
pub const SIGTRAP: int = 22;
pub const SIGURG: int = 23;
pub const SIGVTALRM: int = 24;
pub const SIGXCPU: int = 25;
pub const SIGXFSZ: int = 26;
pub const SIG_ERR: sighandler_t = sighandler_t::MAX;
pub const SIG_DFL: int = 0;
pub const SIG_IGN: int = 1;
pub const SA_RESTART: int = 1;

pub const AF_LOCAL: sa_family_t = windows_sys::Win32::Networking::WinSock::AF_UNIX as _;
pub const AF_UNIX: sa_family_t = windows_sys::Win32::Networking::WinSock::AF_UNIX as _;
pub const AF_INET: sa_family_t = windows_sys::Win32::Networking::WinSock::AF_INET as _;
pub const PF_INET: sa_family_t = AF_INET;
pub const PF_UNIX: sa_family_t = windows_sys::Win32::Networking::WinSock::PF_UNIX as _;
pub const SO_PASSCRED: int = 1;
pub const SO_PEERCRED: int = 2;
pub const SO_SNDBUF: int = windows_sys::Win32::Networking::WinSock::SO_SNDBUF as _;
pub const SO_RCVBUF: int = windows_sys::Win32::Networking::WinSock::SO_RCVBUF as _;
pub const SO_RCVTIMEO: int = windows_sys::Win32::Networking::WinSock::SO_RCVTIMEO as _;
pub const SO_SNDTIMEO: int = windows_sys::Win32::Networking::WinSock::SO_SNDTIMEO as _;
pub const SOCK_STREAM: int = windows_sys::Win32::Networking::WinSock::SOCK_STREAM as _;
pub const SOCK_DGRAM: int = windows_sys::Win32::Networking::WinSock::SOCK_DGRAM as _;
pub const SOCK_NONBLOCK: int = O_NONBLOCK;
pub const IPPROTO_UDP: int = windows_sys::Win32::Networking::WinSock::IPPROTO_UDP as _;
pub const MSG_PEEK: int = windows_sys::Win32::Networking::WinSock::MSG_PEEK as _;
pub const SCM_MAX_FD: u32 = 253;
pub const SCM_RIGHTS: int = 128;
pub const SCM_CREDENTIALS: int = 0x02;
pub const SOL_SOCKET: int = windows_sys::Win32::Networking::WinSock::SOL_SOCKET as _;
pub const SUN_PATH_LEN: usize = 108;
pub const SA_DATA_LEN: usize = 14;

pub const S_IFMT: mode_t = 0o0170000;
pub const S_IFIFO: mode_t = 0o0010000;
pub const S_IFCHR: mode_t = 0o0020000;
pub const S_IFDIR: mode_t = 0o0040000;
pub const S_IFBLK: mode_t = 0o0060000;
pub const S_IFREG: mode_t = 0o0100000;
pub const S_IFLNK: mode_t = 0o0120000;
pub const S_IFSOCK: mode_t = 0o0140000;

pub const S_ISUID: mode_t = 0o04000;
pub const S_ISGID: mode_t = 0o02000;
pub const S_ISVTX: mode_t = 0o01000;

pub const S_IXUSR: mode_t = 0o0100;
pub const S_IWUSR: mode_t = 0o0200;
pub const S_IRUSR: mode_t = 0o0400;
pub const S_IRWXU: mode_t = S_IRUSR | S_IWUSR | S_IXUSR;

pub const S_IXGRP: mode_t = 0o0010;
pub const S_IWGRP: mode_t = 0o0020;
pub const S_IRGRP: mode_t = 0o0040;
pub const S_IRWXG: mode_t = S_IRGRP | S_IWGRP | S_IXGRP;

pub const S_IXOTH: mode_t = 0o0001;
pub const S_IWOTH: mode_t = 0o0002;
pub const S_IROTH: mode_t = 0o0004;
pub const S_IRWXO: mode_t = S_IROTH | S_IWOTH | S_IXOTH;

pub const CLOCK_REALTIME: clockid_t = 1;
pub const CLOCK_MONOTONIC: clockid_t = 2;
pub const CLOCK_TIMER_ABSTIME: int = 4;

pub const F_OK: int = 1;
pub const R_OK: int = 2;
pub const W_OK: int = 4;
pub const X_OK: int = 8;

pub const _SC_ARG_MAX: int = 1;
pub const _SC_CHILD_MAX: int = 2;
pub const _SC_CLK_TCK: int = 3;
pub const _SC_NGROUPS_MAX: int = 4;
pub const _SC_OPEN_MAX: int = 5;
pub const _SC_STREAM_MAX: int = 6;
pub const _SC_TZNAME_MAX: int = 7;
pub const _SC_JOB_CONTROL: int = 8;
pub const _SC_SAVED_IDS: int = 9;
pub const _SC_REALTIME_SIGNALS: int = 10;
pub const _SC_PRIORITY_SCHEDULING: int = 11;
pub const _SC_TIMERS: int = 12;
pub const _SC_ASYNCHRONOUS_IO: int = 13;
pub const _SC_PRIORITIZED_IO: int = 14;
pub const _SC_SYNCHRONIZED_IO: int = 15;
pub const _SC_FSYNC: int = 16;
pub const _SC_MAPPED_FILES: int = 17;
pub const _SC_MEMLOCK: int = 18;
pub const _SC_MEMLOCK_RANGE: int = 19;
pub const _SC_MEMORY_PROTECTION: int = 20;
pub const _SC_MESSAGE_PASSING: int = 21;
pub const _SC_SEMAPHORES: int = 22;
pub const _SC_SHARED_MEMORY_OBJECTS: int = 23;
pub const _SC_AIO_LISTIO_MAX: int = 24;
pub const _SC_AIO_MAX: int = 25;
pub const _SC_AIO_PRIO_DELTA_MAX: int = 26;
pub const _SC_DELAYTIMER_MAX: int = 27;
pub const _SC_MQ_OPEN_MAX: int = 28;
pub const _SC_MQ_PRIO_MAX: int = 29;
pub const _SC_VERSION: int = 30;
pub const _SC_PAGESIZE: int = 31;
pub const _SC_RTSIG_MAX: int = 32;
pub const _SC_SEM_NSEMS_MAX: int = 33;
pub const _SC_SEM_VALUE_MAX: int = 34;
pub const _SC_SIGQUEUE_MAX: int = 35;
pub const _SC_TIMER_MAX: int = 36;
pub const _SC_BC_BASE_MAX: int = 37;
pub const _SC_BC_DIM_MAX: int = 38;
pub const _SC_BC_SCALE_MAX: int = 39;
pub const _SC_BC_STRING_MAX: int = 40;
pub const _SC_COLL_WEIGHTS_MAX: int = 41;
pub const _SC_EXPR_NEST_MAX: int = 42;
pub const _SC_LINE_MAX: int = 43;
pub const _SC_RE_DUP_MAX: int = 44;
pub const _SC_2_VERSION: int = 45;
pub const _SC_2_C_BIND: int = 46;
pub const _SC_2_C_DEV: int = 47;
pub const _SC_2_FORT_DEV: int = 48;
pub const _SC_2_FORT_RUN: int = 49;
pub const _SC_2_SW_DEV: int = 50;
pub const _SC_2_LOCALEDEF: int = 51;
pub const _SC_THREADS: int = 52;
pub const _SC_THREAD_SAFE_FUNCTIONS: int = 53;
pub const _SC_GETGR_R_SIZE_MAX: int = 54;
pub const _SC_GETPW_R_SIZE_MAX: int = 55;
pub const _SC_LOGIN_NAME_MAX: int = 56;
pub const _SC_TTY_NAME_MAX: int = 57;
pub const _SC_THREAD_DESTRUCTOR_ITERATIONS: int = 58;
pub const _SC_THREAD_KEYS_MAX: int = 59;
pub const _SC_THREAD_STACK_MIN: int = 60;
pub const _SC_THREAD_THREADS_MAX: int = 61;
pub const _SC_THREAD_ATTR_STACKADDR: int = 62;
pub const _SC_THREAD_ATTR_STACKSIZE: int = 63;
pub const _SC_THREAD_PRIORITY_SCHEDULING: int = 64;
pub const _SC_THREAD_PRIO_INHERIT: int = 65;
pub const _SC_THREAD_PRIO_PROTECT: int = 66;
pub const _SC_THREAD_PROCESS_SHARED: int = 67;
pub const _SC_NPROCESSORS_CONF: int = 68;
pub const _SC_NPROCESSORS_ONLN: int = 69;
pub const _SC_PHYS_PAGES: int = 70;
pub const _SC_ATEXIT_MAX: int = 71;
pub const _SC_XOPEN_VERSION: int = 72;
pub const _SC_XOPEN_XCU_VERSION: int = 73;
pub const _SC_XOPEN_UNIX: int = 74;
pub const _SC_XOPEN_CRYPT: int = 75;
pub const _SC_XOPEN_ENH_I18N: int = 76;
pub const _SC_XOPEN_SHM: int = 77;
pub const _SC_2_CHAR_TERM: int = 78;
pub const _SC_2_UPE: int = 79;
pub const _SC_XOPEN_LEGACY: int = 80;
pub const _SC_XOPEN_REALTIME: int = 81;
pub const _SC_XOPEN_REALTIME_THREADS: int = 82;
pub const _SC_ADVISORY_INFO: int = 83;
pub const _SC_BARRIERS: int = 84;
pub const _SC_CLOCK_SELECTION: int = 85;
pub const _SC_CPUTIME: int = 86;
pub const _SC_THREAD_CPUTIME: int = 87;
pub const _SC_MONOTONIC_CLOCK: int = 88;
pub const _SC_READER_WRITER_LOCKS: int = 89;
pub const _SC_SPIN_LOCKS: int = 90;
pub const _SC_REGEXP: int = 91;
pub const _SC_SHELL: int = 92;
pub const _SC_SPAWN: int = 93;
pub const _SC_SPORADIC_SERVER: int = 94;
pub const _SC_THREAD_SPORADIC_SERVER: int = 95;
pub const _SC_TIMEOUTS: int = 96;
pub const _SC_TYPED_MEMORY_OBJECTS: int = 97;
pub const _SC_2_PBS: int = 98;
pub const _SC_2_PBS_ACCOUNTING: int = 99;
pub const _SC_2_PBS_LOCATE: int = 100;
pub const _SC_2_PBS_MESSAGE: int = 101;
pub const _SC_2_PBS_TRACK: int = 102;
pub const _SC_SYMLOOP_MAX: int = 103;
pub const _SC_2_PBS_CHECKPOINT: int = 104;
pub const _SC_V6_ILP32_OFF32: int = 105;
pub const _SC_V6_ILP32_OFFBIG: int = 106;
pub const _SC_V6_LP64_OFF64: int = 107;
pub const _SC_V6_LPBIG_OFFBIG: int = 108;
pub const _SC_HOST_NAME_MAX: int = 109;
pub const _SC_TRACE: int = 110;
pub const _SC_TRACE_EVENT_FILTER: int = 111;
pub const _SC_TRACE_INHERIT: int = 112;
pub const _SC_TRACE_LOG: int = 113;
pub const _SC_IPV6: int = 114;
pub const _SC_RAW_SOCKETS: int = 115;
pub const _SC_XOPEN_STREAMS: int = 116;
pub const _SC_UIO_MAXIOV: int = 117;
pub const _SC_IOV_MAX: int = 118;
pub const _SC_AVPHYS_PAGES: int = 119;
pub const _SC_PASS_MAX: int = 120;
pub const _SC_XOPEN_XPG2: int = 121;
pub const _SC_XOPEN_XPG3: int = 122;
pub const _SC_XOPEN_XPG4: int = 123;
pub const _SC_NZERO: int = 124;
pub const _SC_XBS5_ILP32_OFF32: int = 125;
pub const _SC_XBS5_ILP32_OFFBIG: int = 126;
pub const _SC_XBS5_LP64_OFF64: int = 127;
pub const _SC_XBS5_LPBIG_OFFBIG: int = 128;
pub const _SC_STREAMS: int = 129;
pub const _SC_V7_ILP32_OFF32: int = 130;
pub const _SC_V7_ILP32_OFFBIG: int = 131;
pub const _SC_V7_LP64_OFF64: int = 132;
pub const _SC_V7_LPBIG_OFFBIG: int = 133;
pub const _SC_SS_REPL_MAX: int = 134;
pub const _SC_TRACE_EVENT_NAME_MAX: int = 135;
pub const _SC_TRACE_NAME_MAX: int = 136;
pub const _SC_TRACE_SYS_MAX: int = 137;
pub const _SC_THREAD_ROBUST_PRIO_INHERIT: int = 138;
pub const _SC_THREAD_ROBUST_PRIO_PROTECT: int = 139;

pub const _PC_SOCK_MAXBUF: int = 100001;
pub const _PC_2_SYMLINKS: int = 100002;
pub const _SC_TRACE_USER_EVENT_MAX: int = 100003;
pub const _PC_LINK_MAX: int = 100004;
pub const _PC_MAX_CANON: int = 100005;
pub const _PC_MAX_INPUT: int = 100006;
pub const _PC_NAME_MAX: int = 100007;
pub const _PC_PATH_MAX: int = 100008;
pub const _PC_PIPE_BUF: int = 100009;
pub const _PC_CHOWN_RESTRICTED: int = 100010;
pub const _PC_NO_TRUNC: int = 100011;
pub const _PC_VDISABLE: int = 100012;
pub const _PC_SYNC_IO: int = 100013;
pub const _PC_ASYNC_IO: int = 100014;
pub const _PC_PRIO_IO: int = 100015;
pub const _PC_FILESIZEBITS: int = 100016;
pub const _PC_REC_INCR_XFER_SIZE: int = 100017;
pub const _PC_REC_MAX_XFER_SIZE: int = 100018;
pub const _PC_REC_MIN_XFER_SIZE: int = 100019;
pub const _PC_REC_XFER_ALIGN: int = 100020;
pub const _PC_ALLOC_SIZE_MIN: int = 100021;
pub const _PC_SYMLINK_MAX: int = 100022;
