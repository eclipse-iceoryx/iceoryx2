// SPDX-License-Identifier: Apache-2.0 OR MIT
#![cfg(all(target_os = "linux", feature = "dma-buf"))]

//! Integration test: fd inode is preserved across the SCM_RIGHTS round-trip.
//!
//! Publisher holds a memfd wrapped as DmaBuf; after round-trip the subscriber
//! receives a DmaBuf whose underlying fd points to the same kernel inode as
//! measured by (st_dev, st_ino). This validates that SCM_RIGHTS transfers the
//! same kernel object, not a copy.

use iceoryx2_dmabuf::{DmaBufPublisher, DmaBufSubscriber};
use std::os::fd::{AsFd as _, FromRawFd as _, OwnedFd};
use std::time::Duration;

/// Poll until predicate returns true, or deadline elapses (last check included).
/// Synchronous test-only helper; sleep is intentional, not inside any async context.
fn wait_until<F: FnMut() -> bool>(deadline: Duration, mut f: F) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < deadline {
        if f() {
            return true;
        }
        // Intentional: synchronous test polling helper, not in production or async code.
        std::thread::sleep(Duration::from_millis(2));
    }
    f()
}

/// Create a memfd of `len` bytes via libc.
fn memfd_raw(name: &core::ffi::CStr, len: i64) -> OwnedFd {
    // SAFETY: memfd_create is a well-known Linux syscall; name is a valid NUL-terminated CStr.
    let raw = unsafe { libc::memfd_create(name.as_ptr(), 0) };
    assert!(raw >= 0, "memfd_create failed");
    // SAFETY: raw is a valid fd returned by memfd_create; ftruncate sets the size.
    let _ = unsafe { libc::ftruncate(raw, len) };
    // SAFETY: raw is a valid, owned file descriptor.
    unsafe { OwnedFd::from_raw_fd(raw) }
}

#[test]
fn fd_identity_preserved_through_roundtrip() {
    const SERVICE: &str = "dmabuf/test/identity";
    const BUF_SIZE: i64 = 4096;
    const DEADLINE: Duration = Duration::from_millis(500);

    let owned_fd = memfd_raw(c"dmabuf-identity-test", BUF_SIZE);

    // Record (st_dev, st_ino) before publishing.
    let stat_before = rustix::fs::fstat(&owned_fd).expect("fstat publisher fd");
    let (pub_dev, pub_ino) = (stat_before.st_dev, stat_before.st_ino);

    // Wrap as DmaBuf (zero syscall — just consumes the OwnedFd).
    let buf: dma_buf::DmaBuf = owned_fd.into();

    let mut pubr = DmaBufPublisher::<u64>::create(SERVICE).expect("publisher create");
    let mut subr = DmaBufSubscriber::<u64>::create(SERVICE).expect("subscriber create");

    // Allow the UDS fd-channel handshake to complete before publishing.
    // Intentional blocking settle delay in synchronous test code.
    std::thread::sleep(Duration::from_millis(50));

    pubr.publish(0xCAFE_BABE_u64, &buf).expect("publish");

    // Poll until a sample arrives; capture it to avoid re-calling receive.
    let mut result: Option<(u64, dma_buf::DmaBuf)> = None;
    let arrived = wait_until(DEADLINE, || {
        match subr.receive().expect("receive must not error") {
            Some(pair) => {
                result = Some(pair);
                true
            }
            None => false,
        }
    });
    assert!(arrived, "no sample arrived within {DEADLINE:?}");

    let (meta, recv_buf) = result.expect("result set when arrived is true");
    assert_eq!(meta, 0xCAFE_BABE_u64, "meta mismatch");

    let stat_after = rustix::fs::fstat(recv_buf.as_fd()).expect("fstat subscriber fd");

    assert_eq!(
        (pub_dev, pub_ino),
        (stat_after.st_dev, stat_after.st_ino),
        "inode mismatch: publisher ({pub_dev}, {pub_ino}) != subscriber ({}, {})",
        stat_after.st_dev,
        stat_after.st_ino,
    );
}
