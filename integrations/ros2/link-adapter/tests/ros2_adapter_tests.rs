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

use iceoryx2_bb_testing::instantiate_conformance_tests;
use iceoryx2_integrations_ros2_link_adapter::ros_header::RosHeader;
use iceoryx2_link_conformance_tests::fixture::GatewayLinkFixture;
use iceoryx2_link_conformance_tests::parameters::{FixedSizePayload, PublishSubscribe};

use crate::fixture::{
    Passthrough, PassthroughWithHeader, PlainStruct, PlainStructWithHeader, PrefixMapped,
    Ros2Fixture, SerializedString, StaticMapped, UInt64,
};

type Ipc = iceoryx2::service::ipc::Service;
type Local = iceoryx2::service::local::Service;

/// The user header of services mirrored by default.
type NoHeader = ();

instantiate_conformance_tests!(
    iceoryx2_link_conformance_tests::adapter_discovery,
    Ros2Fixture<PrefixMapped, PlainStruct>
);
instantiate_conformance_tests!(
    iceoryx2_link_conformance_tests::adapter_publish_subscribe,
    Ros2Fixture<PrefixMapped, PlainStruct>
);
instantiate_conformance_tests!(
    iceoryx2_link_conformance_tests::adapter_wake,
    Ros2Fixture<PrefixMapped, PlainStruct>
);

mod ipc {
    use super::*;

    mod plain_struct {
        use super::*;

        mod prefix_mapping {
            use super::*;

            mod publish_subscribe {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Ipc,
                    PublishSubscribe<PrefixMapped, FixedSizePayload<UInt64>, NoHeader>,
                    Ros2Fixture<PrefixMapped, PlainStruct>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                    Ipc,
                    PublishSubscribe<PrefixMapped, FixedSizePayload<UInt64>, NoHeader>,
                    Ros2Fixture<PrefixMapped, PlainStruct>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_discovery,
                    Ipc,
                    PublishSubscribe<PrefixMapped, FixedSizePayload<UInt64>, NoHeader>,
                    GatewayLinkFixture<Ros2Fixture<PrefixMapped, PlainStruct>>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_wake,
                    Ipc,
                    PublishSubscribe<PrefixMapped, FixedSizePayload<UInt64>, NoHeader>,
                    GatewayLinkFixture<Ros2Fixture<PrefixMapped, PlainStruct>>
                );
            }

