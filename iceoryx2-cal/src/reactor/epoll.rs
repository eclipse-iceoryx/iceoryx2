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
use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::file_descriptor::FileDescriptor;

use crate::reactor::{Reactor, ReactorBuilder, ReactorCreateError, ReactorGuard};

impl<'reactor, 'attachment> ReactorGuard<'reactor, 'attachment>
    for EpollGuard<'reactor, 'attachment>
{
    fn file_descriptor(&self) -> &FileDescriptor {
        self.file_descriptor()
    }
}

impl Reactor for Epoll {
    type Guard<'reactor, 'attachment> = EpollGuard<'reactor, 'attachment>;
    type Builder = EpollBuilder;

    fn capacity(&self) -> usize {
        Self::capacity()
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
        F: iceoryx2_bb_posix::file_descriptor_set::SynchronousMultiplexing + std::fmt::Debug,
    >(
        &'reactor self,
        value: &'attachment F,
    ) -> Result<Self::Guard<'reactor, 'attachment>, super::ReactorAttachError> {
        match self
            .add(value.file_descriptor())
            .event_type(EventType::ReadyToRead)
            .attach()
        {
            Ok(guard) => Ok(guard),
            Err(e) => todo!(),
        }
    }

    fn try_wait<F: FnMut(&FileDescriptor)>(
        &self,
        mut fn_call: F,
    ) -> Result<usize, super::ReactorWaitError> {
        match self.try_wait(|event| {
            if let EpollEvent::FileDescriptor(fdev) = event {
                fn_call(fdev.file_descriptor());
            }
        }) {
            Ok(n) => Ok(n),
            Err(e) => todo!(),
        }
    }

    fn timed_wait<F: FnMut(&FileDescriptor)>(
        &self,
        mut fn_call: F,
        timeout: std::time::Duration,
    ) -> Result<usize, super::ReactorWaitError> {
        match self.timed_wait(
            |event| {
                if let EpollEvent::FileDescriptor(fdev) = event {
                    fn_call(fdev.file_descriptor());
                }
            },
            timeout,
        ) {
            Ok(n) => Ok(n),
            Err(e) => todo!(),
        }
    }

    fn blocking_wait<F: FnMut(&FileDescriptor)>(
        &self,
        mut fn_call: F,
    ) -> Result<usize, super::ReactorWaitError> {
        match self.blocking_wait(|event| {
            if let EpollEvent::FileDescriptor(fdev) = event {
                fn_call(fdev.file_descriptor());
            }
        }) {
            Ok(n) => Ok(n),
            Err(e) => todo!(),
        }
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
