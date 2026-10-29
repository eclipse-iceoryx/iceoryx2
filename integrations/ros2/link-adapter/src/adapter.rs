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
use std::rc::Rc;
use std::sync::{Arc, Mutex, Once};

use iceoryx2::service::messaging_pattern::MessagingPattern;

use iceoryx2_link_adapter::{Adapter, EndpointTypes, UnsupportedEndpoints};
use iceoryx2_link_backend::{Reactive, WakeHandle};
use iceoryx2_log::{fail, origin, warn};

use crate::config::Config;
use crate::config::{NameError, TypeName};
use crate::endpoint_description::{
    EndpointDescription, EndpointName, EndpointSettings, MessageType,
};
use crate::endpoints;
use crate::qos::QosProfile;
use crate::rcl::{
    self, EndpointInfo, GraphListener, RclClientBuilder, RclNode, RclNodeBuilder,
    RclPublisherBuilder, RclServiceBuilder, RclSubscriptionBuilder, TopicName,
};
use crate::ros_message::RosMessage;
use crate::typesupport;

const SUPPORTS_PUBLISH_SUBSCRIBE: [&str; 3] = [
    "rmw_fastrtps_cpp",
    "rmw_fastrtps_dynamic_cpp",
    "rmw_cyclonedds_cpp",
];
const SUPPORTS_REQUEST_RESPONSE: [&str; 2] = ["rmw_fastrtps_cpp", "rmw_fastrtps_dynamic_cpp"];

/// Warns once per process that the rmw does not propagate request-response.
static UNSUPPORTED_RMW_WARNING: Once = Once::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreationError {
    Node,
    TypeSupport,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListError {
    Graph,
}

impl core::fmt::Display for ListError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ListError::{self:?}")
    }
}

impl core::error::Error for ListError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenError {
    TypeSupport,
    Publisher,
    Subscription,
    Client,
    Service,
    Callback,
    UnsupportedPattern,
    UnsupportedRmw,
}

impl core::fmt::Display for OpenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "OpenError::{self:?}")
    }
}

impl core::error::Error for OpenError {}

/// Adapter for bridging between ROS 2 and iceoryx2.
pub struct Ros2Adapter {
    wake: Arc<Mutex<Option<WakeHandle>>>,
    /// Wakes the link on graph changes.
    graph_listener: Option<GraphListener>,
    pub(crate) node: Rc<RclNode>,
}

impl Ros2Adapter {
    /// Whether the adapter propagates services of `pattern` on the rmw the
    /// process runs on.
    pub fn supports(pattern: MessagingPattern) -> bool {
        let rmws: &[&str] = match pattern {
            MessagingPattern::PublishSubscribe => &SUPPORTS_PUBLISH_SUBSCRIBE,
            MessagingPattern::RequestResponse => &SUPPORTS_REQUEST_RESPONSE,
            _ => &[],
        };
        rmws.contains(&rcl::rmw::implementation_identifier())
    }

    /// Creates the adapter's node and preloads the typesupport for types
    /// specified in the `config`.
    pub fn new(config: &Config) -> Result<Self, CreationError> {
        let origin = origin!("Ros2Adapter::new");

        let node = fail!(
            from origin,
            when RclNodeBuilder::new(rcl::NodeName::from(&config.node_name))
                .rosout(config.rosout)
                .create(),
            with CreationError::Node,
            "Failed to create the ROS 2 node"
        );

        if !Self::supports(MessagingPattern::RequestResponse) {
            UNSUPPORTED_RMW_WARNING.call_once(|| {
                warn!(
                    "Request-response services are not propagated on {}, only on rmw_fastrtps",
                    rcl::rmw::implementation_identifier()
                );
            });
        }

        for type_name in &config.preload_types {
            fail!(
                from origin,
                when typesupport::preload(type_name.as_str()),
                with CreationError::TypeSupport,
                "Failed to preload typesupport for type '{}'", type_name.as_str()
            );
        }

        Ok(Self {
            wake: Arc::new(Mutex::new(None)),
            graph_listener: None,
            node: Rc::new(node),
        })
    }
}

/// Whether a topic belongs to the topics every ROS 2 node publishes on.
fn is_internal_topic(topic: &TopicName) -> bool {
    const INTERNAL_TOPICS: [&str; 2] = ["/rosout", "/parameter_events"];
    INTERNAL_TOPICS.contains(&topic.as_str())
}

