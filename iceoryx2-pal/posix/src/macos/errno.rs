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
use core::{ffi::CStr, fmt::Display};

macro_rules! ErrnoEnumGenerator {
    (assign $($entry:ident = $value:expr),*; map $($map_entry:ident),*) => {
        #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
        #[repr(i32)]
        pub enum Errno {
            $($entry = $value),*,
            $($map_entry = crate::internal::$map_entry as _),*,
            NOTIMPLEMENTED = i32::MAX
        }

        // we explicitly only want to convert from enum to i32 and not the other way around
        #[allow(clippy::from_over_into)]
        impl Into<Errno> for u32 {
        #[deny(clippy::from_over_into)]
            fn into(self) -> Errno {
                match self {
                    $($value => Errno::$entry),*,
                    $($crate::internal::$map_entry => Errno::$map_entry),*,
                    _ => Errno::NOTIMPLEMENTED
                }
            }
        }

        #[allow(clippy::from_over_into)]
        impl Into<Errno> for i32 {
        #[deny(clippy::from_over_into)]
            fn into(self) -> Errno {
                (self as u32).into()
            }
        }

        impl Display for Errno {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                const BUFFER_SIZE: usize = 1024;
                let mut buffer: [c_char; BUFFER_SIZE] = [0; BUFFER_SIZE];
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
        unsafe { *crate::internal::__error() }.into()
    }

    pub fn set(value: Errno) {
        unsafe { *internal::__error() = value as i32 };
    }

    pub fn reset() {
        unsafe { *crate::internal::__error() = 0 };
    }
}

#[cfg(target_os = "linux")]
pub unsafe fn strerror_r(errnum: int, buf: *mut c_char, buflen: size_t) -> int {
    use core::sync::atomic::{AtomicBool, Ordering};
    static IS_LOCKED: AtomicBool = AtomicBool::new(false);

    while IS_LOCKED
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_err()
    {}

    let raw_string = strerror(errnum);
    crate::posix::string::strncpy(buf, raw_string, buflen);

    IS_LOCKED.store(false, Ordering::Relaxed);

    0
}

pub unsafe fn strerror_r(errnum: int, buf: *mut c_char, buflen: size_t) -> int {
    internal::strerror_r(errnum, buf, buflen)
}

pub unsafe fn strerror(errnum: int) -> *const c_char {
    crate::internal::strerror(errnum)
}

mod internal {
    use super::*;
    extern "C" {
        pub(super) fn __error() -> *mut int;
        pub(super) fn strerror_r(errnum: int, buf: *mut c_char, buflen: size_t) -> int;
    }
}
