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

pub struct Testing;

impl iceoryx2_tunnel_backend::traits::testing::Testing for Testing {
    fn sync(id: String, timeout: Duration) -> bool {
        let start_time = std::time::Instant::now();

        let config = Config::default();
        let session = zenoh::open(config.clone()).wait().unwrap();
        let subscriber = session.declare_subscriber(id.clone()).wait().unwrap();

        while subscriber.sender_count() == 0 {
            if start_time.elapsed() >= timeout {
                return false;
            }
        }

        true
    }
}
