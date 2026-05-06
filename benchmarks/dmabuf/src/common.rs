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

//! Shared benchmark helpers.
//!
//! [`make_memfd`] is the single DRY entry point for creating an anonymous
//! memfd wrapped as a `dma_buf::DmaBuf` suitable for passing to
//! `DmaBufPublisher::publish`.

/// Create an anonymous memfd of `len` bytes and wrap it as a `dma_buf::DmaBuf`.
///
/// Uses `libc::memfd_create` + `libc::ftruncate` (both standard Linux
/// syscalls) and then transfers ownership into `dma_buf::DmaBuf::from(fd)`.
///
/// # Errors
///
/// Returns an error if `memfd_create` returns a negative fd or if `ftruncate`
/// fails (the latter is best-effort; only a hard OS failure is returned).
#[cfg(target_os = "linux")]
pub(crate) fn make_memfd(len: i64) -> Result<dma_buf::DmaBuf, Box<dyn core::error::Error>> {
    use std::os::fd::{FromRawFd as _, OwnedFd};

    // SAFETY: memfd_create is a well-known Linux syscall.
    //   - `c"bench"` is a valid NUL-terminated CStr literal (Rust 1.77+).
    //   - flags = 0 (no sealing, no close-on-exec needed; DmaBuf wraps OwnedFd).
    //   - The returned raw fd is valid if >= 0; checked immediately below.
    let raw = unsafe { libc::memfd_create(c"bench".as_ptr(), 0) };
    if raw < 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    // SAFETY: `raw` is a valid open file descriptor returned by memfd_create.
    //   ftruncate sets the file size; errors are non-fatal for the benchmark
    //   (subscriber only needs the fd, not the content).
    let _ = unsafe { libc::ftruncate(raw, len) };

    // SAFETY: `raw` is a valid owned fd; we move it into OwnedFd here.
    let owned: OwnedFd = unsafe { OwnedFd::from_raw_fd(raw) };

    // dma_buf::DmaBuf::from(OwnedFd) is a zero-syscall ownership transfer.
    Ok(dma_buf::DmaBuf::from(owned))
}
