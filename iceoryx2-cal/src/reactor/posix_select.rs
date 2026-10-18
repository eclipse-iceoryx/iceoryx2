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

use alloc::sync::Arc;
use core::{fmt::Debug, time::Duration};

use alloc::format;

use iceoryx2_bb_posix::{
    clock::{NanosleepError, nanosleep},
    file_descriptor::FileDescriptor,
    file_descriptor_set::{
        FileDescriptorSet, FileDescriptorSetAddError, FileDescriptorSetGuard,
        FileDescriptorSetWaitError, FileEvent,
    },
};
use iceoryx2_log::fail;

use crate::{
    clock::{ClockReactorSchedule, ClockReactorScheduleError, ScheduleHandle, Timeout},
    reactor::{ReactorAttachError, ReactorWaitError},
};

impl crate::reactor::ReactorGuard<'_, '_> for FileDescriptorSetGuard<'_, '_> {
    fn file_descriptor(&self) -> &FileDescriptor {
        self.file_descriptor()
    }
}

#[derive(Debug)]
pub struct Reactor {
    set: FileDescriptorSet,
    clock: Arc<dyn ClockReactorSchedule>,
}

impl Reactor {
    fn new(clock: Arc<dyn ClockReactorSchedule>) -> Self {
        Self {
            set: FileDescriptorSet::new(),
            clock,
        }
    }

    fn wait(
        &self,
        mut fn_call: impl FnMut(&FileDescriptor),
        mut wait_call: impl FnMut(
            &mut dyn FnMut(&FileDescriptor),
            FileEvent,
        ) -> Result<usize, FileDescriptorSetWaitError>,
        timeout: Duration,
    ) -> Result<usize, super::ReactorWaitError> {
        let msg = "Unable to wait on Reactor";
        if self.set.is_empty() {
            match nanosleep(timeout) {
                Ok(()) => Ok(0),
                Err(NanosleepError::InterruptedBySignal(_)) => {
                    fail!(from self, with ReactorWaitError::Interrupt,
                        "{} since an interrupt signal was received while waiting.",
                        msg);
                }
                Err(v) => {
                    fail!(from self, with ReactorWaitError::InternalError,
                        "{} since an unknown failure occurred while waiting ({:?}).",
                        msg, v);
                }
            }
        } else {
            match wait_call(&mut fn_call, FileEvent::Read) {
                Ok(number_of_notifications) => Ok(number_of_notifications),
                Err(FileDescriptorSetWaitError::Interrupt) => {
                    fail!(from self, with ReactorWaitError::Interrupt,
                        "{} since an interrupt signal was received while waiting.",
                        msg);
                }
                Err(FileDescriptorSetWaitError::InsufficientPermissions) => {
                    fail!(from self, with ReactorWaitError::Interrupt,
                        "{} due to insufficient permissions.",
                        msg);
                }
                Err(v) => {
                    fail!(from self, with ReactorWaitError::InternalError,
                        "{} since an unknown failure occurred in the underlying FileDescriptorSet ({:?}).",
                        msg, v);
                }
            }
        }
    }
}

impl crate::reactor::Reactor for Reactor {
    type Guard<'reactor, 'attachment> = FileDescriptorSetGuard<'reactor, 'attachment>;
    type Builder = ReactorBuilder;

    fn capacity(&self) -> usize {
        FileDescriptorSet::capacity()
    }

    fn len(&self) -> usize {
        self.set.len()
    }

    fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    fn attach<
        'reactor,
        'attachment,
        F: iceoryx2_bb_posix::file_descriptor_set::SynchronousMultiplexing + Debug + ?Sized,
    >(
        &'reactor self,
        value: &'attachment F,
    ) -> Result<Self::Guard<'reactor, 'attachment>, super::ReactorAttachError> {
        let msg = format!("Unable to attach {value:?} to the reactor");
        match self.set.add(value) {
            Ok(guard) => Ok(guard),
            Err(FileDescriptorSetAddError::CapacityExceeded) => {
                fail!(from self, with ReactorAttachError::CapacityExceeded,
                        "{msg} since the capacity of the underlying file descriptor set was exceeded.");
            }
            Err(FileDescriptorSetAddError::AlreadyAttached) => {
                fail!(from self, with ReactorAttachError::AlreadyAttached,
                        "{msg} since it is already attached.");
            }
        }
    }

    fn try_wait<F: FnMut(&FileDescriptor)>(
        &self,
        fn_call: F,
    ) -> Result<usize, super::ReactorWaitError> {
        self.wait(
            fn_call,
            |f, event| self.set.timed_wait(Duration::ZERO, event, f),
            Duration::ZERO,
        )
    }

    fn timed_wait<F: FnMut(&FileDescriptor)>(
        &self,
        mut fn_call: F,
        timeout: core::time::Duration,
    ) -> Result<usize, super::ReactorWaitError> {
        let msg = "Unable to wait on posix_select::Reactor";
        match self.clock.schedule(Timeout::By(timeout)) {
            Ok(ScheduleHandle::TranslatedTimeout(t)) => {
                let now = fail!(from self, when self.clock.now(),
                                with ReactorWaitError::InternalError,
                                "{msg} since the current time could not be acquired.");
                let translated_timeout = t.as_duration(now);

                self.wait(
                    fn_call,
                    |f, event| self.set.timed_wait(translated_timeout, event, f),
                    translated_timeout,
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

                let timeout_trigger_fd = unsafe { listener.file_descriptor().native_handle() };
                let mut number_of_timeout_triggers = 0;
                self.wait(
                    |fd: &FileDescriptor| {
                        if unsafe { fd.native_handle() == timeout_trigger_fd } {
                            number_of_timeout_triggers += 1;
                        } else {
                            fn_call(fd);
                        }
                    },
                    |f, event| self.set.blocking_wait(event, f),
                    Duration::MAX,
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
        fn_call: F,
    ) -> Result<usize, super::ReactorWaitError> {
        self.wait(
            fn_call,
            |f, event| self.set.blocking_wait(event, f),
            Duration::MAX,
        )
    }
}

pub struct ReactorBuilder {}

impl crate::reactor::ReactorBuilder<Reactor> for ReactorBuilder {
    fn new() -> Self {
        Self {}
    }

    fn create(
        self,
        clock: Arc<dyn ClockReactorSchedule>,
    ) -> Result<Reactor, super::ReactorCreateError> {
        Ok(Reactor::new(clock))
    }
}
