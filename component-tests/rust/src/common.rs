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
    fn run_test(
        &mut self,
        node: &iceoryx2::node::Node<iceoryx2::service::ipc::Service>,
    ) -> Result<(), Box<dyn core::error::Error>>;
}

#[derive(Debug)]
pub struct GenericTestError {}

impl core::fmt::Display for GenericTestError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Test Failed")
    }
}

impl core::error::Error for GenericTestError {}

#[derive(Debug)]
pub struct TimeoutError {}

impl core::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Communication time out")
    }
}

impl core::error::Error for TimeoutError {}

pub fn wait_for<T, F: Fn() -> Option<T>>(
    node: &iceoryx2::prelude::Node<iceoryx2::prelude::ipc::Service>,
    predicate: &F,
    timeout: core::time::Duration,
    cycle_time: core::time::Duration,
) -> Result<T, Box<dyn core::error::Error>> {
    let mut time_passed = core::time::Duration::from_millis(0);
    while time_passed < timeout {
        if node.wait(cycle_time).is_err() {
            return Err(Box::new(GenericTestError {}));
        }
        time_passed += cycle_time;
        if let Some(v) = predicate() {
            return Ok(v);
        }
    }
    Err(Box::new(TimeoutError {}))
}

pub fn wait_for_pred<F: Fn() -> bool>(
    node: &iceoryx2::prelude::Node<iceoryx2::prelude::ipc::Service>,
    predicate: &F,
    timeout: core::time::Duration,
    cycle_time: core::time::Duration,
) -> Result<(), Box<dyn core::error::Error>> {
    wait_for(
        node,
        &|| -> Option<()> {
            if predicate() {
                Some(())
            } else {
                None
            }
        },
        timeout,
        cycle_time,
    )
}

pub fn await_response<
    ServiceType: iceoryx2::service::Service,
    RequestType: core::fmt::Debug + iceoryx2::prelude::ZeroCopySend + ?Sized,
    ResponseType: core::fmt::Debug + iceoryx2::prelude::ZeroCopySend,
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
    cycle_time: core::time::Duration,
) -> Result<iceoryx2::response::Response<ServiceType, ResponseType, ()>, Box<dyn core::error::Error>>
{
    wait_for(
        node,
        &|| -> Option<iceoryx2::response::Response<ServiceType, ResponseType, ()>> {
            if pending_response.has_response() {
                pending_response.receive().ok().flatten()
            } else {
                None
            }
        },
        timeout,
        cycle_time,
    )
}
