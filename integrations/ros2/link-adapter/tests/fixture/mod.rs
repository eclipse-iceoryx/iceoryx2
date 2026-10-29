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

mod endpoints;
mod mapping;
mod payload;
mod serialization;
mod translation;

pub use mapping::{PrefixMapped, StaticMapped};
pub use payload::{
    AddTwoIntsRequest, AddTwoIntsResponse, SerializedAddTwoIntsRequest,
    SerializedAddTwoIntsResponse, SerializedString, UInt64,
};
pub use translation::{Passthrough, PassthroughWithHeader, PlainStruct, PlainStructWithHeader};

pub use iceoryx2_integrations_ros2_link_adapter::testing::supports_pattern;

use core::marker::PhantomData;
use std::rc::Rc;

use iceoryx2::service::Service;
use iceoryx2::service::messaging_pattern::MessagingPattern;
use iceoryx2::testing::{generate_isolated_config, generate_node_name};
use iceoryx2_bb_concurrency::cell::RefCell;
use iceoryx2_integrations_ros2_link_adapter::testing::{GraphHandle, PeerNode};
use iceoryx2_integrations_ros2_link_adapter::{
    Config, EndpointDescription, MessageType, NodeName, Ros2Adapter,
};
use iceoryx2_link_adapter::Mapping;
use iceoryx2_link_adapter::{EndpointTypes, Translator};
use iceoryx2_link_backend::service_description::{
    PublishSubscribeSettings, ServiceDescription, ServiceTypes,
};
use iceoryx2_link_conformance_tests::fixture::{AdapterFixture, GatewayFixture};
use iceoryx2_link_conformance_tests::parameters::{PublishSubscribeName, RequestResponseName};
use iceoryx2_link_conformance_tests::testing::{
    describe_publish_subscribe, describe_request_response,
};
use rosidl_runtime_rs::RmwMessage;

use endpoints::RclEndpoints;
use mapping::MappingUnderTest;
use translation::TranslatorUnderTest;

/// The ROS 2 service type the request-response suites run over.
pub const SERVICE_TYPE: &str = "example_interfaces/srv/AddTwoInts";

/// The settings to use in all suites.
pub const PUBLISH_SUBSCRIBE_SETTINGS: PublishSubscribeSettings = PublishSubscribeSettings {
    max_subscribers: 8,
    max_publishers: 2,
    max_nodes: 20,
    history_size: 5,
    subscriber_max_buffer_size: 2,
    subscriber_max_borrowed_samples: 2,
    safe_overflow: true,
};

/// The fixture for the adapter and gateway suites on ROS 2, with the
/// mapping `M` and the translator `T` under test.
pub struct Ros2Fixture<M, T> {
    peer: PeerNode,
    /// The graphs of the peer's node and of every gateway's node created,
    /// which the remote endpoints wait on for matching.
    graphs: Rc<RefCell<Vec<GraphHandle>>>,
    _under_test: PhantomData<(M, T)>,
}

impl<M: MappingUnderTest, T: TranslatorUnderTest> AdapterFixture for Ros2Fixture<M, T> {
    type SampleTypes = MessageType;
    type Adapter = Ros2Adapter;
    type RemoteEndpoints = RclEndpoints<T>;

    fn new() -> Self {
        let peer = PeerNode::new();
        let graphs = Rc::new(RefCell::new(vec![peer.graph()]));
        Self {
            peer,
            graphs,
            _under_test: PhantomData,
        }
    }

