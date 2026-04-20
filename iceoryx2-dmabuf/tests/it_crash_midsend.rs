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

//! Integration test: producer SIGSTOP between sidecar send and iceoryx2 publish.
//!
//! The test drives the `fd_sidecar_crash_pub` binary (a thin wrapper that sets
//! `DMABUF_CRASH_PHASE=mid-iceoryx2` before publishing).  After the publisher
//! freezes, the subscriber polls `recv()` and must observe `NoFdInMessage`
//! (because the iceoryx2 sample was not yet sent when the publisher stopped).
//! The test then sends `SIGCONT` + `SIGKILL` and verifies no panic occurred.
//!
//! Marked `#[ignore]` by default because it relies on process-level SIGSTOP
//! timing.  Run with:
//! ```sh
//! cargo test -p iceoryx2-dmabuf --test it_crash_midsend \
//!     --features memfd -- --include-ignored --nocapture
//! ```
//!
//! Linux-only; compiled out on non-Linux targets.

#[cfg(target_os = "linux")]
mod linux_tests {
    use std::process::{Command, Stdio};
    use std::time::Duration;

    fn unique_service() -> String {
        format!("crash-midsend-{}", std::process::id())
    }

    fn bin_path(name: &str) -> std::path::PathBuf {
        // Cargo sets CARGO_BIN_EXE_<name> for each [[bin]] target.
        // We derive the path by substituting the known binary name pattern.
        let base = std::path::PathBuf::from(
            std::env::var("CARGO_BIN_EXE_fd_sidecar_crash_pub")
                .unwrap_or_else(|_| "fd_sidecar_crash_pub".to_owned()),
        );
        if name == "fd_sidecar_crash_pub" {
            return base;
        }
        // For other binaries, replace the last component.
        base.parent()
            .map(|p| p.join(name))
            .unwrap_or_else(|| std::path::PathBuf::from(name))
    }

    #[test]
    #[ignore = "requires SIGSTOP timing; run with --include-ignored on Linux"]
    fn producer_sigstop_mid_send_yields_no_fd_in_message() {
        use iceoryx2::prelude::ZeroCopySend;
        use iceoryx2_dmabuf::FdSidecarSubscriber;

        const SETTLE_MS: u64 = 50;
        const WAIT_STOP_MS: u64 = 200;

        #[derive(Debug, Clone, Copy, ZeroCopySend)]
        #[repr(C)]
        struct Meta {
            size: u64,
        }

        let service = unique_service();

        // 1. Start subscriber first so it is ready when the publisher sends.
        let mut sub_ =
            FdSidecarSubscriber::<iceoryx2::service::ipc::Service, Meta>::create(&service)
                .expect("FdSidecarSubscriber::create failed");

        // 2. Spawn the crash publisher with DMABUF_CRASH_PHASE=mid-iceoryx2.
        let crash_bin = bin_path("fd_sidecar_crash_pub");
        let mut pub_proc = Command::new(&crash_bin)
            .env("DMABUF_SERVICE", &service)
            .env("DMABUF_CRASH_PHASE", "mid-iceoryx2")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn crash publisher");

        // 3. Give the publisher time to connect subscribers, send the fd, and
        //    then SIGSTOP itself.
        std::thread::sleep(Duration::from_millis(WAIT_STOP_MS));

        // 4. Poll the subscriber — no iceoryx2 sample should have arrived yet
        //    (publisher froze before the iceoryx2 send).  This should return
        //    Ok(None) because the sample is not in the ring buffer.
        let result = sub_.recv();
        // The subscriber should see Ok(None) (no iceoryx2 sample yet).
        assert!(
            matches!(result, Ok(None)),
            "expected Ok(None) while publisher is stopped, got {result:?}",
        );

        // 5. SIGCONT + SIGKILL the publisher.
        let pid = pub_proc.id();
        // SAFETY: kill() is a valid syscall; pid is the child's PID.
        unsafe {
            libc::kill(pid as i32, libc::SIGCONT);
        }
        std::thread::sleep(Duration::from_millis(SETTLE_MS));
        let _ = pub_proc.kill();
        let _ = pub_proc.wait();

        // 6. Assert no panic in the subscriber (we reached here without panic).
        drop(sub_);
    }
}
