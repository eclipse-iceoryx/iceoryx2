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

use std::{fmt::Debug, time::Duration};

use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::{
    file_descriptor::FileDescriptor,
    file_descriptor_set::{
        FileDescriptorSet, FileDescriptorSetGuard, FileDescriptorSetWaitError, FileEvent,
    },
};

use crate::reactor::{ReactorAttachError, ReactorWaitError};

impl crate::reactor::ReactorGuard<'_, '_> for FileDescriptorSetGuard<'_, '_> {}

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

    fn wait<F: FnMut(&FileDescriptor)>(
        &self,
        fn_call: F,
        timeout: std::time::Duration,
    ) -> Result<(), super::ReactorWaitError> {
        let msg = "Unable to wait on Reactor";
        match self.set.timed_wait(timeout, FileEvent::Read, fn_call) {
            Ok(()) => Ok(()),
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

impl crate::reactor::Reactor for Reactor {
    type Guard<'reactor, 'attachment> = FileDescriptorSetGuard<'reactor, 'attachment>;
    type Builder = ReactorBuilder;

    fn capacity() -> usize {
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
        Ok(fail!(from self, when self.set.add(value),
                with ReactorAttachError::CapacityExceeded,
                "Unable to attach {:?} to reactor since the capacity of the underlying file descriptor set was exceeded.",
                value))
    }

    fn try_wait<F: FnMut(&FileDescriptor)>(
        &self,
        fn_call: F,
    ) -> Result<(), super::ReactorWaitError> {
        self.wait(fn_call, Duration::ZERO)
    }

    fn timed_wait<F: FnMut(&FileDescriptor)>(
        &self,
        fn_call: F,
        timeout: std::time::Duration,
    ) -> Result<(), super::ReactorWaitError> {
        self.wait(fn_call, timeout)
    }

    fn blocking_wait<F: FnMut(&FileDescriptor)>(
        &self,
        fn_call: F,
    ) -> Result<(), super::ReactorWaitError> {
        const INFINITE_TIMEOUT: Duration = Duration::from_secs(3600 * 24 * 365);
        self.wait(fn_call, INFINITE_TIMEOUT)
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
