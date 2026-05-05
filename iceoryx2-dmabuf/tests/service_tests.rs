// SPDX-License-Identifier: Apache-2.0 OR MIT
#![cfg(target_os = "linux")]

use iceoryx2_dmabuf::service::Service;
use std::os::fd::{AsFd as _, FromRawFd as _, OwnedFd};
use std::time::Duration;

fn memfd(name: &core::ffi::CStr) -> OwnedFd {
    let raw = unsafe { libc::memfd_create(name.as_ptr(), 0) };
    assert!(raw >= 0);
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
fn service_open_create_idempotent() {
    let factory_a = Service::open_or_create::<u64>("dmabuf/test/idem").expect("first");
    let factory_b = Service::open_or_create::<u64>("dmabuf/test/idem").expect("second");
    drop(factory_a);
    drop(factory_b);
}

#[test]
fn publish_receive_memfd_through_dmabuf_service() {
    let factory = Service::open_or_create::<u64>("dmabuf/test/rt").expect("factory");
    let mut pubr = factory.publisher_builder().create().expect("publisher");
    let mut subr = factory.subscriber_builder().create().expect("subscriber");

    // Wait for fd-channel handshake.
    std::thread::sleep(Duration::from_millis(50));

    let fd = memfd(c"rt");
    pubr.publish(42u64, fd.as_fd(), 4096).expect("publish");

    let recvd = wait_until(Duration::from_millis(500), || {
        matches!(subr.receive(), Ok(Some((42, _, 4096))))
    });
    assert!(recvd, "did not receive");
}
