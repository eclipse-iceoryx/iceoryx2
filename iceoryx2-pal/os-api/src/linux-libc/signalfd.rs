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

#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

use iceoryx2_pal_posix::posix::{self};

pub type signalfd_siginfo = libc::signalfd_siginfo;
pub const SFD_NONBLOCK: u32 = libc::SFD_NONBLOCK as _;
pub const SFD_CLOEXEC: u32 = libc::SFD_CLOEXEC as _;

pub unsafe fn signalfd(
    fd: posix::int,
    mask: *const posix::sigset_t,
    flags: posix::int,
) -> posix::int {
    libc::signalfd(fd, mask.cast(), flags)
}
