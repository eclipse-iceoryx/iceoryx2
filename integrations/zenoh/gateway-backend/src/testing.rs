// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use core::time::Duration;

use zenoh::Config;
use zenoh::Wait;

/// Poll interval while waiting for the probe publisher to match.
const POLL_PERIOD: Duration = Duration::from_millis(10);

pub struct Testing;

impl iceoryx2_gateway_backend::traits::testing::Testing for Testing {
    /// Waits until the subscriber declarations made by gateways for the given
    /// service have propagated through the zenoh mesh.
    fn sync(id: String, timeout: Duration) -> bool {
        let start_time = std::time::Instant::now();

        let session = zenoh::open(Config::default()).wait().unwrap();
        let publisher = session
            .declare_publisher(format!("iox2/*/{id}/**"))
            .wait()
            .unwrap();

        loop {
            if publisher.matching_status().wait().unwrap().matching() {
                return true;
            }
            if start_time.elapsed() >= timeout {
                return false;
            }
            std::thread::sleep(POLL_PERIOD);
        }
    }
}
