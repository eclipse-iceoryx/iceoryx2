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
use std::{
    ffi::CStr,
    fmt::Display,
    sync::atomic::{AtomicU32, Ordering},
};

macro_rules! ErrnoEnumGenerator {
    (assign $($entry:ident = $value:expr),*; map $($map_entry:ident),*) => {
        #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
        #[repr(i32)]
        pub enum Errno {
            $($entry = $value),*,
            $($map_entry = crate::internal::$map_entry as _),*,
            NOTIMPLEMENTED = i32::MAX
        }

        impl From<u32> for Errno {
            fn from(value: u32) -> Self {
                match value {
                    $($value => Errno::$entry),*,
                    $($crate::internal::$map_entry => Errno::$map_entry),*,
                    _ => Errno::NOTIMPLEMENTED
                }
            }
        }

        impl From<i32> for Errno {
            fn from(value: i32) -> Self {
                Errno::from(value as u32)
            }
        }

        impl Display for Errno {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                const BUFFER_SIZE: usize = 1024;
                let mut buffer: [char; BUFFER_SIZE] = [0; BUFFER_SIZE];
                unsafe { strerror_r(*self as i32, buffer.as_mut_ptr(), BUFFER_SIZE) };
                let s = match unsafe { CStr::from_ptr(buffer.as_ptr()) }.to_str() {
                    Ok(v) => v.to_string(),
                    Err(_) => "".to_string(),
                };

                match self {
                    $(Errno::$entry => {
                        write!(f, "errno {{ name = \"{}\", value = {}, details = \"{}\" }}",
                            stringify!($entry), Errno::$entry as i32, s)
                    }),*,
                    $(Errno::$map_entry => {
                        write!(f, "errno {{ name = \"{}\", value = {}, details = \"{}\" }}",
                            stringify!($map_entry), Errno::$map_entry as i32, s)
                    }),*,
                    Errno::NOTIMPLEMENTED => {
                        write!(f, "errno {{ name = \"NOTIMPLEMENTED\", value = {}, details = \"???\" }}",
                            Errno::NOTIMPLEMENTED as i32)
                    }
                }
            }
        }
    };
}

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

static GLOBAL_ERRNO_VALUE: AtomicU32 = AtomicU32::new(Errno::ESUCCES as _);

impl Errno {
    pub fn get() -> Errno {
        GLOBAL_ERRNO_VALUE.load(Ordering::Relaxed).into()
    }

    pub fn reset() {
        GLOBAL_ERRNO_VALUE.store(Errno::ESUCCES as _, Ordering::Relaxed);
    }

    pub(crate) fn set(value: Errno) {
        GLOBAL_ERRNO_VALUE.store(value as _, Ordering::Relaxed);
    }
}

pub unsafe fn strerror_r(errnum: int, buf: *mut char, buflen: size_t) -> int {
    let error = strerror(errnum);
    let len = || -> usize {
        for n in 0..buflen {
            if *error.add(n) == 0 {
                return n;
            }
        }
        buflen
    }();

    std::ptr::copy_nonoverlapping(error, buf, len);

    0
}

pub unsafe fn strerror(errnum: int) -> *const char {
    let errno: Errno = errnum.into();
    match errno {
        Errno::EINVAL => "Invalid input argument value.\0".as_ptr() as *const char,
        Errno::ENOSYS => "The feature is not defined and supported.\0".as_ptr() as *const char,
        Errno::ETIMEDOUT => "A user-provided timeout was hit.\0".as_ptr() as *const char,
        Errno::ENOENT => "A required system-resource does not exist.\0".as_ptr() as *const char,
        Errno::ENOTSUP => "The feature is not supported on this system.\0".as_ptr() as *const char,
        Errno::EBUSY => {
            "The resource is currently busy and unaccessable.\0".as_ptr() as *const char
        }
        _ => "Unknown error has occurred.\0".as_ptr() as *const char,
    }
}
