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

use iceoryx2::prelude::ZeroCopySend;
use std::time::Duration;

#[allow(dead_code)]
pub const TIMEOUT_DURATION: Duration = Duration::from_secs(5);
pub const HISTORY_SIZE: usize = 10;

pub trait Config {
    type PayloadType: Send + Sync + core::fmt::Debug + PartialEq + ZeroCopySend + 'static;

    fn ping_service_name(&self) -> &str;
    fn pong_service_name(&self) -> &str;
    #[allow(dead_code)]
    fn payload(&self) -> Self::PayloadType;
}

pub struct PrimitiveType;

impl Config for PrimitiveType {
    type PayloadType = u64;

    fn ping_service_name(&self) -> &str {
        "tunnel-end-to-end-test/ping"
    }

    fn pong_service_name(&self) -> &str {
        "tunnel-end-to-end-test/pong"
    }

    fn payload(&self) -> Self::PayloadType {
        42
    }
}