/// Whether a service type belongs to the services every ROS 2 node serves.
fn is_internal_service(type_name: &rcl::TypeName) -> bool {
    const INTERNAL_PACKAGES: [&str; 2] =
        ["rcl_interfaces/srv/", "type_description_interfaces/srv/"];
    INTERNAL_PACKAGES
        .iter()
        .any(|package| type_name.as_str().starts_with(package))
}

/// Whether `endpoint` belongs to the node named `own`.
fn is_own(own: &rcl::NodeName, endpoint: &EndpointInfo) -> bool {
    own.as_str() == endpoint.node_name
}

/// The request type of the ROS 2 service with the type `srv`.
fn request_type(srv: &rcl::TypeName) -> Result<TypeName, NameError> {
    TypeName::new(&format!("{}_Request", srv.as_str()))
}

/// The response type of the ROS 2 service with the type `srv`.
fn response_type(srv: &rcl::TypeName) -> Result<TypeName, NameError> {
    TypeName::new(&format!("{}_Response", srv.as_str()))
}

impl Ros2Adapter {
    /// Lists the topics of the graph that another node publishes or
    /// subscribes to.
    fn list_topics(&self, callback: &mut dyn FnMut(&EndpointDescription)) -> Result<(), ListError> {
        let origin = origin!("Ros2Adapter::list_topics");

        let graph = fail!(
            from origin,
            when self.node.topic_names_and_types(),
            with ListError::Graph,
            "Failed to query the topics of the graph"
        );

        let own = self.node.fully_qualified_name();
        for (topic, types) in graph {
            // ROS 2's own topics have no consumer on the iceoryx2 side.
            if is_internal_topic(&topic) {
                continue;
            }

            // A topic with several types has no single C struct to translate
            // so is skipped.
            let [type_name] = types.as_slice() else {
                warn!(
                    "Topic '{}' has {} types and is not listed",
                    topic.as_str(),
                    types.len()
                );
                continue;
            };

            // List all publisher and subscribers. Own endpoints are filtered
            // out.
            let publishers = fail!(
                from origin,
                when self.node.publishers(&topic),
                with ListError::Graph,
                "Failed to query the publishers of topic '{}'", topic.as_str()
            );
            let subscriptions = fail!(
                from origin,
                when self.node.subscriptions(&topic),
                with ListError::Graph,
                "Failed to query the subscriptions of topic '{}'", topic.as_str()
            );
            let Some(remote) = publishers
                .iter()
                .chain(subscriptions.iter())
                .find(|endpoint| !is_own(own.name(), endpoint))
            else {
                continue;
            };

            // Hand the descriptions to the gateway.
            callback(&EndpointDescription {
                settings: EndpointSettings {
                    name: EndpointName::Topic(topic.clone().into()),
                    qos: remote.qos.clone(),
                },
                types: EndpointTypes::PublishSubscribe(MessageType {
                    type_name: type_name.clone().into(),
                }),
            });
        }

        Ok(())
    }

