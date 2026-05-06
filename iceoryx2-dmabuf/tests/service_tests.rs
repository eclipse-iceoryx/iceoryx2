// SPDX-License-Identifier: Apache-2.0 OR MIT
#![cfg(target_os = "linux")]

use iceoryx2_dmabuf::service::Service;
use iceoryx2_dmabuf::service_publisher::DmaBufServicePublisher;
use iceoryx2_dmabuf::service_subscriber::DmaBufServiceSubscriber;
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

fn unique_service_name(tag: &str) -> String {
    format!("dmabuf/test/{}/{}", std::process::id(), tag)
}

#[test]
fn service_open_create_idempotent() {
    let name = unique_service_name("idem");
    let factory_a = Service::open_or_create::<u64>(&name).expect("first");
    let factory_b = Service::open_or_create::<u64>(&name).expect("second");
    drop(factory_a);
    drop(factory_b);
}

#[test]
fn publish_receive_memfd_through_dmabuf_service() {
    let factory = Service::open_or_create::<u64>(&unique_service_name("rt")).expect("factory");
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

// TEST 6 — spec §31: user-header is () — no internal token leaks into the
// iceoryx2 metadata channel.
//
// The iceoryx2 service is built with user-header = () (see service.rs).
// We verify this structurally: DmaBufServicePublisher::create and
// DmaBufServiceSubscriber::create both accept the same PortFactory<_, Meta, ()>
// which is only possible if the user-header type is (). If it were anything
// else the factory call would fail to compile.
//
// We cannot inspect the PortFactory type parameter from outside the crate
// (meta_factory is private), so this test is a build-time proof: the
// publish/receive round-trip compiles only with user-header = ().
#[test]
fn meta_user_header_is_callee_owned() {
    // Structural test: open_or_create<u64> with user-header=().
    // If the service were built with a non-() user-header, this line would
    // not compile because DmaBufServicePublisher::create requires
    // PortFactory<ipc::Service, Meta, ()>.
    let svc_name = unique_service_name("user_header");
    let factory = Service::open_or_create::<u64>(&svc_name).expect("service");

    // Confirm publisher and subscriber can be built from the same factory.
    // This is only possible if both agree on user-header = ().
    let _pub: DmaBufServicePublisher<u64> =
        factory.publisher_builder().create().expect("publisher");
    let _sub: DmaBufServiceSubscriber<u64> =
        factory.subscriber_builder().create().expect("subscriber");

    // No token field leaks through the iceoryx2 metadata sample type.
    drop(_pub);
    drop(_sub);
    drop(factory);
}

// TEST 7 — migration from it_service_gone: subscriber does not panic when
// publisher is dropped while the subscriber is mid-receive.
//
// Contract: sub.receive() after publisher drop must return Ok(None) or Err(...).
// Neither panic nor undefined behaviour is acceptable.
#[test]
fn service_dropped_subscriber_recovers() {
    let svc_name = unique_service_name("svcgone");

    // Create subscriber first (requires publisher to already have bound the socket).
    // We build publisher first so the socket exists when subscriber connects.
    let factory_pub = Service::open_or_create::<u64>(&svc_name).expect("factory-pub");
    let mut pub_ = factory_pub.publisher_builder().create().expect("publisher");

    let factory_sub = Service::open_or_create::<u64>(&svc_name).expect("factory-sub");
    let mut sub = factory_sub
        .subscriber_builder()
        .create()
        .expect("subscriber");

    // Let the fd-channel handshake complete.
    std::thread::sleep(Duration::from_millis(50));

    // Publish one frame.
    let fd = memfd(c"gone");
    pub_.publish(7u64, fd.as_fd(), 64).expect("publish");

    // Drain the frame so the subscriber is in a clean state.
    let _ = wait_until(Duration::from_millis(300), || {
        matches!(sub.receive(), Ok(Some(_)))
    });

    // Drop the publisher (and its factory/node).
    drop(pub_);
    drop(factory_pub);

    // Subscriber attempts receive after publisher dropped. Must not panic.
    let result = sub.receive();
    // Both Ok(None) (no more samples) and Err(_) are acceptable outcomes.
    // The spec contract is "no panic, no UB".
    let _ = result;
}