            mod publish_subscribe_with_ros_header {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Ipc,
                    PublishSubscribe<PrefixMapped, FixedSizePayload<UInt64>, RosHeader>,
                    Ros2Fixture<PrefixMapped, PlainStructWithHeader>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                    Ipc,
                    PublishSubscribe<PrefixMapped, FixedSizePayload<UInt64>, RosHeader>,
                    Ros2Fixture<PrefixMapped, PlainStructWithHeader>
                );
            }
        }

        mod static_mapping {
            use super::*;

            mod publish_subscribe {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Ipc,
                    PublishSubscribe<StaticMapped, FixedSizePayload<UInt64>, NoHeader>,
                    Ros2Fixture<StaticMapped, PlainStruct>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                    Ipc,
                    PublishSubscribe<StaticMapped, FixedSizePayload<UInt64>, NoHeader>,
                    Ros2Fixture<StaticMapped, PlainStruct>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_discovery,
                    Ipc,
                    PublishSubscribe<StaticMapped, FixedSizePayload<UInt64>, NoHeader>,
                    GatewayLinkFixture<Ros2Fixture<StaticMapped, PlainStruct>>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_wake,
                    Ipc,
                    PublishSubscribe<StaticMapped, FixedSizePayload<UInt64>, NoHeader>,
                    GatewayLinkFixture<Ros2Fixture<StaticMapped, PlainStruct>>
                );
            }

            mod publish_subscribe_with_ros_header {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Ipc,
                    PublishSubscribe<StaticMapped, FixedSizePayload<UInt64>, RosHeader>,
                    Ros2Fixture<StaticMapped, PlainStructWithHeader>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                    Ipc,
                    PublishSubscribe<StaticMapped, FixedSizePayload<UInt64>, RosHeader>,
                    Ros2Fixture<StaticMapped, PlainStructWithHeader>
                );
            }
        }
    }

    mod passthrough {
        use super::*;

        mod prefix_mapping {
            use super::*;

            mod publish_subscribe {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Ipc,
                    PublishSubscribe<PrefixMapped, SerializedString, NoHeader>,
                    Ros2Fixture<PrefixMapped, Passthrough>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                    Ipc,
                    PublishSubscribe<PrefixMapped, SerializedString, NoHeader>,
                    Ros2Fixture<PrefixMapped, Passthrough>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_discovery,
                    Ipc,
                    PublishSubscribe<PrefixMapped, SerializedString, NoHeader>,
                    GatewayLinkFixture<Ros2Fixture<PrefixMapped, Passthrough>>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_wake,
                    Ipc,
                    PublishSubscribe<PrefixMapped, SerializedString, NoHeader>,
                    GatewayLinkFixture<Ros2Fixture<PrefixMapped, Passthrough>>
                );
            }

            mod publish_subscribe_with_ros_header {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Ipc,
                    PublishSubscribe<PrefixMapped, SerializedString, RosHeader>,
                    Ros2Fixture<PrefixMapped, PassthroughWithHeader>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                    Ipc,
                    PublishSubscribe<PrefixMapped, SerializedString, RosHeader>,
                    Ros2Fixture<PrefixMapped, PassthroughWithHeader>
                );
            }
        }

        mod static_mapping {
            use super::*;

            mod publish_subscribe {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Ipc,
                    PublishSubscribe<StaticMapped, SerializedString, NoHeader>,
                    Ros2Fixture<StaticMapped, Passthrough>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                    Ipc,
                    PublishSubscribe<StaticMapped, SerializedString, NoHeader>,
                    Ros2Fixture<StaticMapped, Passthrough>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_discovery,
                    Ipc,
                    PublishSubscribe<StaticMapped, SerializedString, NoHeader>,
                    GatewayLinkFixture<Ros2Fixture<StaticMapped, Passthrough>>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_wake,
                    Ipc,
                    PublishSubscribe<StaticMapped, SerializedString, NoHeader>,
                    GatewayLinkFixture<Ros2Fixture<StaticMapped, Passthrough>>
                );
            }

            mod publish_subscribe_with_ros_header {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Ipc,
                    PublishSubscribe<StaticMapped, SerializedString, RosHeader>,
                    Ros2Fixture<StaticMapped, PassthroughWithHeader>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                    Ipc,
                    PublishSubscribe<StaticMapped, SerializedString, RosHeader>,
                    Ros2Fixture<StaticMapped, PassthroughWithHeader>
                );
            }
        }
    }
}

mod local {
    use super::*;

    mod plain_struct {
        use super::*;

        mod prefix_mapping {
            use super::*;

            mod publish_subscribe {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Local,
                    PublishSubscribe<PrefixMapped, FixedSizePayload<UInt64>, NoHeader>,
                    Ros2Fixture<PrefixMapped, PlainStruct>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                    Local,
                    PublishSubscribe<PrefixMapped, FixedSizePayload<UInt64>, NoHeader>,
                    Ros2Fixture<PrefixMapped, PlainStruct>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_discovery,
                    Local,
                    PublishSubscribe<PrefixMapped, FixedSizePayload<UInt64>, NoHeader>,
                    GatewayLinkFixture<Ros2Fixture<PrefixMapped, PlainStruct>>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_wake,
                    Local,
                    PublishSubscribe<PrefixMapped, FixedSizePayload<UInt64>, NoHeader>,
                    GatewayLinkFixture<Ros2Fixture<PrefixMapped, PlainStruct>>
                );
            }