    /// Lists the services of the graph that another node serves or calls.
    ///
    /// The graph does not tell a service's QoS. Every ROS 2 service runs on
    /// the default profile.
    fn list_services(
        &self,
        callback: &mut dyn FnMut(&EndpointDescription),
    ) -> Result<(), ListError> {
        let origin = origin!("Ros2Adapter::list_services");

        if !Self::supports(MessagingPattern::RequestResponse) {
            return Ok(());
        }

        let graph = fail!(
            from origin,
            when self.node.service_names_and_types(),
            with ListError::Graph,
            "Failed to query the services of the graph"
        );

        // A service is listed when a node other than the gateway's serves or
        // calls it. The graph query does not say which nodes do, so every
        // other node must be asked for its services explicitly.
        //
        // Counting the servers and clients of a service is only available
        // from `jazzy` or later.
        let own = self.node.fully_qualified_name();
        let nodes = fail!(
            from origin,
            when self.node.nodes(),
            with ListError::Graph,
            "Failed to query the nodes of the graph"
        );

        let mut remote = BTreeSet::new();
        for node in nodes.iter().filter(|node| **node != own) {
            remote.extend(fail!(
                from origin,
                when self.node.servers_of(node),
                with ListError::Graph,
                "Failed to query the servers of node {}", node
            ));
            remote.extend(fail!(
                from origin,
                when self.node.clients_of(node),
                with ListError::Graph,
                "Failed to query the clients of node {}", node
            ));
        }

        for (service, types) in graph {
            // A service with several types has no single pair of C structs
            // to translate so is skipped.
            let [service_type] = types.as_slice() else {
                warn!(
                    "Service '{}' has {} types and is not listed",
                    service.as_str(),
                    types.len()
                );
                continue;
            };

            // ROS 2's own services have no consumer on the iceoryx2 side.
            if is_internal_service(service_type) {
                continue;
            }

            if !remote.contains(&service) {
                continue;
            }

            let (Ok(request), Ok(response)) =
                (request_type(service_type), response_type(service_type))
            else {
                warn!(
                    "Service '{}' has the type '{}' which names no request and response types and is not listed",
                    service.as_str(),
                    service_type.as_str()
                );
                continue;
            };

            // Hand the descriptions to the gateway.
            callback(&EndpointDescription {
                settings: EndpointSettings {
                    name: EndpointName::Service(service.into()),
                    qos: QosProfile::default(),
                },
                types: EndpointTypes::RequestResponse {
                    request: MessageType { type_name: request },
                    response: MessageType {
                        type_name: response,
                    },
                },
            });
        }

        Ok(())
    }
}

impl Adapter for Ros2Adapter {
    type ListError = ListError;
    type OpenError = OpenError;
    type EndpointSettings = EndpointSettings;
    type EndpointTypes = EndpointTypes<MessageType>;
    type PublishSubscribeEndpoints = endpoints::PublishSubscribeEndpoints;
    type RequestResponseEndpoints = endpoints::RequestResponseEndpoints;
    type EventEndpoints = UnsupportedEndpoints;

    fn endpoints(
        &self,
        callback: &mut dyn FnMut(&EndpointDescription),
    ) -> Result<(), Self::ListError> {
        self.list_topics(callback)?;
        self.list_services(callback)
    }

    fn publish_subscribe(
        &mut self,
        description: &EndpointDescription,
    ) -> Result<Self::PublishSubscribeEndpoints, Self::OpenError> {
        let origin = origin!("Ros2Adapter::publish_subscribe");

        let (EndpointName::Topic(topic), EndpointTypes::PublishSubscribe(types)) =
            (&description.settings.name, &description.types)
        else {
            fail!(
                from origin,
                with OpenError::UnsupportedPattern,
                "'{}' is not a publish-subscribe topic", description.settings.name
            );
        };

        // Get the typesupport.
        let type_name = types.type_name.as_str();
        let type_support = fail!(
            from origin,
            when typesupport::get::<typesupport::MessageKind>(type_name),
            with OpenError::TypeSupport,
            "Failed to load typesupport for type '{}'", type_name
        );

        // Create the endpoints.
        let topic = topic.into();
        let publisher = fail!(
            from origin,
            when RclPublisherBuilder::new(Rc::clone(&self.node), &topic, Rc::clone(&type_support))
                .qos(description.settings.qos.clone())
                .create(),
            with OpenError::Publisher,
            "Failed to create the publisher on topic '{}'", topic.as_str()
        );
        let mut subscription = fail!(
            from origin,
            when RclSubscriptionBuilder::new(Rc::clone(&self.node), &topic, type_support)
                .qos(description.settings.qos.clone())
                .create(),
            with OpenError::Subscription,
            "Failed to create the subscription on topic '{}'", topic.as_str()
        );

        // Wake the link on every message received if a wake is attached.
        let wake = Arc::clone(&self.wake);
        fail!(
            from origin,
            when subscription.on_new_message(Box::new(move |_| signal(&wake))),
            with OpenError::Callback,
            "Failed to register the new-message callback on topic '{}'", topic.as_str()
        );

        Ok(endpoints::PublishSubscribeEndpoints::new(
            publisher,
            subscription,
        ))
    }

