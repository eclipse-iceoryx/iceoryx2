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

use crate::{
    clock::{ClockReactorSchedule, ClockReactorScheduleError, ScheduleHandle, Timeout},
    reactor::{
        Reactor, ReactorAttachError, ReactorBuilder, ReactorCreateError, ReactorGuard,
        ReactorWaitError,
    },
};
use alloc::format;
use alloc::sync::Arc;
use iceoryx2_bb_linux::epoll::{EpollAttachmentError, EpollWaitError};
use iceoryx2_bb_posix::file_descriptor::FileDescriptor;
use iceoryx2_log::{fail, warn};

#[derive(Debug)]
pub struct EpollReactor {
    epoll: Epoll,
    clock: Arc<dyn ClockReactorSchedule>,
}

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

impl Reactor for EpollReactor {
    type Guard<'reactor, 'attachment> = EpollGuard<'reactor, 'attachment>;
    type Builder = EpollBuilder;

    fn capacity(&self) -> usize {
        match Epoll::capacity() {
            Ok(v) => v,
            Err(e) => {
                warn!(from self, "Unable to acquire the epoll capacity ({e:?}). Falling back to Epoll::max_wait_events().");
                Epoll::max_wait_events()
            }
        }
    }

    fn len(&self) -> usize {
        self.epoll.len()
    }

    fn is_empty(&self) -> bool {
        self.epoll.is_empty()
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
            .epoll
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
            &self.epoll,
            "Unable to try wait on reactor::Epoll",
            self.epoll.try_wait(|event| {
                wait_call(&self.epoll, event, &mut fn_call);
            }),
        )
    }

    fn timed_wait<F: FnMut(&FileDescriptor)>(
        &self,
        mut fn_call: F,
        timeout: core::time::Duration,
    ) -> Result<usize, super::ReactorWaitError> {
        let msg = "Unable to wait with timeout on reactor::Epoll";

        match self.clock.schedule(Timeout::By(timeout)) {
            Ok(ScheduleHandle::TranslatedTimeout(t)) => {
                let now = fail!(from self, when self.clock.now(),
                                with ReactorWaitError::InternalError,
                                "{msg} since the current time could not be acquired.");

                let translated_timeout = t.as_duration(now);
                handle_wait_error(
                    &self.epoll,
                    msg,
                    self.epoll.timed_wait(
                        |event| {
                            wait_call(&self.epoll, event, &mut fn_call);
                        },
                        translated_timeout,
                    ),
                )
            }
            Ok(ScheduleHandle::GuardAndListener((_guard, listener))) => {
                let _attachment = match self.attach(&*listener) {
                    Ok(v) => v,
                    Err(ReactorAttachError::CapacityExceeded) => {
                        fail!(from self, with ReactorWaitError::CapacityExceeded,
                            "{msg} since the wait trigger could not be attached since it would exceed the capacity.");
                    }
                    Err(ReactorAttachError::InsufficientResources) => {
                        fail!(from self, with ReactorWaitError::InsufficientResources,
                            "{msg} since the wait trigger could not be attached due to insufficient resources.");
                    }
                    Err(e) => {
                        fail!(from self, with ReactorWaitError::InternalError,
                            "{msg} since the wait trigger could not be attached due to an internal error. [{e:?}]");
                    }
                };

                let mut number_of_timeout_triggers = 0;
                handle_wait_error(
                    &self.epoll,
                    msg,
                    self.epoll.blocking_wait(|event| {
                        if let EpollEvent::FileDescriptor(fd_event) = &event
                            && fd_event.originates_from(listener.file_descriptor())
                        {
                            number_of_timeout_triggers += 1;
                            return;
                        }
                        wait_call(&self.epoll, event, &mut fn_call);
                    }),
                )
                .map(|count| count - number_of_timeout_triggers)
            }
            Err(ClockReactorScheduleError::ClockScheduleCapacityExceeded) => {
                fail!(from self, with ReactorWaitError::ClockScheduleCapacityExceeded,
                    "{msg} since the clocks schedule capacity is exceeded.");
            }
            Err(ClockReactorScheduleError::ListenerAttachmentFailed) => {
                fail!(from self, with ReactorWaitError::ListenerAttachmentError,
                    "{msg} since the listener could not be attached.");
            }
            Err(ClockReactorScheduleError::InternalError) => {
                fail!(from self, with ReactorWaitError::InternalError,
                    "{msg} since an internal error occured.");
            }
        }
    }

    fn blocking_wait<F: FnMut(&FileDescriptor)>(
        &self,
        mut fn_call: F,
    ) -> Result<usize, super::ReactorWaitError> {
        handle_wait_error(
            &self.epoll,
            "Unable to blocking wait on reactor::Epoll",
            self.epoll.blocking_wait(|event| {
                wait_call(&self.epoll, event, &mut fn_call);
            }),
        )
    }
}

impl ReactorBuilder<EpollReactor> for EpollBuilder {
    fn new() -> Self {
        EpollBuilder::new().set_close_on_exec(true)
    }

    fn create(
        self,
        clock: Arc<dyn ClockReactorSchedule>,
    ) -> Result<EpollReactor, ReactorCreateError> {
        let msg = "Unable to create epoll::Reactor";
        let origin = format!("{self:?}");
        match self.create() {
            Ok(epoll) => Ok(EpollReactor { epoll, clock }),
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
