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

//! Error-path integration tests for TokenMismatch and NoFdInMessage.
#![cfg(target_os = "linux")]
mod common;

use common::{TestGuard, test_service_name};
use iceoryx2::service::ipc;
use iceoryx2_dmabuf::{FdSidecarError, FdSidecarPublisher, FdSidecarSubscriber};
use rustix::fs::{MemfdFlags, memfd_create};
use std::os::fd::AsFd as _;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Meta {
    v: u64,
}
unsafe impl iceoryx2::prelude::ZeroCopySend for Meta {}

/// Injects a forged sidecar message with token=9999 to trigger TokenMismatch.
///
/// After a legit round-trip (token=1 consumed), the publisher injects
/// token=9999 directly to the subscriber's server-side stream via
/// `inject_raw_for_test`. The next `pub_.send()` produces an iceoryx2
/// sample with token=2. When `sub_.recv()` reads the iceoryx2 sample
/// (expecting token=2) and polls the UDS stream, it finds token=9999 (>2)
/// and returns `TokenMismatch`.
#[test]
fn token_mismatch_on_forged_message() {
    let svc = test_service_name("token-mismatch");
    let _guard = TestGuard::new(&svc);

    let mut pub_ = FdSidecarPublisher::<ipc::Service, Meta>::create(&svc).unwrap();
    let mut sub_ = FdSidecarSubscriber::<ipc::Service, Meta>::create(&svc).unwrap();

    // Wait for the subscriber's UDS stream to be accepted by the publisher.
    std::thread::sleep(std::time::Duration::from_millis(20));

    // Send and consume a legitimate frame (token=1).
    let fd1 = memfd_create("tok-mismatch-1", MemfdFlags::CLOEXEC).unwrap();
    pub_.send(Meta { v: 1 }, fd1).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));
    while let Ok(Some(_)) = sub_.recv() {}

    // Inject forged token=9999 directly to the subscriber's UDS stream.
    let junk_fd = memfd_create("tok-mismatch-junk", MemfdFlags::CLOEXEC).unwrap();
    pub_.inject_raw_for_test(9999u64, junk_fd.as_fd()).unwrap();

    // Send a second legitimate frame (token=2). The subscriber's UDS queue is
    // now: [token=9999 (forged), token=2 (legit)].
    // recv_fd_matching(expected=2) reads token=9999 first → TokenMismatch.
    let fd2 = memfd_create("tok-mismatch-2", MemfdFlags::CLOEXEC).unwrap();
    pub_.send(Meta { v: 2 }, fd2).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));

    let err = sub_.recv();
    assert!(
        matches!(err, Err(FdSidecarError::TokenMismatch { .. })),
        "expected Err(TokenMismatch {{ .. }}), got {err:?}"
    );
}

/// Expects NoFdInMessage when an iceoryx2 metadata sample is queued but no
/// matching fd is ever delivered on the UDS sidecar.
///
/// Subscriber connects first, then `send_metadata_only_for_test` publishes the
/// iceoryx2 sample while deliberately skipping the `SCM_RIGHTS` fd delivery.
/// `sub_.recv()` dequeues the sample (token=1), calls `recv_fd_matching_impl`
/// with a 50 ms timeout, finds no fd, and returns `NoFdInMessage`.
///
/// This approach is deterministic: the subscriber is already registered before
/// the sample is enqueued, so iceoryx2 never silently drops the sample
/// (which it would if the subscriber connected after a publish without history
/// enabled on the service).
#[test]
fn no_fd_in_message_after_timeout() {
    let svc = test_service_name("no-fd-timeout");
    let _guard = TestGuard::new(&svc);

    // Subscriber connects FIRST so iceoryx2 will deliver the subsequent sample.
    let mut pub_ = FdSidecarPublisher::<ipc::Service, Meta>::create(&svc).unwrap();
    let mut sub_ = FdSidecarSubscriber::<ipc::Service, Meta>::create(&svc).unwrap();

    // Wait for the subscriber's UDS stream to be accepted by the publisher.
    std::thread::sleep(std::time::Duration::from_millis(20));

    // Publish the iceoryx2 sample WITHOUT sending the fd on the sidecar.
    // The subscriber will find the sample in its queue but no matching fd.
    pub_.send_metadata_only_for_test(Meta { v: 1 }).unwrap();

    let start = std::time::Instant::now();
    let result = sub_.recv();
    let elapsed = start.elapsed();
    assert!(
        matches!(result, Err(FdSidecarError::NoFdInMessage)),
        "expected Err(NoFdInMessage), got {result:?}"
    );
    assert!(
        elapsed >= std::time::Duration::from_millis(40),
        "timeout too short: {elapsed:?}"
    );
}