    fn request_response(
        &mut self,
        description: &EndpointDescription,
    ) -> Result<Self::RequestResponseEndpoints, Self::OpenError> {
        let origin = origin!("Ros2Adapter::request_response");

        if !Self::supports(MessagingPattern::RequestResponse) {
            fail!(
                from origin,
                with OpenError::UnsupportedRmw,
                "Request-response services are not propagated on {}", rcl::rmw::implementation_identifier()
            );
        }

        let (EndpointName::Service(service), EndpointTypes::RequestResponse { request, response }) =
            (&description.settings.name, &description.types)
        else {
            fail!(
                from origin,
                with OpenError::UnsupportedPattern,
                "'{}' is not a request-response service", description.settings.name
            );
        };

        // Get the typesupport. The service's type is the request type
        // without its suffix.
        let Some(service_type) = request.type_name.as_str().strip_suffix("_Request") else {
            fail!(
                from origin,
                with OpenError::TypeSupport,
                "The type '{}' is not the request type of a service", request.type_name.as_str()
            );
        };
        let type_support = fail!(
            from origin,
            when typesupport::get::<typesupport::ServiceKind>(service_type),
            with OpenError::TypeSupport,
            "Failed to load typesupport for service type '{}'", service_type
        );
        let request = fail!(
            from origin,
            when RosMessage::new(request.type_name.as_str()),
            with OpenError::TypeSupport,
            "Failed to load typesupport for type '{}'", request.type_name.as_str()
        );
        let response = fail!(
            from origin,
            when RosMessage::new(response.type_name.as_str()),
            with OpenError::TypeSupport,
            "Failed to load typesupport for type '{}'", response.type_name.as_str()
        );

        // Create the endpoints.
        let service_name = rcl::ServiceName::from(service);
        let mut client = fail!(
            from origin,
            when RclClientBuilder::new(Rc::clone(&self.node), &service_name, Rc::clone(&type_support))
                .qos(description.settings.qos.clone())
                .create(),
            with OpenError::Client,
            "Failed to create the client on service '{}'", service_name.as_str()
        );
        let mut server = fail!(
            from origin,
            when RclServiceBuilder::new(Rc::clone(&self.node), &service_name, type_support)
                .qos(description.settings.qos.clone())
                .create(),
            with OpenError::Service,
            "Failed to create the server on service '{}'", service_name.as_str()
        );

        // Wake the link on every response and request received if a wake is
        // attached.
        let wake = Arc::clone(&self.wake);
        fail!(
            from origin,
            when client.on_new_response(Box::new(move |_| signal(&wake))),
            with OpenError::Callback,
            "Failed to register the new-response callback on service '{}'", service_name.as_str()
        );
        let wake = Arc::clone(&self.wake);
        fail!(
            from origin,
            when server.on_new_request(Box::new(move |_| signal(&wake))),
            with OpenError::Callback,
            "Failed to register the new-request callback on service '{}'", service_name.as_str()
        );

        Ok(endpoints::RequestResponseEndpoints::new(
            client, server, request, response,
        ))
    }

    fn event(
        &mut self,
        description: &EndpointDescription,
    ) -> Result<Self::EventEndpoints, Self::OpenError> {
        let origin = origin!("Ros2Adapter::event");

        fail!(
            from origin,
            with OpenError::UnsupportedPattern,
            "ROS 2 has no events, '{}' cannot be opened as one", description.settings.name
        );
    }
}

/// Signals the attached wake, if any. Called from an rcl thread.
fn signal(wake: &Mutex<Option<WakeHandle>>) {
    if let Ok(wake) = wake.lock()
        && let Some(wake) = wake.as_ref()
    {
        wake.signal();
    }
}

impl Reactive for Ros2Adapter {
    fn attach(&mut self, wake: WakeHandle) {
        let origin = origin!("Ros2Adapter::attach");

        match self.wake.lock() {
            Ok(mut attached) => *attached = Some(wake),
            Err(_) => warn!(
                from origin,
                "The wake handle mutex is poisoned, the link cannot be woken by the adapter"
            ),
        }

        let wake = Arc::clone(&self.wake);
        match GraphListener::spawn(&self.node, Box::new(move || signal(&wake))) {
            Ok(listener) => self.graph_listener = Some(listener),
            Err(error) => warn!(
                from origin,
                "The link is not woken on graph changes, the graph listener failed: {}", error
            ),
        }
    }
}
