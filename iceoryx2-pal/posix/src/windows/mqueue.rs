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

#![allow(non_camel_case_types, non_snake_case)]
#![allow(clippy::missing_safety_doc)]
#![allow(unused_variables)]

use crate::posix::constants::*;
use crate::posix::types::*;

pub unsafe fn mq_open4(name: *const char, flags: int, mode: mode_t, attr: *mut mq_attr) -> mqd_t {
    MQ_INVALID
}

pub unsafe fn mq_open2(name: *const char, flags: int) -> mqd_t {
    MQ_INVALID
}

pub unsafe fn mq_close(mqdes: mqd_t) -> int {
    -1
}

pub unsafe fn mq_unlink(name: *const char) -> int {
    -1
}

pub unsafe fn mq_getattr(mqdes: mqd_t, attr: *mut mq_attr) -> int {
    -1
}

pub unsafe fn mq_setattr(mqdes: mqd_t, newattr: *const mq_attr, oldattr: *mut mq_attr) -> int {
    -1
}

pub unsafe fn mq_receive(
    mqdes: mqd_t,
    msg_ptr: *mut char,
    msg_len: size_t,
    msg_prio: *mut uint,
) -> ssize_t {
    -1
}

pub unsafe fn mq_timedreceive(
    mqdes: mqd_t,
    msg_ptr: *mut char,
    msg_len: size_t,
    msg_prio: *mut uint,
    abs_timeout: *const timespec,
) -> ssize_t {
    -1
}

pub unsafe fn mq_send(mqdes: mqd_t, msg_ptr: *const char, msg_len: size_t, msg_prio: uint) -> int {
    -1
}

pub unsafe fn mq_timedsend(
    mqdes: mqd_t,
    msg_ptr: *const char,
    msg_len: size_t,
    msg_prio: uint,
    abs_timeout: *const timespec,
) -> int {
    -1
}
