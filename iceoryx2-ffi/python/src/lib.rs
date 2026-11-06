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
pub mod backpressure_strategy;
pub mod cleanup_state;
pub mod client;
pub mod config;
pub mod duration;
pub mod dynamic_config_blackboard;
pub mod dynamic_config_event;
pub mod dynamic_config_publish_subscribe;
pub mod dynamic_config_request_response;
pub mod entry_handle;
pub mod entry_handle_mut;
pub mod entry_value_uninit;
pub mod error;
pub mod event_activation;
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
pub mod node_name;
pub mod node_state;
pub mod notifier;
pub mod parc;
pub mod path;
pub mod pending_response;
pub mod port_factory_blackboard;
pub mod port_factory_client;
pub mod port_factory_event;
pub mod port_factory_listener;
pub mod port_factory_notifier;
pub mod port_factory_publish_subscribe;
pub mod port_factory_publisher;
pub mod port_factory_reader;
pub mod port_factory_request_response;
pub mod port_factory_server;
pub mod port_factory_subscriber;
pub mod port_factory_writer;
pub mod port_name;
pub mod publisher;
pub mod reader;
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
pub mod service_builder_blackboard;
pub mod service_builder_event;
pub mod service_builder_publish_subscribe;
pub mod service_builder_request_response;
pub mod service_details;
pub mod service_hash;
pub mod service_name;
pub mod service_type;
pub mod signal_handling_mode;
pub mod static_config_blackboard;
pub mod static_config_event;
pub mod static_config_publish_subscribe;
pub mod static_config_request_response;
pub mod subscriber;
pub mod testing;
pub mod type_detail;
pub mod type_name;
pub mod type_storage;
pub mod type_variant;
pub mod unique_client_id;
pub mod unique_listener_id;
pub mod unique_node_id;
pub mod unique_notifier_id;
pub mod unique_publisher_id;
pub mod unique_reader_id;
pub mod unique_server_id;
pub mod unique_service_id;
pub mod unique_subscriber_id;
pub mod unique_writer_id;
pub mod waitset;
pub mod waitset_attachment_id;
pub mod waitset_builder;
pub mod waitset_guard;
pub mod waitset_run_result;
pub mod writer;

use pyo3::prelude::*;

pub(crate) use service_type::IpcService;
pub(crate) use service_type::LocalService;

/// Registers a submodule of the package in `sys.modules` so that it can be imported under its
/// dotted name.
fn make_submodule_importable(py: Python<'_>, m: &Bound<'_, PyModule>, name: &str) -> PyResult<()> {
    let qualified_name = format!("iceoryx2.{name}");
    let submodule = m.getattr(name)?;
    submodule.setattr("__name__", &qualified_name)?;
    py.import("sys")?
        .getattr("modules")?
        .set_item(&qualified_name, submodule)
}

/// iceoryx2 Python language bindings
#[pymodule]
mod _iceoryx2 {
    use pyo3::prelude::*;

