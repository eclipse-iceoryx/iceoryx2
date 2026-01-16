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

//! [`Epoll`] is a safe abstraction over the event file descriptor in linux. It allows users to
//! attach [`FileDescriptor`]s with a set of [`EventType`]s and [`InputFlag`]s. Additionally,
//! [`Epoll`] can also handle [`FetchableSignal`]s via a wakeup and informing the user what
//! [`FetchableSignal`] was raised.
//!
//! # Example
//!
//! ## Handle Events
//!
//! ```
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_linux::epoll::*;
//! use iceoryx2_bb_posix::socket_pair::StreamingSocket;
//! use iceoryx2_bb_posix::file_descriptor::FileDescriptorBased;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//!
//! let epoll = EpollBuilder::new().create()?;
//! let (socket_1, socket_2) = StreamingSocket::create_pair()?;
//!
//! let epoll_guard = epoll
//!     .add(socket_1.file_descriptor())
//!     .event_type(EventType::ReadyToRead)
//!     .attach()?;
//!
//! socket_2.try_send(b"hello world")?;
//!
//! let number_of_triggers = epoll.blocking_wait(|event| {
//!     if let EpollEvent::FileDescriptor(fd_event) = event {
//!         if fd_event.originates_from(socket_1.file_descriptor()) {
//!             let mut raw_data = [0u8; 20];
//!             socket_1.try_receive(&mut raw_data);
//!         }
//!     }
//! })?;
//!
//! # Ok(())
//! # }
//! ```

extern crate alloc;

use alloc::format;

use core::mem::MaybeUninit;
use core::time::Duration;
use iceoryx2_bb_concurrency::atomic::Ordering;

use iceoryx2_bb_concurrency::atomic::AtomicUsize;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_posix::{
    file::{AccessMode, FileBuilder, FileOpenError, FileReadError},
    file_descriptor::{FileDescriptor, FileDescriptorBased},
    signal::FetchableSignal,
    signal_set::FetchableSignalSet,
};
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_log::{fail, warn};
use iceoryx2_pal_os_api::linux;
use iceoryx2_pal_posix::posix::{self};

use crate::signalfd::{SignalFd, SignalFdBuilder, SignalFdReadError, SignalInfo};

const MAX_USER_WATCHES_FILE: &str = "/proc/sys/fs/epoll/max_user_watches";

/// Errors that can occur when [`EpollBuilder::create()`] creates a new [`Epoll`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EpollCreateError {
    /// The process file handle limit has been reached.
    PerProcessFileHandleLimitReached,
    /// The system wide file handle limit has been reached.
    SystemWideFileHandleLimitReached,
    /// The system has not enough memory to create the [`Epoll`]
    InsufficientMemory,
    /// The syscall [`linux::epoll_create()`] returned a broken [`FileDescriptor`].
    SysCallReturnedInvalidFileDescriptor,
    /// [`EpollBuilder`] was configured to handle some [`FetchableSignal`]s but the underlying
    /// [`SignalFd`] could not be created.
    UnableToEnableSignalHandling,
    /// An error occurred that was not described in the linux man-page.
    UnknownError(i32),
}

impl core::fmt::Display for EpollCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "EpollCreateError::{self:?}")
    }
}

impl core::error::Error for EpollCreateError {}

/// Can be emitted by [`Epoll::add()`] when a new [`FileDescriptor`] shall be attached.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EpollAttachmentError {
    /// The [`FileDescriptor`] is already attached.
    AlreadyAttached,
    /// The system has not enough memory to attach the [`FileDescriptor`].
    InsufficientMemory,
    /// The maximum supported amount of [`FileDescriptor`]s are already attached to [`Epoll`].
    ExceedsMaxSupportedAttachments,
    /// A [`FileDescriptor`] that does not support event multiplexing was given. For instance a
    /// [`FileDescriptor`] of a [`File`](iceoryx2_bb_posix::file::File) or a
    /// [`UnnamedSemaphore`](iceoryx2_bb_posix::semaphore::UnnamedSemaphore).
    ProvidedFileDescriptorDoesNotSupportEventMultiplexing,
    /// An error occurred that was not described in the linux man-page.
    UnknownError(i32),
}

