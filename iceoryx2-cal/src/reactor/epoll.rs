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

pub use iceoryx2_bb_linux::epoll::{
    Epoll, EpollBuilder, EpollCreateError, EpollEvent, EpollGuard, EventType,
};

use alloc::format;

use iceoryx2_bb_linux::epoll::{EpollAttachmentError, EpollWaitError};
use iceoryx2_bb_posix::file_descriptor::FileDescriptor;
use iceoryx2_log::{fail, warn};

use crate::reactor::{
    Reactor, ReactorAttachError, ReactorBuilder, ReactorCreateError, ReactorGuard, ReactorWaitError,
};

impl<'reactor, 'attachment> ReactorGuard<'reactor, 'attachment>
    for EpollGuard<'reactor, 'attachment>
{
    fn file_descriptor(&self) -> &FileDescriptor {
        self.file_descriptor()
    }
}

fn handle_wait_error(
    this: &Epoll,
    msg: &str,
    epoll_wait_state: Result<usize, EpollWaitError>,
) -> Result<usize, ReactorWaitError> {
    match epoll_wait_state {
        Ok(value) => Ok(value),
        Err(EpollWaitError::Interrupt) => {
            fail!(from this, with ReactorWaitError::Interrupt,
                "{msg} since an interrupt signal was raised.");
        }
        Err(EpollWaitError::UnknownError(value)) => {
            fail!(from this, with ReactorWaitError::InternalError,
                "{msg} due to an internal error ({value}).");
        }
    }
}

fn wait_call<F: FnMut(&FileDescriptor)>(this: &Epoll, event: EpollEvent<'_>, fn_call: &mut F) {
    if let EpollEvent::FileDescriptor(fdev) = event {
        let native_handle = unsafe { fdev.native_fd_handle() };
        match FileDescriptor::non_owning_new(native_handle) {
            Some(fd) => fn_call(&fd),
            None => {
                warn!(from this,
                    "The file descriptor {native_handle} is no longer valid but still attached to the reactor. Skipping attachment!");
            }
        }
    }
}

impl Reactor for Epoll {
    type Guard<'reactor, 'attachment> = EpollGuard<'reactor, 'attachment>;
    type Builder = EpollBuilder;

    fn capacity(&self) -> usize {
        match Self::capacity() {
            Ok(v) => v,
            Err(e) => {
                warn!(from self, "Unable to acquire the epoll capacity ({e:?}). Falling back to Epoll::max_wait_events().");
                Self::max_wait_events()
            }
        }
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn attach<
        'reactor,
        'attachment,
        F: iceoryx2_bb_posix::file_descriptor_set::SynchronousMultiplexing + core::fmt::Debug + ?Sized,
    >(
        &'reactor self,
        value: &'attachment F,
    ) -> Result<Self::Guard<'reactor, 'attachment>, ReactorAttachError> {
        let msg = "Unable to attach file descriptor to reactor::Epoll";

        match self
            .add(value.file_descriptor())
            .event_type(EventType::ReadyToRead)
            .attach()
        {
            Ok(guard) => Ok(guard),
            Err(EpollAttachmentError::ExceedsMaxSupportedAttachments) => {
                fail!(from self, with ReactorAttachError::CapacityExceeded,
                    "{msg} since it would exceed the maximum capacity of {}.", self.capacity());
            }
            Err(EpollAttachmentError::AlreadyAttached) => {
                fail!(from self, with ReactorAttachError::AlreadyAttached,
                    "{msg} since the file descriptor {:?} is already attached.", value);
            }
            Err(EpollAttachmentError::InsufficientMemory) => {
                fail!(from self, with ReactorAttachError::InsufficientResources,
                    "{msg} due to insufficient memory.");
            }
            Err(e) => {
                fail!(from self, with ReactorAttachError::InternalError,
                    "{msg} due to an internal error ({e:?}).");
            }
        }
    }

    fn try_wait<F: FnMut(&FileDescriptor)>(
        &self,
        mut fn_call: F,
    ) -> Result<usize, super::ReactorWaitError> {
        handle_wait_error(
            self,
            "Unable to try wait on reactor::Epoll",
            self.try_wait(|event| {
                wait_call(self, event, &mut fn_call);
            }),
        )
    }

    fn timed_wait<F: FnMut(&FileDescriptor)>(
        &self,
        mut fn_call: F,
        timeout: core::time::Duration,
    ) -> Result<usize, super::ReactorWaitError> {
        handle_wait_error(
            self,
            "Unable to wait with timeout on reactor::Epoll",
            self.timed_wait(
                |event| {
                    wait_call(self, event, &mut fn_call);
                },
                timeout,
            ),
        )
    }

    fn blocking_wait<F: FnMut(&FileDescriptor)>(
        &self,
        mut fn_call: F,
    ) -> Result<usize, super::ReactorWaitError> {
        handle_wait_error(
            self,
            "Unable to blocking wait on reactor::Epoll",
            self.blocking_wait(|event| {
                wait_call(self, event, &mut fn_call);
            }),
        )
    }
}

impl ReactorBuilder<Epoll> for EpollBuilder {
    fn new() -> Self {
        EpollBuilder::new().set_close_on_exec(true)
    }

    fn create(self) -> Result<Epoll, ReactorCreateError> {
        let msg = "Unable to create epoll::Reactor";
        let origin = format!("{self:?}");
        match self.create() {
            Ok(v) => Ok(v),
            Err(EpollCreateError::InsufficientMemory)
            | Err(EpollCreateError::PerProcessFileHandleLimitReached)
            | Err(EpollCreateError::SystemWideFileHandleLimitReached) => {
                fail!(from origin, with ReactorCreateError::InsufficientResources,
                   "{msg} due to insufficient system resources.");
            }
            Err(e) => {
                fail!(from origin, with ReactorCreateError::InternalError,
                    "{msg} due to an internal error ({e:?}).");
            }
        }
    }
}
