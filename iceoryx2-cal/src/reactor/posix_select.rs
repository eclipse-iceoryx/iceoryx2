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

use core::{fmt::Debug, time::Duration};

use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::{
    clock::{nanosleep, NanosleepError},
    file_descriptor::FileDescriptor,
    file_descriptor_set::{
        FileDescriptorSet, FileDescriptorSetAddError, FileDescriptorSetGuard,
        FileDescriptorSetWaitError, FileEvent,
    },
};

use crate::reactor::{ReactorAttachError, ReactorWaitError};

impl crate::reactor::ReactorGuard<'_, '_> for FileDescriptorSetGuard<'_, '_> {
    fn file_descriptor(&self) -> &FileDescriptor {
        self.file_descriptor()
    }
}

#[derive(Debug)]
pub struct Reactor {
    set: FileDescriptorSet,
}

impl Reactor {
    fn new() -> Self {
        Self {
            set: FileDescriptorSet::new(),
        }
    }

    fn wait<
        F: FnMut(&FileDescriptor),
        W: FnMut(F, FileEvent) -> Result<usize, FileDescriptorSetWaitError>,
    >(
        &self,
        fn_call: F,
        mut wait_call: W,
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
                    fail!(from self, with ReactorWaitError::UnknownError,
                        "{} since an unknown failure occurred while waiting ({:?}).",
                        msg, v);
                }
            }
        } else {
            match wait_call(fn_call, FileEvent::Read) {
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
                    fail!(from self, with ReactorWaitError::UnknownError,
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
        F: iceoryx2_bb_posix::file_descriptor_set::SynchronousMultiplexing + Debug,
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
            |f: F, event: FileEvent| self.set.timed_wait(Duration::ZERO, event, f),
            Duration::ZERO,
        )
    }

    fn timed_wait<F: FnMut(&FileDescriptor)>(
        &self,
        fn_call: F,
        timeout: core::time::Duration,
    ) -> Result<usize, super::ReactorWaitError> {
        self.wait(
            fn_call,
            |f: F, event: FileEvent| self.set.timed_wait(timeout, event, f),
            timeout,
        )
    }

    fn blocking_wait<F: FnMut(&FileDescriptor)>(
        &self,
        fn_call: F,
    ) -> Result<usize, super::ReactorWaitError> {
        self.wait(
            fn_call,
            |f: F, event: FileEvent| self.set.blocking_wait(event, f),
            Duration::MAX,
        )
    }
}

pub struct ReactorBuilder {}

impl crate::reactor::ReactorBuilder<Reactor> for ReactorBuilder {
    fn new() -> Self {
        Self {}
    }

    fn create(self) -> Result<Reactor, super::ReactorCreateError> {
        Ok(Reactor::new())
    }
}
