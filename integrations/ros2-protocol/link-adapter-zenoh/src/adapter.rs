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

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use iceoryx2_bb_elementary::generation::{Generation, GenerationCounter};
use iceoryx2_link_adapter::{Adapter, EndpointTypes, UnsupportedEndpoints};
use iceoryx2_log::{fail, origin, warn};
use zenoh::liveliness::LivelinessToken;
use zenoh::pubsub::Subscriber;
use zenoh::sample::{Sample, SampleKind};
use zenoh::{Session, Wait};

use crate::codec::{EntityKind, NodeInfo, Token, TopicInfo, TypeName, liveliness_subscription_key};
use crate::endpoint_description::{
    EndpointDescription, EndpointName, EndpointSettings, MessageType,
};
use crate::graph::Graph;

const DOMAIN_ID_VARIABLE: &str = "ROS_DOMAIN_ID";
const DEFAULT_DOMAIN_ID: usize = 0;
const ROOT_NAMESPACE: &str = "/";
const NAMESPACE_SEPARATOR: char = '/';
const NAME_UNDERSCORE: char = '_';
const DEFAULT_NODE_NAMESPACE: &str = ROOT_NAMESPACE;
const DEFAULT_NODE_NAME: &str = "iceoryx2_link_gateway";
const NODE_ID: &str = "0";
const NODE_ENTITY_ID: &str = "0";
const NODE_ENCLAVE: &str = "/";
const INTERNAL_TOPICS: [&str; 2] = ["/rosout", "/parameter_events"];
const INTERNAL_SERVICE_TYPES: [&str; 2] =
    ["rcl_interfaces/srv/", "type_description_interfaces/srv/"];

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CreationError {
    Session,
    Subscriber,
    NodeToken,
    InvalidNode,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ListError {
    Graph,
}

impl core::fmt::Display for ListError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ListError::{self:?}")
    }
}

impl core::error::Error for ListError {}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OpenError {
    UnsupportedPattern,
}

impl core::fmt::Display for OpenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "OpenError::{self:?}")
    }
}

impl core::error::Error for OpenError {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Config {
    pub domain_id: usize,
    pub node_namespace: String,
    pub node_name: String,
}

impl Default for Config {
    fn default() -> Self {
        let domain_id = match std::env::var(DOMAIN_ID_VARIABLE) {
            Ok(value) => value.parse().unwrap_or_else(|_| {
                warn!(
                    "{} '{}' is not a domain id, using {}",
                    DOMAIN_ID_VARIABLE, value, DEFAULT_DOMAIN_ID
                );
                DEFAULT_DOMAIN_ID
            }),
            Err(_) => DEFAULT_DOMAIN_ID,
        };
        Self {
            domain_id,
            node_namespace: DEFAULT_NODE_NAMESPACE.to_string(),
            node_name: DEFAULT_NODE_NAME.to_string(),
        }
    }
}

/// An [`Adapter`] to a ROS 2 graph on `rmw_zenoh_cpp`.
pub struct RmwZenohAdapter {
    graph: Arc<Mutex<Graph>>,
    generation: Arc<GenerationCounter>,
    _node_token: LivelinessToken,
    _subscriber: Subscriber<()>,
    session: Session,
}

impl RmwZenohAdapter {
    pub fn create(zenoh_config: zenoh::Config, config: &Config) -> Result<Self, CreationError> {
        let origin = origin!("RmwZenohAdapter::create");

        let valid_namespace = config.node_namespace == ROOT_NAMESPACE
            || config
                .node_namespace
                .strip_prefix(NAMESPACE_SEPARATOR)
                .is_some_and(|tokens| tokens.split(NAMESPACE_SEPARATOR).all(is_name_token));
        if !valid_namespace || !is_name_token(&config.node_name) {
            fail!(
                from origin,
                with CreationError::InvalidNode,
                "The node '{}' in the namespace '{}' breaks the rmw naming rules",
                config.node_name,
                config.node_namespace
            );
        }

        let session = fail!(
            from origin,
            when zenoh::open(zenoh_config).wait(),
            with CreationError::Session,
            "Failed to open the zenoh session"
        );

        let graph = Arc::new(Mutex::new(Graph::default()));
        let generation = Arc::new(GenerationCounter::new());
        let subscriber = fail!(
            from origin,
            when session
                .liveliness()
                .declare_subscriber(liveliness_subscription_key(config.domain_id))
                .history(true)
                .callback({
                    let graph = graph.clone();
                    let generation = generation.clone();
                    move |sample: Sample| {
                        let key = sample.key_expr().as_str();
                        let Ok(mut graph) = graph.lock() else {
                            warn!(from origin, "Skipping the token {} on a poisoned graph", key);
                            return;
                        };
                        let changed = match sample.kind() {
                            SampleKind::Put => graph.put(key).unwrap_or_else(|error| {
                                warn!(from origin, "Skipping the token {} that does not parse: {}", key, error);
                                false
                            }),
                            SampleKind::Delete => graph.delete(key),
                        };
                        if changed {
                            generation.advance();
                        }
                    }
                })
                .wait(),
            with CreationError::Subscriber,
            "Failed to subscribe to the liveliness tokens of domain {}", config.domain_id
        );

        let node = Token {
            zid: session.zid().to_string(),
            node_id: NODE_ID.to_string(),
            entity_id: NODE_ENTITY_ID.to_string(),
            kind: EntityKind::Node,
            node: NodeInfo {
                domain_id: config.domain_id,
                enclave: NODE_ENCLAVE.to_string(),
                namespace: config.node_namespace.clone(),
                name: config.node_name.clone(),
            },
            topic: None,
        };
        let node_token = fail!(
            from origin,
            when session.liveliness().declare_token(node.key()).wait(),
            with CreationError::NodeToken,
            "Failed to declare the liveliness token of the node {}", config.node_name
        );

        Ok(Self {
            graph,
            generation,
            _node_token: node_token,
            _subscriber: subscriber,
            session,
        })
    }
}

fn is_name_token(token: &str) -> bool {
    token
        .chars()
        .next()
        .is_some_and(|first| !first.is_ascii_digit())
        && token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == NAME_UNDERSCORE)
}

