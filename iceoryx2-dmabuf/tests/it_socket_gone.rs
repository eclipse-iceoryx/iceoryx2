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

//! Integration test: sidecar socket unlink causes subscriber `SideChannelIo`,
//! then `reconnect()` restores operation.
//!
//! Sequence:
//! 1. Publisher sends frames 1–3; subscriber receives them successfully.
//! 2. The publisher's sidecar socket file is unlinked while the publisher
//!    drops, closing all subscriber connections.
//! 3. Subscriber's next `recv()` returns either `Ok(None)` (no iceoryx2
//!    sample) or `Err(FdSidecarError::SideChannelIo | NoFdInMessage)` because the
//!    sidecar is gone.
//! 4. A new publisher binds a new socket at the same path.
//! 5. `sub.reconnect()` re-establishes the sidecar connection.
//! 6. Publisher sends frame 4; subscriber receives it successfully.
//!
//! Linux-only; compiled out on non-Linux targets.

#[cfg(target_os = "linux")]
mod linux_tests {
    use iceoryx2::prelude::ZeroCopySend;
    use iceoryx2_dmabuf::{FdSidecarError, FdSidecarPublisher, FdSidecarSubscriber};
    use rustix::fs::{MemfdFlags, memfd_create};
    use std::time::Duration;

    const SETTLE_MS: u64 = 20;
    const RECONNECT_SETTLE_MS: u64 = 50;

    #[derive(Debug, Clone, Copy, ZeroCopySend)]
    #[repr(C)]
    struct Meta {
        seq: u64,
    }

    fn make_fd() -> std::os::fd::OwnedFd {
        memfd_create("socket-gone-frame", MemfdFlags::CLOEXEC).expect("memfd_create failed")
    }

    fn recv_timeout(
        sub: &mut FdSidecarSubscriber<iceoryx2::service::ipc::Service, Meta>,
        max_ms: u64,
    ) -> Option<(Meta, std::os::fd::OwnedFd)> {
        let deadline = std::time::Instant::now() + Duration::from_millis(max_ms);
        while std::time::Instant::now() < deadline {
            match sub.recv() {
                Ok(Some(pair)) => return Some(pair),
                Ok(None) => {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(_) => return None,
            }
        }
        None
    }

    #[test]
    fn sidecar_socket_unlink_then_reconnect() {
        let service = format!("socket-gone-{}", std::process::id());

        let mut pub_ =
            FdSidecarPublisher::<iceoryx2::service::ipc::Service, Meta>::create(&service)
                .expect("FdSidecarPublisher::create failed");
        let mut sub_ =
            FdSidecarSubscriber::<iceoryx2::service::ipc::Service, Meta>::create(&service)
                .expect("FdSidecarSubscriber::create failed");

        std::thread::sleep(Duration::from_millis(SETTLE_MS));

        // Phase 1: frames 1–3 succeed.
        for seq in 1..=3u64 {
            let fd = make_fd();
            pub_.send(Meta { seq }, fd).expect("send failed");
            std::thread::sleep(Duration::from_millis(SETTLE_MS));
            let received = recv_timeout(&mut sub_, 500);
            assert!(received.is_some(), "expected frame {seq} from subscriber");
        }

        // Phase 2: drop the publisher (sidecar connections closed; socket file
        // is also removed by FdSidecarPublisher's ScmRightsPublisher Drop impl).
        drop(pub_);
        std::thread::sleep(Duration::from_millis(SETTLE_MS));

        // Phase 3: subscriber detects degraded sidecar.  The next poll should
        // see Ok(None) (no iceoryx2 sample) or an IO error.  Either is valid.
        let poll_result = sub_.recv();
        let degraded = matches!(
            &poll_result,
            Ok(None)
                | Err(FdSidecarError::SideChannelIo(_))
                | Err(FdSidecarError::NoFdInMessage)
                | Err(FdSidecarError::IceoryxReceive(_))
        );
        assert!(
            degraded,
            "expected degraded result after publisher drop, got {poll_result:?}",
        );

        // Phase 4: new publisher re-binds the same service (new socket).
        let mut pub2 =
            FdSidecarPublisher::<iceoryx2::service::ipc::Service, Meta>::create(&service)
                .expect("FdSidecarPublisher2::create failed");

        std::thread::sleep(Duration::from_millis(RECONNECT_SETTLE_MS));

        // Phase 5: subscriber reconnects to the new socket.
        sub_.reconnect().expect("reconnect failed");
        std::thread::sleep(Duration::from_millis(SETTLE_MS));

        // Phase 6: frame 4 must be received successfully.
        let fd4 = make_fd();
        pub2.send(Meta { seq: 4 }, fd4)
            .expect("send frame 4 failed");
        std::thread::sleep(Duration::from_millis(SETTLE_MS));

        let received4 = recv_timeout(&mut sub_, 500);
        assert!(
            received4.is_some(),
            "subscriber did not receive frame 4 after reconnect",
        );
        let (meta4, _fd) = received4.expect("Some guaranteed by assert above");
        assert_eq!(meta4.seq, 4, "frame 4 meta.seq mismatch");
    }
}
