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

use iceoryx2_dmabuf::external_buffer::ExternalFdBuffer;
use iceoryx2_dmabuf::shm::{FdBackedSharedMemory, linux::Linux};
use std::os::fd::{FromRawFd as _, OwnedFd};

fn memfd(name: &core::ffi::CStr, len: i64) -> OwnedFd {
    // SAFETY: memfd_create is well-defined; ftruncate sizes the empty fd.
    let raw = unsafe { libc::memfd_create(name.as_ptr(), 0) };
    assert!(raw >= 0, "memfd_create");
    let rc = unsafe { libc::ftruncate(raw, len) };
    assert_eq!(rc, 0, "ftruncate");
    unsafe { OwnedFd::from_raw_fd(raw) }
}

#[test]
fn external_fd_buffer_carries_fd_and_len() {
    let fd = memfd(c"buf", 0);
    let buf = ExternalFdBuffer::new(fd, 4096);
    assert_eq!(buf.len, 4096);
}

#[test]
fn fd_backed_shm_from_owned_fd_succeeds() {
    let fd = memfd(c"shm", 4096);
    let shm = Linux::from_owned_fd(fd, 4096).expect("from_owned_fd");
    assert_eq!(shm.len(), 4096);
    assert!(!shm.payload_ptr().is_null());
}

#[test]
fn fd_backed_shm_payload_writeable() {
    let fd = memfd(c"shmw", 4096);
    let shm = Linux::from_owned_fd(fd, 4096).expect("mmap");
    // SAFETY: payload_ptr is a real mapped region of size shm.len() owned by us.
    unsafe {
        let slice = std::slice::from_raw_parts_mut(shm.payload_ptr(), 64);
        slice.copy_from_slice(&[0xABu8; 64]);
        let read = std::slice::from_raw_parts(shm.payload_ptr(), 64);
        assert!(read.iter().all(|&b| b == 0xAB));
    }
}
