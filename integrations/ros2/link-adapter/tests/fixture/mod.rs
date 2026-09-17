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

pub mod prefix_mapping;
mod remote_endpoints;
pub mod static_mapping;
mod wire_form;

pub use prefix_mapping::PrefixMapped;
pub use static_mapping::StaticMapped;
pub use wire_form::{Passthrough, PlainStruct};

use core::marker::PhantomData;

use iceoryx2::service::Service;
use iceoryx2::testing::generate_service_name;
use iceoryx2_integrations_ros2_link_adapter::qos::Durability;
use iceoryx2_integrations_ros2_link_adapter::testing::PeerNode;
use iceoryx2_integrations_ros2_link_adapter::{
    Config, QosProfile, Ros2Adapter, TopicDescription, TopicName, TopicSettings, TopicTypes,
    TypeName,
};
use iceoryx2_link_adapter::{Mapping, Translator};
use iceoryx2_link_backend::service_description::ServiceDescription;
use iceoryx2_link_conformance_tests::fixture::{AdapterFixture, GatewayFixture};
use iceoryx2_link_conformance_tests::parameters::PublishSubscribeName;

use remote_endpoints::{RemoteMessageEndpoints, RemotePayloadEndpoints};
use wire_form::WireForm;

/// A mapping under test, also the source of the names it covers.
pub trait MappingUnderTest: PublishSubscribeName + 'static {
    type Mapping: Mapping<EndpointSettings = TopicSettings>;

    /// The mapping, over services of the payload type `payload_type`.
    fn mapping(payload_type: &str) -> Self::Mapping;
}

/// The ROS 2 graph, the adapters on it and a peer node holding the
/// remote endpoints, the gateways mapping through `M` and translating
/// through `W`.
pub struct Ros2Fixture<M, W> {
    peer: PeerNode,
    _under_test: PhantomData<(M, W)>,
}

impl<M: MappingUnderTest, W: WireForm> AdapterFixture for Ros2Fixture<M, W> {
    type Adapter = Ros2Adapter;
    type RemoteEndpoints = RemoteMessageEndpoints;

    fn new() -> Self {
        Self {
            peer: PeerNode::new(),
            _under_test: PhantomData,
        }
    }

    fn adapter(&mut self) -> Ros2Adapter {
        let config = Config {
            rosout: false,
            ..Config::default()
        };
        Ros2Adapter::new(&config).expect("the adapter is created")
    }

    fn remote_endpoints(&mut self) -> RemoteMessageEndpoints {
        let topic = TopicName::new(&format!("/{}", generate_service_name().as_str()))
            .expect("a valid topic name");
        // Transient local so that payloads sent before matching is complete
        // are still received, on the gateway's endpoints too, they take the
        // description's QoS.
        let qos = QosProfile {
            durability: Durability::TransientLocal,
            ..QosProfile::default()
        };
        let description = TopicDescription {
            settings: TopicSettings {
                topic,
                qos: qos.clone(),
            },
            types: TopicTypes {
                type_name: TypeName::new(W::TYPE_NAME).expect("a valid type name"),
            },
        };
        RemoteMessageEndpoints::new(&self.peer, description, qos)
    }
}

impl<S: Service, M: MappingUnderTest, W: WireForm> GatewayFixture<S> for Ros2Fixture<M, W> {
    type Mapping = M::Mapping;
    type Translator = W::Translator;
    type RemoteEndpoints = RemotePayloadEndpoints<W>;

    fn mapping(&self) -> M::Mapping {
        M::mapping(W::TYPE_NAME)
    }

    fn translator(&self) -> W::Translator {
        W::Translator::default()
    }

    /// The topic and type the gateway's own mapping and translator take
    /// the service to.
    fn remote_endpoints_on(&mut self, service: &ServiceDescription) -> RemotePayloadEndpoints<W> {
        let settings = <Self as GatewayFixture<S>>::mapping(self)
            .remote(service.settings())
            .expect("the mapping succeeds")
            .expect("the mapping covers the service");
        let types = <Self as GatewayFixture<S>>::translator(self)
            .remote(service.types())
            .expect("the translator covers the service");
        let qos = settings.qos.clone();
        let endpoints =
            RemoteMessageEndpoints::new(&self.peer, TopicDescription { settings, types }, qos);
        RemotePayloadEndpoints {
            service: service.clone(),
            endpoints,
            _wire_form: PhantomData,
        }
    }
}
