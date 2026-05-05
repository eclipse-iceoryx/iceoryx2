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

//! Plain wrapper struct: an externally-allocated buffer identified by an
//! `OwnedFd` and a known length. Used as the configuration handle for
//! [`crate::shm::FdBackedSharedMemory::from_owned_fd`].
//!
//! Compared to a tuple `(OwnedFd, usize)`, this struct keeps the field names
//! self-documenting and makes future extensions (alignment, allocator-id, …)
//! source-compatible.

use std::os::fd::OwnedFd;

/// An externally-allocated buffer identified by an fd and a byte length.
#[derive(Debug)]
pub struct ExternalFdBuffer {
    /// The file descriptor referencing the buffer (DMA-BUF, memfd, …).
    pub fd: OwnedFd,
    /// Size of the buffer in bytes.
    pub len: usize,
}

impl ExternalFdBuffer {
    /// Wraps an externally-allocated fd of size `len` bytes.
    #[must_use]
    pub fn new(fd: OwnedFd, len: usize) -> Self {
        Self { fd, len }
    }
}
