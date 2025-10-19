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

#![allow(clippy::missing_safety_doc)]

use crate::posix::types::*;
use crate::ErrnoEnumGenerator;
use core::{ffi::CStr, fmt::Display};

ErrnoEnumGenerator!(
  assign
    ESUCCES = 0;
  map
    EPERM,
    ENOENT,
    ESRCH,
    EINTR,
    EIO,
    ENXIO,
    E2BIG,
    ENOEXEC,
    EBADF,
    ECHILD,
    EAGAIN,
    ENOMEM,
    EACCES,
    EFAULT,
    ENOTBLK,
    EBUSY,
    EEXIST,
    EXDEV,
    ENODEV,
    ENOTDIR,
    EISDIR,
    EINVAL,
    ENFILE,
    EMFILE,
    ENOTTY,
    ETXTBSY,
    EFBIG,
    ENOSPC,
    ESPIPE,
    EROFS,
    EMLINK,
    EPIPE,
    EDOM,
    ERANGE,
    //WOULDBLOCK = AGAIN

    // GNU extensions for POSIX
    EDEADLK,
    ENAMETOOLONG,
    ENOLCK,
    ENOSYS,
    ENOTEMPTY,
    ELOOP,
    ENOMSG,
    EIDRM,
    // ECHRNG,
    // EL2NSYNC,
    // EL3HLT,
    // EL3RST,
    // ELNRNG,
    // EUNATCH,
    // ENOCSI,
    // EL2HLT,
    // EBADE,
    // EBADR,
    // EXFULL,
    // ENOANO,
    // EBADRQC,
    // EBADSLT,
    EMULTIHOP,
    EOVERFLOW,
    // ENOTUNIQ,
    // EBADFD,
    EBADMSG,
    // EREMCHG,
    // ELIBACC,
    // ELIBBAD,
    // ELIBSCN,
    // ELIBMAX,
    // ELIBEXEC,
    EILSEQ,
    // ERESTART,
    // ESTRPIPE,
    EUSERS,
    ENOTSOCK,
    EDESTADDRREQ,
    EMSGSIZE,
    EPROTOTYPE,
    ENOPROTOOPT,
    EPROTONOSUPPORT,
    ESOCKTNOSUPPORT,
    ENOTSUP,
    EPFNOSUPPORT,
    EAFNOSUPPORT,
    EADDRINUSE,
    EADDRNOTAVAIL,
    ENETDOWN,
    ENETUNREACH,
    ENETRESET,
    ECONNABORTED,
    ECONNRESET,
    ENOBUFS,
    EISCONN,
    ENOTCONN,
    ESHUTDOWN,
    ETOOMANYREFS,
    ETIMEDOUT,
    ECONNREFUSED,
    EHOSTDOWN,
    EHOSTUNREACH,
    EALREADY,
    EINPROGRESS,
    ESTALE,
    EDQUOT,
    // ENOMEDIUM,
    // EMEDIUMTYPE,
    ECANCELED,
    // ENOKEY,
    // EKEYEXPIRED,
    // EKEYREVOKED,
    // EKEYREJECTED,
    EOWNERDEAD,
    ENOTRECOVERABLE
    // ERFKILL,
    // EHWPOISON,
);

impl Errno {
    pub fn get() -> Errno {
        unsafe { *libc::__errno_location() }.into()
    }

    pub fn set(value: Errno) {
        unsafe { *libc::__errno_location() = value as i32 };
    }

    pub fn reset() {
        unsafe { *libc::__errno_location() = 0 };
    }
}

pub unsafe fn strerror_r(errnum: int, buf: *mut c_char, buflen: size_t) -> int {
    use core::sync::atomic::Ordering;
    use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;
    static IS_LOCKED: IoxAtomicBool = IoxAtomicBool::new(false);

    while IS_LOCKED
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_err()
    {}

    let raw_string = strerror(errnum);
    crate::posix::string::strncpy(buf, raw_string, buflen);

    IS_LOCKED.store(false, Ordering::Relaxed);

    0
}

pub unsafe fn strerror(errnum: int) -> *const c_char {
    libc::strerror(errnum)
}
