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

use iceoryx2_bb_testing::instantiate_conformance_tests;

use iceoryx2::service::ipc::Service as Ipc;
use iceoryx2::service::local::Service as Local;
use iceoryx2_tunnel_zenoh::testing;
use iceoryx2_tunnel_zenoh::ZenohBackend;

mod ipc {
    use super::*;
    instantiate_conformance_tests!(
        iceoryx2_tunnel_conformance_tests::event_propagation,
        super::Ipc,
        super::ZenohBackend<super::Ipc>,
        super::testing::Testing
    );
}

mod local {
    use super::*;
    instantiate_conformance_tests!(
        iceoryx2_tunnel_conformance_tests::event_propagation,
        super::Local,
        super::ZenohBackend<super::Local>,
        super::testing::Testing
    );
}
