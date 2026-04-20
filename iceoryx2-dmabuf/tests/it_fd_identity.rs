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

//! Integration test: fd identity across a two-process publisher/subscriber.
//!
//! The test spawns the `fd_sidecar_fd_identity` binary in both "pub" and "sub"
//! roles.  The publisher prints its `(st_dev, st_ino)` before publishing; the
//! subscriber prints its `(st_dev, st_ino)` after receiving the fd.  The
//! parent process asserts the pairs are equal, proving that `SCM_RIGHTS`
//! transmitted the exact same kernel inode — not a copy.
//!
//! Linux-only: the entire module is compiled out on non-Linux targets.

#[cfg(target_os = "linux")]
mod linux_tests {
    use std::process::Command;
    use std::time::Duration;

    // Service name salted with the process ID to avoid collisions when tests
    // run in parallel or when a previous run left stale sockets.
    fn unique_service() -> String {
        format!("fd-identity-{}", std::process::id())
    }

    /// Path to the `fd_sidecar_fd_identity` binary built alongside the tests.
    fn bin_path() -> std::path::PathBuf {
        // `CARGO_BIN_EXE_fd_sidecar_fd_identity` is set by Cargo when the test is
        // run via `cargo test`.
        let exe = env!("CARGO_BIN_EXE_fd_sidecar_fd_identity");
        std::path::PathBuf::from(exe)
    }

    /// Parse a `KEY:val1:val2` line from process stdout.
    fn parse_stat_line(output: &str, key: &str) -> Option<(u64, u64)> {
        for line in output.lines() {
            if let Some(rest) = line.strip_prefix(key) {
                let mut parts = rest.splitn(2, ':');
                let dev: u64 = parts.next()?.parse().ok()?;
                let ino: u64 = parts.next()?.parse().ok()?;
                return Some((dev, ino));
            }
        }
        None
    }

    #[test]
    fn fd_identity_pub_sub_same_inode() {
        const WAIT_PUB_MS: u64 = 50;

        let service = unique_service();
        let bin = bin_path();

        // Start publisher first.
        let pub_proc = Command::new(&bin)
            .env("DMABUF_ROLE", "pub")
            .env("DMABUF_SERVICE", &service)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("spawn publisher");

        // Give the publisher time to bind the socket and print its stat line
        // before the subscriber tries to connect.
        std::thread::sleep(Duration::from_millis(WAIT_PUB_MS));

        // Start subscriber.
        let sub_out = Command::new(&bin)
            .env("DMABUF_ROLE", "sub")
            .env("DMABUF_SERVICE", &service)
            .output()
            .expect("run subscriber");

        // Wait for publisher to finish.
        let pub_out = pub_proc.wait_with_output().expect("wait publisher");

        let pub_stdout = String::from_utf8_lossy(&pub_out.stdout);
        let sub_stdout = String::from_utf8_lossy(&sub_out.stdout);

        assert!(
            pub_out.status.success(),
            "publisher exited non-zero:\nstdout: {pub_stdout}\nstderr: {}",
            String::from_utf8_lossy(&pub_out.stderr),
        );
        assert!(
            sub_out.status.success(),
            "subscriber exited non-zero:\nstdout: {sub_stdout}\nstderr: {}",
            String::from_utf8_lossy(&sub_out.stderr),
        );

        let pub_stat = parse_stat_line(&pub_stdout, "PUB_STAT:")
            .expect("publisher did not print PUB_STAT line");
        let sub_stat = parse_stat_line(&sub_stdout, "SUB_STAT:")
            .expect("subscriber did not print SUB_STAT line");

        assert_eq!(
            pub_stat, sub_stat,
            "inode mismatch: publisher stat={pub_stat:?}, subscriber stat={sub_stat:?}",
        );
    }
}
