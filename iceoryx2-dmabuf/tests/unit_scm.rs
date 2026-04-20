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

use iceoryx2::port::side_channel::Role;
use iceoryx2_dmabuf::scm::ScmRightsPublisher;

#[cfg(target_os = "linux")]
#[test]
fn publisher_opens_and_drops() {
    let result = ScmRightsPublisher::open("test-service-open-drop", Role::Publisher);
    assert!(result.is_ok(), "expected Ok, got {result:?}");
    drop(result.ok());
    // Socket file cleaned up on drop
    let path = iceoryx2_dmabuf::uds_path_for_service("test-service-open-drop");
    assert!(!std::path::Path::new(&path).exists());
}

// Compiled and run on macOS; compiled out on Linux.
#[cfg(not(target_os = "linux"))]
#[test]
fn non_linux_returns_unsupported_platform() {
    let result = ScmRightsPublisher::open("test-service-macos", Role::Publisher);
    assert!(
        matches!(
            result,
            Err(iceoryx2_dmabuf::FdSidecarError::UnsupportedPlatform)
        ),
        "expected UnsupportedPlatform, got {result:?}"
    );
}

/// Linux-only in-process round-trip: `send_fd` + `recv_fd_matching`.
///
/// Two threads share a publisher/subscriber pair on a unique service name.
/// The publisher sends an `OwnedFd` (an anonymous pipe read-end); the
/// subscriber receives it via `recv_fd_matching` and verifies the token matches.
#[cfg(target_os = "linux")]
#[test]
fn send_fd_recv_fd_matching_roundtrip() {
    use core::num::NonZeroU64;
    use iceoryx2_dmabuf::scm::ScmRightsSubscriber;
    use std::os::fd::{AsFd as _, OwnedFd};
    use std::time::Duration;

    const SERVICE: &str = "unit-scm-roundtrip";
    const TOKEN: u64 = 42;

    // Open publisher (binds socket, spawns accept thread).
    let pub_ = ScmRightsPublisher::open(SERVICE, Role::Publisher).expect("publisher open failed");

    // Give the accept thread a moment to start listening.
    std::thread::sleep(Duration::from_millis(5));

    // Open subscriber (connects to the publisher socket).
    let mut sub =
        ScmRightsSubscriber::open(SERVICE, Role::Subscriber).expect("subscriber open failed");

    // Wait for the accept thread to register the connection.
    std::thread::sleep(Duration::from_millis(20));

    // Create a pipe; we send the read-end as our "fd".
    // SAFETY: pipe2 is a standard Linux syscall.
    let (pipe_read, pipe_write): (OwnedFd, OwnedFd) = {
        use std::os::fd::FromRawFd as _;
        let mut fds = [0i32; 2];
        let rc = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
        assert_eq!(rc, 0, "pipe2 failed");
        unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) }
    };

    let token = NonZeroU64::new(TOKEN).expect("token must be non-zero");

    // Send the fd with the token (via inherent method).
    pub_.send_fd_impl(token, pipe_read.as_fd())
        .expect("send_fd_impl failed");

    // Receive the fd with the matching token (via inherent method).
    let received = sub
        .recv_fd_matching_impl(token, Duration::from_millis(500))
        .expect("recv_fd_matching_impl failed");

    // Drop the original pipe ends; the received fd is an independent kernel ref.
    drop(pipe_read);
    drop(pipe_write);
    drop(received);

    // Clean up the publisher.
    drop(pub_);
    let path = iceoryx2_dmabuf::uds_path_for_service(SERVICE);
    assert!(
        !std::path::Path::new(&path).exists(),
        "socket not cleaned up"
    );
}
