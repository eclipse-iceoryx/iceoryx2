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

pub mod active_request;
pub mod alignment;
pub mod allocation_strategy;
pub mod attribute;
pub mod attribute_key;
pub mod attribute_set;
pub mod attribute_specifier;
pub mod attribute_value;
pub mod attribute_verifier;
pub mod cleanup_state;
pub mod client;
pub mod config;
pub mod duration;
pub mod error;
pub mod event_id;
pub mod file_descriptor;
pub mod file_name;
pub mod file_path;
pub mod header_publish_subscribe;
pub mod listener;
pub mod log;
pub mod log_level;
pub mod message_type_details;
pub mod messaging_pattern;
pub mod node;
pub mod node_builder;
pub mod node_id;
pub mod node_name;
pub mod node_state;
pub mod notifier;
pub mod parc;
pub mod path;
pub mod pending_response;
pub mod port_factory_client;
pub mod port_factory_event;
pub mod port_factory_listener;
pub mod port_factory_notifier;
pub mod port_factory_publish_subscribe;
pub mod port_factory_publisher;
pub mod port_factory_request_response;
pub mod port_factory_server;
pub mod port_factory_subscriber;
pub mod publisher;
pub mod request_header;
pub mod request_mut;
pub mod request_mut_uninit;
pub mod response;
pub mod response_header;
pub mod response_mut;
pub mod response_mut_uninit;
pub mod sample;
pub mod sample_mut;
pub mod sample_mut_uninit;
pub mod server;
pub mod service;
pub mod service_builder;
pub mod service_builder_event;
pub mod service_builder_publish_subscribe;
pub mod service_builder_request_response;
pub mod service_details;
pub mod service_id;
pub mod service_name;
pub mod service_type;
pub mod signal_handling_mode;
pub mod static_config_event;
pub mod static_config_publish_subscribe;
pub mod static_config_request_response;
pub mod subscriber;
pub mod testing;
pub mod type_detail;
pub mod type_name;
pub mod type_storage;
pub mod type_variant;
pub mod unable_to_deliver_strategy;
pub mod unique_client_id;
pub mod unique_listener_id;
pub mod unique_notifier_id;
pub mod unique_publisher_id;
pub mod unique_server_id;
pub mod unique_subscriber_id;
pub mod waitset;
pub mod waitset_attachment_id;
pub mod waitset_builder;
pub mod waitset_guard;
pub mod waitset_run_result;

use pyo3::prelude::*;
use pyo3::wrap_pymodule;

pub(crate) use service_type::IpcService;
pub(crate) use service_type::LocalService;

