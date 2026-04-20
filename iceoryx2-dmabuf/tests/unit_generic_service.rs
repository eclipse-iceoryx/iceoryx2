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

#![cfg(target_os = "linux")]
mod common;

use common::{TestGuard, test_service_name};
use iceoryx2::service::local;
use iceoryx2_dmabuf::{FdSidecarPublisher, FdSidecarSubscriber};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Meta {
    v: u64,
}
unsafe impl iceoryx2::prelude::ZeroCopySend for Meta {}

#[test]
fn local_service_smoke() {
    use rustix::fs::{MemfdFlags, memfd_create};

    let svc = test_service_name("local-smoke");
    let _guard = TestGuard::new(&svc);

    let fd = memfd_create("local-test", MemfdFlags::CLOEXEC).unwrap();

    let mut pub_ = FdSidecarPublisher::<local::Service, Meta>::create(&svc).unwrap();
    let mut sub_ = FdSidecarSubscriber::<local::Service, Meta>::create(&svc).unwrap();

    // Allow the subscriber's UDS stream to connect to the publisher socket.
    std::thread::sleep(std::time::Duration::from_millis(20));

    pub_.send(Meta { v: 99 }, fd).unwrap();

    // Allow the iceoryx2 sample to land in the subscriber queue.
    std::thread::sleep(std::time::Duration::from_millis(20));

    let result = sub_.recv().unwrap();
    assert!(result.is_some(), "expected Some from recv, got None");
    let (meta, _fd) = result.unwrap();
    assert_eq!(meta.v, 99, "meta.v mismatch");
}
