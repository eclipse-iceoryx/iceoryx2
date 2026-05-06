// SPDX-License-Identifier: Apache-2.0 OR MIT
#![cfg(all(target_os = "linux", feature = "dma-buf"))]

use iceoryx2_dmabuf::{DmaBufPublisher, DmaBufSubscriber};
use std::os::fd::{FromRawFd as _, OwnedFd};
use std::time::Duration;

fn memfd(name: &core::ffi::CStr, len: i64) -> OwnedFd {
    // SAFETY: memfd_create is a standard Linux syscall; name is a valid CStr.
    let raw = unsafe { libc::memfd_create(name.as_ptr(), 0) };
    assert!(raw >= 0, "memfd_create failed");
    // SAFETY: raw is a valid fd returned by memfd_create.
    let _ = unsafe { libc::ftruncate(raw, len) };
    // SAFETY: raw is a valid, owned file descriptor.
    unsafe { OwnedFd::from_raw_fd(raw) }
}

fn wait_until<F: FnMut() -> bool>(deadline: Duration, mut f: F) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < deadline {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    f()
}

#[test]
fn typed_publish_receive_via_memfd_wrapped_in_dmabuf() {
    let mut pubr =
        DmaBufPublisher::<u64>::create("dmabuf/test/typed-rt").expect("publisher create");
    let mut subr =
        DmaBufSubscriber::<u64>::create("dmabuf/test/typed-rt").expect("subscriber create");

    // Wait for the UDS fd-channel handshake.
    std::thread::sleep(Duration::from_millis(50));

    let fd = memfd(c"rt", 4096);
    let buf: dma_buf::DmaBuf = fd.into();
    pubr.publish(42u64, &buf).expect("publish");

    let recvd = wait_until(Duration::from_millis(500), || {
        matches!(subr.receive(), Ok(Some((42, _))))
    });
    assert!(recvd, "did not receive typed sample within deadline");
}
