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

#![allow(non_camel_case_types, non_snake_case)]
#![allow(clippy::missing_safety_doc)]
#![allow(unused_variables)]

use core::ffi::CStr;
use core::unimplemented;

use alloc::string::ToString;
use iceoryx2_pal_concurrency_sync::atomic::AtomicU32;

use crate::posix::types::*;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum Errno {
    ESUCCES = 0,
    EPERM = 1,
    ENOENT = 2,
    ESRCH = 3,
    EINTR = 4,
    EIO = 5,
    ENXIO = 6,
    E2BIG = 7,
    ENOEXEC = 8,
    EBADF = 9,
    ECHILD = 10,
    EAGAIN = 11,
    ENOMEM = 12,
    EACCES = 13,
    EFAULT = 14,
    ENOTBLK = 15,
    EBUSY = 16,
    EEXIST = 17,
    EXDEV = 18,
    ENODEV = 19,
    ENOTDIR = 20,
    EISDIR = 21,
    EINVAL = 22,
    ENFILE = 23,
    EMFILE = 24,
    ENOTTY = 25,
    ETXTBSY = 26,
    EFBIG = 27,
    ENOSPC = 28,
    ESPIPE = 29,
    EROFS = 30,
    EMLINK = 31,
    EPIPE = 32,
    EDOM = 33,
    ERANGE = 34,
    // GNU extensions for POSIX
    EDEADLK = 35,
    ENAMETOOLONG = 36,
    ENOLCK = 37,
    ENOSYS = 38,
    ENOTEMPTY = 39,
    ELOOP = 40,
    ENOMSG = 42,
    EIDRM = 43,
    ECHRNG = 44,
    EL2NSYNC = 45,
    EL3HLT = 46,
    EL3RST = 47,
    ELNRNG = 48,
    EUNATCH = 49,
    ENOCSI = 50,
    EL2HLT = 51,
    EBADE = 52,
    EBADR = 53,
    EXFULL = 54,
    ENOANO = 55,
    EBADRQC = 56,
    EBADSLT = 57,
    EMULTIHOP = 72,
    EOVERFLOW = 75,
    EBADMSG = 74,
    ENOTUNIQ = 76,
    EBADFD = 77,
    EILSEQ = 84,
    EREMCHG = 78,
    ELIBACC = 79,
    ELIBBAD = 80,
    ELIBSCN = 81,
    ELIBMAX = 82,
    ELIBEXEC = 83,
    EUSERS = 87,
    ENOTSOCK = 88,
    EDESTADDRREQ = 89,
    EMSGSIZE = 90,
    EPROTOTYPE = 91,
    ENOPROTOOPT = 92,
    EPROTONOSUPPORT = 93,
    ESOCKTNOSUPPORT = 94,
    ENOTSUP = 95,
    EPFNOSUPPORT = 96,
    EAFNOSUPPORT = 97,
    EADDRINUSE = 98,
    EADDRNOTAVAIL = 99,
    ENETDOWN = 100,
    ENETUNREACH = 101,
    ENETRESET = 102,
    ECONNABORTED = 103,
    ECONNRESET = 104,
    ENOBUFS = 105,
    EISCONN = 106,
    ENOTCONN = 107,
    ESHUTDOWN = 108,
    ETOOMANYREFS = 109,
    ETIMEDOUT = 110,
    ECONNREFUSED = 111,
    EHOSTDOWN = 112,
    EHOSTUNREACH = 113,
    EALREADY = 114,
    EINPROGRESS = 115,
    ESTALE = 116,
    ERESTART = 117,
    ESTRPIPE = 118,
    EDQUOT = 122,
    ENOMEDIUM = 123,
    EMEDIUMTYPE = 124,
    ECANCELED = 125,
    ENOKEY = 126,
    EKEYEXPIRED = 127,
    EKEYREVOKED = 128,
    EKEYREJECTED = 129,
    EOWNERDEAD = 130,
    ENOTRECOVERABLE = 131,
    ERFKILL = 132,
    EHWPOISON = 133,
    NOTIMPLEMENTED = i32::MAX,
}

pub static GLOBAL_ERRNO_VALUE: AtomicU32 = AtomicU32::new(Errno::ESUCCES as _);
impl Errno {
    pub fn get() -> Errno {
        GLOBAL_ERRNO_VALUE
            .load(core::sync::atomic::Ordering::Relaxed)
            .into()
    }

