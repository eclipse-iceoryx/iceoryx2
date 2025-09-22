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

use std::time::Duration;

use iceoryx2_bb_posix::file_descriptor::FileDescriptor;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EpollCreateError {}

pub struct EpollGuard<'epoll> {
    epoll: &'epoll Epoll,
}

pub struct Epoll {
    epoll_fd: FileDescriptor,
}

impl Epoll {
    pub fn create() -> Result<Self, EpollCreateError> {
        todo!()
    }

    pub fn attach<'epoll>(&'epoll mut self, fd: &FileDescriptor) -> EpollGuard<'epoll> {
        todo!()
    }

    pub fn try_wait(&self) {
        todo!()
    }

    pub fn timed_wait(&self, timeout: Duration) {
        todo!()
    }

    pub fn blocking_wait(&self) {
        todo!()
    }
}
