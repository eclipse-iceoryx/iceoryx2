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

//! Proptest roundtrip: random fd-payload sizes 1 B – 16 MiB.
//!
//! For each generated `size`:
//! 1. `memfd_create` a region of `size` bytes.
//! 2. Write `size` bytes of pattern `0xAB`.
//! 3. Publish via `FdSidecarPublisher`.
//! 4. Receive via `FdSidecarSubscriber`.
//! 5. `mmap` the received fd and assert all bytes match.
//!
//! Limited to 20 cases to keep CI time bounded.
//!
//! Linux-only; compiled out on non-Linux targets.

#[cfg(target_os = "linux")]
mod linux_tests {
    use iceoryx2::prelude::ZeroCopySend;
    use iceoryx2_dmabuf::{FdSidecarPublisher, FdSidecarSubscriber};
    use proptest::prelude::*;
    use rustix::fs::{MemfdFlags, memfd_create};
    use std::io::Write as _;
    use std::os::fd::{AsFd as _, AsRawFd as _, FromRawFd as _};
    use std::time::Duration;

    const MAX_SIZE: u32 = 16 * 1024 * 1024; // 16 MiB
    const SETTLE_MS: u64 = 20;
    const RECV_TIMEOUT_MS: u64 = 500;
    const POLL_MS: u64 = 10;
    const PATTERN_BYTE: u8 = 0xAB;

    /// Counter for unique service names across proptest cases.
    static CASE_ID: iceoryx2_pal_concurrency_sync::atomic::AtomicU64 =
        iceoryx2_pal_concurrency_sync::atomic::AtomicU64::new(0);

    #[derive(Debug, Clone, Copy, ZeroCopySend)]
    #[repr(C)]
    struct Meta {
        size: u64,
    }

    /// Create a `memfd_create` fd containing `size` bytes of `PATTERN_BYTE`.
    fn make_payload_fd(size: usize) -> std::os::fd::OwnedFd {
        let fd = memfd_create("prop-roundtrip", MemfdFlags::CLOEXEC).expect("memfd_create failed");
        {
            let raw = fd.as_fd().as_raw_fd();
            // SAFETY: fd is valid and owned; ManuallyDrop prevents double-close.
            let mut file = std::mem::ManuallyDrop::new(unsafe { std::fs::File::from_raw_fd(raw) });
            let payload = vec![PATTERN_BYTE; size];
            file.write_all(&payload).expect("write_all failed");
        }
        fd
    }

    /// Run a single publisher→subscriber roundtrip for `size` bytes.
    fn one_roundtrip(size: u32) -> Result<(), TestCaseError> {
        let case = CASE_ID.fetch_add(1, iceoryx2_pal_concurrency_sync::atomic::Ordering::Relaxed);
        let service = format!("prop-roundtrip-{}-{}", std::process::id(), case);
        let size = size as usize;

        let mut pub_ =
            FdSidecarPublisher::<iceoryx2::service::ipc::Service, Meta>::create(&service)
                .map_err(|e| TestCaseError::fail(format!("publisher create: {e:?}")))?;
        let mut sub_ =
            FdSidecarSubscriber::<iceoryx2::service::ipc::Service, Meta>::create(&service)
                .map_err(|e| TestCaseError::fail(format!("subscriber create: {e:?}")))?;

        // Allow subscriber to connect.
        std::thread::sleep(Duration::from_millis(SETTLE_MS));

        let fd = make_payload_fd(size);
        pub_.send(Meta { size: size as u64 }, fd)
            .map_err(|e| TestCaseError::fail(format!("send: {e:?}")))?;

        std::thread::sleep(Duration::from_millis(SETTLE_MS));

        // Poll until we receive the frame.
        let deadline = std::time::Instant::now() + Duration::from_millis(RECV_TIMEOUT_MS);
        let (meta, owned_fd) = loop {
            if std::time::Instant::now() >= deadline {
                return Err(TestCaseError::fail("recv timeout"));
            }
            match sub_
                .recv()
                .map_err(|e| TestCaseError::fail(format!("recv: {e:?}")))?
            {
                Some(pair) => break pair,
                None => {
                    std::thread::sleep(Duration::from_millis(POLL_MS));
                }
            }
        };

        prop_assert_eq!(meta.size as usize, size, "meta.size mismatch");

        // mmap the received fd and verify every byte.
        // SAFETY: fd received via SCM_RIGHTS; publisher no longer writes to it.
        let mmap = unsafe { memmap2::MmapOptions::new().map(&owned_fd) }
            .map_err(|e| TestCaseError::fail(format!("mmap: {e}")))?;

        prop_assert_eq!(mmap.len(), size, "mmap length mismatch");
        prop_assert!(
            mmap.iter().all(|&b| b == PATTERN_BYTE),
            "byte mismatch in received fd (size={size})",
        );

        Ok(())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn roundtrip_random_size(size in 1_u32..=MAX_SIZE) {
            one_roundtrip(size)?;
        }
    }
}