            mod publish_subscribe_with_ros_header {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Local,
                    PublishSubscribe<PrefixMapped, FixedSizePayload<UInt64>, RosHeader>,
                    Ros2Fixture<PrefixMapped, PlainStructWithHeader>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                    Local,
                    PublishSubscribe<PrefixMapped, FixedSizePayload<UInt64>, RosHeader>,
                    Ros2Fixture<PrefixMapped, PlainStructWithHeader>
                );
            }
        }

        mod static_mapping {
            use super::*;

            mod publish_subscribe {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Local,
                    PublishSubscribe<StaticMapped, FixedSizePayload<UInt64>, NoHeader>,
                    Ros2Fixture<StaticMapped, PlainStruct>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                    Local,
                    PublishSubscribe<StaticMapped, FixedSizePayload<UInt64>, NoHeader>,
                    Ros2Fixture<StaticMapped, PlainStruct>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_discovery,
                    Local,
                    PublishSubscribe<StaticMapped, FixedSizePayload<UInt64>, NoHeader>,
                    GatewayLinkFixture<Ros2Fixture<StaticMapped, PlainStruct>>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_wake,
                    Local,
                    PublishSubscribe<StaticMapped, FixedSizePayload<UInt64>, NoHeader>,
                    GatewayLinkFixture<Ros2Fixture<StaticMapped, PlainStruct>>
                );
            }

            mod publish_subscribe_with_ros_header {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Local,
                    PublishSubscribe<StaticMapped, FixedSizePayload<UInt64>, RosHeader>,
                    Ros2Fixture<StaticMapped, PlainStructWithHeader>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                    Local,
                    PublishSubscribe<StaticMapped, FixedSizePayload<UInt64>, RosHeader>,
                    Ros2Fixture<StaticMapped, PlainStructWithHeader>
                );
            }
        }
    }

    mod passthrough {
        use super::*;

        mod prefix_mapping {
            use super::*;

            mod publish_subscribe {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Local,
                    PublishSubscribe<PrefixMapped, SerializedString, NoHeader>,
                    Ros2Fixture<PrefixMapped, Passthrough>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                    Local,
                    PublishSubscribe<PrefixMapped, SerializedString, NoHeader>,
                    Ros2Fixture<PrefixMapped, Passthrough>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_discovery,
                    Local,
                    PublishSubscribe<PrefixMapped, SerializedString, NoHeader>,
                    GatewayLinkFixture<Ros2Fixture<PrefixMapped, Passthrough>>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_wake,
                    Local,
                    PublishSubscribe<PrefixMapped, SerializedString, NoHeader>,
                    GatewayLinkFixture<Ros2Fixture<PrefixMapped, Passthrough>>
                );
            }

            mod publish_subscribe_with_ros_header {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Local,
                    PublishSubscribe<PrefixMapped, SerializedString, RosHeader>,
                    Ros2Fixture<PrefixMapped, PassthroughWithHeader>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                    Local,
                    PublishSubscribe<PrefixMapped, SerializedString, RosHeader>,
                    Ros2Fixture<PrefixMapped, PassthroughWithHeader>
                );
            }
        }

        mod static_mapping {
            use super::*;

            mod publish_subscribe {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Local,
                    PublishSubscribe<StaticMapped, SerializedString, NoHeader>,
                    Ros2Fixture<StaticMapped, Passthrough>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                    Local,
                    PublishSubscribe<StaticMapped, SerializedString, NoHeader>,
                    Ros2Fixture<StaticMapped, Passthrough>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_discovery,
                    Local,
                    PublishSubscribe<StaticMapped, SerializedString, NoHeader>,
                    GatewayLinkFixture<Ros2Fixture<StaticMapped, Passthrough>>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_wake,
                    Local,
                    PublishSubscribe<StaticMapped, SerializedString, NoHeader>,
                    GatewayLinkFixture<Ros2Fixture<StaticMapped, Passthrough>>
                );
            }

            mod publish_subscribe_with_ros_header {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Local,
                    PublishSubscribe<StaticMapped, SerializedString, RosHeader>,
                    Ros2Fixture<StaticMapped, PassthroughWithHeader>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
                    Local,
                    PublishSubscribe<StaticMapped, SerializedString, RosHeader>,
                    Ros2Fixture<StaticMapped, PassthroughWithHeader>
                );
            }
        }
    }
}