    pub fn reset() {
        Errno::set(Errno::ESUCCES);
    }

    pub(crate) fn set(value: Errno) {
        GLOBAL_ERRNO_VALUE.store(value as _, core::sync::atomic::Ordering::Relaxed);
    }
}

pub unsafe fn strerror_r(errnum: int, buf: *mut c_char, buflen: size_t) -> int {
    unimplemented!("")
}

pub unsafe fn strerror(errnum: int) -> *const c_char {
    unimplemented!("")
}

#[allow(clippy::from_over_into)]
impl Into<Errno> for u32 {
    #[deny(clippy::from_over_into)]
    fn into(self) -> Errno {
        match self {
            0 => Errno::ESUCCES,
            1 => Errno::EPERM,
            2 => Errno::ENOENT,
            3 => Errno::ESRCH,
            4 => Errno::EINTR,
            5 => Errno::EIO,
            6 => Errno::ENXIO,
            7 => Errno::E2BIG,
            8 => Errno::ENOEXEC,
            9 => Errno::EBADF,
            10 => Errno::ECHILD,
            11 => Errno::EAGAIN,
            12 => Errno::ENOMEM,
            13 => Errno::EACCES,
            14 => Errno::EFAULT,
            15 => Errno::ENOTBLK,
            16 => Errno::EBUSY,
            17 => Errno::EEXIST,
            18 => Errno::EXDEV,
            19 => Errno::ENODEV,
            20 => Errno::ENOTDIR,
            21 => Errno::EISDIR,
            22 => Errno::EINVAL,
            23 => Errno::ENFILE,
            24 => Errno::EMFILE,
            25 => Errno::ENOTTY,
            26 => Errno::ETXTBSY,
            27 => Errno::EFBIG,
            28 => Errno::ENOSPC,
            29 => Errno::ESPIPE,
            30 => Errno::EROFS,
            31 => Errno::EMLINK,
            32 => Errno::EPIPE,
            33 => Errno::EDOM,
            34 => Errno::ERANGE,
            35 => Errno::EDEADLK,
            36 => Errno::ENAMETOOLONG,
            37 => Errno::ENOLCK,
            38 => Errno::ENOSYS,
            39 => Errno::ENOTEMPTY,
            40 => Errno::ELOOP,
            42 => Errno::ENOMSG,
            43 => Errno::EIDRM,
            72 => Errno::EMULTIHOP,
            74 => Errno::EBADMSG,
            75 => Errno::EOVERFLOW,
            84 => Errno::EILSEQ,
            87 => Errno::EUSERS,
            88 => Errno::ENOTSOCK,
            89 => Errno::EDESTADDRREQ,
            90 => Errno::EMSGSIZE,
            91 => Errno::EPROTOTYPE,
            92 => Errno::ENOPROTOOPT,
            93 => Errno::EPROTONOSUPPORT,
            94 => Errno::ESOCKTNOSUPPORT,
            95 => Errno::ENOTSUP,
            96 => Errno::EPFNOSUPPORT,
            97 => Errno::EAFNOSUPPORT,
            98 => Errno::EADDRINUSE,
            99 => Errno::EADDRNOTAVAIL,
            100 => Errno::ENETDOWN,
            101 => Errno::ENETUNREACH,
            102 => Errno::ENETRESET,
            103 => Errno::ECONNABORTED,
            104 => Errno::ECONNRESET,
            105 => Errno::ENOBUFS,
            106 => Errno::EISCONN,
            107 => Errno::ENOTCONN,
            108 => Errno::ESHUTDOWN,
            109 => Errno::ETOOMANYREFS,
            110 => Errno::ETIMEDOUT,
            111 => Errno::ECONNREFUSED,
            112 => Errno::EHOSTDOWN,
            113 => Errno::EHOSTUNREACH,
            114 => Errno::EALREADY,
            115 => Errno::EINPROGRESS,
            116 => Errno::ESTALE,
            122 => Errno::EDQUOT,
            125 => Errno::ECANCELED,
            130 => Errno::EOWNERDEAD,
            131 => Errno::ENOTRECOVERABLE,
            44 => Errno::ECHRNG,
            45 => Errno::EL2NSYNC,
            46 => Errno::EL3HLT,
            47 => Errno::EL3RST,
            48 => Errno::ELNRNG,
            49 => Errno::EUNATCH,
            50 => Errno::ENOCSI,
            51 => Errno::EL2HLT,
            52 => Errno::EBADE,
            53 => Errno::EBADR,
            54 => Errno::EXFULL,
            55 => Errno::ENOANO,
            56 => Errno::EBADRQC,
            57 => Errno::EBADSLT,
            76 => Errno::ENOTUNIQ,
            77 => Errno::EBADFD,
            78 => Errno::EREMCHG,
            79 => Errno::ELIBACC,
            80 => Errno::ELIBBAD,
            81 => Errno::ELIBSCN,
            82 => Errno::ELIBMAX,
            83 => Errno::ELIBEXEC,
            117 => Errno::ERESTART,
            118 => Errno::ESTRPIPE,
            123 => Errno::ENOMEDIUM,
            124 => Errno::EMEDIUMTYPE,
            126 => Errno::ENOKEY,
            127 => Errno::EKEYEXPIRED,
            128 => Errno::EKEYREVOKED,
            129 => Errno::EKEYREJECTED,
            132 => Errno::ERFKILL,
            133 => Errno::EHWPOISON,
            _ => Errno::NOTIMPLEMENTED,
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

impl core::fmt::Display for Errno {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        const BUFFER_SIZE: usize = 1024;
        let mut buffer: [c_char; BUFFER_SIZE] = [0; BUFFER_SIZE];
        unsafe { strerror_r(*self as i32, buffer.as_mut_ptr(), BUFFER_SIZE) };
        let s = match unsafe { CStr::from_ptr(buffer.as_ptr()) }.to_str() {
            Ok(v) => v.to_string(),
            Err(_) => "".to_string(),
        };

        match self {
            Errno::ESUCCES => core::write!(
                f,
                "errno {{ name = \"ESUCCES\", value = {}, details = \"{}\" }}",
                Errno::ESUCCES as i32,
                s
            ),
            Errno::EPERM => core::write!(
                f,
                "errno {{ name = \"EPERM\", value = {}, details = \"{}\" }}",
                Errno::EPERM as i32,
                s
            ),
            Errno::ENOENT => core::write!(
                f,
                "errno {{ name = \"ENOENT\", value = {}, details = \"{}\" }}",
                Errno::ENOENT as i32,
                s
            ),
            Errno::ESRCH => core::write!(
                f,
                "errno {{ name = \"ESRCH\", value = {}, details = \"{}\" }}",
                Errno::ESRCH as i32,
                s
            ),
            Errno::EINTR => core::write!(
                f,
                "errno {{ name = \"EINTR\", value = {}, details = \"{}\" }}",
                Errno::EINTR as i32,
                s
            ),
            Errno::EIO => core::write!(
                f,
                "errno {{ name = \"EIO\", value = {}, details = \"{}\" }}",
                Errno::EIO as i32,
                s
            ),
            Errno::ENXIO => core::write!(
                f,
                "errno {{ name = \"ENXIO\", value = {}, details = \"{}\" }}",
                Errno::ENXIO as i32,
                s
            ),
            Errno::E2BIG => core::write!(
                f,
                "errno {{ name = \"E2BIG\", value = {}, details = \"{}\" }}",
                Errno::E2BIG as i32,
                s
            ),
            Errno::ENOEXEC => core::write!(
                f,
                "errno {{ name = \"ENOEXEC\", value = {}, details = \"{}\" }}",
                Errno::ENOEXEC as i32,
                s
            ),
            Errno::EBADF => core::write!(
                f,
                "errno {{ name = \"EBADF\", value = {}, details = \"{}\" }}",
                Errno::EBADF as i32,
                s
            ),
            Errno::ECHILD => core::write!(
                f,
                "errno {{ name = \"ECHILD\", value = {}, details = \"{}\" }}",
                Errno::ECHILD as i32,
                s
            ),
            Errno::EAGAIN => core::write!(
                f,
                "errno {{ name = \"EAGAIN\", value = {}, details = \"{}\" }}",
                Errno::EAGAIN as i32,
                s
            ),
            Errno::ENOMEM => core::write!(
                f,
                "errno {{ name = \"ENOMEM\", value = {}, details = \"{}\" }}",
                Errno::ENOMEM as i32,
                s
            ),
            Errno::EACCES => core::write!(
                f,
                "errno {{ name = \"EACCES\", value = {}, details = \"{}\" }}",
                Errno::EACCES as i32,
                s
            ),
            Errno::EFAULT => core::write!(
                f,
                "errno {{ name = \"EFAULT\", value = {}, details = \"{}\" }}",
                Errno::EFAULT as i32,
                s
            ),
            Errno::ENOTBLK => core::write!(
                f,
                "errno {{ name = \"ENOTBLK\", value = {}, details = \"{}\" }}",
                Errno::ENOTBLK as i32,
                s
            ),
            Errno::EBUSY => core::write!(
                f,
                "errno {{ name = \"EBUSY\", value = {}, details = \"{}\" }}",
                Errno::EBUSY as i32,
                s
            ),
            Errno::EEXIST => core::write!(
                f,
                "errno {{ name = \"EEXIST\", value = {}, details = \"{}\" }}",
                Errno::EEXIST as i32,
                s
            ),
            Errno::EXDEV => core::write!(
                f,
                "errno {{ name = \"EXDEV\", value = {}, details = \"{}\" }}",
                Errno::EXDEV as i32,
                s
            ),
            Errno::ENODEV => core::write!(
                f,
                "errno {{ name = \"ENODEV\", value = {}, details = \"{}\" }}",
                Errno::ENODEV as i32,
                s
            ),
            Errno::ENOTDIR => core::write!(
                f,
                "errno {{ name = \"ENOTDIR\", value = {}, details = \"{}\" }}",
                Errno::ENOTDIR as i32,
                s
            ),
            Errno::EISDIR => core::write!(
                f,
                "errno {{ name = \"EISDIR\", value = {}, details = \"{}\" }}",
                Errno::EISDIR as i32,
                s
            ),
            Errno::EINVAL => core::write!(
                f,
                "errno {{ name = \"EINVAL\", value = {}, details = \"{}\" }}",
                Errno::EINVAL as i32,
                s
            ),
            Errno::ENFILE => core::write!(
                f,
                "errno {{ name = \"ENFILE\", value = {}, details = \"{}\" }}",
                Errno::ENFILE as i32,
                s
            ),
            Errno::EMFILE => core::write!(
                f,
                "errno {{ name = \"EMFILE\", value = {}, details = \"{}\" }}",
                Errno::EMFILE as i32,
                s
            ),
            Errno::ENOTTY => core::write!(
                f,
                "errno {{ name = \"ENOTTY\", value = {}, details = \"{}\" }}",
                Errno::ENOTTY as i32,
                s
            ),
            Errno::ETXTBSY => core::write!(
                f,
                "errno {{ name = \"ETXTBSY\", value = {}, details = \"{}\" }}",
                Errno::ETXTBSY as i32,
                s
            ),
            Errno::EFBIG => core::write!(
                f,
                "errno {{ name = \"EFBIG\", value = {}, details = \"{}\" }}",
                Errno::EFBIG as i32,
                s
            ),
            Errno::ENOSPC => core::write!(
                f,
                "errno {{ name = \"ENOSPC\", value = {}, details = \"{}\" }}",
                Errno::ENOSPC as i32,
                s
            ),
            Errno::ESPIPE => core::write!(
                f,
                "errno {{ name = \"ESPIPE\", value = {}, details = \"{}\" }}",
                Errno::ESPIPE as i32,
                s
            ),
            Errno::EROFS => core::write!(
                f,
                "errno {{ name = \"EROFS\", value = {}, details = \"{}\" }}",
                Errno::EROFS as i32,
                s
            ),
            Errno::EMLINK => core::write!(
                f,
                "errno {{ name = \"EMLINK\", value = {}, details = \"{}\" }}",
                Errno::EMLINK as i32,
                s
            ),
            Errno::EPIPE => core::write!(
                f,
                "errno {{ name = \"EPIPE\", value = {}, details = \"{}\" }}",
                Errno::EPIPE as i32,
                s
            ),
            Errno::EDOM => core::write!(
                f,
                "errno {{ name = \"EDOM\", value = {}, details = \"{}\" }}",
                Errno::EDOM as i32,
                s
            ),
            Errno::ERANGE => core::write!(
                f,
                "errno {{ name = \"ERANGE\", value = {}, details = \"{}\" }}",
                Errno::ERANGE as i32,
                s
            ),
            Errno::EDEADLK => core::write!(
                f,
                "errno {{ name = \"EDEADLK\", value = {}, details = \"{}\" }}",
                Errno::EDEADLK as i32,
                s
            ),
            Errno::ENAMETOOLONG => core::write!(
                f,
                "errno {{ name = \"ENAMETOOLONG\", value = {}, details = \"{}\" }}",
                Errno::ENAMETOOLONG as i32,
                s
            ),
            Errno::ENOLCK => core::write!(
                f,
                "errno {{ name = \"ENOLCK\", value = {}, details = \"{}\" }}",
                Errno::ENOLCK as i32,
                s
            ),
            Errno::ENOSYS => core::write!(
                f,
                "errno {{ name = \"ENOSYS\", value = {}, details = \"{}\" }}",
                Errno::ENOSYS as i32,
                s
            ),
            Errno::ENOTEMPTY => core::write!(
                f,
                "errno {{ name = \"ENOTEMPTY\", value = {}, details = \"{}\" }}",
                Errno::ENOTEMPTY as i32,
                s
            ),
            Errno::ELOOP => core::write!(
                f,
                "errno {{ name = \"ELOOP\", value = {}, details = \"{}\" }}",
                Errno::ELOOP as i32,
                s
            ),
            Errno::ENOMSG => core::write!(
                f,
                "errno {{ name = \"ENOMSG\", value = {}, details = \"{}\" }}",
                Errno::ENOMSG as i32,
                s
            ),
            Errno::EIDRM => core::write!(
                f,
                "errno {{ name = \"EIDRM\", value = {}, details = \"{}\" }}",
                Errno::EIDRM as i32,
                s
            ),
            Errno::EMULTIHOP => core::write!(
                f,
                "errno {{ name = \"EMULTIHOP\", value = {}, details = \"{}\" }}",
                Errno::EMULTIHOP as i32,
                s
            ),
            Errno::EOVERFLOW => core::write!(
                f,
                "errno {{ name = \"EOVERFLOW\", value = {}, details = \"{}\" }}",
                Errno::EOVERFLOW as i32,
                s
            ),
            Errno::EBADMSG => core::write!(
                f,
                "errno {{ name = \"EBADMSG\", value = {}, details = \"{}\" }}",
                Errno::EBADMSG as i32,
                s
            ),
            Errno::EILSEQ => core::write!(
                f,
                "errno {{ name = \"EILSEQ\", value = {}, details = \"{}\" }}",
                Errno::EILSEQ as i32,
                s
            ),
            Errno::EUSERS => core::write!(
                f,
                "errno {{ name = \"EUSERS\", value = {}, details = \"{}\" }}",
                Errno::EUSERS as i32,
                s
            ),
            Errno::ENOTSOCK => core::write!(
                f,
                "errno {{ name = \"ENOTSOCK\", value = {}, details = \"{}\" }}",
                Errno::ENOTSOCK as i32,
                s
            ),
            Errno::EDESTADDRREQ => core::write!(
                f,
                "errno {{ name = \"EDESTADDRREQ\", value = {}, details = \"{}\" }}",
                Errno::EDESTADDRREQ as i32,
                s
            ),
            Errno::EMSGSIZE => core::write!(
                f,
                "errno {{ name = \"EMSGSIZE\", value = {}, details = \"{}\" }}",
                Errno::EMSGSIZE as i32,
                s
            ),
            Errno::EPROTOTYPE => core::write!(
                f,
                "errno {{ name = \"EPROTOTYPE\", value = {}, details = \"{}\" }}",
                Errno::EPROTOTYPE as i32,
                s
            ),
            Errno::ENOPROTOOPT => core::write!(
                f,
                "errno {{ name = \"ENOPROTOOPT\", value = {}, details = \"{}\" }}",
                Errno::ENOPROTOOPT as i32,
                s
            ),
            Errno::EPROTONOSUPPORT => core::write!(
                f,
                "errno {{ name = \"EPROTONOSUPPORT\", value = {}, details = \"{}\" }}",
                Errno::EPROTONOSUPPORT as i32,
                s
            ),
            Errno::ESOCKTNOSUPPORT => core::write!(
                f,
                "errno {{ name = \"ESOCKTNOSUPPORT\", value = {}, details = \"{}\" }}",
                Errno::ESOCKTNOSUPPORT as i32,
                s
            ),
            Errno::ENOTSUP => core::write!(
                f,
                "errno {{ name = \"ENOTSUP\", value = {}, details = \"{}\" }}",
                Errno::ENOTSUP as i32,
                s
            ),
            Errno::EPFNOSUPPORT => core::write!(
                f,
                "errno {{ name = \"EPFNOSUPPORT\", value = {}, details = \"{}\" }}",
                Errno::EPFNOSUPPORT as i32,
                s
            ),
            Errno::EAFNOSUPPORT => core::write!(
                f,
                "errno {{ name = \"EAFNOSUPPORT\", value = {}, details = \"{}\" }}",
                Errno::EAFNOSUPPORT as i32,
                s
            ),
            Errno::EADDRINUSE => core::write!(
                f,
                "errno {{ name = \"EADDRINUSE\", value = {}, details = \"{}\" }}",
                Errno::EADDRINUSE as i32,
                s
            ),
            Errno::EADDRNOTAVAIL => core::write!(
                f,
                "errno {{ name = \"EADDRNOTAVAIL\", value = {}, details = \"{}\" }}",
                Errno::EADDRNOTAVAIL as i32,
                s
            ),
            Errno::ENETDOWN => core::write!(
                f,
                "errno {{ name = \"ENETDOWN\", value = {}, details = \"{}\" }}",
                Errno::ENETDOWN as i32,
                s
            ),
            Errno::ENETUNREACH => core::write!(
                f,
                "errno {{ name = \"ENETUNREACH\", value = {}, details = \"{}\" }}",
                Errno::ENETUNREACH as i32,
                s
            ),
            Errno::ENETRESET => core::write!(
                f,
                "errno {{ name = \"ENETRESET\", value = {}, details = \"{}\" }}",
                Errno::ENETRESET as i32,
                s
            ),
            Errno::ECONNABORTED => core::write!(
                f,
                "errno {{ name = \"ECONNABORTED\", value = {}, details = \"{}\" }}",
                Errno::ECONNABORTED as i32,
                s
            ),
            Errno::ECONNRESET => core::write!(
                f,
                "errno {{ name = \"ECONNRESET\", value = {}, details = \"{}\" }}",
                Errno::ECONNRESET as i32,
                s
            ),
            Errno::ENOBUFS => core::write!(
                f,
                "errno {{ name = \"ENOBUFS\", value = {}, details = \"{}\" }}",
                Errno::ENOBUFS as i32,
                s
            ),
            Errno::EISCONN => core::write!(
                f,
                "errno {{ name = \"EISCONN\", value = {}, details = \"{}\" }}",
                Errno::EISCONN as i32,
                s
            ),
            Errno::ENOTCONN => core::write!(
                f,
                "errno {{ name = \"ENOTCONN\", value = {}, details = \"{}\" }}",
                Errno::ENOTCONN as i32,
                s
            ),
            Errno::ESHUTDOWN => core::write!(
                f,
                "errno {{ name = \"ESHUTDOWN\", value = {}, details = \"{}\" }}",
                Errno::ESHUTDOWN as i32,
                s
            ),
            Errno::ETOOMANYREFS => core::write!(
                f,
                "errno {{ name = \"ETOOMANYREFS\", value = {}, details = \"{}\" }}",
                Errno::ETOOMANYREFS as i32,
                s
            ),
            Errno::ETIMEDOUT => core::write!(
                f,
                "errno {{ name = \"ETIMEDOUT\", value = {}, details = \"{}\" }}",
                Errno::ETIMEDOUT as i32,
                s
            ),
            Errno::ECONNREFUSED => core::write!(
                f,
                "errno {{ name = \"ECONNREFUSED\", value = {}, details = \"{}\" }}",
                Errno::ECONNREFUSED as i32,
                s
            ),
            Errno::EHOSTDOWN => core::write!(
                f,
                "errno {{ name = \"EHOSTDOWN\", value = {}, details = \"{}\" }}",
                Errno::EHOSTDOWN as i32,
                s
            ),
            Errno::EHOSTUNREACH => core::write!(
                f,
                "errno {{ name = \"EHOSTUNREACH\", value = {}, details = \"{}\" }}",
                Errno::EHOSTUNREACH as i32,
                s
            ),
            Errno::EALREADY => core::write!(
                f,
                "errno {{ name = \"EALREADY\", value = {}, details = \"{}\" }}",
                Errno::EALREADY as i32,
                s
            ),
            Errno::EINPROGRESS => core::write!(
                f,
                "errno {{ name = \"EINPROGRESS\", value = {}, details = \"{}\" }}",
                Errno::EINPROGRESS as i32,
                s
            ),
            Errno::ESTALE => core::write!(
                f,
                "errno {{ name = \"ESTALE\", value = {}, details = \"{}\" }}",
                Errno::ESTALE as i32,
                s
            ),
            Errno::EDQUOT => core::write!(
                f,
                "errno {{ name = \"EDQUOT\", value = {}, details = \"{}\" }}",
                Errno::EDQUOT as i32,
                s
            ),
            Errno::ECHRNG => core::write!(
                f,
                "errno {{ name = \"ECHRNG\", value = {}, details = \"{}\" }}",
                Errno::ECHRNG as i32,
                s
            ),
            Errno::EL2NSYNC => core::write!(
                f,
                "errno {{ name = \"EL2NSYNC\", value = {}, details = \"{}\" }}",
                Errno::EL2NSYNC as i32,
                s
            ),
            Errno::EL3HLT => core::write!(
                f,
                "errno {{ name = \"EL3HLT\", value = {}, details = \"{}\" }}",
                Errno::EL3HLT as i32,
                s
            ),
            Errno::EL3RST => core::write!(
                f,
                "errno {{ name = \"EL3RST\", value = {}, details = \"{}\" }}",
                Errno::EL3RST as i32,
                s
            ),
            Errno::ELNRNG => core::write!(
                f,
                "errno {{ name = \"ELNRNG\", value = {}, details = \"{}\" }}",
                Errno::ELNRNG as i32,
                s
            ),
            Errno::EUNATCH => core::write!(
                f,
                "errno {{ name = \"EUNATCH\", value = {}, details = \"{}\" }}",
                Errno::EUNATCH as i32,
                s
            ),
            Errno::ENOCSI => core::write!(
                f,
                "errno {{ name = \"ENOCSI\", value = {}, details = \"{}\" }}",
                Errno::ENOCSI as i32,
                s
            ),
            Errno::EL2HLT => core::write!(
                f,
                "errno {{ name = \"EL2HLT\", value = {}, details = \"{}\" }}",
                Errno::EL2HLT as i32,
                s
            ),
            Errno::EBADE => core::write!(
                f,
                "errno {{ name = \"EBADE\", value = {}, details = \"{}\" }}",
                Errno::EBADE as i32,
                s
            ),
            Errno::EBADR => core::write!(
                f,
                "errno {{ name = \"EBADR\", value = {}, details = \"{}\" }}",
                Errno::EBADR as i32,
                s
            ),
            Errno::EXFULL => core::write!(
                f,
                "errno {{ name = \"EXFULL\", value = {}, details = \"{}\" }}",
                Errno::EXFULL as i32,
                s
            ),
            Errno::ENOANO => core::write!(
                f,
                "errno {{ name = \"ENOANO\", value = {}, details = \"{}\" }}",
                Errno::ENOANO as i32,
                s
            ),
            Errno::EBADRQC => core::write!(
                f,
                "errno {{ name = \"EBADRQC\", value = {}, details = \"{}\" }}",
                Errno::EBADRQC as i32,
                s
            ),
            Errno::EBADSLT => core::write!(
                f,
                "errno {{ name = \"EBADSLT\", value = {}, details = \"{}\" }}",
                Errno::EBADSLT as i32,
                s
            ),
            Errno::ENOTUNIQ => core::write!(
                f,
                "errno {{ name = \"ENOTUNIQ\", value = {}, details = \"{}\" }}",
                Errno::ENOTUNIQ as i32,
                s
            ),
            Errno::EBADFD => core::write!(
                f,
                "errno {{ name = \"EBADFD\", value = {}, details = \"{}\" }}",
                Errno::EBADFD as i32,
                s
            ),
            Errno::EREMCHG => core::write!(
                f,
                "errno {{ name = \"EREMCHG\", value = {}, details = \"{}\" }}",
                Errno::EREMCHG as i32,
                s
            ),
            Errno::ELIBACC => core::write!(
                f,
                "errno {{ name = \"ELIBACC\", value = {}, details = \"{}\" }}",
                Errno::ELIBACC as i32,
                s
            ),
            Errno::ELIBBAD => core::write!(
                f,
                "errno {{ name = \"ELIBBAD\", value = {}, details = \"{}\" }}",
                Errno::ELIBBAD as i32,
                s
            ),
            Errno::ELIBSCN => core::write!(
                f,
                "errno {{ name = \"ELIBSCN\", value = {}, details = \"{}\" }}",
                Errno::ELIBSCN as i32,
                s
            ),
            Errno::ELIBMAX => core::write!(
                f,
                "errno {{ name = \"ELIBMAX\", value = {}, details = \"{}\" }}",
                Errno::ELIBMAX as i32,
                s
            ),
            Errno::ELIBEXEC => core::write!(
                f,
                "errno {{ name = \"ELIBEXEC\", value = {}, details = \"{}\" }}",
                Errno::ELIBEXEC as i32,
                s
            ),
            Errno::ERESTART => core::write!(
                f,
                "errno {{ name = \"ERESTART\", value = {}, details = \"{}\" }}",
                Errno::ERESTART as i32,
                s
            ),
            Errno::ESTRPIPE => core::write!(
                f,
                "errno {{ name = \"ESTRPIPE\", value = {}, details = \"{}\" }}",
                Errno::ESTRPIPE as i32,
                s
            ),
            Errno::ENOMEDIUM => core::write!(
                f,
                "errno {{ name = \"ENOMEDIUM\", value = {}, details = \"{}\" }}",
                Errno::ENOMEDIUM as i32,
                s
            ),
            Errno::EMEDIUMTYPE => core::write!(
                f,
                "errno {{ name = \"EMEDIUMTYPE\", value = {}, details = \"{}\" }}",
                Errno::EMEDIUMTYPE as i32,
                s
            ),
            Errno::ECANCELED => core::write!(
                f,
                "errno {{ name = \"ECANCELED\", value = {}, details = \"{}\" }}",
                Errno::ECANCELED as i32,
                s
            ),
            Errno::ENOKEY => core::write!(
                f,
                "errno {{ name = \"ENOKEY\", value = {}, details = \"{}\" }}",
                Errno::ENOKEY as i32,
                s
            ),
            Errno::EKEYEXPIRED => core::write!(
                f,
                "errno {{ name = \"EKEYEXPIRED\", value = {}, details = \"{}\" }}",
                Errno::EKEYEXPIRED as i32,
                s
            ),
            Errno::EKEYREVOKED => core::write!(
                f,
                "errno {{ name = \"EKEYREVOKED\", value = {}, details = \"{}\" }}",
                Errno::EKEYREVOKED as i32,
                s
            ),
            Errno::EKEYREJECTED => core::write!(
                f,
                "errno {{ name = \"EKEYREJECTED\", value = {}, details = \"{}\" }}",
                Errno::EKEYREJECTED as i32,
                s
            ),
            Errno::EOWNERDEAD => core::write!(
                f,
                "errno {{ name = \"EOWNERDEAD\", value = {}, details = \"{}\" }}",
                Errno::EOWNERDEAD as i32,
                s
            ),
            Errno::ENOTRECOVERABLE => core::write!(
                f,
                "errno {{ name = \"ENOTRECOVERABLE\", value = {}, details = \"{}\" }}",
                Errno::ENOTRECOVERABLE as i32,
                s
            ),
            Errno::ERFKILL => core::write!(
                f,
                "errno {{ name = \"ERFKILL\", value = {}, details = \"{}\" }}",
                Errno::ERFKILL as i32,
                s
            ),
            Errno::EHWPOISON => core::write!(
                f,
                "errno {{ name = \"EHWPOISON\", value = {}, details = \"{}\" }}",
                Errno::EHWPOISON as i32,
                s
            ),
            Errno::NOTIMPLEMENTED => core::write!(
                f,
                "errno {{ name = \"NOTIMPLEMENTED\", value = {}, details = \"???\" }}",
                Errno::NOTIMPLEMENTED as i32
            ),
        }
    }
}
