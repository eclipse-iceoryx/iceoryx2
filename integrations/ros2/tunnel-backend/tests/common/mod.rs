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

use core::time::Duration;

use iceoryx2::prelude::*;

pub const DISCOVERY_RETRY_PERIOD: Duration = Duration::from_millis(50);
pub const DISCOVERY_RETRY_ATTEMPTS: usize = 200;

#[derive(Debug, ZeroCopySend)]
#[type_name("std_msgs/msg/String")]
#[repr(C)]
pub struct RosString(u8);

pub fn service_name(name: &str) -> ServiceName {
    name.try_into().expect("valid service name")
}
