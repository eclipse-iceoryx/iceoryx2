// Copyright (c) 2026 Contributors to the Eclipse Foundation
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
#![cfg(not(target_os = "linux"))]

use std::io;
use std::os::fd::{BorrowedFd, OwnedFd};

use super::FdBackedSharedMemory;

/// Non-Linux stub. Uninhabited — cannot be constructed at runtime.
///
/// `from_owned_fd` always returns `Unsupported`, so no value of this type
/// can ever exist. The trait methods use `match *self {}` to exploit that
/// invariant at compile time, producing no code and no panic.
#[derive(Debug)]
pub enum NonLinux {}

impl FdBackedSharedMemory for NonLinux {
    fn from_owned_fd(_fd: OwnedFd, _len: usize) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "fd-backed SHM is Linux-only",
        ))
    }

    fn as_fd(&self) -> BorrowedFd<'_> {
        // Match an empty enum — the compiler proves this branch is dead.
        match *self {}
    }

    fn len(&self) -> usize {
        match *self {}
    }

    fn payload_ptr(&self) -> *mut u8 {
        match *self {}
    }
}