/// iceoryx2 Python language bindings
#[pymodule]
fn _iceoryx2(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(crate::config::config))?;
    m.add_wrapped(wrap_pymodule!(crate::testing::testing))?;
    m.add_wrapped(wrap_pyfunction!(crate::log::set_log_level))?;
    m.add_wrapped(wrap_pyfunction!(crate::log::set_log_level_from_env_or))?;
    m.add_wrapped(wrap_pyfunction!(
        crate::log::set_log_level_from_env_or_default
    ))?;

    m.add_class::<crate::active_request::ActiveRequest>()?;
    m.add_class::<crate::alignment::Alignment>()?;
    m.add_class::<crate::allocation_strategy::AllocationStrategy>()?;
    m.add_class::<crate::attribute::Attribute>()?;
    m.add_class::<crate::attribute_set::AttributeSet>()?;
    m.add_class::<crate::attribute_verifier::AttributeVerifier>()?;
    m.add_class::<crate::attribute_specifier::AttributeSpecifier>()?;
    m.add_class::<crate::attribute_key::AttributeKey>()?;
    m.add_class::<crate::attribute_value::AttributeValue>()?;
    m.add_class::<crate::client::Client>()?;
    m.add_class::<crate::duration::Duration>()?;
    m.add_class::<crate::event_id::EventId>()?;
    m.add_class::<crate::file_name::FileName>()?;
    m.add_class::<crate::file_path::FilePath>()?;
    m.add_class::<crate::header_publish_subscribe::HeaderPublishSubscribe>()?;
    m.add_class::<crate::listener::Listener>()?;
    m.add_class::<crate::log_level::LogLevel>()?;
    m.add_class::<crate::messaging_pattern::MessagingPattern>()?;
    m.add_class::<crate::message_type_details::MessageTypeDetails>()?;
    m.add_class::<crate::node::Node>()?;
    m.add_class::<crate::node_builder::NodeBuilder>()?;
    m.add_class::<crate::node_id::NodeId>()?;
    m.add_class::<crate::node_name::NodeName>()?;
    m.add_class::<crate::node_state::NodeState>()?;
    m.add_class::<crate::node_state::AliveNodeView>()?;
    m.add_class::<crate::node_state::DeadNodeView>()?;
    m.add_class::<crate::node_state::NodeDetails>()?;
    m.add_class::<crate::notifier::Notifier>()?;
    m.add_class::<crate::path::Path>()?;
    m.add_class::<crate::pending_response::PendingResponse>()?;
    m.add_class::<crate::port_factory_client::PortFactoryClient>()?;
    m.add_class::<crate::port_factory_event::PortFactoryEvent>()?;
    m.add_class::<crate::port_factory_listener::PortFactoryListener>()?;
    m.add_class::<crate::port_factory_notifier::PortFactoryNotifier>()?;
    m.add_class::<crate::port_factory_publisher::PortFactoryPublisher>()?;
    m.add_class::<crate::port_factory_publish_subscribe::PortFactoryPublishSubscribe>()?;
    m.add_class::<crate::port_factory_request_response::PortFactoryRequestResponse>()?;
    m.add_class::<crate::port_factory_server::PortFactoryServer>()?;
    m.add_class::<crate::port_factory_subscriber::PortFactorySubscriber>()?;
    m.add_class::<crate::publisher::Publisher>()?;
    m.add_class::<crate::request_header::RequestHeader>()?;
    m.add_class::<crate::request_mut::RequestMut>()?;
    m.add_class::<crate::request_mut_uninit::RequestMutUninit>()?;
    m.add_class::<crate::response::Response>()?;
    m.add_class::<crate::response_header::ResponseHeader>()?;
    m.add_class::<crate::response_mut::ResponseMut>()?;
    m.add_class::<crate::response_mut_uninit::ResponseMutUninit>()?;
    m.add_class::<crate::sample::Sample>()?;
    m.add_class::<crate::sample_mut::SampleMut>()?;
    m.add_class::<crate::sample_mut_uninit::SampleMutUninit>()?;
    m.add_class::<crate::server::Server>()?;
    m.add_class::<crate::service::Service>()?;
    m.add_class::<crate::service_builder::ServiceBuilder>()?;
    m.add_class::<crate::service_builder_event::ServiceBuilderEvent>()?;
    m.add_class::<crate::service_builder_publish_subscribe::ServiceBuilderPublishSubscribe>()?;
    m.add_class::<crate::service_builder_request_response::ServiceBuilderRequestResponse>()?;
    m.add_class::<crate::service_details::ServiceDetails>()?;
    m.add_class::<crate::service_id::ServiceId>()?;
    m.add_class::<crate::service_name::ServiceName>()?;
    m.add_class::<crate::service_type::ServiceType>()?;
    m.add_class::<crate::signal_handling_mode::SignalHandlingMode>()?;
    m.add_class::<crate::static_config_event::StaticConfigEvent>()?;
    m.add_class::<crate::static_config_publish_subscribe::StaticConfigPublishSubscribe>()?;
    m.add_class::<crate::static_config_request_response::StaticConfigRequestResponse>()?;
    m.add_class::<crate::subscriber::Subscriber>()?;
    m.add_class::<crate::type_detail::TypeDetail>()?;
    m.add_class::<crate::type_variant::TypeVariant>()?;
    m.add_class::<crate::type_name::TypeName>()?;
    m.add_class::<crate::unable_to_deliver_strategy::UnableToDeliverStrategy>()?;
    m.add_class::<crate::unique_client_id::UniqueClientId>()?;
    m.add_class::<crate::unique_listener_id::UniqueListenerId>()?;
    m.add_class::<crate::unique_notifier_id::UniqueNotifierId>()?;
    m.add_class::<crate::unique_publisher_id::UniquePublisherId>()?;
    m.add_class::<crate::unique_server_id::UniqueServerId>()?;
    m.add_class::<crate::unique_subscriber_id::UniqueSubscriberId>()?;
    m.add_class::<crate::waitset::WaitSet>()?;
    m.add_class::<crate::waitset_attachment_id::WaitSetAttachmentId>()?;
    m.add_class::<crate::waitset_builder::WaitSetBuilder>()?;
    m.add_class::<crate::waitset_guard::WaitSetGuard>()?;
    m.add_class::<crate::waitset_run_result::WaitSetRunResult>()?;

    m.add(
        "ClientCreateError",
        py.get_type::<crate::error::ClientCreateError>(),
    )?;
    m.add(
        "ConfigCreationError",
        py.get_type::<crate::error::ConfigCreationError>(),
    )?;
    m.add(
        "ConnectionFailure",
        py.get_type::<crate::error::ConnectionFailure>(),
    )?;
    m.add(
        "EventOpenError",
        py.get_type::<crate::error::EventOpenError>(),
    )?;
    m.add(
        "EventCreateError",
        py.get_type::<crate::error::EventCreateError>(),
    )?;
    m.add(
        "EventOpenOrCreateError",
        py.get_type::<crate::error::EventOpenOrCreateError>(),
    )?;
    m.add(
        "InvalidAlignmentValue",
        py.get_type::<crate::error::InvalidAlignmentValue>(),
    )?;
    m.add("LoanError", py.get_type::<crate::error::LoanError>())?;
    m.add(
        "ListenerCreateError",
        py.get_type::<crate::error::ListenerCreateError>(),
    )?;
    m.add(
        "ListenerWaitError",
        py.get_type::<crate::error::ListenerWaitError>(),
    )?;
    m.add(
        "NodeCreationFailure",
        py.get_type::<crate::error::NodeCreationFailure>(),
    )?;
    m.add(
        "NodeCleanupFailure",
        py.get_type::<crate::error::NodeCleanupFailure>(),
    )?;
    m.add(
        "NodeListFailure",
        py.get_type::<crate::error::NodeListFailure>(),
    )?;
    m.add(
        "NodeWaitFailure",
        py.get_type::<crate::error::NodeWaitFailure>(),
    )?;
    m.add(
        "NotifierCreateError",
        py.get_type::<crate::error::NotifierCreateError>(),
    )?;
    m.add(
        "NotifierNotifyError",
        py.get_type::<crate::error::NotifierNotifyError>(),
    )?;
    m.add("SendError", py.get_type::<crate::error::SendError>())?;
    m.add(
        "SemanticStringError",
        py.get_type::<crate::error::SemanticStringError>(),
    )?;
    m.add(
        "PublisherCreateError",
        py.get_type::<crate::error::PublisherCreateError>(),
    )?;
    m.add(
        "PublishSubscribeOpenError",
        py.get_type::<crate::error::PublishSubscribeOpenError>(),
    )?;
    m.add(
        "PublishSubscribeCreateError",
        py.get_type::<crate::error::PublishSubscribeCreateError>(),
    )?;
    m.add(
        "PublishSubscribeOpenOrCreateError",
        py.get_type::<crate::error::PublishSubscribeOpenOrCreateError>(),
    )?;
    m.add("ReceiveError", py.get_type::<crate::error::ReceiveError>())?;
    m.add(
        "RequestResponseOpenError",
        py.get_type::<crate::error::RequestResponseOpenError>(),
    )?;
    m.add(
        "RequestResponseCreateError",
        py.get_type::<crate::error::RequestResponseCreateError>(),
    )?;
    m.add(
        "RequestResponseOpenOrCreateError",
        py.get_type::<crate::error::RequestResponseOpenOrCreateError>(),
    )?;
    m.add(
        "RequestSendError",
        py.get_type::<crate::error::RequestSendError>(),
    )?;
    m.add(
        "ServerCreateError",
        py.get_type::<crate::error::ServerCreateError>(),
    )?;
    m.add(
        "ServiceDetailsError",
        py.get_type::<crate::error::ServiceDetailsError>(),
    )?;
    m.add(
        "SubscriberCreateError",
        py.get_type::<crate::error::SubscriberCreateError>(),
    )?;
    m.add(
        "WaitSetAttachmentError",
        py.get_type::<crate::error::WaitSetAttachmentError>(),
    )?;
    m.add(
        "WaitSetCreateError",
        py.get_type::<crate::error::WaitSetCreateError>(),
    )?;
    m.add(
        "WaitSetRunError",
        py.get_type::<crate::error::WaitSetRunError>(),
    )?;

    Ok(())
}
