// SPDX-License-Identifier: Apache-2.0 OR MIT
#![cfg(target_os = "linux")]

use iceoryx2_dmabuf::connection::{FdPassingConnection, linux::Linux};
use std::os::fd::{AsFd as _, FromRawFd as _, OwnedFd};
use std::time::Duration;

fn unique_socket_path(tag: &str) -> String {
    format!("/tmp/iox2-conntest-{}-{}.sock", std::process::id(), tag)
}

fn memfd(name: &core::ffi::CStr) -> OwnedFd {
    let raw = unsafe { libc::memfd_create(name.as_ptr(), 0) };
    assert!(raw >= 0);
    unsafe { OwnedFd::from_raw_fd(raw) }
}

/// Poll until a closure returns true, or until timeout.
/// Intentional blocking sleep: this is synchronous test code only,
/// not used in any production or async path.
fn wait_until<F: FnMut() -> bool>(deadline: Duration, mut f: F) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < deadline {
        if f() {
            return true;
        }
        // Intentional: synchronous test polling helper, not in production code.
        std::thread::sleep(Duration::from_millis(2));
    }
    f()
}

#[test]
fn send_recv_memfd_roundtrip() {
    let path = unique_socket_path("rt");
    let publisher = Linux::open_publisher(&path).expect("bind");
    let subscriber = Linux::open_subscriber(&path).expect("connect");

    // Wait for publisher's accept thread to register the subscriber.
    // Test code: unwrap_or(0) is safe here since a poisoned lock means no subscribers.
    assert!(
        wait_until(Duration::from_millis(500), || {
            publisher.connected_subscriber_count().unwrap_or(0) >= 1
        }),
        "subscriber connect not seen"
    );

    let fd = memfd(c"rt");
    // token=0: wire v2, token field unused in this test.
    publisher.send_with_fd(fd.as_fd(), 4096, 0).expect("send");

    // Wait for the message to land.
    let recvd = wait_until(Duration::from_millis(500), || {
        matches!(subscriber.recv_with_fd(), Ok(Some(_)))
    });
    assert!(recvd, "no message arrived");
}

#[test]
fn fanout_one_pub_three_sub_100_frames() {
    let path = unique_socket_path("fanout");
    let publisher = Linux::open_publisher(&path).expect("bind");
    let subs: Vec<_> = (0..3)
        .map(|_| Linux::open_subscriber(&path).expect("connect"))
        .collect();

    // Test code: unwrap_or(0) is safe here since a poisoned lock means no subscribers.
    assert!(wait_until(Duration::from_millis(500), || {
        publisher.connected_subscriber_count().unwrap_or(0) >= 3
    }));

    for _ in 0..100 {
        let fd = memfd(c"f");
        // token=0: wire v2, token field unused in this test.
        publisher.send_with_fd(fd.as_fd(), 1024, 0).expect("send");
    }

    for sub in &subs {
        let mut count = 0;
        wait_until(Duration::from_secs(2), || {
            while let Ok(Some(_)) = sub.recv_with_fd() {
                count += 1;
            }
            count >= 100
        });
        assert!(count >= 100, "subscriber received {count}/100");
    }
}

#[test]
fn subscriber_disconnect_publisher_prunes() {
    let path = unique_socket_path("disc");
    let publisher = Linux::open_publisher(&path).expect("bind");
    {
        let _sub = Linux::open_subscriber(&path).expect("connect");
        // Test code: unwrap_or(0) is safe here since a poisoned lock means no subscribers.
        assert!(wait_until(Duration::from_millis(500), || {
            publisher.connected_subscriber_count().unwrap_or(0) >= 1
        }));
    } // sub dropped
    let fd = memfd(c"d");
    // token=0: wire v2, token field unused in this test.
    let _ = publisher.send_with_fd(fd.as_fd(), 1024, 0);
    let _ = publisher.send_with_fd(fd.as_fd(), 1024, 0);
    // Test code: unwrap_or(0) is safe here since a poisoned lock means no subscribers.
    assert!(wait_until(Duration::from_millis(500), || {
        publisher.connected_subscriber_count().unwrap_or(0) == 0
    }));
}

#[test]
fn back_channel_release_roundtrip() {
    let path = unique_socket_path("ack");
    let publisher = Linux::open_publisher(&path).expect("bind");
    let subscriber = Linux::open_subscriber(&path).expect("connect");
    assert!(wait_until(Duration::from_millis(500), || {
        publisher.connected_subscriber_count().unwrap_or(0) >= 1
    }));

    // Publisher sends an fd with token=42.
    let fd = memfd(c"ack");
    publisher
        .send_with_fd(fd.as_fd(), 4096, 42)
        .expect("send fd");

    // Subscriber receives + releases.
    let recvd = wait_until(Duration::from_millis(500), || {
        matches!(subscriber.recv_with_fd(), Ok(Some(_)))
    });
    assert!(recvd, "no fd received");
    subscriber.send_release_ack(42).expect("send ack");

    // Publisher drains the ack.
    let acked = wait_until(Duration::from_millis(500), || {
        matches!(publisher.recv_release_ack(), Ok(Some(42)))
    });
    assert!(acked, "no ack drained");
}
