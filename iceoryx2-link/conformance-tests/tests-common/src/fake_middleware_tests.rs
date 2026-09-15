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
use iceoryx2_link_adapter::Passthrough;
use iceoryx2_link_conformance_tests::fixture::GatewayLinkFixture;
use iceoryx2_link_conformance_tests::parameters::{AnyName, FixedSizePayload, PublishSubscribe};
use iceoryx2_link_testing::FakeSwapTranslator;

use crate::fixture::FakeMiddlewareFixture;

instantiate_conformance_tests!(
    iceoryx2_link_conformance_tests::adapter_discovery,
    FakeMiddlewareFixture<Passthrough>
);
instantiate_conformance_tests!(
    iceoryx2_link_conformance_tests::adapter_publish_subscribe,
    FakeMiddlewareFixture<Passthrough>
);
instantiate_conformance_tests!(
    iceoryx2_link_conformance_tests::adapter_wake,
    FakeMiddlewareFixture<Passthrough>
);

mod ipc {
    use super::*;

    mod passthrough {
        use super::*;

        mod publish_subscribe {
            use super::*;

            instantiate_conformance_tests!(
                iceoryx2_link_conformance_tests::gateway_discovery,
                Ipc,
                PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
                FakeMiddlewareFixture<Passthrough>
            );
            instantiate_conformance_tests!(
                iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                Ipc,
                PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
                FakeMiddlewareFixture<Passthrough>
            );
            instantiate_conformance_tests!(
                iceoryx2_link_conformance_tests::gateway_publish_subscribe_header,
                Ipc,
                PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
                FakeMiddlewareFixture<Passthrough>
            );
            instantiate_conformance_tests!(
                iceoryx2_link_conformance_tests::link_discovery,
                Ipc,
                PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
                GatewayLinkFixture<FakeMiddlewareFixture<Passthrough>>
            );
            instantiate_conformance_tests!(
                iceoryx2_link_conformance_tests::link_wake,
                Ipc,
                PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
                GatewayLinkFixture<FakeMiddlewareFixture<Passthrough>>
            );
        }
    }

    mod transcoded {
        use super::*;

        mod publish_subscribe {
            use super::*;

            instantiate_conformance_tests!(
                iceoryx2_link_conformance_tests::gateway_discovery,
                Ipc,
                PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
                FakeMiddlewareFixture<FakeSwapTranslator>
            );
            instantiate_conformance_tests!(
                iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                Ipc,
                PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
                FakeMiddlewareFixture<FakeSwapTranslator>
            );
            instantiate_conformance_tests!(
                iceoryx2_link_conformance_tests::gateway_publish_subscribe_header,
                Ipc,
                PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
                FakeMiddlewareFixture<FakeSwapTranslator>
            );
            instantiate_conformance_tests!(
                iceoryx2_link_conformance_tests::link_discovery,
                Ipc,
                PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
                GatewayLinkFixture<FakeMiddlewareFixture<FakeSwapTranslator>>
            );
        }
    }
}

mod local {
    use super::*;

    mod passthrough {
        use super::*;

        mod publish_subscribe {
            use super::*;

            instantiate_conformance_tests!(
                iceoryx2_link_conformance_tests::gateway_discovery,
                Local,
                PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
                FakeMiddlewareFixture<Passthrough>
            );
            instantiate_conformance_tests!(
                iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                Local,
                PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
                FakeMiddlewareFixture<Passthrough>
            );
            instantiate_conformance_tests!(
                iceoryx2_link_conformance_tests::gateway_publish_subscribe_header,
                Local,
                PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
                FakeMiddlewareFixture<Passthrough>
            );
            instantiate_conformance_tests!(
                iceoryx2_link_conformance_tests::link_discovery,
                Local,
                PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
                GatewayLinkFixture<FakeMiddlewareFixture<Passthrough>>
            );
            instantiate_conformance_tests!(
                iceoryx2_link_conformance_tests::link_wake,
                Local,
                PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
                GatewayLinkFixture<FakeMiddlewareFixture<Passthrough>>
            );
        }
    }

    mod transcoded {
        use super::*;

        mod publish_subscribe {
            use super::*;

            instantiate_conformance_tests!(
                iceoryx2_link_conformance_tests::gateway_discovery,
                Local,
                PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
                FakeMiddlewareFixture<FakeSwapTranslator>
            );
            instantiate_conformance_tests!(
                iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                Local,
                PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
                FakeMiddlewareFixture<FakeSwapTranslator>
            );
            instantiate_conformance_tests!(
                iceoryx2_link_conformance_tests::gateway_publish_subscribe_header,
                Local,
                PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
                FakeMiddlewareFixture<FakeSwapTranslator>
            );
            instantiate_conformance_tests!(
                iceoryx2_link_conformance_tests::link_discovery,
                Local,
                PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
                GatewayLinkFixture<FakeMiddlewareFixture<FakeSwapTranslator>>
            );
        }
    }
}
