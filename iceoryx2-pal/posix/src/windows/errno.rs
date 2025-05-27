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
#![allow(unused_variables)]

use crate::posix::types::*;
use crate::ErrnoEnumGenerator;
use core::{cell::Cell, ffi::CStr, fmt::Display};

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
//    ENOTBLK,
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
 //   EMULTIHOP,
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
//    EUSERS,
    ENOTSOCK,
    EDESTADDRREQ,
    EMSGSIZE,
    EPROTOTYPE,
    ENOPROTOOPT,
    EPROTONOSUPPORT,
 //   ESOCKTNOSUPPORT,
    ENOTSUP,
 //   EPFNOSUPPORT,
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
 //   ESHUTDOWN,
 //   ETOOMANYREFS,
    ETIMEDOUT,
    ECONNREFUSED,
 //   EHOSTDOWN,
    EHOSTUNREACH,
    EALREADY,
    EINPROGRESS,
 //   ESTALE,
 //   EDQUOT,
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

thread_local! {
    pub static GLOBAL_ERRNO_VALUE: Cell<u32> = const { Cell::new(Errno::ESUCCES as _) };
}

impl Errno {
    pub fn get() -> Errno {
        GLOBAL_ERRNO_VALUE.get().into()
    }

    pub fn reset() {
        Errno::set(Errno::ESUCCES);
    }

    pub(crate) fn set(value: Errno) {
        GLOBAL_ERRNO_VALUE.set(value as _);
    }
}

pub unsafe fn strerror_r(errnum: int, buf: *mut c_char, buflen: size_t) -> int {
    let error = strerror(errnum);
    let len = || -> usize {
        for n in 0..buflen {
            if *error.add(n) == 0 {
                return n;
            }
        }
        buflen
    }();

    core::ptr::copy_nonoverlapping(error, buf, len);

    0
}

pub unsafe fn strerror(errnum: int) -> *const c_char {
    let errno: Errno = errnum.into();
    match errno {
        Errno::EINVAL => c"Invalid input argument value.".as_ptr() as *const c_char,
        Errno::ENOSYS => c"The feature is not defined and supported.".as_ptr() as *const c_char,
        Errno::ETIMEDOUT => c"A user-provided timeout was hit.".as_ptr() as *const c_char,
        Errno::ENOENT => c"A required system-resource does not exist.".as_ptr() as *const c_char,
        Errno::ENOTSUP => c"The feature is not supported on this system.".as_ptr() as *const c_char,
        Errno::EBUSY => {
            c"The resource is currently busy and unaccessable.".as_ptr() as *const c_char
        }
        _ => c"Unknown error has occurred.".as_ptr() as *const c_char,
    }
}
