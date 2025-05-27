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
use crate::posix::Errno;
use crate::posix::MemZeroedStruct;

pub unsafe fn clock_gettime(clock_id: clockid_t, tp: *mut timespec) -> int {
    crate::internal::clock_gettime(clock_id, tp)
}

pub unsafe fn clock_settime(clock_id: clockid_t, tp: *const timespec) -> int {
    crate::internal::clock_settime(clock_id, tp)
}

pub unsafe fn clock_nanosleep(
    clock_id: clockid_t,
    _flags: int,
    rqtp: *const timespec,
    rmtp: *mut timespec,
) -> int {
    if clock_id != crate::posix::CLOCK_REALTIME {
        return Errno::ENOTSUP as _;
    }

    let mut now = timespec::new_zeroed();
    if clock_gettime(clock_id, &mut now) == -1 {
        return Errno::EINVAL as _;
    }

    let wait_time = if (now.tv_sec > (*rqtp).tv_sec)
        || (now.tv_sec == (*rqtp).tv_sec && now.tv_nsec >= (*rqtp).tv_nsec)
    {
        return 0;
    } else if now.tv_nsec <= (*rqtp).tv_nsec {
        timespec {
            tv_sec: (*rqtp).tv_sec - now.tv_sec,
            tv_nsec: (*rqtp).tv_nsec - now.tv_nsec,
        }
    } else {
        timespec {
            tv_sec: (*rqtp).tv_sec - now.tv_sec - 1,
            tv_nsec: 1000000000 + (*rqtp).tv_nsec - now.tv_nsec,
        }
    };

    if crate::internal::nanosleep(&wait_time, rmtp) == 0 {
        0
    } else {
        Errno::get() as _
    }
}