impl core::fmt::Display for EpollAttachmentError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "EpollAttachmentError::{self:?}")
    }
}

impl core::error::Error for EpollAttachmentError {}

/// Can be emitted by [`Epoll::capacity()`] when the epoll capacity is read from the proc
/// file system.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EpollGetCapacityError {
    /// The proc file containing the capacity does not exist.
    ProcFileDoesNotExist,
    /// The content of the proc file could not be read.
    ProcFileReadFailure,
    /// The process does not have the permission to open the proc file for reading.
    InsufficientPermissions,
    /// Insufficient memory to read from the proc file.
    InsufficientMemory,
    /// The process file handle limit has been reached.
    PerProcessFileHandleLimitReached,
    /// The system wide file handle limit has been reached.
    SystemWideFileHandleLimitReached,
    /// [`FetchableSignal::Interrupt`] was received (SIGINT).
    Interrupt,
    /// The proc file does not contain a number but something else.
    InvalidProcFileContent,
    /// An undocumented error occurred.
    UnknownError,
}

impl core::fmt::Display for EpollGetCapacityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "EpollGetCapacityError::{self:?}")
    }
}

impl core::error::Error for EpollGetCapacityError {}

/// Errors that can be returned by [`Epoll::try_wait()`], [`Epoll::timed_wait()`] or
/// [`Epoll::blocking_wait()`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EpollWaitError {
    /// [`FetchableSignal::Interrupt`] was received (SIGINT).
    Interrupt,
    /// An error occurred that was not described in the linux man-page.
    UnknownError(i32),
}

impl core::fmt::Display for EpollWaitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "EpollWaitError::{self:?}")
    }
}

impl core::error::Error for EpollWaitError {}

/// Defines the type of event to which [`Epoll`] shall listen with the attached
/// [`FileDescriptor`]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum EventType {
    /// Detect when the [`FileDescriptor`] has data to read.
    ReadyToRead = linux::EPOLL_EVENTS_EPOLLIN as _,
    /// Detect when the [`FileDescriptor`] is able to write data.
    ReadyToWrite = linux::EPOLL_EVENTS_EPOLLOUT as _,
    /// Detect when the [`FileDescriptor`]s counterpart closed the connection.
    ConnectionClosed = linux::EPOLL_EVENTS_EPOLLRDHUP as _,
    /// Detect an exceptional condition on the [`FileDescriptor`].
    ExceptionalCondition = linux::EPOLL_EVENTS_EPOLLPRI as _,
    /// Detect an error condition on the [`FileDescriptor`].
    ErrorCondition = linux::EPOLL_EVENTS_EPOLLERR as _,
    /// Detect when the [`FileDescriptor`]s counterpart closed the connection.
    Hangup = linux::EPOLL_EVENTS_EPOLLHUP as _,
}

/// The input flags for the [`FileDescriptor`] attachments.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum InputFlag {
    /// Use edge triggered notifications instead of level triggered notifications.
    /// See `man epoll` for more details.
    EdgeTriggeredNotification = linux::EPOLL_EVENTS_EPOLLET as _,
    /// Enable one-shot notification for the attached [`FileDescriptor`]. If the state changed
    /// the user is notified once in the wait calls. The user must detach and reattach the
    /// [`FileDescriptor`] to rearm it again.
    OneShotNotification = linux::EPOLL_EVENTS_EPOLLONESHOT as _,
    /// Ensures that the system does not enter "suspend" or "hibernate" while this event is
    /// pending or being processed.
    BlockSuspension = linux::EPOLL_EVENTS_EPOLLWAKEUP as _,
    /// Sets an exclusive wakeup mode for the [`FileDescriptor`]. Useful when multiple epoll
    /// file descriptors are attached to the same target file.
    ExclusiveWakeup = linux::EPOLL_EVENTS_EPOLLEXCLUSIVE as _,
}

/// Returned by [`EpollAttachmentBuilder::attach()`] and represents an [`Epoll`] attachment. As
/// soon as the [`EpollGuard`] goes out of scope the attachment is detached.
pub struct EpollGuard<'epoll, 'file_descriptor> {
    epoll: &'epoll Epoll,
    fd: &'file_descriptor FileDescriptor,
}

