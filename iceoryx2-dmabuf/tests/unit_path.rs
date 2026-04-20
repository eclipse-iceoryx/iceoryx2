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

use iceoryx2_dmabuf::uds_path_for_service;

#[test]
fn same_service_same_path() {
    let a = uds_path_for_service("frame-plane/video/0");
    let b = uds_path_for_service("frame-plane/video/0");
    assert_eq!(a, b);
}

#[test]
fn different_service_different_path() {
    let a = uds_path_for_service("frame-plane/video/0");
    let b = uds_path_for_service("frame-plane/video/1");
    assert_ne!(a, b);
}

#[test]
fn path_is_under_tmp() {
    let p = uds_path_for_service("any-service");
    assert!(p.starts_with("/tmp/iox2-dmabuf/"));
}

#[test]
fn path_basename_is_45_chars() {
    let p = uds_path_for_service("any-service");
    let path = std::path::Path::new(&p);
    let basename = path.file_name().and_then(|n| n.to_str());
    assert!(basename.is_some(), "path must have a valid UTF-8 basename");
    let basename = basename.unwrap_or("");
    // 40 hex chars + ".sock" = 45
    assert_eq!(
        basename.len(),
        45,
        "basename must be 40 hex + '.sock'; got {basename}"
    );
}
