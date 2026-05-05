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
#![cfg(target_os = "linux")]

use std::io;
use std::os::fd::{AsFd as _, AsRawFd as _, BorrowedFd, OwnedFd};

use super::FdBackedSharedMemory;

/// Linux fd-backed SHM. mmap on construction, munmap on drop.
pub struct Linux {
    fd: OwnedFd,
    base: *mut u8,
    len: usize,
}

// SAFETY: Linux owns its mmap region; the OwnedFd is Send. Reads/writes
// through the *mut u8 are the caller's responsibility (they must use the
// dma_buf::DmaBuf wrapper or equivalent for CPU-sync on cache-incoherent
// SoCs). Marking only Send (not Sync) preserves the invariant that the
// caller owns synchronization.
unsafe impl Send for Linux {}

impl Drop for Linux {
    fn drop(&mut self) {
        // SAFETY: base/len are owned by this struct; the mapping was created
        // by our `from_owned_fd` and has not been unmapped elsewhere.
        #[allow(unsafe_code)]
        unsafe {
            libc::munmap(self.base.cast(), self.len);
        }
    }
}

impl FdBackedSharedMemory for Linux {
    fn from_owned_fd(fd: OwnedFd, len: usize) -> io::Result<Self> {
        // SAFETY: fd is a valid OwnedFd; mmap with PROT_READ|PROT_WRITE and
        // MAP_SHARED is well-defined for memfd/DMA-BUF fds of at least `len`
        // bytes. Returns MAP_FAILED on error.
        #[allow(unsafe_code)]
        let base = unsafe {
            libc::mmap(
                core::ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd.as_raw_fd(),
                0,
            )
        };
        if base == libc::MAP_FAILED {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            fd,
            base: base.cast(),
            len,
        })
    }

    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }

    fn len(&self) -> usize {
        self.len
    }

    fn payload_ptr(&self) -> *mut u8 {
        self.base
    }
}
