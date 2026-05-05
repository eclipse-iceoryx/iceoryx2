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

//! Standalone fd-backed shared memory.
//!
//! `FdBackedSharedMemory` is a NEW trait — NOT a sub-trait of
//! `iceoryx2_cal::shared_memory::SharedMemory`. The cal-layer trait is built
//! around `PointerOffset` and pool allocators; DMA-BUF is a different
//! lifecycle. See `iceoryx2-dmabuf/specs/arch-dmabuf-service-variant.adoc`
//! decision D1 (post-spike pivot).

use std::io;
use std::os::fd::{BorrowedFd, OwnedFd};

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(not(target_os = "linux"))]
pub mod non_linux;

/// Trait for shared memory backed by an externally-supplied file descriptor
/// (DMA-BUF, memfd). The fd is mmapped on construction and munmapped on drop.
pub trait FdBackedSharedMemory: Sized {
    /// Maps `fd` into the process address space for `len` bytes.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if the underlying `mmap` call fails, or if the
    /// platform does not support fd-backed SHM.
    fn from_owned_fd(fd: OwnedFd, len: usize) -> io::Result<Self>;

    /// Borrows the underlying file descriptor.
    fn as_fd(&self) -> BorrowedFd<'_>;

    /// Returns the mapped size in bytes.
    fn len(&self) -> usize;

    /// Returns `true` if the mapped region is zero-length.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a raw pointer to the start of the mapped payload.
    ///
    /// The caller must respect [`len`](Self::len) bounds and handle
    /// CPU-cache synchronization (e.g. `DMA_BUF_IOCTL_SYNC`) on
    /// cache-incoherent SoCs.
    fn payload_ptr(&self) -> *mut u8;
}
