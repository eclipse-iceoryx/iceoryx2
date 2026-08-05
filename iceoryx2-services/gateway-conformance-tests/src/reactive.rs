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

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod reactive {
    use core::fmt::Debug;
    use core::time::Duration;

    use iceoryx2::prelude::*;
    use iceoryx2::testing::*;

    use iceoryx2::testing::generate_service_name;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_gateway::Gateway;
    use iceoryx2_gateway_backend::traits::{Backend, ReactiveBackendBuilder, testing::Testing};

    const TIMEOUT: Duration = Duration::from_millis(250);
    const MAX_ATTEMPTS: usize = 25;

    // Two hosts: A is a polled gateway that publishes; B is a reactive gateway
    // whose wake listener must fire when A's data arrives over the backend.
    #[conformance_test]
    pub fn wakes_on_publish_subscribe_data<S, B, T, W>()
    where
        S: Service,
        B: Backend<S> + Debug,
        T: Testing,
        W: Service,
        for<'b> B::Builder<'b>: ReactiveBackendBuilder<S, WakeService = W>,
    {
        let service_name = generate_service_name();

        // === Host A: polled gateway + publisher ===
        let iceoryx_config_a = generate_isolated_config();
        let mut gateway_a = Gateway::<S, B>::new()
            .iceoryx_config(iceoryx_config_a.clone())
            .polled()
            .create()
            .unwrap();

        let node_a = NodeBuilder::new()
            .config(&iceoryx_config_a)
            .create::<S>()
            .unwrap();
        let service_a = node_a
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open_or_create()
            .unwrap();
        let publisher_a = service_a.publisher_builder().create().unwrap();

        gateway_a.discover_over_iceoryx().unwrap();
        assert_that!(gateway_a.bridged_services().contains(service_a.service_hash()), eq true);

        // === Host B: reactive gateway — wake listener is what we're testing ===
        let iceoryx_config_b = generate_isolated_config();
        let (mut gateway_b, wake_listener) = Gateway::<S, B>::new()
            .iceoryx_config(iceoryx_config_b.clone())
            .reactive()
            .create::<W>()
            .unwrap();

        // Wait for B to discover A's service over the backend.
        T::retry(
            || {
                gateway_b.discover_over_backend().unwrap();
                if gateway_b
                    .bridged_services()
                    .contains(service_a.service_hash())
                {
                    Ok(())
                } else {
                    Err("backend discovery did not propagate to host B")
                }
            },
            TIMEOUT,
            Some(MAX_ATTEMPTS),
        )
        .unwrap();

        // Publish from A; the wake on B must fire because data arrives on the backend.
        T::retry(
            || {
                publisher_a.send_copy(42u64).map_err(|_| "send failed")?;
                gateway_a.propagate().map_err(|_| "propagate failed")?;
                if wake_listener.try_wait(|_| {}).unwrap() == 0 {
                    Err("wake did not fire")
                } else {
                    Ok(())
                }
            },
            TIMEOUT,
            Some(MAX_ATTEMPTS),
        )
        .unwrap();
    }

    // Two hosts: A is a polled gateway that fires an event; B is a reactive
    // gateway whose wake listener must fire when A's event arrives over the backend.
    #[conformance_test]
    pub fn wakes_on_event<S, B, T, W>()
    where
        S: Service,
        B: Backend<S> + Debug,
        T: Testing,
        W: Service,
        for<'b> B::Builder<'b>: ReactiveBackendBuilder<S, WakeService = W>,
    {
        let service_name = generate_service_name();

        // === Host A: polled gateway + notifier ===
        let iceoryx_config_a = generate_isolated_config();
        let mut gateway_a = Gateway::<S, B>::new()
            .iceoryx_config(iceoryx_config_a.clone())
            .polled()
            .create()
            .unwrap();

        let node_a = NodeBuilder::new()
            .config(&iceoryx_config_a)
            .create::<S>()
            .unwrap();
        let service_a = node_a
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        let notifier_a = service_a.notifier_builder().create().unwrap();

        gateway_a.discover_over_iceoryx().unwrap();
        assert_that!(gateway_a.bridged_services().contains(service_a.service_hash()), eq true);

        // === Host B: reactive gateway ===
        let iceoryx_config_b = generate_isolated_config();
        let (mut gateway_b, wake_listener) = Gateway::<S, B>::new()
            .iceoryx_config(iceoryx_config_b.clone())
            .reactive()
            .create::<W>()
            .unwrap();

        T::retry(
            || {
                gateway_b.discover_over_backend().unwrap();
                if gateway_b
                    .bridged_services()
                    .contains(service_a.service_hash())
                {
                    Ok(())
                } else {
                    Err("backend discovery did not propagate to host B")
                }
            },
            TIMEOUT,
            Some(MAX_ATTEMPTS),
        )
        .unwrap();

        // Notify from A; the wake on B must fire because data arrives on the backend.
        T::retry(
            || {
                notifier_a.notify().map_err(|_| "notify failed")?;
                gateway_a.propagate().map_err(|_| "propagate failed")?;
                if wake_listener.try_wait(|_| {}).unwrap() == 0 {
                    Err("wake did not fire")
                } else {
                    Ok(())
                }
            },
            TIMEOUT,
            Some(MAX_ATTEMPTS),
        )
        .unwrap();
    }
}
