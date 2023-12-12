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

pub mod posix_select;

use std::{fmt::Debug, time::Duration};

use iceoryx2_bb_posix::{
    file_descriptor::FileDescriptor, file_descriptor_set::SynchronousMultiplexing,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactorCreateError {
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactorAttachError {
    CapacityExceeded,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactorWaitError {
    Interrupt,
    InsufficientPermissions,
    UnknownError,
}

pub trait ReactorGuard<'reactor, 'attachment> {}

pub trait Reactor: Sized {
    type Guard<'reactor, 'attachment>: ReactorGuard<'reactor, 'attachment>
    where
        Self: 'reactor;
    type Builder: ReactorBuilder<Self>;

    fn capacity() -> usize;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;

    fn attach<'reactor, 'attachment, F: SynchronousMultiplexing + Debug>(
        &'reactor self,
        value: &'attachment F,
    ) -> Result<Self::Guard<'reactor, 'attachment>, ReactorAttachError>;

    fn try_wait<F: FnMut(&FileDescriptor)>(&self, fn_call: F) -> Result<(), ReactorWaitError>;
    fn timed_wait<F: FnMut(&FileDescriptor)>(
        &self,
        fn_call: F,
        timeout: Duration,
    ) -> Result<(), ReactorWaitError>;
    fn blocking_wait<F: FnMut(&FileDescriptor)>(&self, fn_call: F) -> Result<(), ReactorWaitError>;
}

pub trait ReactorBuilder<T: Reactor> {
    fn new() -> Self;
    fn create(self) -> Result<T, ReactorCreateError>;
}
