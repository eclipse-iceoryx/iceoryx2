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

#![allow(clippy::missing_safety_doc)]

use crate::posix::types::*;
use crate::ErrnoEnumGenerator;
use core::{ffi::CStr, fmt::Display};

extern crate alloc;
use alloc::string::ToString;

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
    ENOMSG,
    EIDRM,
    ECHRNG,
    EL2NSYNC,
    EL3HLT,
    EL3RST,
    ELNRNG,
    EUNATCH,
    ENOCSI,
    EL2HLT,
    EDEADLK,
    ENOLCK,
    ECANCELED,
    ENOTSUP,
    EDQUOT,
    EBADE,
    EBADR,
    EXFULL,
    ENOANO,
    EBADRQC,
    EBADSLT,
    EDEADLOCK,
    EBFONT,
    EOWNERDEAD,
    ENOSTR,
    ENODATA,
    ETIME,
    ENOSR,
    ENONET,
    ENOPKG,
    EREMOTE,
    ENOLINK,
    EADV,
    ESRMNT,
    ECOMM,
    EPROTO,
    EMULTIHOP,
    EBADMSG,
    ENAMETOOLONG,
    EOVERFLOW,
    ENOTUNIQ,
    EBADFD,
    EREMCHG,
    ELIBACC,
    ELIBBAD,
    ELIBSCN,
    ELIBMAX,
    ELIBEXEC,
    EILSEQ,
    ENOSYS,
    ELOOP,
    ERESTART,
    ESTRPIPE,
    ENOTEMPTY,
    EUSERS,
    ENOTRECOVERABLE,
    EOPNOTSUPP,
    EFPOS,
    ESTALE,
//     EWOULDBLOCK, // Duplicate value, same as EAGAIN
    EINPROGRESS,
    EALREADY,
    ENOTSOCK,
    EDESTADDRREQ,
    EMSGSIZE,
    EPROTOTYPE,
    ENOPROTOOPT,
    EPROTONOSUPPORT,
    ESOCKTNOSUPPORT,
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
    EBADRPC,
    ERPCMISMATCH,
    EPROGUNAVAIL,
    EPROGMISMATCH,
    EPROCUNAVAIL,
    ENOREMOTE,
    ENONDP,
    EBADFSYS,
    EMORE,
    ECTRLTERM,
    ENOLIC,
    ESRVRFAULT,
    EENDIAN,
    ESECTYPEINVAL
);

impl Errno {
    pub fn get() -> Errno {
        unsafe { *crate::internal::__get_errno_ptr() }.into()
    }

    pub fn set(value: Errno) {
        unsafe { *crate::internal::__get_errno_ptr() = value as i32 };
    }

    pub fn reset() {
        unsafe { *crate::internal::__get_errno_ptr() = 0 };
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
    crate::internal::strerror(errnum)
}
