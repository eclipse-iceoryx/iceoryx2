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

pub type EPOLL_EVENTS = crate::internal::EPOLL_EVENTS;

pub const EPOLL_CLOEXEC: usize = crate::internal::EPOLL_CLOEXEC as _;
pub const EPOLL_EVENTS_EPOLLIN: EPOLL_EVENTS = crate::internal::EPOLL_EVENTS_EPOLLIN;
pub const EPOLL_EVENTS_EPOLLPRI: EPOLL_EVENTS = crate::internal::EPOLL_EVENTS_EPOLLPRI;
pub const EPOLL_EVENTS_EPOLLOUT: EPOLL_EVENTS = crate::internal::EPOLL_EVENTS_EPOLLOUT;
pub const EPOLL_EVENTS_EPOLLRDNORM: EPOLL_EVENTS = crate::internal::EPOLL_EVENTS_EPOLLRDNORM;
pub const EPOLL_EVENTS_EPOLLRDBAND: EPOLL_EVENTS = crate::internal::EPOLL_EVENTS_EPOLLRDBAND;
pub const EPOLL_EVENTS_EPOLLWRNORM: EPOLL_EVENTS = crate::internal::EPOLL_EVENTS_EPOLLWRNORM;
pub const EPOLL_EVENTS_EPOLLWRBAND: EPOLL_EVENTS = crate::internal::EPOLL_EVENTS_EPOLLWRBAND;
pub const EPOLL_EVENTS_EPOLLMSG: EPOLL_EVENTS = crate::internal::EPOLL_EVENTS_EPOLLMSG;
pub const EPOLL_EVENTS_EPOLLERR: EPOLL_EVENTS = crate::internal::EPOLL_EVENTS_EPOLLERR;
pub const EPOLL_EVENTS_EPOLLHUP: EPOLL_EVENTS = crate::internal::EPOLL_EVENTS_EPOLLHUP;
pub const EPOLL_EVENTS_EPOLLRDHUP: EPOLL_EVENTS = crate::internal::EPOLL_EVENTS_EPOLLRDHUP;
pub const EPOLL_EVENTS_EPOLLEXCLUSIVE: EPOLL_EVENTS = crate::internal::EPOLL_EVENTS_EPOLLEXCLUSIVE;
pub const EPOLL_EVENTS_EPOLLWAKEUP: EPOLL_EVENTS = crate::internal::EPOLL_EVENTS_EPOLLWAKEUP;
pub const EPOLL_EVENTS_EPOLLONESHOT: EPOLL_EVENTS = crate::internal::EPOLL_EVENTS_EPOLLONESHOT;
pub const EPOLL_EVENTS_EPOLLET: EPOLL_EVENTS = crate::internal::EPOLL_EVENTS_EPOLLET;

pub const EPOLL_CTL_ADD: u32 = crate::internal::EPOLL_CTL_ADD;
pub const EPOLL_CTL_DEL: u32 = crate::internal::EPOLL_CTL_DEL;
pub const EPOLL_CTL_MOD: u32 = crate::internal::EPOLL_CTL_MOD;

pub type epoll_event = crate::internal::epoll_event;

pub unsafe fn epoll_addr_of_event_data(event: *const epoll_event) -> *const u8 {
    let event_ref = &*event;
    core::ptr::addr_of!(event_ref.data.u64_).cast()
}

pub unsafe fn epoll_addr_of_event_data_mut(event: *mut epoll_event) -> *mut u8 {
    let event_ref = &mut *event;
    core::ptr::addr_of!(event_ref.data.u64_).cast_mut().cast()
}

pub unsafe fn epoll_create(size: posix::int) -> posix::int {
    crate::internal::epoll_create(size)
}

pub unsafe fn epoll_create1(flags: posix::int) -> posix::int {
    crate::internal::epoll_create1(flags)
}

pub unsafe fn epoll_ctl(
    epfd: posix::int,
    op: posix::int,
    fd: posix::int,
    event: *mut epoll_event,
) -> posix::int {
    crate::internal::epoll_ctl(epfd, op, fd, event)
}

pub unsafe fn epoll_wait(
    epfd: posix::int,
    events: *mut epoll_event,
    maxevents: posix::int,
    timeout: posix::int,
) -> posix::int {
    crate::internal::epoll_wait(epfd, events, maxevents, timeout)
}
