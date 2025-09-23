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

use core::mem::MaybeUninit;
use std::time::Duration;

use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::{
    file_descriptor::FileDescriptor, signal::FetchableSignal, signal_set::FetchableSignalSet,
};
use iceoryx2_pal_os_api::linux;
use iceoryx2_pal_posix::posix;

use crate::signalfd::{SignalFd, SignalFdBuilder};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EpollCreateError {
    PerProcessFileHandleLimitReached,
    SystemWideFileHandleLimitReached,
    InsufficientMemory,
    SysCallReturnedInvalidFileDescriptor,
    UnableToEnableSignalHandling,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EpollAttachmentError {}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EpollWaitError {
    Interrupt,
    UnknownError(i32),
}

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

#[derive(Debug)]
pub struct EpollBuilder {
    has_close_on_exec_flag: bool,
    signal_set: FetchableSignalSet,
    has_enabled_signal_handling: bool,
}

impl EpollBuilder {
    pub fn new() -> Self {
        Self {
            has_close_on_exec_flag: false,
            signal_set: FetchableSignalSet::new_empty(),
            has_enabled_signal_handling: false,
        }
    }

    pub fn set_close_on_exec(mut self, value: bool) -> Self {
        self.has_close_on_exec_flag = value;
        self
    }

    pub fn handle_signal(mut self, signal: FetchableSignal) -> Self {
        self.signal_set.add(signal);
        self.has_enabled_signal_handling = true;
        self
    }

    pub fn create(self) -> Result<Epoll, EpollCreateError> {
        let msg = "Unable to create epoll file descriptor";
        let mut flags = 0;
        if self.has_close_on_exec_flag {
            flags |= linux::EPOLL_CLOEXEC;
        }

        let epoll_fd = unsafe { linux::epoll_create1(flags as _) };
        if epoll_fd == -1 {
            match posix::Errno::get() {
                posix::Errno::EMFILE => {
                    fail!(from self, with EpollCreateError::PerProcessFileHandleLimitReached,
                        "{msg} since it would exceed the process limit for file descriptors.");
                }
                posix::Errno::ENFILE => {
                    fail!(from self, with EpollCreateError::SystemWideFileHandleLimitReached,
                        "{msg} since it would exceed the system limit for file descriptors.");
                }
                posix::Errno::ENOMEM => {
                    fail!(from self, with EpollCreateError::InsufficientMemory,
                        "{msg} due to insufficient memory.");
                }
                e => {
                    fail!(from self, with EpollCreateError::UnknownError(e as i32),
                        "{msg} since an unknown error occurred ({e:?}).");
                }
            }
        }

        let epoll_fd = match FileDescriptor::new(epoll_fd) {
            Some(fd) => fd,
            None => {
                fail!(from self, with EpollCreateError::SysCallReturnedInvalidFileDescriptor,
                        "{msg} since the epoll_create1() syscall returned an invalid file descriptor.");
            }
        };

        if !self.has_enabled_signal_handling {
            return Ok(Epoll {
                epoll_fd,
                signal_fd: None,
            });
        }

        let origin = format!("{self:?}");
        let signal_fd = match SignalFdBuilder::new(self.signal_set)
            .set_close_on_exec(self.has_close_on_exec_flag)
            .create_non_blocking()
        {
            Ok(signal_fd) => signal_fd,
            Err(e) => {
                fail!(from origin, with EpollCreateError::UnableToEnableSignalHandling,
                        "{msg} since the signal fd, required for signal handling, could not be created ({e:?}).");
            }
        };

        Ok(Epoll {
            epoll_fd,
            signal_fd: Some(signal_fd),
        })
    }
}

pub struct EpollEvent<'a> {
    data: &'a linux::epoll_event,
}

#[derive(Debug)]
pub struct Epoll {
    epoll_fd: FileDescriptor,
    signal_fd: Option<SignalFd>,
}

impl Epoll {
    fn remove(&self, fd_value: i32) {
        unsafe {
            linux::epoll_ctl(
                self.epoll_fd.native_handle(),
                linux::EPOLL_CTL_DEL as _,
                fd_value,
                core::ptr::null_mut(),
            )
        };
    }

    pub fn add<'epoll, 'fd>(
        &'epoll mut self,
        fd: &'fd FileDescriptor,
    ) -> EpollAttachmentBuilder<'epoll, 'fd> {
        EpollAttachmentBuilder { epoll: self, fd }
    }

    pub fn try_wait<F: FnMut(EpollEvent)>(
        &self,
        event_call: &mut F,
    ) -> Result<usize, EpollWaitError> {
        self.wait_impl(0, event_call)
    }

    pub fn timed_wait<F: FnMut(EpollEvent)>(
        &self,
        event_call: &mut F,
        timeout: Duration,
    ) -> Result<usize, EpollWaitError> {
        self.wait_impl(timeout.as_millis().max(i32::MAX as _) as i32, event_call)
    }

    pub fn blocking_wait<F: FnMut(EpollEvent)>(
        &self,
        event_call: &mut F,
    ) -> Result<usize, EpollWaitError> {
        self.wait_impl(-1, event_call)
    }

    fn wait_impl<F: FnMut(EpollEvent)>(
        &self,
        timeout: posix::int,
        event_call: &mut F,
    ) -> Result<usize, EpollWaitError> {
        let msg = "Unable to wait on epoll";
        const MAX_EVENTS: usize = 512;
        let mut events: [MaybeUninit<linux::epoll_event>; 512] = [MaybeUninit::uninit(); 512];

        let number_of_fds = unsafe {
            linux::epoll_wait(
                self.epoll_fd.native_handle(),
                events.as_mut_ptr().cast(),
                MAX_EVENTS as _,
                timeout,
            )
        };

        if number_of_fds == -1 {
            match posix::Errno::get() {
                posix::Errno::EINTR => {
                    fail!(from self, with EpollWaitError::Interrupt,
                        "{msg} with a timeout of {timeout}ms since an interrupt signal was raised."
                    );
                }
                e => {
                    fail!(from self, with EpollWaitError::UnknownError(e as i32),
                        "{msg} with a timeout of {timeout}ms due to an unknown failure ({e:?})."
                    );
                }
            }
        }

        for i in 0..number_of_fds {
            event_call(EpollEvent {
                data: unsafe { events[i as usize].assume_init_ref() },
            });
        }

        Ok(number_of_fds as usize)
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
