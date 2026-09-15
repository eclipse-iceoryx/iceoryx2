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

mod remote_endpoints;
mod wire_form;
pub(crate) use remote_endpoints::RemoteMessageEndpoints;
use remote_endpoints::endpoint_for;
pub(crate) use wire_form::{Payload, WireForm};

use core::marker::PhantomData;

use iceoryx2::config::Config;
use iceoryx2::service::local::Service as Local;
use iceoryx2::testing::generate_service_name;
use iceoryx2_link_conformance_tests::fixture::AdapterFixture;
use iceoryx2_link_conformance_tests::parameters::FixedSizePayload;
use iceoryx2_link_conformance_tests::testing::describe;
use iceoryx2_link_testing::{FakeAdapter, FakeMiddleware};

/// A fake middleware whose data `T` translates, and its adapters
/// and remote endpoints.
pub(crate) struct FakeMiddlewareFixture<T> {
    pub(crate) middleware: FakeMiddleware,
    _translator: PhantomData<T>,
}

impl<T: WireForm> AdapterFixture for FakeMiddlewareFixture<T> {
    type Adapter = FakeAdapter;
    type RemoteEndpoints = RemoteMessageEndpoints;

    fn new() -> Self {
        Self {
            middleware: FakeMiddleware::new(),
            _translator: PhantomData,
        }
    }

    fn adapter(&mut self) -> Self::Adapter {
        self.middleware.adapter()
    }

    fn remote_endpoints(&mut self) -> Self::RemoteEndpoints {
        let description = endpoint_for::<T>(&describe::<Local, FixedSizePayload<Payload>, ()>(
            &generate_service_name(),
            &Config::default(),
        ));
        let remote = self.middleware.remote_endpoints(&description);
        RemoteMessageEndpoints {
            description,
            remote,
        }
    }
}
