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

use std::time::Duration;

use iceoryx2_bb_posix::{file_descriptor::FileDescriptor, signal::FetchableSignal};
use iceoryx2_pal_os_api::linux;
use iceoryx2_pal_posix::posix::{self, MemZeroedStruct};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EpollCreateError {}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EpollAttachmentError {}

#[repr(u32)]
pub enum EventType {
    ReadyToRead = linux::EPOLL_EVENTS_EPOLLIN,
    ReadyToWrite = linux::EPOLL_EVENTS_EPOLLOUT,
    ConnectionClosed = linux::EPOLL_EVENTS_EPOLLRDHUP,
    ExceptionalCondition = linux::EPOLL_EVENTS_EPOLLPRI,
    ErrorCondition = linux::EPOLL_EVENTS_EPOLLERR,
    Hangup = linux::EPOLL_EVENTS_EPOLLHUP,
}

#[repr(u32)]
pub enum InputFlag {
    EdgeTriggeredNotification = linux::EPOLL_EVENTS_EPOLLET,
    OneShotNotification = linux::EPOLL_EVENTS_EPOLLONESHOT,
    BlockSuspension = linux::EPOLL_EVENTS_EPOLLWAKEUP,
    ExclusiveWakeup = linux::EPOLL_EVENTS_EPOLLEXCLUSIVE,
}

pub struct EpollGuard<'epoll, 'file_descriptor> {
    epoll: &'epoll Epoll,
    fd_value: &'file_descriptor FileDescriptor,
}

impl Drop for EpollGuard<'_, '_> {
    fn drop(&mut self) {
        self.epoll.remove(unsafe { self.fd_value.native_handle() })
    }
}

pub struct EpollBuilder {
    has_close_on_exec_flag: bool,
    signal_set: posix::sigset_t,
}

// add signal set
//   sigaddset() - add new signal
//   sigdelset() - delete attached signal
//   sigismember() - check if signal is contained in sigset
//   sigfillset() - initialise signal set, every signal is included
//   sigemptyset() - initialises signal set, so thet every signal is excluded
//   sigpending() - creates a signal set, contains signals that are blocked from delivery
//
// add signalfd
//   signalfd() - creates signalfd
//   close() - remove
//   read() - read signal -> use struct signalfd_siginfo
impl EpollBuilder {
    pub fn new() -> Self {
        Self {
            has_close_on_exec_flag: false,
            signal_set: posix::sigset_t::new_zeroed(),
        }
    }

    pub fn set_close_on_exec_flag(mut self) -> Self {
        self.has_close_on_exec_flag = true;
        self
    }

    pub fn handle_signal(mut self, signal: FetchableSignal) -> Self {
        todo!()
    }

    pub fn create(self) -> Result<Epoll, EpollCreateError> {
        todo!()
    }
}

pub struct Epoll {
    epoll_fd: FileDescriptor,
}

impl Epoll {
    fn remove(&self, fd_value: i32) {
        todo!()
    }

    pub fn add<'epoll, 'fd>(
        &'epoll mut self,
        fd: &'fd FileDescriptor,
    ) -> EpollAttachmentBuilder<'epoll, 'fd> {
        todo!()
    }

    pub fn try_wait(&self) {
        //epoll_pwait2
        //when signal was specified then use sigwaitinfo()
        todo!()
    }

    pub fn timed_wait(&self, timeout: Duration) {
        todo!()
    }

    pub fn blocking_wait(&self) {
        todo!()
    }
}

pub struct EpollAttachmentBuilder<'epoll, 'fd> {
    epoll: &'epoll Epoll,
    fd: &'fd FileDescriptor,
}

impl<'epoll, 'fd> EpollAttachmentBuilder<'epoll, 'fd> {
    // data that the kernel provides when triggered
    pub fn data<T>(mut self, value: T) -> Self {
        todo!()
    }

    // can be multiple
    pub fn event_type(mut self, event_type: EventType) -> Self {
        todo!()
    }

    // can be multiple
    pub fn flags(mut self, input_flag: InputFlag) -> Self {
        todo!()
    }

    pub fn attach(mut self) -> Result<EpollGuard<'epoll, 'fd>, EpollAttachmentError> {
        todo!()
    }
}