    fn config(&self) -> iceoryx2::config::Config {
        let mut config = generate_isolated_config();

        let defaults = &mut config.defaults.publish_subscribe;
        defaults.max_subscribers = PUBLISH_SUBSCRIBE_SETTINGS.max_subscribers;
        defaults.max_publishers = PUBLISH_SUBSCRIBE_SETTINGS.max_publishers;
        defaults.max_nodes = PUBLISH_SUBSCRIBE_SETTINGS.max_nodes;
        defaults.publisher_history_size = PUBLISH_SUBSCRIBE_SETTINGS.history_size;
        defaults.subscriber_max_buffer_size = PUBLISH_SUBSCRIBE_SETTINGS.subscriber_max_buffer_size;
        defaults.subscriber_max_borrowed_samples =
            PUBLISH_SUBSCRIBE_SETTINGS.subscriber_max_borrowed_samples;
        defaults.enable_safe_overflow = PUBLISH_SUBSCRIBE_SETTINGS.safe_overflow;

        config
    }

    fn adapter(&mut self) -> Ros2Adapter {
        let config = Config {
            node_name: NodeName::new(generate_node_name().as_str()).expect("a valid node name"),
            rosout: false,
            ..Config::default()
        };
        let adapter = Ros2Adapter::new(&config).expect("the adapter is created");
        self.graphs.borrow_mut().push(adapter.graph());
        adapter
    }

    fn remote_endpoints(&mut self, pattern: MessagingPattern) -> RclEndpoints<T> {
        let config = self.config();
        let service_description = match pattern {
            MessagingPattern::PublishSubscribe => {
                describe_publish_subscribe::<iceoryx2::service::local::Service, T::SamplePayload, ()>(
                    &<M as PublishSubscribeName>::service_name(),
                    &config,
                )
            }
            MessagingPattern::RequestResponse => {
                describe_request_response::<
                    iceoryx2::service::local::Service,
                    T::RequestPayload,
                    (),
                    T::ResponsePayload,
                    (),
                >(&<M as RequestResponseName>::service_name(), &config)
            }
            _ => panic!("the fixture has no remote endpoints of {pattern:?}"),
        };
        self.rcl_endpoints_on(service_description)
    }
}

impl<S: Service, M: MappingUnderTest, T: TranslatorUnderTest> GatewayFixture<S>
    for Ros2Fixture<M, T>
{
    type Mapping = M::Mapping;
    type Translator = T::Translator;
    type RemoteEndpoints = RclEndpoints<T>;

    fn mapping(&self) -> M::Mapping {
        M::mapping(<T::SampleMessage as RmwMessage>::TYPE_NAME, &self.config())
    }

    fn translator(&self) -> T::Translator {
        T::translator()
    }

    fn remote_endpoints_on(&mut self, service: &ServiceDescription) -> RclEndpoints<T> {
        self.rcl_endpoints_on(service.clone())
    }
}

impl<M: MappingUnderTest, T: TranslatorUnderTest> Ros2Fixture<M, T> {
    /// The peer's endpoints on the topic or ROS 2 service, and of the types,
    /// the mapping and the translator under test take `service_description`
    /// to.
    fn rcl_endpoints_on(&self, service_description: ServiceDescription) -> RclEndpoints<T> {
        let mapping = M::mapping(<T::SampleMessage as RmwMessage>::TYPE_NAME, &self.config());
        let translator = T::translator();

        let settings = mapping
            .remote(service_description.settings())
            .expect("the mapping succeeds")
            .expect("the mapping covers the service");
        let types = match service_description.types() {
            ServiceTypes::PublishSubscribe(types) => EndpointTypes::PublishSubscribe(
                translator
                    .remote(types)
                    .expect("the translator covers the service"),
            ),
            ServiceTypes::RequestResponse(types) => EndpointTypes::RequestResponse {
                request: translator
                    .remote(&types.request)
                    .expect("the translator covers the request"),
                response: translator
                    .remote(&types.response)
                    .expect("the translator covers the response"),
            },
            ServiceTypes::Event => panic!("ROS 2 has no events"),
        };
        let endpoint_description = EndpointDescription { settings, types };

        RclEndpoints::new(
            &self.peer,
            Rc::clone(&self.graphs),
            service_description,
            endpoint_description,
        )
    }
}
