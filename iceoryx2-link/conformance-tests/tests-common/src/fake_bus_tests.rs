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

use iceoryx2::service::ipc::Service as Ipc;
use iceoryx2::service::local::Service as Local;
use iceoryx2_bb_testing::instantiate_conformance_tests;
use iceoryx2_link_conformance_tests::fixture::TunnelLinkFixture;
use iceoryx2_link_conformance_tests::parameters::{
    AnyName, Event, FixedSizePayload, PublishSubscribe,
};

use crate::fixture::FakeBusFixture;

instantiate_conformance_tests!(
    iceoryx2_link_conformance_tests::carrier_discovery,
    FakeBusFixture
);
instantiate_conformance_tests!(
    iceoryx2_link_conformance_tests::carrier_propagation,
    FakeBusFixture
);
instantiate_conformance_tests!(
    iceoryx2_link_conformance_tests::carrier_wake,
    FakeBusFixture
);

mod ipc {
    use super::*;

    mod publish_subscribe {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::tunnel_discovery,
            Ipc,
            PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
            FakeBusFixture
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::tunnel_publish_subscribe,
            Ipc,
            PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
            FakeBusFixture
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::link_discovery,
            Ipc,
            PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
            TunnelLinkFixture<FakeBusFixture>
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::link_wake,
            Ipc,
            PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
            TunnelLinkFixture<FakeBusFixture>
        );
    }

    mod event {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::tunnel_discovery,
            Ipc,
            Event<AnyName>,
            FakeBusFixture
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::tunnel_event,
            Ipc,
            Event<AnyName>,
            FakeBusFixture
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::link_discovery,
            Ipc,
            Event<AnyName>,
            TunnelLinkFixture<FakeBusFixture>
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::link_wake,
            Ipc,
            Event<AnyName>,
            TunnelLinkFixture<FakeBusFixture>
        );
    }
}

mod local {
    use super::*;

    mod publish_subscribe {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::tunnel_discovery,
            Local,
            PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
            FakeBusFixture
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::tunnel_publish_subscribe,
            Local,
            PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
            FakeBusFixture
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::link_discovery,
            Local,
            PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
            TunnelLinkFixture<FakeBusFixture>
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::link_wake,
            Local,
            PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
            TunnelLinkFixture<FakeBusFixture>
        );
    }

    mod event {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::tunnel_discovery,
            Local,
            Event<AnyName>,
            FakeBusFixture
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::tunnel_event,
            Local,
            Event<AnyName>,
            FakeBusFixture
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::link_discovery,
            Local,
            Event<AnyName>,
            TunnelLinkFixture<FakeBusFixture>
        );
        instantiate_conformance_tests!(
            iceoryx2_link_conformance_tests::link_wake,
            Local,
            Event<AnyName>,
            TunnelLinkFixture<FakeBusFixture>
        );
    }
}
