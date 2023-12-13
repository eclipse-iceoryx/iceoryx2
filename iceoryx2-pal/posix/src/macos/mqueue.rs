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

use crate::posix::types::*;

pub unsafe fn mq_open4(
    _name: *const char,
    _flags: int,
    _mode: mode_t,
    _attr: *mut mq_attr,
) -> mqd_t {
    //crate::internal::mq_open(name, flags, mode, attr)
    -1
}

pub unsafe fn mq_open2(_name: *const char, _flags: int) -> mqd_t {
    //crate::internal::mq_open(name, flags)
    -1
}

pub unsafe fn mq_close(_mqdes: mqd_t) -> int {
    //crate::internal::mq_close(mqdes)
    -1
}

pub unsafe fn mq_unlink(_name: *const char) -> int {
    //crate::internal::mq_unlink(name)
    -1
}

pub unsafe fn mq_getattr(_mqdes: mqd_t, _attr: *mut mq_attr) -> int {
    //crate::internal::mq_getattr(mqdes, attr)
    -1
}

pub unsafe fn mq_setattr(_mqdes: mqd_t, _newattr: *const mq_attr, _oldattr: *mut mq_attr) -> int {
    //crate::internal::mq_setattr(mqdes, newattr, oldattr)
    -1
}

pub unsafe fn mq_receive(
    _mqdes: mqd_t,
    _msg_ptr: *mut char,
    _msg_len: size_t,
    _msg_prio: *mut uint,
) -> ssize_t {
    //crate::internal::mq_receive(mqdes, msg_ptr, msg_len, msg_prio)
    -1
}

pub unsafe fn mq_timedreceive(
    _mqdes: mqd_t,
    _msg_ptr: *mut char,
    _msg_len: size_t,
    _msg_prio: *mut uint,
    _abs_timeout: *const timespec,
) -> ssize_t {
    //crate::internal::mq_timedreceive(mqdes, msg_ptr, msg_len, msg_prio, abs_timeout)
    -1
}

pub unsafe fn mq_send(
    _mqdes: mqd_t,
    _msg_ptr: *const char,
    _msg_len: size_t,
    _msg_prio: uint,
) -> int {
    //crate::internal::mq_send(mqdes, msg_ptr, msg_len, msg_prio)
    -1
}

pub unsafe fn mq_timedsend(
    _mqdes: mqd_t,
    _msg_ptr: *const char,
    _msg_len: size_t,
    _msg_prio: uint,
    _abs_timeout: *const timespec,
) -> int {
    //crate::internal::mq_timedsend(mqdes, msg_ptr, msg_len, msg_prio, abs_timeout)
    -1
}
