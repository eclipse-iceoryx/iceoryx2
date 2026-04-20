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

//! Integration test: service drop causes subscriber recv to return cleanly.
//!
//! Publisher sends 5 frames then drops.  Subscriber polls in a loop; after the
//! producer drops, the test asserts:
//! - No panic occurs.
//! - No hang (enforced by a 500 ms timeout).
//! - The result is either `Ok(None)` (no more samples) or an `Err` wrapping
//!   `FdSidecarError::IceoryxReceive` — iceoryx2 may or may not error when the
//!   publisher drops depending on service lifetime semantics.
//!
//! On non-Linux: asserts `FdSidecarPublisher::create` returns `UnsupportedPlatform`.

#[cfg(not(target_os = "linux"))]
mod non_linux_test {
    use iceoryx2::prelude::ZeroCopySend;

    #[derive(Debug, Clone, Copy, ZeroCopySend)]
    #[repr(C)]
    struct Meta {
        seq: u64,
    }

    #[test]
    fn service_gone_non_linux_returns_unsupported() {
        let result =
            iceoryx2_dmabuf::FdSidecarPublisher::<iceoryx2::service::ipc::Service, Meta>::create(
                "service-gone-non-linux-test",
            );
        assert!(
            matches!(
                result,
                Err(iceoryx2_dmabuf::FdSidecarError::UnsupportedPlatform)
            ),
            "expected UnsupportedPlatform on non-Linux",
        );
    }
}

#[cfg(target_os = "linux")]
mod linux_tests {
    use iceoryx2::prelude::ZeroCopySend;
    use iceoryx2_dmabuf::{FdSidecarPublisher, FdSidecarSubscriber};
    use rustix::fs::{MemfdFlags, memfd_create};
    use std::time::Duration;

    const N_FRAMES: u64 = 5;
    const SETTLE_MS: u64 = 20;
    const POLL_MS: u64 = 10;
    const TIMEOUT_MS: u64 = 500;

    #[derive(Debug, Clone, Copy, ZeroCopySend)]
    #[repr(C)]
    struct Meta {
        seq: u64,
    }

    fn make_fd() -> std::os::fd::OwnedFd {
        memfd_create("service-gone-frame", MemfdFlags::CLOEXEC).expect("memfd_create failed")
    }

    #[test]
    fn publisher_drop_subscriber_recv_no_panic_no_hang() {
        let service = format!("service-gone-{}", std::process::id());

        let mut pub_ =
            FdSidecarPublisher::<iceoryx2::service::ipc::Service, Meta>::create(&service)
                .expect("FdSidecarPublisher::create failed");
        let mut sub_ =
            FdSidecarSubscriber::<iceoryx2::service::ipc::Service, Meta>::create(&service)
                .expect("FdSidecarSubscriber::create failed");

        std::thread::sleep(Duration::from_millis(SETTLE_MS));

        // Send N frames and drain them.
        for seq in 1..=N_FRAMES {
            let fd = make_fd();
            pub_.send(Meta { seq }, fd).expect("send failed");
            std::thread::sleep(Duration::from_millis(SETTLE_MS));

            let received = sub_.recv().expect("recv during publish phase failed");
            assert!(received.is_some(), "expected Some for frame {seq}");
        }

        // Drop the publisher.
        drop(pub_);

        // Poll after publisher drop — must not panic and must not hang.
        let deadline = std::time::Instant::now() + Duration::from_millis(TIMEOUT_MS);
        let mut saw_clean = false;
        while std::time::Instant::now() < deadline {
            match sub_.recv() {
                Ok(None) => {
                    saw_clean = true;
                    break;
                }
                Ok(Some(_)) => {
                    // Stale sample in the ring buffer — consume and continue.
                }
                Err(iceoryx2_dmabuf::FdSidecarError::IceoryxReceive(_)) => {
                    saw_clean = true;
                    break;
                }
                Err(e) => {
                    panic!("unexpected error after publisher drop: {e:?}");
                }
            }
            std::thread::sleep(Duration::from_millis(POLL_MS));
        }

        assert!(
            saw_clean,
            "subscriber did not observe Ok(None) or IceoryxReceive within {TIMEOUT_MS} ms",
        );
    }
}
