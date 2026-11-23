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

use iceoryx2::service::messaging_pattern::MessagingPattern;
use iceoryx2_bb_testing::instantiate_conformance_tests;
use iceoryx2_bb_testing_macros::requires;
use iceoryx2_integrations_ros2_link_adapter::ros_header::RosHeader;
use iceoryx2_link_conformance_tests::fixture::GatewayLinkFixture;
use iceoryx2_link_conformance_tests::parameters::{
    FixedSizePayload, PublishSubscribe, RequestResponse,
};

use crate::fixture::{
    AddTwoIntsRequest, AddTwoIntsResponse, Passthrough, PassthroughWithHeader, PlainStruct,
    PlainStructWithHeader, PrefixMapped, Ros2Fixture, SerializedAddTwoIntsRequest,
    SerializedAddTwoIntsResponse, SerializedString, StaticMapped, UInt64,
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
#[requires(crate::fixture::supports_pattern(MessagingPattern::RequestResponse))]
mod request_response {
    use super::*;

    use core::time::Duration;

    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::test;
    use iceoryx2_integrations_ros2_link_adapter::testing::PeerNode;
    use iceoryx2_integrations_ros2_link_adapter::{EndpointName, QosProfile, TypeName};
    use iceoryx2_link_adapter::{
        Adapter, RequestResponseEndpoints, ResponseKind, SampleBytes, SampleBytesRef, TakeOutcome,
    };
    use iceoryx2_link_conformance_tests::fixture::{
        AdapterFixture, Describable, PropagatesRequestResponseMessages,
    };
    use iceoryx2_link_conformance_tests::testing::{ClientRequestIds, retry};

    use crate::fixture::SERVICE_TYPE;

    const TIMEOUT: Duration = Duration::from_secs(10);
    const REQUEST: u64 = 7;
    const NO_HEADER: &[u8] = &[];

    type Fixture = Ros2Fixture<PrefixMapped, PlainStruct>;

    instantiate_conformance_tests!(
        iceoryx2_link_conformance_tests::adapter_request_response,
        Ros2Fixture<PrefixMapped, PlainStruct>
    );

    #[test]
    fn requests_end_when_another_node_only_calls_the_service() {
        let mut fixture = Fixture::new();

        let mut adapter = fixture.adapter();
        let remote = fixture.remote_endpoints(MessagingPattern::RequestResponse);
        let mut own = adapter
            .request_response(remote.description())
            .expect("the adapter's endpoints open");
        let EndpointName::Service(service) = remote.description().settings.name.clone() else {
            panic!("the endpoints are on a ROS 2 service");
        };
        let request = remote.encode_request(&remote.request(REQUEST));
        let ids = ClientRequestIds::new();
        assert_that!(remote.sync(TIMEOUT), eq true);
        drop(remote);

        let peer = PeerNode::new();
        let _client = peer.client(
            &service,
            &TypeName::new(SERVICE_TYPE).expect("a valid type name"),
            QosProfile::default(),
        );
        let graph = adapter.graph();
        retry(
            || match (
                graph.discovered_client(&service),
                graph.discovered_server(&service),
            ) {
                (true, false) => Ok(()),
                _ => Err("the graph does not show only a client"),
            },
            TIMEOUT,
        )
        .expect("only a client is on the graph");

        let id = ids.next();
        own.send_request(
            id,
            SampleBytesRef {
                header: NO_HEADER,
                payload: &request,
            },
        )
        .expect("sending the request succeeds");

        assert_that!(own.take_response(|_| None::<&mut SampleBytes>), eq Ok(TakeOutcome::Taken((id, ResponseKind::End))));
        assert_that!(own.take_response(|_| None::<&mut SampleBytes>), eq Ok(TakeOutcome::Empty));
    }

    #[test]
    fn requests_end_when_the_last_server_leaves() {
        let mut fixture = Fixture::new();

        let mut adapter = fixture.adapter();
        let remote = fixture.remote_endpoints(MessagingPattern::RequestResponse);
        let mut own = adapter
            .request_response(remote.description())
            .expect("the adapter's endpoints open");
        let EndpointName::Service(service) = remote.description().settings.name.clone() else {
            panic!("the endpoints are on a ROS 2 service");
        };
        let ids = ClientRequestIds::new();
        assert_that!(remote.sync(TIMEOUT), eq true);
        let sent = [ids.next(), ids.next()];
        for id in sent {
            own.send_request(
                id,
                SampleBytesRef {
                    header: NO_HEADER,
                    payload: &remote.encode_request(&remote.request(REQUEST)),
                },
            )
            .expect("sending the request succeeds");
            retry(
                || remote.receive_request().ok_or("no request arrived"),
                TIMEOUT,
            )
            .expect("the request reaches the remote endpoints");
        }
        assert_that!(own.take_response(|_| None::<&mut SampleBytes>), eq Ok(TakeOutcome::Empty));

        drop(remote);
        let graph = adapter.graph();
        retry(
            || match graph.discovered_server(&service) {
                false => Ok(()),
                true => Err("the graph still shows the server"),
            },
            TIMEOUT,
        )
        .expect("the server left the graph");

        for id in sent {
            assert_that!(own.take_response(|_| None::<&mut SampleBytes>), eq Ok(TakeOutcome::Taken((id, ResponseKind::End))));
        }
        assert_that!(own.take_response(|_| None::<&mut SampleBytes>), eq Ok(TakeOutcome::Empty));
    }
}
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
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe,
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
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe,
                    Ipc,
                    PublishSubscribe<PrefixMapped, FixedSizePayload<UInt64>, RosHeader>,
                    Ros2Fixture<PrefixMapped, PlainStructWithHeader>
                );
            }

            #[requires(crate::fixture::supports_pattern(MessagingPattern::RequestResponse))]
            mod request_response {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Ipc,
                    RequestResponse<
                        PrefixMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        NoHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        NoHeader,
                    >,
                    Ros2Fixture<PrefixMapped, PlainStruct>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_request_response,
                    Ipc,
                    RequestResponse<
                        PrefixMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        NoHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        NoHeader,
                    >,
                    Ros2Fixture<PrefixMapped, PlainStruct>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_discovery,
                    Ipc,
                    RequestResponse<
                        PrefixMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        NoHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        NoHeader,
                    >,
                    GatewayLinkFixture<Ros2Fixture<PrefixMapped, PlainStruct>>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_wake,
                    Ipc,
                    RequestResponse<
                        PrefixMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        NoHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        NoHeader,
                    >,
                    GatewayLinkFixture<Ros2Fixture<PrefixMapped, PlainStruct>>
                );
            }

            #[requires(crate::fixture::supports_pattern(MessagingPattern::RequestResponse))]
            mod request_response_with_ros_header {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Ipc,
                    RequestResponse<
                        PrefixMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        RosHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        RosHeader,
                    >,
                    Ros2Fixture<PrefixMapped, PlainStructWithHeader>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_request_response,
                    Ipc,
                    RequestResponse<
                        PrefixMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        RosHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        RosHeader,
                    >,
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
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe,
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
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe,
                    Ipc,
                    PublishSubscribe<StaticMapped, FixedSizePayload<UInt64>, RosHeader>,
                    Ros2Fixture<StaticMapped, PlainStructWithHeader>
                );
            }

            #[requires(crate::fixture::supports_pattern(MessagingPattern::RequestResponse))]
            mod request_response {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Ipc,
                    RequestResponse<
                        StaticMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        NoHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        NoHeader,
                    >,
                    Ros2Fixture<StaticMapped, PlainStruct>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_request_response,
                    Ipc,
                    RequestResponse<
                        StaticMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        NoHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        NoHeader,
                    >,
                    Ros2Fixture<StaticMapped, PlainStruct>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_discovery,
                    Ipc,
                    RequestResponse<
                        StaticMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        NoHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        NoHeader,
                    >,
                    GatewayLinkFixture<Ros2Fixture<StaticMapped, PlainStruct>>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_wake,
                    Ipc,
                    RequestResponse<
                        StaticMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        NoHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        NoHeader,
                    >,
                    GatewayLinkFixture<Ros2Fixture<StaticMapped, PlainStruct>>
                );
            }

            #[requires(crate::fixture::supports_pattern(MessagingPattern::RequestResponse))]
            mod request_response_with_ros_header {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Ipc,
                    RequestResponse<
                        StaticMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        RosHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        RosHeader,
                    >,
                    Ros2Fixture<StaticMapped, PlainStructWithHeader>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_request_response,
                    Ipc,
                    RequestResponse<
                        StaticMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        RosHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        RosHeader,
                    >,
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
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe,
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
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe,
                    Ipc,
                    PublishSubscribe<PrefixMapped, SerializedString, RosHeader>,
                    Ros2Fixture<PrefixMapped, PassthroughWithHeader>
                );
            }

            #[requires(crate::fixture::supports_pattern(MessagingPattern::RequestResponse))]
            mod request_response {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Ipc,
                    RequestResponse<
                        PrefixMapped,
                        SerializedAddTwoIntsRequest,
                        NoHeader,
                        SerializedAddTwoIntsResponse,
                        NoHeader,
                    >,
                    Ros2Fixture<PrefixMapped, Passthrough>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_request_response,
                    Ipc,
                    RequestResponse<
                        PrefixMapped,
                        SerializedAddTwoIntsRequest,
                        NoHeader,
                        SerializedAddTwoIntsResponse,
                        NoHeader,
                    >,
                    Ros2Fixture<PrefixMapped, Passthrough>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_discovery,
                    Ipc,
                    RequestResponse<
                        PrefixMapped,
                        SerializedAddTwoIntsRequest,
                        NoHeader,
                        SerializedAddTwoIntsResponse,
                        NoHeader,
                    >,
                    GatewayLinkFixture<Ros2Fixture<PrefixMapped, Passthrough>>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_wake,
                    Ipc,
                    RequestResponse<
                        PrefixMapped,
                        SerializedAddTwoIntsRequest,
                        NoHeader,
                        SerializedAddTwoIntsResponse,
                        NoHeader,
                    >,
                    GatewayLinkFixture<Ros2Fixture<PrefixMapped, Passthrough>>
                );
            }

            #[requires(crate::fixture::supports_pattern(MessagingPattern::RequestResponse))]
            mod request_response_with_ros_header {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Ipc,
                    RequestResponse<
                        PrefixMapped,
                        SerializedAddTwoIntsRequest,
                        RosHeader,
                        SerializedAddTwoIntsResponse,
                        RosHeader,
                    >,
                    Ros2Fixture<PrefixMapped, PassthroughWithHeader>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_request_response,
                    Ipc,
                    RequestResponse<
                        PrefixMapped,
                        SerializedAddTwoIntsRequest,
                        RosHeader,
                        SerializedAddTwoIntsResponse,
                        RosHeader,
                    >,
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
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe,
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
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe,
                    Ipc,
                    PublishSubscribe<StaticMapped, SerializedString, RosHeader>,
                    Ros2Fixture<StaticMapped, PassthroughWithHeader>
                );
            }

            #[requires(crate::fixture::supports_pattern(MessagingPattern::RequestResponse))]
            mod request_response {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Ipc,
                    RequestResponse<
                        StaticMapped,
                        SerializedAddTwoIntsRequest,
                        NoHeader,
                        SerializedAddTwoIntsResponse,
                        NoHeader,
                    >,
                    Ros2Fixture<StaticMapped, Passthrough>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_request_response,
                    Ipc,
                    RequestResponse<
                        StaticMapped,
                        SerializedAddTwoIntsRequest,
                        NoHeader,
                        SerializedAddTwoIntsResponse,
                        NoHeader,
                    >,
                    Ros2Fixture<StaticMapped, Passthrough>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_discovery,
                    Ipc,
                    RequestResponse<
                        StaticMapped,
                        SerializedAddTwoIntsRequest,
                        NoHeader,
                        SerializedAddTwoIntsResponse,
                        NoHeader,
                    >,
                    GatewayLinkFixture<Ros2Fixture<StaticMapped, Passthrough>>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_wake,
                    Ipc,
                    RequestResponse<
                        StaticMapped,
                        SerializedAddTwoIntsRequest,
                        NoHeader,
                        SerializedAddTwoIntsResponse,
                        NoHeader,
                    >,
                    GatewayLinkFixture<Ros2Fixture<StaticMapped, Passthrough>>
                );
            }

            #[requires(crate::fixture::supports_pattern(MessagingPattern::RequestResponse))]
            mod request_response_with_ros_header {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Ipc,
                    RequestResponse<
                        StaticMapped,
                        SerializedAddTwoIntsRequest,
                        RosHeader,
                        SerializedAddTwoIntsResponse,
                        RosHeader,
                    >,
                    Ros2Fixture<StaticMapped, PassthroughWithHeader>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_request_response,
                    Ipc,
                    RequestResponse<
                        StaticMapped,
                        SerializedAddTwoIntsRequest,
                        RosHeader,
                        SerializedAddTwoIntsResponse,
                        RosHeader,
                    >,
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
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe,
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
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe,
                    Local,
                    PublishSubscribe<PrefixMapped, FixedSizePayload<UInt64>, RosHeader>,
                    Ros2Fixture<PrefixMapped, PlainStructWithHeader>
                );
            }

            #[requires(crate::fixture::supports_pattern(MessagingPattern::RequestResponse))]
            mod request_response {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Local,
                    RequestResponse<
                        PrefixMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        NoHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        NoHeader,
                    >,
                    Ros2Fixture<PrefixMapped, PlainStruct>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_request_response,
                    Local,
                    RequestResponse<
                        PrefixMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        NoHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        NoHeader,
                    >,
                    Ros2Fixture<PrefixMapped, PlainStruct>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_discovery,
                    Local,
                    RequestResponse<
                        PrefixMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        NoHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        NoHeader,
                    >,
                    GatewayLinkFixture<Ros2Fixture<PrefixMapped, PlainStruct>>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_wake,
                    Local,
                    RequestResponse<
                        PrefixMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        NoHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        NoHeader,
                    >,
                    GatewayLinkFixture<Ros2Fixture<PrefixMapped, PlainStruct>>
                );
            }

            #[requires(crate::fixture::supports_pattern(MessagingPattern::RequestResponse))]
            mod request_response_with_ros_header {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Local,
                    RequestResponse<
                        PrefixMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        RosHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        RosHeader,
                    >,
                    Ros2Fixture<PrefixMapped, PlainStructWithHeader>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_request_response,
                    Local,
                    RequestResponse<
                        PrefixMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        RosHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        RosHeader,
                    >,
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
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe,
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
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe,
                    Local,
                    PublishSubscribe<StaticMapped, FixedSizePayload<UInt64>, RosHeader>,
                    Ros2Fixture<StaticMapped, PlainStructWithHeader>
                );
            }

            #[requires(crate::fixture::supports_pattern(MessagingPattern::RequestResponse))]
            mod request_response {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Local,
                    RequestResponse<
                        StaticMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        NoHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        NoHeader,
                    >,
                    Ros2Fixture<StaticMapped, PlainStruct>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_request_response,
                    Local,
                    RequestResponse<
                        StaticMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        NoHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        NoHeader,
                    >,
                    Ros2Fixture<StaticMapped, PlainStruct>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_discovery,
                    Local,
                    RequestResponse<
                        StaticMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        NoHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        NoHeader,
                    >,
                    GatewayLinkFixture<Ros2Fixture<StaticMapped, PlainStruct>>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_wake,
                    Local,
                    RequestResponse<
                        StaticMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        NoHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        NoHeader,
                    >,
                    GatewayLinkFixture<Ros2Fixture<StaticMapped, PlainStruct>>
                );
            }

            #[requires(crate::fixture::supports_pattern(MessagingPattern::RequestResponse))]
            mod request_response_with_ros_header {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Local,
                    RequestResponse<
                        StaticMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        RosHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        RosHeader,
                    >,
                    Ros2Fixture<StaticMapped, PlainStructWithHeader>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_request_response,
                    Local,
                    RequestResponse<
                        StaticMapped,
                        FixedSizePayload<AddTwoIntsRequest>,
                        RosHeader,
                        FixedSizePayload<AddTwoIntsResponse>,
                        RosHeader,
                    >,
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
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe,
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
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe,
                    Local,
                    PublishSubscribe<PrefixMapped, SerializedString, RosHeader>,
                    Ros2Fixture<PrefixMapped, PassthroughWithHeader>
                );
            }

            #[requires(crate::fixture::supports_pattern(MessagingPattern::RequestResponse))]
            mod request_response {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Local,
                    RequestResponse<
                        PrefixMapped,
                        SerializedAddTwoIntsRequest,
                        NoHeader,
                        SerializedAddTwoIntsResponse,
                        NoHeader,
                    >,
                    Ros2Fixture<PrefixMapped, Passthrough>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_request_response,
                    Local,
                    RequestResponse<
                        PrefixMapped,
                        SerializedAddTwoIntsRequest,
                        NoHeader,
                        SerializedAddTwoIntsResponse,
                        NoHeader,
                    >,
                    Ros2Fixture<PrefixMapped, Passthrough>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_discovery,
                    Local,
                    RequestResponse<
                        PrefixMapped,
                        SerializedAddTwoIntsRequest,
                        NoHeader,
                        SerializedAddTwoIntsResponse,
                        NoHeader,
                    >,
                    GatewayLinkFixture<Ros2Fixture<PrefixMapped, Passthrough>>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_wake,
                    Local,
                    RequestResponse<
                        PrefixMapped,
                        SerializedAddTwoIntsRequest,
                        NoHeader,
                        SerializedAddTwoIntsResponse,
                        NoHeader,
                    >,
                    GatewayLinkFixture<Ros2Fixture<PrefixMapped, Passthrough>>
                );
            }

            #[requires(crate::fixture::supports_pattern(MessagingPattern::RequestResponse))]
            mod request_response_with_ros_header {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Local,
                    RequestResponse<
                        PrefixMapped,
                        SerializedAddTwoIntsRequest,
                        RosHeader,
                        SerializedAddTwoIntsResponse,
                        RosHeader,
                    >,
                    Ros2Fixture<PrefixMapped, PassthroughWithHeader>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_request_response,
                    Local,
                    RequestResponse<
                        PrefixMapped,
                        SerializedAddTwoIntsRequest,
                        RosHeader,
                        SerializedAddTwoIntsResponse,
                        RosHeader,
                    >,
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
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe,
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
                    iceoryx2_link_conformance_tests::gateway_publish_subscribe,
                    Local,
                    PublishSubscribe<StaticMapped, SerializedString, RosHeader>,
                    Ros2Fixture<StaticMapped, PassthroughWithHeader>
                );
            }

            #[requires(crate::fixture::supports_pattern(MessagingPattern::RequestResponse))]
            mod request_response {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Local,
                    RequestResponse<
                        StaticMapped,
                        SerializedAddTwoIntsRequest,
                        NoHeader,
                        SerializedAddTwoIntsResponse,
                        NoHeader,
                    >,
                    Ros2Fixture<StaticMapped, Passthrough>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_request_response,
                    Local,
                    RequestResponse<
                        StaticMapped,
                        SerializedAddTwoIntsRequest,
                        NoHeader,
                        SerializedAddTwoIntsResponse,
                        NoHeader,
                    >,
                    Ros2Fixture<StaticMapped, Passthrough>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_discovery,
                    Local,
                    RequestResponse<
                        StaticMapped,
                        SerializedAddTwoIntsRequest,
                        NoHeader,
                        SerializedAddTwoIntsResponse,
                        NoHeader,
                    >,
                    GatewayLinkFixture<Ros2Fixture<StaticMapped, Passthrough>>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::link_wake,
                    Local,
                    RequestResponse<
                        StaticMapped,
                        SerializedAddTwoIntsRequest,
                        NoHeader,
                        SerializedAddTwoIntsResponse,
                        NoHeader,
                    >,
                    GatewayLinkFixture<Ros2Fixture<StaticMapped, Passthrough>>
                );
            }

            #[requires(crate::fixture::supports_pattern(MessagingPattern::RequestResponse))]
            mod request_response_with_ros_header {
                use super::*;

                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_discovery,
                    Local,
                    RequestResponse<
                        StaticMapped,
                        SerializedAddTwoIntsRequest,
                        RosHeader,
                        SerializedAddTwoIntsResponse,
                        RosHeader,
                    >,
                    Ros2Fixture<StaticMapped, PassthroughWithHeader>
                );
                instantiate_conformance_tests!(
                    iceoryx2_link_conformance_tests::gateway_request_response,
                    Local,
                    RequestResponse<
                        StaticMapped,
                        SerializedAddTwoIntsRequest,
                        RosHeader,
                        SerializedAddTwoIntsResponse,
                        RosHeader,
                    >,
                    Ros2Fixture<StaticMapped, PassthroughWithHeader>
                );
            }
        }
    }
}
