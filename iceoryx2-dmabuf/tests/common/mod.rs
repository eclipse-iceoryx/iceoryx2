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

/// Removes the UDS socket file for `service_name` on `Drop`.
/// Use in every integration test to prevent artifact leaks on failure.
pub struct TestGuard {
    socket_path: String,
}

impl TestGuard {
    pub fn new(service_name: &str) -> Self {
        let path = iceoryx2_dmabuf::uds_path_for_service(service_name);
        Self { socket_path: path }
    }
}

impl Drop for TestGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

/// Stable, PID-qualified service name for use in tests.
/// `local_id` is a short in-test disambiguator (e.g. "local-smoke").
pub fn test_service_name(local_id: &str) -> String {
    format!("{}/{}-{}", module_path!(), local_id, std::process::id())
}