fn single_type<'a>(name: &str, entities: &[&'a TopicInfo]) -> Option<(TypeName, &'a TopicInfo)> {
    let first = entities.first()?;
    let types: BTreeSet<(&str, &str)> = entities
        .iter()
        .map(|entity| (entity.type_name.as_str(), entity.type_hash.as_str()))
        .collect();
    if types.len() > 1 {
        warn!("'{}' has {} types and is not listed", name, types.len());
        return None;
    }
    match TypeName::from_rmw(&first.type_name) {
        Ok(type_name) => Some((type_name, first)),
        Err(error) => {
            warn!(
                "'{}' has the type '{}' which is no rmw type name and is not listed: {}",
                name, first.type_name, error
            );
            None
        }
    }
}

impl Adapter for RmwZenohAdapter {
    type ListError = ListError;
    type OpenError = OpenError;
    type EndpointSettings = EndpointSettings;
    type EndpointTypes = EndpointTypes<MessageType>;
    type PublishSubscribeEndpoints = UnsupportedEndpoints;
    type RequestResponseEndpoints = UnsupportedEndpoints;
    type EventEndpoints = UnsupportedEndpoints;

    fn generation(&self) -> Generation {
        self.generation.current()
    }

    fn endpoints(
        &self,
        callback: &mut dyn FnMut(&EndpointDescription),
    ) -> Result<(), Self::ListError> {
        let origin = origin!("RmwZenohAdapter::endpoints");

        let own = self.session.zid().to_string();
        let mut descriptions = Vec::new();
        {
            let graph = fail!(
                from origin,
                when self.graph.lock(),
                with ListError::Graph,
                "Failed to lock the poisoned graph"
            );

            for (name, entities) in graph.topics(|token| token.zid != own) {
                if INTERNAL_TOPICS.contains(&name) {
                    continue;
                }
                let Some((type_name, entity)) = single_type(name, &entities) else {
                    continue;
                };
                descriptions.push(EndpointDescription {
                    settings: EndpointSettings {
                        name: EndpointName::Topic(name.to_string()),
                        qos: entity.qos,
                    },
                    types: EndpointTypes::PublishSubscribe(MessageType {
                        type_name: type_name.ros(),
                        type_hash: entity.type_hash.clone(),
                    }),
                });
            }

            for (name, entities) in graph.services(|token| token.zid != own) {
                let Some((type_name, entity)) = single_type(name, &entities) else {
                    continue;
                };
                let ros_type_name = type_name.ros();
                if INTERNAL_SERVICE_TYPES
                    .iter()
                    .any(|prefix| ros_type_name.starts_with(prefix))
                {
                    continue;
                }
                descriptions.push(EndpointDescription {
                    settings: EndpointSettings {
                        name: EndpointName::Service(name.to_string()),
                        qos: entity.qos,
                    },
                    types: EndpointTypes::RequestResponse {
                        request: MessageType {
                            type_name: type_name.request(),
                            type_hash: entity.type_hash.clone(),
                        },
                        response: MessageType {
                            type_name: type_name.response(),
                            type_hash: entity.type_hash.clone(),
                        },
                    },
                });
            }
        }

        for description in &descriptions {
            callback(description);
        }
        Ok(())
    }

    fn publish_subscribe(
        &mut self,
        _description: &EndpointDescription,
    ) -> Result<Self::PublishSubscribeEndpoints, Self::OpenError> {
        Err(OpenError::UnsupportedPattern)
    }

    fn request_response(
        &mut self,
        _description: &EndpointDescription,
    ) -> Result<Self::RequestResponseEndpoints, Self::OpenError> {
        Err(OpenError::UnsupportedPattern)
    }

    fn event(
        &mut self,
        _description: &EndpointDescription,
    ) -> Result<Self::EventEndpoints, Self::OpenError> {
        Err(OpenError::UnsupportedPattern)
    }
}
