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
use core::time::Duration;
use std::sync::atomic::Ordering;

use iceoryx2_bb_log::{fail, warn};
use iceoryx2_bb_posix::{
    file_descriptor::{FileDescriptor, FileDescriptorBased},
    signal::FetchableSignal,
    signal_set::FetchableSignalSet,
};
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicUsize;
use iceoryx2_pal_os_api::linux;
use iceoryx2_pal_posix::posix::{self};

use crate::signalfd::{SignalFd, SignalFdBuilder, SignalFdReadError, SignalInfo};

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
pub enum EpollAttachmentError {
    AlreadyAttached,
    InsufficientMemory,
    ExceedsMaxSupportedAttachments,
    ProvidedFileDescriptorDoesNotSupportEventMultiplexing,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EpollWaitError {
    Interrupt,
    UnknownError(i32),
}

#[repr(u32)]
pub enum EventType {
    ReadyToRead = linux::EPOLL_EVENTS_EPOLLIN as _,
    ReadyToWrite = linux::EPOLL_EVENTS_EPOLLOUT as _,
    ConnectionClosed = linux::EPOLL_EVENTS_EPOLLRDHUP as _,
    ExceptionalCondition = linux::EPOLL_EVENTS_EPOLLPRI as _,
    ErrorCondition = linux::EPOLL_EVENTS_EPOLLERR as _,
    Hangup = linux::EPOLL_EVENTS_EPOLLHUP as _,
}

#[repr(u32)]
pub enum InputFlag {
    EdgeTriggeredNotification = linux::EPOLL_EVENTS_EPOLLET as _,
    OneShotNotification = linux::EPOLL_EVENTS_EPOLLONESHOT as _,
    BlockSuspension = linux::EPOLL_EVENTS_EPOLLWAKEUP as _,
    ExclusiveWakeup = linux::EPOLL_EVENTS_EPOLLEXCLUSIVE as _,
}

pub struct EpollGuard<'epoll, 'file_descriptor> {
    epoll: &'epoll Epoll,
    fd: &'file_descriptor FileDescriptor,
}

impl Drop for EpollGuard<'_, '_> {
    fn drop(&mut self) {
        self.epoll.remove(unsafe { self.fd.native_handle() })
    }
}

#[derive(Debug)]
pub struct EpollBuilder {
    has_close_on_exec_flag: bool,
    signal_set: FetchableSignalSet,
    has_enabled_signal_handling: bool,
}

impl Default for EpollBuilder {
    fn default() -> Self {
        Self::new()
    }
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
                len: IoxAtomicUsize::new(0),
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

        let signal_fd_native_handle = unsafe { signal_fd.file_descriptor().native_handle() };
        let mut epoll_event: linux::epoll_event = unsafe { core::mem::zeroed() };
        epoll_event.events = EventType::ReadyToRead as _;
        unsafe {
            core::ptr::copy_nonoverlapping(
                (&signal_fd_native_handle as *const i32) as *const u8,
                linux::epoll_addr_of_event_data_mut(&mut epoll_event),
                core::mem::size_of::<i32>(),
            )
        };

        if unsafe {
            linux::epoll_ctl(
                epoll_fd.native_handle(),
                linux::EPOLL_CTL_ADD as _,
                signal_fd.file_descriptor().native_handle(),
                &mut epoll_event,
            )
        } == -1
        {
            match posix::Errno::get() {
                posix::Errno::ENOMEM => {
                    fail!(from origin, with EpollCreateError::InsufficientMemory,
                        "{msg} since there is not enough memory available to attach the signalfd for signal handling.");
                }
                e => {
                    fail!(from origin, with EpollCreateError::UnknownError(e as i32),
                        "{msg} due to an unknown error while attaching the signalfd for signal handling.");
                }
            }
        }

        Ok(Epoll {
            epoll_fd,
            signal_fd: Some(signal_fd),
            len: IoxAtomicUsize::new(0),
        })
    }
}

pub enum EpollEvent<'a> {
    FileDescriptor(FileDescriptorEvent<'a>),
    Signal(SignalInfo),
}

pub struct FileDescriptorEvent<'a> {
    data: &'a linux::epoll_event,
}

impl FileDescriptorEvent<'_> {
    pub fn file_descriptor_native_handle(&self) -> i32 {
        let mut fd_value: i32 = 0;
        unsafe {
            core::ptr::copy_nonoverlapping(
                linux::epoll_addr_of_event_data(self.data),
                (&mut fd_value as *mut i32) as *mut u8,
                core::mem::size_of::<i32>(),
            )
        };
        fd_value
    }

    pub fn has_event(&self, event_type: EventType) -> bool {
        self.data.events & event_type as u32 != 0
    }
}

