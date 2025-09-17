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

use std::time::Duration;

#[allow(dead_code)]
pub const TIMEOUT_DURATION: Duration = Duration::from_secs(5);
pub const HISTORY_SIZE: usize = 10;
pub const PING_SERVICE_NAME: &str = "tunnel-end-to-end-test/ping";
pub const PONG_SERVICE_NAME: &str = "tunnel-end-to-end-test/pong";