impl<'epoll, 'file_descriptor> EpollGuard<'epoll, 'file_descriptor> {
    /// Returns a reference of the attached [`FileDescriptor`]
    pub fn file_descriptor(&self) -> &'file_descriptor FileDescriptor {
        self.fd
    }
}

impl Drop for EpollGuard<'_, '_> {
    fn drop(&mut self) {
        self.epoll.remove(unsafe { self.fd.native_handle() })
    }
}

/// The builder to create a new [`Epoll`].
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
    /// Creates a new builder instance.
    pub fn new() -> Self {
        Self {
            has_close_on_exec_flag: false,
            signal_set: FetchableSignalSet::new_empty(),
            has_enabled_signal_handling: false,
        }
    }

    /// Defines if the underlying [`FileDescriptor`] shall be closed when the
    /// [`Process`](iceoryx2_bb_posix::process::Process) is forked.
    pub fn set_close_on_exec(mut self, value: bool) -> Self {
        self.has_close_on_exec_flag = value;
        self
    }

    /// Defines all [`FetchableSignal`]s the [`Epoll`] shall handle. When one of the defined
    /// [`FetchableSignal`]s is raised [`Epoll::try_wait()`], [`Epoll::timed_wait()`] or
    /// [`Epoll::blocking_wait()`] will wake up and provide the signal details in
    /// [`EventType`] as [`SignalInfo`].
    pub fn handle_signal(mut self, signal: FetchableSignal) -> Self {
        self.signal_set.add(signal);
        self.has_enabled_signal_handling = true;
        self
    }

    /// Creates a new [`Epoll`].
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
                len: AtomicUsize::new(0),
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

        let mut epoll_event: linux::epoll_event = unsafe { core::mem::zeroed() };
        epoll_event.events = EventType::ReadyToRead as _;
        unsafe {
            core::ptr::copy_nonoverlapping(
                (signal_fd.file_descriptor() as *const _) as *const u8,
                linux::epoll_addr_of_event_data_mut(&mut epoll_event),
                core::mem::size_of::<FileDescriptor>(),
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
            len: AtomicUsize::new(0),
        })
    }
}

/// Represents an event that activated the [`Epoll`].
pub enum EpollEvent<'a> {
    /// A [`FileDescriptor`] was activated.
    FileDescriptor(FileDescriptorEvent<'a>),
    /// A [`FetchableSignal`] was raised.
    Signal(SignalInfo),
}

/// Describes an event on a [`FileDescriptor`].
pub struct FileDescriptorEvent<'a> {
    data: &'a linux::epoll_event,
}

impl FileDescriptorEvent<'_> {
    /// Returns `true` if the [`FileDescriptorEvent`] originated from the provided
    /// [`FileDescriptor`], otherwise `false`.
    pub fn originates_from(&self, file_descriptor: &FileDescriptor) -> bool {
        unsafe { file_descriptor.native_handle() == self.native_fd_handle() }
    }

    /// Returns the native handle of the corresponding [`FileDescriptor`]
    ///
    /// # Safety
    ///
    /// * the user must not modify or close the provided native handle
    pub unsafe fn native_fd_handle(&self) -> i32 {
        let mut native_handle: i32 = 0;
        unsafe {
            core::ptr::copy_nonoverlapping(
                linux::epoll_addr_of_event_data(self.data),
                (&mut native_handle as *mut i32).cast(),
                core::mem::size_of::<i32>(),
            )
        }
        native_handle
    }

    /// Returns `true` if the [`FileDescriptorEvent`] was caused by the provided [`EventType`],
    /// otherwise `false`.
    pub fn has_event(&self, event_type: EventType) -> bool {
        self.data.events & event_type as u32 != 0
    }
}

/// Abstraction of the event multiplexer epoll.
#[derive(Debug)]
pub struct Epoll {
    epoll_fd: FileDescriptor,
    signal_fd: Option<SignalFd>,
    len: AtomicUsize,
}