#[derive(Debug)]
pub struct Epoll {
    epoll_fd: FileDescriptor,
    signal_fd: Option<SignalFd>,
    len: IoxAtomicUsize,
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
        self.len.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }

    pub fn add<'epoll, 'fd>(
        &'epoll self,
        fd: &'fd FileDescriptor,
    ) -> EpollAttachmentBuilder<'epoll, 'fd> {
        EpollAttachmentBuilder {
            epoll: self,
            fd,
            events_flag: 0,
        }
    }

    pub fn try_wait<F: FnMut(EpollEvent)>(&self, event_call: F) -> Result<usize, EpollWaitError> {
        self.wait_impl(0, event_call)
    }

    pub fn timed_wait<F: FnMut(EpollEvent)>(
        &self,
        event_call: F,
        timeout: Duration,
    ) -> Result<usize, EpollWaitError> {
        self.wait_impl(timeout.as_millis().min(i32::MAX as _) as i32, event_call)
    }

    pub fn blocking_wait<F: FnMut(EpollEvent)>(
        &self,
        event_call: F,
    ) -> Result<usize, EpollWaitError> {
        self.wait_impl(-1, event_call)
    }

    fn wait_impl<F: FnMut(EpollEvent)>(
        &self,
        timeout: posix::int,
        mut event_call: F,
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

        match self.signal_fd.as_ref() {
            Some(signal_fd) => {
                let signal_fd_native_handle =
                    unsafe { signal_fd.file_descriptor().native_handle() };

                for i in 0..number_of_fds {
                    let fd_event = FileDescriptorEvent {
                        data: unsafe { events[i as usize].assume_init_ref() },
                    };

                    if fd_event.file_descriptor_native_handle() == signal_fd_native_handle {
                        while let Some(signal) = match signal_fd.try_read() {
                            Ok(v) => v,
                            Err(SignalFdReadError::Interrupt) => {
                                fail!(from self, with EpollWaitError::Interrupt,
                                    "{msg} with a timeout of {timeout}ms since an interrupt signal was raised while acquiring the raised signals.");
                            }
                            Err(e) => {
                                warn!("Epoll wait will continue but a failure occurred while reading the raised signal ({e:?}).");
                                None
                            }
                        } {
                            event_call(EpollEvent::Signal(signal));
                        }
                    } else {
                        event_call(EpollEvent::FileDescriptor(fd_event));
                    }
                }
            }
            None => {
                for i in 0..number_of_fds {
                    event_call(EpollEvent::FileDescriptor(FileDescriptorEvent {
                        data: unsafe { events[i as usize].assume_init_ref() },
                    }));
                }
            }
        }

        Ok(number_of_fds as usize)
    }
}

#[derive(Debug)]
pub struct EpollAttachmentBuilder<'epoll, 'fd> {
    epoll: &'epoll Epoll,
    fd: &'fd FileDescriptor,
    events_flag: u32,
}

impl<'epoll, 'fd> EpollAttachmentBuilder<'epoll, 'fd> {
    // can be multiple
    pub fn event_type(mut self, event_type: EventType) -> Self {
        self.events_flag |= event_type as u32;
        self
    }

    // can be multiple
    pub fn flags(mut self, input_flag: InputFlag) -> Self {
        self.events_flag |= input_flag as u32;
        self
    }

    pub fn attach(self) -> Result<EpollGuard<'epoll, 'fd>, EpollAttachmentError> {
        let msg = "Unable to attach file descriptor to epoll";
        let mut epoll_event: linux::epoll_event = unsafe { core::mem::zeroed() };

        epoll_event.events = self.events_flag;
        unsafe {
            core::ptr::copy_nonoverlapping(
                (&self.fd.native_handle() as *const _) as *const u8,
                linux::epoll_addr_of_event_data_mut(&mut epoll_event),
                core::mem::size_of::<i32>(),
            )
        }

        if unsafe {
            linux::epoll_ctl(
                self.epoll.epoll_fd.native_handle(),
                linux::EPOLL_CTL_ADD as _,
                self.fd.native_handle(),
                &mut epoll_event,
            )
        } == -1
        {
            match posix::Errno::get() {
                posix::Errno::EEXIST => {
                    fail!(from self, with EpollAttachmentError::AlreadyAttached,
                        "{msg} since it is already attached.");
                }
                posix::Errno::ENOMEM => {
                    fail!(from self, with EpollAttachmentError::InsufficientMemory,
                        "{msg} due to insufficient memory.");
                }
                posix::Errno::ENOSPC => {
                    fail!(from self, with EpollAttachmentError::ExceedsMaxSupportedAttachments,
                        "{msg} since it would exceed the system limit of the number of attachments.");
                }
                posix::Errno::EPERM => {
                    fail!(from self, with EpollAttachmentError::ProvidedFileDescriptorDoesNotSupportEventMultiplexing,
                        "{msg} since the provided file descriptor does not support event multiplexing.");
                }
                e => {
                    fail!(from self, with EpollAttachmentError::UnknownError(e as i32),
                        "{msg} due to an unknown error ({e:?}).");
                }
            }
        }

        self.epoll.len.fetch_add(1, Ordering::Relaxed);

        Ok(EpollGuard {
            epoll: self.epoll,
            fd: self.fd,
        })
    }
}