    #[pymodule_export]
    use crate::{
        active_request::ActiveRequest,
        alignment::Alignment,
        allocation_strategy::AllocationStrategy,
        attribute::Attribute,
        attribute_key::AttributeKey,
        attribute_set::AttributeSet,
        attribute_specifier::AttributeSpecifier,
        attribute_value::AttributeValue,
        attribute_verifier::AttributeVerifier,
        backpressure_strategy::BackpressureStrategy,
        client::Client,
        config::config,
        duration::Duration,
        dynamic_config_blackboard::DynamicConfigBlackboard,
        dynamic_config_event::DynamicConfigEvent,
        dynamic_config_publish_subscribe::DynamicConfigPublishSubscribe,
        dynamic_config_request_response::DynamicConfigRequestResponse,
        entry_handle::EntryHandle,
        entry_handle_mut::EntryHandleMut,
        entry_value_uninit::EntryValueUninit,
        error::{
            AllocationGrowError, BlackboardCreateError, BlackboardOpenError, ClientCreateError,
            ConfigCreationError, ConnectionFailure, EntryHandleError, EntryHandleMutError,
            EventCreateError, EventOpenError, EventOpenOrCreateError, InvalidAlignmentValue,
            ListenerCreateError, ListenerWaitError, LoanError, NodeCleanupFailure,
            NodeCreationFailure, NodeListFailure, NodeWaitFailure, NotifierCreateError,
            NotifierNotifyError, PublishSubscribeCreateError, PublishSubscribeOpenError,
            PublishSubscribeOpenOrCreateError, PublisherCreateError, ReaderCreateError,
            ReceiveError, RequestResponseCreateError, RequestResponseOpenError,
            RequestResponseOpenOrCreateError, RequestSendError, SemanticStringError, SendError,
            ServerCreateError, ServiceDetailsError, ServiceRemoveError, SubscriberCreateError,
            WaitSetAttachmentError, WaitSetCreateError, WaitSetRunError, WriterCreateError,
        },
        event_activation::EventActivation,
        event_id::EventId,
        file_descriptor::FileDescriptor,
        file_name::FileName,
        file_path::FilePath,
        header_publish_subscribe::HeaderPublishSubscribe,
        listener::Listener,
        log::{set_log_level, set_log_level_from_env_or, set_log_level_from_env_or_default},
        log_level::LogLevel,
        message_type_details::MessageTypeDetails,
        messaging_pattern::MessagingPattern,
        node::Node,
        node_builder::NodeBuilder,
        node_name::NodeName,
        node_state::AliveNodeView,
        node_state::DeadNodeView,
        node_state::NodeDetails,
        node_state::NodeState,
        notifier::Notifier,
        path::Path,
        pending_response::PendingResponse,
        port_factory_blackboard::PortFactoryBlackboard,
        port_factory_client::PortFactoryClient,
        port_factory_event::PortFactoryEvent,
        port_factory_listener::PortFactoryListener,
        port_factory_notifier::PortFactoryNotifier,
        port_factory_publish_subscribe::PortFactoryPublishSubscribe,
        port_factory_publisher::PortFactoryPublisher,
        port_factory_reader::PortFactoryReader,
        port_factory_request_response::PortFactoryRequestResponse,
        port_factory_server::PortFactoryServer,
        port_factory_subscriber::PortFactorySubscriber,
        port_factory_writer::PortFactoryWriter,
        port_name::PortName,
        publisher::Publisher,
        reader::Reader,
        request_header::RequestHeader,
        request_mut::RequestMut,
        request_mut_uninit::RequestMutUninit,
        response::Response,
        response_header::ResponseHeader,
        response_mut::ResponseMut,
        response_mut_uninit::ResponseMutUninit,
        sample::Sample,
        sample_mut::SampleMut,
        sample_mut_uninit::SampleMutUninit,
        server::Server,
        service::Service,
        service_builder::ServiceBuilder,
        service_builder_blackboard::ServiceBuilderBlackboardCreator,
        service_builder_blackboard::ServiceBuilderBlackboardOpener,
        service_builder_event::ServiceBuilderEvent,
        service_builder_publish_subscribe::ServiceBuilderPublishSubscribe,
        service_builder_request_response::ServiceBuilderRequestResponse,
        service_details::ServiceDetails,
        service_hash::ServiceHash,
        service_name::ServiceName,
        service_type::ServiceType,
        signal_handling_mode::SignalHandlingMode,
        static_config_blackboard::StaticConfigBlackboard,
        static_config_event::StaticConfigEvent,
        static_config_publish_subscribe::StaticConfigPublishSubscribe,
        static_config_request_response::StaticConfigRequestResponse,
        subscriber::Subscriber,
        testing::testing,
        type_detail::TypeDetail,
        type_name::TypeName,
        type_variant::TypeVariant,
        unique_client_id::UniqueClientId,
        unique_listener_id::UniqueListenerId,
        unique_node_id::UniqueNodeId,
        unique_notifier_id::UniqueNotifierId,
        unique_publisher_id::UniquePublisherId,
        unique_reader_id::UniqueReaderId,
        unique_server_id::UniqueServerId,
        unique_service_id::UniqueServiceId,
        unique_subscriber_id::UniqueSubscriberId,
        unique_writer_id::UniqueWriterId,
        waitset::WaitSet,
        waitset_attachment_id::WaitSetAttachmentId,
        waitset_builder::WaitSetBuilder,
        waitset_guard::WaitSetGuard,
        waitset_run_result::WaitSetRunResult,
        writer::Writer,
    };

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        let py = m.py();
        pyo3_log::init();
        super::make_submodule_importable(py, m, "config")?;
        Ok(())
    }
}