impl Epoll {
    fn remove(&self, fd_value: i32) {
        if unsafe {
            linux::epoll_ctl(
                self.epoll_fd.native_handle(),
                linux::EPOLL_CTL_DEL as _,
                fd_value,
                core::ptr::null_mut(),
            )
        } == -1
        {
            warn!(from self,
                "This should never happen! Failed to detach {fd_value} from epoll due to ({:?}). This might cause unexpected behavior.",
                posix::Errno::get());
        } else {
            self.len.fetch_sub(1, Ordering::Relaxed);
        }
    }

    /// Returns the number of wait events that can be handle at most with one
    /// [`Epoll::try_wait()`], [`Epoll::timed_wait()`] or [`Epoll::blocking_wait()`] call.
    pub const fn max_wait_events() -> usize {
        512
    }

    /// Returns the maximum supported [`Epoll`] capacity for the current system.
    pub fn capacity() -> Result<usize, EpollGetCapacityError> {
        let origin = "Epoll::capacity()";
        let msg = "Unable to acquire the capacity of epoll";
        let file_path = unsafe { FilePath::new_unchecked(MAX_USER_WATCHES_FILE.as_bytes()) };
        let proc_stat_file = match FileBuilder::new(&file_path)
            .has_ownership(false)
            .open_existing(AccessMode::Read)
        {
            Ok(file) => file,
            Err(FileOpenError::FileDoesNotExist) => {
                fail!(from origin, with EpollGetCapacityError::ProcFileDoesNotExist,
                    "{msg} since the file {MAX_USER_WATCHES_FILE} does not exist.");
            }
            Err(FileOpenError::InsufficientPermissions) => {
                fail!(from origin, with EpollGetCapacityError::InsufficientPermissions,
                    "{msg} since the file {MAX_USER_WATCHES_FILE} could not be opened due to insufficient permissions.");
            }
            Err(FileOpenError::Interrupt) => {
                fail!(from origin, with EpollGetCapacityError::Interrupt,
                    "{msg} since an interrupt signal was raised while opening the file {MAX_USER_WATCHES_FILE}.");
            }
            Err(FileOpenError::InsufficientMemory) => {
                fail!(from origin, with EpollGetCapacityError::InsufficientMemory,
                    "{msg} since the file {MAX_USER_WATCHES_FILE} could not be opened due to insufficient memory.");
            }
            Err(FileOpenError::PerProcessFileHandleLimitReached) => {
                fail!(from origin, with EpollGetCapacityError::PerProcessFileHandleLimitReached,
                    "{msg} since the process file handle limit was reached while opening {MAX_USER_WATCHES_FILE}.");
            }
            Err(FileOpenError::SystemWideFileHandleLimitReached) => {
                fail!(from origin, with EpollGetCapacityError::SystemWideFileHandleLimitReached,
                    "{msg} since the system wide file handle limit was reached while opening {MAX_USER_WATCHES_FILE}.");
            }
            Err(e) => {
                fail!(from origin, with EpollGetCapacityError::UnknownError,
                    "{msg} due to an unknown error while opening {MAX_USER_WATCHES_FILE} ({e:?}).");
            }
        };

        let mut buffer = [0u8; 32];
        let bytes_read = match proc_stat_file.read(&mut buffer) {
            Ok(v) => v,
            Err(FileReadError::Interrupt) => {
                fail!(from origin, with EpollGetCapacityError::Interrupt,
                    "{msg} since an interrupt signal was raised while reading the file {MAX_USER_WATCHES_FILE}.");
            }
            Err(e) => {
                fail!(from origin, with EpollGetCapacityError::ProcFileReadFailure,
                    "{msg} since the content of the file {MAX_USER_WATCHES_FILE} could not be read ({e:?}).");
            }
        };

        let file_content = match core::str::from_utf8(&buffer[0..bytes_read as usize - 1]) {
            Ok(v) => v,
            Err(e) => {
                fail!(from origin, with EpollGetCapacityError::InvalidProcFileContent,
                "{msg} since the file {MAX_USER_WATCHES_FILE} contains invalid content. Expected an UTF-8 string. ({e:?})");
            }
        };

        match file_content.parse::<usize>() {
            Ok(v) => Ok(v),
            Err(e) => {
                fail!(from origin, with EpollGetCapacityError::InvalidProcFileContent,
                    "{msg} since the file {MAX_USER_WATCHES_FILE} contains invalid content. Expected a number. ({e:?})");
            }
        }
    }

