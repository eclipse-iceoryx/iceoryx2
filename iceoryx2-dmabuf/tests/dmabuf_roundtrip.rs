// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

//! End-to-end roundtrip test for `FdSidecarPublisher::send` + `FdSidecarSubscriber::recv`.
//!
//! * **Linux** (`cfg(target_os = "linux")`): in-process test using
//!   `memfd_create`; publishes 4096 magic bytes, subscriber receives and
//!   `mmap`s them, asserts bytes match.
//! * **non-Linux**: a portable test that asserts `FdSidecarPublisher::create`
//!   returns `FdSidecarError::UnsupportedPlatform`.

// ── macOS (non-Linux) test ────────────────────────────────────────────────────
//
// This test is compiled and run on macOS.  It verifies the graceful
// `UnsupportedPlatform` error without any Linux-specific syscalls.
#[cfg(not(target_os = "linux"))]
mod non_linux_test {
    use iceoryx2::prelude::ZeroCopySend;

    #[derive(Debug, Clone, Copy, ZeroCopySend)]
    #[repr(C)]
    struct TestMeta {
        seq: u64,
    }

    #[test]
    fn create_returns_unsupported_platform_on_non_linux() {
        let result =
            iceoryx2_dmabuf::FdSidecarPublisher::<iceoryx2::service::ipc::Service, TestMeta>::create(
                "dmabuf-roundtrip-macos-test",
            );

        let is_unsupported = matches!(
            result,
            Err(iceoryx2_dmabuf::FdSidecarError::UnsupportedPlatform)
        );
        assert!(is_unsupported, "expected UnsupportedPlatform on non-Linux");
    }
}

// ── Linux in-process roundtrip ─────────────────────────────────────────────────
#[cfg(target_os = "linux")]
mod linux_tests {
    use iceoryx2::prelude::ZeroCopySend;
    use iceoryx2_dmabuf::{FdSidecarPublisher, FdSidecarSubscriber};
    use std::io::Write as _;
    use std::os::fd::AsRawFd as _;

    /// Simple metadata struct carried in the iceoryx2 user payload.
    #[derive(Debug, Clone, Copy, ZeroCopySend)]
    #[repr(C)]
    struct TestMeta {
        seq: u64,
    }

    const SERVICE_NAME: &str = "dmabuf-roundtrip-test";
    const MAGIC: u8 = 0xAB;
    const SIZE: usize = 4096;
    const SETTLE_MS: u64 = 20;

    /// Create a `memfd_create`'d fd containing `SIZE` bytes of `MAGIC`.
    fn make_memfd() -> std::os::fd::OwnedFd {
        use rustix::fs::{MemfdFlags, memfd_create};
        use std::os::fd::{AsFd as _, FromRawFd as _};

        let fd = memfd_create("test-frame", MemfdFlags::CLOEXEC).expect("memfd_create failed");

        // Write SIZE bytes via a borrowed File view, using ManuallyDrop to
        // avoid a double-close when the temporary File is dropped.
        let raw = fd.as_fd().as_raw_fd();
        // SAFETY: the fd is valid and owned by this function; ManuallyDrop
        // prevents double-close.
        let mut file = std::mem::ManuallyDrop::new(unsafe { std::fs::File::from_raw_fd(raw) });
        let payload = vec![MAGIC; SIZE];
        file.write_all(&payload).expect("write_all failed");

        fd
    }

    struct SocketGuard(String);
    impl Drop for SocketGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    #[test]
    fn send_recv_one_frame_in_process() {
        // Remove the side-channel socket on drop so parallel test runs don't
        // collide on a stale socket file.
        let _guard = SocketGuard(iceoryx2_dmabuf::uds_path_for_service(SERVICE_NAME));

        let mut pub_ =
            FdSidecarPublisher::<iceoryx2::service::ipc::Service, TestMeta>::create(SERVICE_NAME)
                .expect("FdSidecarPublisher::create failed");

        let mut sub_ =
            FdSidecarSubscriber::<iceoryx2::service::ipc::Service, TestMeta>::create(SERVICE_NAME)
                .expect("FdSidecarSubscriber::create failed");

        // Allow the subscriber's UDS stream to connect to the publisher socket.
        std::thread::sleep(std::time::Duration::from_millis(SETTLE_MS));

        let fd = make_memfd();
        pub_.send(TestMeta { seq: 1 }, fd).expect("send failed");

        // Give the iceoryx2 sample a moment to land in the subscriber queue.
        std::thread::sleep(std::time::Duration::from_millis(SETTLE_MS));

        let received = sub_.recv().expect("recv failed");
        assert!(received.is_some(), "expected Some from recv, got None");

        let (meta, owned_fd) = received.unwrap(); // test-only unwrap
        assert_eq!(meta.seq, 1, "meta.seq mismatch");

        // mmap and verify bytes.
        // SAFETY: fd received via SCM_RIGHTS; publisher no longer writes to it.
        let mmap = unsafe { memmap2::MmapOptions::new().map(&owned_fd) }
            .expect("mmap of received fd failed");
        assert_eq!(mmap.len(), SIZE, "mmap length mismatch");
        assert!(
            mmap.iter().all(|&b| b == MAGIC),
            "received bytes do not match published magic"
        );
    }
}
