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

use iceoryx2_pal_posix::posix;

pub type EPOLL_EVENTS = posix::int;
pub type epoll_event = libc::epoll_event;

pub const EPOLL_CLOEXEC: usize = libc::EPOLL_CLOEXEC as _;
pub const EPOLL_EVENTS_EPOLLIN: EPOLL_EVENTS = libc::EPOLLIN;
pub const EPOLL_EVENTS_EPOLLPRI: EPOLL_EVENTS = libc::EPOLLPRI;
pub const EPOLL_EVENTS_EPOLLOUT: EPOLL_EVENTS = libc::EPOLLOUT;
pub const EPOLL_EVENTS_EPOLLRDNORM: EPOLL_EVENTS = libc::EPOLLRDNORM;
pub const EPOLL_EVENTS_EPOLLRDBAND: EPOLL_EVENTS = libc::EPOLLRDBAND;
pub const EPOLL_EVENTS_EPOLLWRNORM: EPOLL_EVENTS = libc::EPOLLWRNORM;
pub const EPOLL_EVENTS_EPOLLWRBAND: EPOLL_EVENTS = libc::EPOLLWRBAND;
pub const EPOLL_EVENTS_EPOLLMSG: EPOLL_EVENTS = libc::EPOLLMSG;
pub const EPOLL_EVENTS_EPOLLERR: EPOLL_EVENTS = libc::EPOLLERR;
pub const EPOLL_EVENTS_EPOLLHUP: EPOLL_EVENTS = libc::EPOLLHUP;
pub const EPOLL_EVENTS_EPOLLRDHUP: EPOLL_EVENTS = libc::EPOLLRDHUP;
pub const EPOLL_EVENTS_EPOLLEXCLUSIVE: EPOLL_EVENTS = libc::EPOLLEXCLUSIVE;
pub const EPOLL_EVENTS_EPOLLWAKEUP: EPOLL_EVENTS = libc::EPOLLWAKEUP;
pub const EPOLL_EVENTS_EPOLLONESHOT: EPOLL_EVENTS = libc::EPOLLONESHOT;
pub const EPOLL_EVENTS_EPOLLET: EPOLL_EVENTS = libc::EPOLLET;

pub const EPOLL_CTL_ADD: u32 = libc::EPOLL_CTL_ADD as _;
pub const EPOLL_CTL_DEL: u32 = libc::EPOLL_CTL_DEL as _;
pub const EPOLL_CTL_MOD: u32 = libc::EPOLL_CTL_MOD as _;

pub unsafe fn epoll_addr_of_event_data(event: *const epoll_event) -> *const u8 {
    let event_ref = &*event;
    core::ptr::addr_of!(event_ref.u64).cast()
}

pub unsafe fn epoll_addr_of_event_data_mut(event: *mut epoll_event) -> *mut u8 {
    let event_ref = &mut *event;
    core::ptr::addr_of!(event_ref.u64).cast_mut().cast()
}

pub unsafe fn epoll_create(size: posix::int) -> posix::int {
    libc::epoll_create(size)
}

pub unsafe fn epoll_create1(flags: posix::int) -> posix::int {
    libc::epoll_create1(flags)
}

pub unsafe fn epoll_ctl(
    epfd: posix::int,
    op: posix::int,
    fd: posix::int,
    event: *mut epoll_event,
) -> posix::int {
    libc::epoll_ctl(epfd, op, fd, event)
}

pub unsafe fn epoll_wait(
    epfd: posix::int,
    events: *mut epoll_event,
    maxevents: posix::int,
    timeout: posix::int,
) -> posix::int {
    libc::epoll_wait(epfd, events, maxevents, timeout)
}
