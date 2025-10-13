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

pub trait ComponentTest {
    fn test_name(&self) -> &'static str;
    fn run_test(&mut self, node: &iceoryx2::node::Node<iceoryx2::service::ipc::Service>) -> Result<(), Box<dyn core::error::Error>>;
}

#[derive(Debug)]
pub struct GenericTestError {
}

impl std::fmt::Display for GenericTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Test Failed")
    }
}

impl core::error::Error for GenericTestError {
}

pub fn await_response<
    ServiceType: iceoryx2::service::Service,
    RequestType: std::fmt::Debug + iceoryx2::prelude::ZeroCopySend + ?Sized,
    ResponseType: std::fmt::Debug + iceoryx2::prelude::ZeroCopySend,
>(
    node: &iceoryx2::prelude::Node<iceoryx2::prelude::ipc::Service>,
    pending_response: &iceoryx2::pending_response::PendingResponse<
        ServiceType,
        RequestType,
        (),
        ResponseType,
        (),
    >,
    timeout: core::time::Duration,
    cycle_time: core::time::Duration
) -> Option<iceoryx2::response::Response<ServiceType, ResponseType, ()>> {
    let mut time_passed = core::time::Duration::from_millis(0);
    while time_passed < timeout {
        if node.wait(cycle_time).is_err() {
            break;
        }
        time_passed += cycle_time;
        if pending_response.has_response() {
            return pending_response.receive().ok().flatten();
        }
    }
    None
}
