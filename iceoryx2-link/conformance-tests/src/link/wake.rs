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
pub mod link_wake {
    use core::time::Duration;

    use iceoryx2::service::Service;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_link_backend::Reactive;

    use crate::fixture::LinkFixture;

    use crate::parameters::AnyService;
    use crate::testing::{retry, side};

    const TIMEOUT: Duration = Duration::from_secs(10);

    #[conformance_test]
    pub fn a_link_is_woken_by_the_opposing_side<S: Service, X: AnyService, F: LinkFixture<S>>()
    where
        F::Backend: Reactive,
    {
        let mut fixture = F::new();

        // === SETUP ===
        // Two links over one mechanism, side B waiting on its listener.
        let mut a = side::<S, _>(|config| fixture.backend(config));
        let mut b = side::<S, _>(|config| fixture.backend(config));
        let listener = b.link.listener().expect("the listener is created");
        let mut woken = false;
        listener
            .try_wait(|_| woken = true)
            .expect("waiting succeeds");
        assert_that!(woken, eq false);

        // === OFFER ===
        // Side A bridges a service its application created, which reaches
        // the mechanism and wakes side B.
        let _service = X::create::<S>(&a.node, &X::service_name());

        retry(
            || {
                a.link.discover().expect("discovery succeeds");
                let mut woken = false;
                listener
                    .try_wait(|_| woken = true)
                    .expect("waiting succeeds");
                match woken {
                    true => Ok(()),
                    false => Err("side B was not woken"),
                }
            },
            TIMEOUT,
        )
        .expect("the opposing side wakes the link");
    }
}