    /// Returns `true` when [`Epoll`] has no attached [`FileDescriptor`]s, otherwise `false`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of attached [`FileDescriptor`]s.
    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }

    /// Returns an [`EpollAttachmentBuilder`] to attach a new [`FileDescriptor`] to [`Epoll`].
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

    /// Non-blocking call, that returns the number of activated attachments and calls the provided
    /// callback for every activated attachment and with [`EpollEvent`] as callback argument
    /// that contains the information about the activated attachment.
    pub fn try_wait<F: FnMut(EpollEvent)>(&self, event_call: F) -> Result<usize, EpollWaitError> {
        self.wait_impl(0, event_call)
    }

    /// Blocking call, that returns the number of activated attachments and calls the provided
    /// callback for every activated attachment and with [`EpollEvent`] as callback argument
    /// that contains the information about the activated attachment.
    /// If the timeout has passed and no activation has happened it will return 0.
    pub fn timed_wait<F: FnMut(EpollEvent)>(
        &self,
        event_call: F,
        timeout: Duration,
    ) -> Result<usize, EpollWaitError> {
        // the smallest time period epoll can wait is 1ms, to introduce some waiting for
        // smaller time periods we always round the timeout up to the next millisecond
        let timeout_in_ms = timeout.as_nanos().div_ceil(1_000_000) as i32;
        self.wait_impl(timeout_in_ms, event_call)
    }

    /// Blocking call, that returns the number of activated attachments and calls the provided
    /// callback for every activated attachment and with [`EpollEvent`] as callback argument
    /// that contains the information about the activated attachment.
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

        let mut events: [MaybeUninit<linux::epoll_event>; Self::max_wait_events()] =
            [MaybeUninit::uninit(); Self::max_wait_events()];

        let number_of_fds = unsafe {
            linux::epoll_wait(
                self.epoll_fd.native_handle(),
                events.as_mut_ptr().cast(),
                Self::max_wait_events() as _,
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
                for i in 0..number_of_fds {
                    let fd_event = FileDescriptorEvent {
                        data: unsafe { events[i as usize].assume_init_ref() },
                    };

                    if fd_event.originates_from(signal_fd.file_descriptor()) {
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

/// Builder created by [`Epoll::add()`] that configures the [`EventType`]s and the [`InputFlag`]s
/// of the attachment.
#[derive(Debug)]
pub struct EpollAttachmentBuilder<'epoll, 'fd> {
    epoll: &'epoll Epoll,
    fd: &'fd FileDescriptor,
    events_flag: u32,
}

impl<'epoll, 'fd> EpollAttachmentBuilder<'epoll, 'fd> {
    /// The user can call this multiple times to define multiple [`EventType`]s for the attachment.
    /// It defines the [`EventType`] that shall cause a wakeup in [`Epoll`].
    pub fn event_type(mut self, event_type: EventType) -> Self {
        self.events_flag |= event_type as u32;
        self
    }

    /// The user can call this multiple times to attach multiple [`InputFlag`]s for the attachment.
    pub fn flags(mut self, input_flag: InputFlag) -> Self {
        self.events_flag |= input_flag as u32;
        self
    }

    /// Attaches the [`FileDescriptor`] to [`Epoll`] and returns an [`EpollGuard`]. As soon as the
    /// [`EpollGuard`] goes out-of-scope the attachment is released.
    pub fn attach(self) -> Result<EpollGuard<'epoll, 'fd>, EpollAttachmentError> {
        let msg = "Unable to attach file descriptor to epoll";
        let mut epoll_event: linux::epoll_event = unsafe { core::mem::zeroed() };

        epoll_event.events = self.events_flag;
        unsafe {
            core::ptr::copy_nonoverlapping(
                (self.fd as *const _) as *const u8,
                linux::epoll_addr_of_event_data_mut(&mut epoll_event),
                core::mem::size_of::<FileDescriptor>(),
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
