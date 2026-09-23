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
pub use payload::{SerializedString, UInt64};
pub use translation::{Passthrough, PassthroughWithHeader, PlainStruct, PlainStructWithHeader};

use core::marker::PhantomData;

use iceoryx2::service::Service;
use iceoryx2::testing::{generate_isolated_config, generate_service_name};
use iceoryx2_integrations_ros2_link_adapter::qos::Durability;
use iceoryx2_integrations_ros2_link_adapter::testing::PeerNode;
use iceoryx2_integrations_ros2_link_adapter::{
    Config, QosProfile, Ros2Adapter, TopicDescription, TopicName, TopicSettings, TopicTypes,
    TypeName,
};
use iceoryx2_link_adapter::Mapping;
use iceoryx2_link_adapter::Translator;
use iceoryx2_link_backend::service_description::ServiceDescription;
use iceoryx2_link_conformance_tests::fixture::{AdapterFixture, GatewayFixture};
use rosidl_runtime_rs::RmwMessage;

use endpoints::{MappedRclEndpoints, RclEndpoints};
use mapping::MappingUnderTest;
use translation::TranslatorUnderTest;

/// The fixture for the adapter and gateway suites on ROS 2, with the
/// mapping `M` and the translator `T` under test.
pub struct Ros2Fixture<M, T> {
    peer: PeerNode,
    _under_test: PhantomData<(M, T)>,
}

impl<M: MappingUnderTest, T: TranslatorUnderTest> AdapterFixture for Ros2Fixture<M, T> {
    type Adapter = Ros2Adapter;
    type RemoteEndpoints = RclEndpoints<T>;

    fn new() -> Self {
        Self {
            peer: PeerNode::new(),
            _under_test: PhantomData,
        }
    }

    fn config(&self) -> iceoryx2::config::Config {
        let mut config = generate_isolated_config();

        // Keep history so samples are not lost to late matching.
        config.defaults.publish_subscribe.publisher_history_size = 5;
        config
    }

    fn adapter(&mut self) -> Ros2Adapter {
        let config = Config {
            rosout: false,
            ..Config::default()
        };
        Ros2Adapter::new(&config).expect("the adapter is created")
    }

    fn remote_endpoints(&mut self) -> RclEndpoints<T> {
        let topic = TopicName::new(&format!("/{}", generate_service_name().as_str()))
            .expect("a valid topic name");
        // Transient local durability keeps messages sent before matching
        // completes. The gateway's endpoints take the same QoS from the
        // description.
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
                type_name: TypeName::new(<T::Message as RmwMessage>::TYPE_NAME)
                    .expect("a valid type name"),
            },
        };
        RclEndpoints::new(&self.peer, description, qos)
    }
}

impl<S: Service, M: MappingUnderTest, T: TranslatorUnderTest> GatewayFixture<S>
    for Ros2Fixture<M, T>
{
    type Mapping = M::Mapping;
    type Translator = T::Translator;
    type RemoteEndpoints = MappedRclEndpoints<T>;

    fn mapping(&self) -> M::Mapping {
        M::mapping(<T::Message as RmwMessage>::TYPE_NAME)
    }

    fn translator(&self) -> T::Translator {
        T::translator()
    }

    /// Remote endpoints on topics and types the gateway maps and
    /// translates `service` to.
    fn remote_endpoints_on(&mut self, service: &ServiceDescription) -> MappedRclEndpoints<T> {
        let settings = <Self as GatewayFixture<S>>::mapping(self)
            .remote(service.settings())
            .expect("the mapping succeeds")
            .expect("the mapping covers the service");
        let types = <Self as GatewayFixture<S>>::translator(self)
            .remote(service.types())
            .expect("the translator covers the service");

        let qos = settings.qos.clone();
        let endpoints = RclEndpoints::new(&self.peer, TopicDescription { settings, types }, qos);

        MappedRclEndpoints {
            service: service.clone(),
            endpoints,
        }
    }
}
