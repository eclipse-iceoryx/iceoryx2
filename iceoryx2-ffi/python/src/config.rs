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

use crate::duration::Duration;
use crate::error::ConfigCreationError;
use crate::file_name::FileName;
use crate::file_path::FilePath;
use crate::parc::Parc;
use crate::path::Path;
use crate::unable_to_deliver_strategy::UnableToDeliverStrategy;
use pyo3::prelude::*;

#[pyclass]
/// All configurable settings of a `Node`.
pub struct Node(Parc<iceoryx2::config::Config>);

#[pymethods]
impl Node {
    pub fn __str__(&self) -> String {
        format!("{:?}", self.0.lock().global.node)
    }

    #[getter]
    /// The directory in which all node files are stored
    pub fn directory(&self) -> Path {
        Path(self.0.lock().global.node.directory.clone())
    }

    #[setter]
    /// Set the directory in which all node files are stored
    pub fn set_directory(&mut self, value: &Path) {
        self.0.lock().global.node.directory = value.0.clone()
    }

    #[getter]
    /// The suffix of the monitor token
    pub fn monitor_suffix(&self) -> FileName {
        FileName(self.0.lock().global.node.monitor_suffix.clone())
    }

    #[setter]
    /// Set the suffix of the monitor token
    pub fn set_monitor_suffix(&mut self, value: &FileName) {
        self.0.lock().global.node.monitor_suffix = value.0.clone()
    }

    #[getter]
    /// The suffix of the files where the node configuration is stored.
    pub fn static_config_suffix(&self) -> FileName {
        FileName(self.0.lock().global.node.static_config_suffix.clone())
    }

    #[setter]
    /// Set the suffix of the files where the node configuration is stored.
    pub fn set_static_config_suffix(&mut self, value: &FileName) {
        self.0.lock().global.node.static_config_suffix = value.0.clone()
    }

    #[getter]
    /// The suffix of the service tags.
    pub fn service_tag_suffix(&self) -> FileName {
        FileName(self.0.lock().global.node.service_tag_suffix.clone())
    }

    #[setter]
    /// Set the suffix of the service tags.
    pub fn set_service_tag_suffix(&mut self, value: &FileName) {
        self.0.lock().global.node.service_tag_suffix = value.0.clone()
    }

    #[getter]
    /// When true, the `NodeBuilder` checks for dead nodes and
    /// cleans up all their stale resources whenever a new [`Node`](Node) is
    /// created.
    pub fn cleanup_dead_nodes_on_creation(&self) -> bool {
        self.0.lock().global.node.cleanup_dead_nodes_on_creation
    }

    #[setter]
    /// Enable/disable the cleanup dead nodes on creation
    pub fn set_cleanup_dead_nodes_on_creation(&mut self, value: bool) {
        self.0.lock().global.node.cleanup_dead_nodes_on_creation = value
    }

    #[getter]
    /// When true, the `NodeBuilder` checks for dead nodes and
    /// cleans up all their stale resources whenever an existing `Node` is
    /// going out of scope.
    pub fn cleanup_dead_nodes_on_destruction(&self) -> bool {
        self.0.lock().global.node.cleanup_dead_nodes_on_destruction
    }

    #[setter]
    /// Enable/disable the cleanup dead nodes on destruction
    pub fn set_cleanup_dead_nodes_on_destruction(&mut self, value: bool) {
        self.0.lock().global.node.cleanup_dead_nodes_on_destruction = value
    }
}

#[pyclass]
/// All configurable settings of a `Service`.
pub struct Service(Parc<iceoryx2::config::Config>);

#[pymethods]
impl Service {
    pub fn __str__(&self) -> String {
        format!("{:?}", self.0.lock().global.service)
    }

    #[getter]
    /// The directory in which all service files are stored
    pub fn directory(&self) -> Path {
        Path(self.0.lock().global.service.directory.clone())
    }

    #[setter]
    /// Set the directory in which all service files are stored
    pub fn set_directory(&self, value: &Path) {
        self.0.lock().global.service.directory = value.0.clone()
    }

    #[getter]
    /// The suffix of the ports data segment
    pub fn data_segment_suffix(&self) -> FileName {
        FileName(self.0.lock().global.service.data_segment_suffix.clone())
    }

    #[setter]
    /// Set the suffix of the ports data segment
    pub fn set_data_segment_suffix(&self, value: &FileName) {
        self.0.lock().global.service.data_segment_suffix = value.0.clone()
    }

    #[getter]
    /// The suffix of the static config file
    pub fn static_config_storage_suffix(&self) -> FileName {
        FileName(
            self.0
                .lock()
                .global
                .service
                .static_config_storage_suffix
                .clone(),
        )
    }

    #[setter]
    /// Set the suffix of the static config file
    pub fn set_static_config_storage_suffix(&self, value: &FileName) {
        self.0.lock().global.service.static_config_storage_suffix = value.0.clone()
    }

    #[getter]
    /// The suffix of the dynamic config file
    pub fn dynamic_config_storage_suffix(&self) -> FileName {
        FileName(
            self.0
                .lock()
                .global
                .service
                .dynamic_config_storage_suffix
                .clone(),
        )
    }

    #[setter]
    /// Set the suffix of the dynamic config file
    pub fn set_dynamic_config_storage_suffix(&self, value: &FileName) {
        self.0.lock().global.service.dynamic_config_storage_suffix = value.0.clone()
    }

    #[getter]
    /// Defines the time of how long another process will wait until the service creation is
    /// finalized
    pub fn creation_timeout(&self) -> Duration {
        Duration(self.0.lock().global.service.creation_timeout)
    }

    #[setter]
    /// Set the creation timeout
    pub fn set_creation_timeout(&self, value: &Duration) {
        self.0.lock().global.service.creation_timeout = value.0
    }

    #[getter]
    /// The suffix of a one-to-one connection
    pub fn connection_suffix(&self) -> FileName {
        FileName(self.0.lock().global.service.connection_suffix.clone())
    }

    #[setter]
    /// Set the suffix of a one-to-one connection
    pub fn set_connection_suffix(&self, value: &FileName) {
        self.0.lock().global.service.connection_suffix = value.0.clone()
    }

    #[getter]
    /// The suffix of a one-to-one connection
    pub fn event_connection_suffix(&self) -> FileName {
        FileName(self.0.lock().global.service.event_connection_suffix.clone())
    }

    #[setter]
    /// Set the suffix of a one-to-one connection
    pub fn set_event_connection_suffix(&self, value: &FileName) {
        self.0.lock().global.service.event_connection_suffix = value.0.clone()
    }
}

#[pyclass]
/// Default settings for the publish-subscribe messaging pattern. These settings are used unless
/// the user specifies custom QoS or port settings.
pub struct PublishSubscribe(Parc<iceoryx2::config::Config>);

#[pymethods]
impl PublishSubscribe {
    pub fn __str__(&self) -> String {
        format!("{:?}", self.0.lock().defaults.publish_subscribe)
    }

    #[getter]
    /// The maximum amount of supported `Subscriber`s
    pub fn max_subscribers(&self) -> usize {
        self.0.lock().defaults.publish_subscribe.max_subscribers
    }

    #[setter]
    /// Set the maximum amount of supported `Subscriber`s
    pub fn set_max_subscribers(&self, value: usize) {
        self.0.lock().defaults.publish_subscribe.max_subscribers = value
    }

    #[getter]
    /// The maximum amount of supported `Publisher`s
    pub fn max_publishers(&self) -> usize {
        self.0.lock().defaults.publish_subscribe.max_publishers
    }

    #[setter]
    /// Set the maximum amount of supported `Publisher`s
    pub fn set_max_publishers(&self, value: usize) {
        self.0.lock().defaults.publish_subscribe.max_publishers = value
    }

    #[getter]
    /// The maximum amount of supported `Node`s. Defines indirectly how many
    /// processes can open the service at the same time.
    pub fn max_nodes(&self) -> usize {
        self.0.lock().defaults.publish_subscribe.max_nodes
    }

    #[setter]
    /// Set the maximum amount of supported `Node`s.
    pub fn set_max_nodes(&self, value: usize) {
        self.0.lock().defaults.publish_subscribe.max_nodes = value
    }

    #[getter]
    /// The maximum buffer size a `Subscriber` can have
    pub fn subscriber_max_buffer_size(&self) -> usize {
        self.0
            .lock()
            .defaults
            .publish_subscribe
            .subscriber_max_buffer_size
    }

    #[setter]
    /// Set the maximum buffer size a `Subscriber` can have
    pub fn set_subscriber_max_buffer_size(&self, value: usize) {
        self.0
            .lock()
            .defaults
            .publish_subscribe
            .subscriber_max_buffer_size = value
    }

    #[getter]
    /// The maximum amount of `Sample`s a `Subscriber` can hold at the same time.
    pub fn subscriber_max_borrowed_samples(&self) -> usize {
        self.0
            .lock()
            .defaults
            .publish_subscribe
            .subscriber_max_borrowed_samples
    }

    #[setter]
    /// Set the maximum amount of `Sample`s a `Subscriber` can hold at the same time.
    pub fn set_subscriber_max_borrowed_samples(&self, value: usize) {
        self.0
            .lock()
            .defaults
            .publish_subscribe
            .subscriber_max_borrowed_samples = value
    }

    #[getter]
    /// The maximum amount of `SampleMut`s a `Publisher` can loan at the same time.
    pub fn publisher_max_loaned_samples(&self) -> usize {
        self.0
            .lock()
            .defaults
            .publish_subscribe
            .publisher_max_loaned_samples
    }

    #[setter]
    /// The maximum amount of `SampleMut`s a `Publisher` can loan at the same time.
    pub fn set_publisher_max_loaned_samples(&self, value: usize) {
        self.0
            .lock()
            .defaults
            .publish_subscribe
            .publisher_max_loaned_samples = value
    }

    #[getter]
    /// The maximum history size a `Subscriber` can request from a `Publisher`.
    pub fn publisher_history_size(&self) -> usize {
        self.0
            .lock()
            .defaults
            .publish_subscribe
            .publisher_history_size
    }

    #[setter]
    /// Set the maximum history size a `Subscriber` can request from a `Publisher`.
    pub fn set_publisher_history_size(&self, value: usize) {
        self.0
            .lock()
            .defaults
            .publish_subscribe
            .publisher_history_size = value
    }

    #[getter]
    /// Defines how the `Subscriber` buffer behaves when it is
    /// full. When safe overflow is activated, the `Publisher` will
    /// replace the oldest `Sample` with the newest one.
    pub fn enable_safe_overflow(&self) -> bool {
        self.0
            .lock()
            .defaults
            .publish_subscribe
            .enable_safe_overflow
    }

    #[setter]
    /// Enables/disables safe overflow
    pub fn set_enable_safe_overflow(&self, value: bool) {
        self.0
            .lock()
            .defaults
            .publish_subscribe
            .enable_safe_overflow = value
    }

    #[getter]
    /// If safe overflow is deactivated it defines the deliver strategy of the
    /// `Publisher` when the `Subscriber`s buffer is full.
    pub fn unable_to_deliver_strategy(&self) -> UnableToDeliverStrategy {
        self.0
            .lock()
            .defaults
            .publish_subscribe
            .unable_to_deliver_strategy
            .into()
    }

    #[setter]
    /// Define the unable to deliver strategy
    pub fn set_unable_to_deliver_strategy(&self, value: &UnableToDeliverStrategy) {
        self.0
            .lock()
            .defaults
            .publish_subscribe
            .unable_to_deliver_strategy = (value.clone()).into()
    }

    #[getter]
    /// Defines the size of the internal `Subscriber`
    /// buffer that contains expired connections. An
    /// connection is expired when the `Publisher`
    /// disconnected from a service and the connection
    /// still contains unconsumed `Sample`s.
    pub fn subscriber_expired_connection_buffer(&self) -> usize {
        self.0
            .lock()
            .defaults
            .publish_subscribe
            .subscriber_expired_connection_buffer
    }

    #[setter]
    /// Set the expired connection buffer size
    pub fn set_subscriber_expired_connection_buffer(&self, value: usize) {
        self.0
            .lock()
            .defaults
            .publish_subscribe
            .subscriber_expired_connection_buffer = value
    }
}

#[pyclass]
/// Default settings for the event messaging pattern. These settings are used unless
/// the user specifies custom QoS or port settings.
pub struct Event(Parc<iceoryx2::config::Config>);

#[pymethods]
impl Event {
    pub fn __str__(&self) -> String {
        format!("{:?}", self.0.lock().defaults.event)
    }

    #[getter]
    /// The maximum amount of supported `Listener`
    pub fn max_listeners(&self) -> usize {
        self.0.lock().defaults.event.max_listeners
    }

    #[setter]
    /// Set the maximum amount of supported `Listener`
    pub fn set_max_listeners(&self, value: usize) {
        self.0.lock().defaults.event.max_listeners = value
    }

    #[getter]
    /// The maximum amount of supported `Notifier`
    pub fn max_notifiers(&self) -> usize {
        self.0.lock().defaults.event.max_notifiers
    }

    #[setter]
    /// Set the maximum amount of supported `Notifier`
    pub fn set_max_notifiers(&self, value: usize) {
        self.0.lock().defaults.event.max_notifiers = value
    }

    #[getter]
    /// The maximum amount of supported `Node`s. Defines indirectly how many
    /// processes can open the service at the same time.
    pub fn max_nodes(&self) -> usize {
        self.0.lock().defaults.event.max_nodes
    }

    #[setter]
    /// Set the maximum amount of supported `Node`s.
    pub fn set_max_nodes(&self, value: usize) {
        self.0.lock().defaults.event.max_nodes = value
    }

    #[getter]
    /// The largest event id supported by the event service
    pub fn event_id_max_value(&self) -> usize {
        self.0.lock().defaults.event.event_id_max_value
    }

    #[setter]
    /// Set the largest event id supported by the event service
    pub fn set_event_id_max_value(&self, value: usize) {
        self.0.lock().defaults.event.event_id_max_value = value
    }

    #[getter]
    /// Defines the maximum allowed time between two consecutive notifications. If a notifiation
    /// is not sent after the defined time, every `Listener`
    /// that is attached to a `WaitSet` will be notified.
    pub fn deadline(&self) -> Duration {
        Duration(
            self.0
                .lock()
                .defaults
                .event
                .deadline
                .unwrap_or(core::time::Duration::ZERO),
        )
    }

    #[setter]
    /// Sets the deadline of the event service.
    pub fn set_deadline(&self, value: &Duration) {
        if value.0.is_zero() {
            self.0.lock().defaults.event.deadline = None
        } else {
            self.0.lock().defaults.event.deadline = Some(value.0)
        }
    }

    #[getter]
    /// Defines the event id value that is emitted after a new notifier was created. If it is
    /// not set then `usize::MAX` is returned
    pub fn get_notifier_created_event(&self) -> usize {
        self.0
            .lock()
            .defaults
            .event
            .notifier_created_event
            .unwrap_or(usize::MAX)
    }

    #[getter]
    /// Returns true if the notifier created event was set, otherwise false.
    pub fn has_notifier_created_event(&self) -> bool {
        self.0
            .lock()
            .defaults
            .event
            .notifier_created_event
            .is_some()
    }

    #[setter]
    /// Sets the event id value that is emitted after a new notifier was created.
    pub fn set_notifier_created_event(&self, value: usize) {
        self.0.lock().defaults.event.notifier_created_event = Some(value)
    }

    /// Do not emit an event whenever a notifier was created.
    pub fn disable_notifier_created_event(&self) {
        self.0.lock().defaults.event.notifier_created_event = None
    }

    #[getter]
    /// Defines the event id value that is emitted before a new notifier is dropped.
    pub fn get_notifier_dropped_event(&self) -> usize {
        self.0
            .lock()
            .defaults
            .event
            .notifier_dropped_event
            .unwrap_or(usize::MAX)
    }

    #[getter]
    /// Returns true if the notifier dropped event was set, otherwise false.
    pub fn has_notifier_dropped_event(&self) -> bool {
        self.0
            .lock()
            .defaults
            .event
            .notifier_dropped_event
            .is_some()
    }

    #[setter]
    /// Sets the event id value that is emitted before a new notifier is dropped.
    pub fn set_notifier_dropped_event(&self, value: usize) {
        self.0.lock().defaults.event.notifier_dropped_event = Some(value)
    }

    /// Do not emit an event whenever a notifier was dropped.
    pub fn disable_notifier_dropped_event(&self) {
        self.0.lock().defaults.event.notifier_dropped_event = None
    }

    #[getter]
    /// Defines the event id value that is emitted if a notifier was identified as dead.
    pub fn get_notifier_dead_event(&self) -> usize {
        self.0
            .lock()
            .defaults
            .event
            .notifier_dead_event
            .unwrap_or(usize::MAX)
    }

    #[getter]
    /// Returns true if the notifier dead event was set, otherwise false.
    pub fn has_notifier_dead_event(&self) -> bool {
        self.0.lock().defaults.event.notifier_dead_event.is_some()
    }

    #[setter]
    /// Sets the event id value that is emitted if a notifier was identified as dead.
    pub fn set_notifier_dead_event(&self, value: usize) {
        self.0.lock().defaults.event.notifier_dead_event = Some(value)
    }

    /// Do not emit an event whenever a notifier was identified as dead.
    pub fn disable_notifier_dead_event(&self) {
        self.0.lock().defaults.event.notifier_dead_event = None
    }
}

#[pyclass]
/// Default settings for the request response messaging pattern. These settings are used unless
/// the user specifies custom QoS or port settings.
pub struct RequestResponse(Parc<iceoryx2::config::Config>);

#[pymethods]
impl RequestResponse {
    pub fn __str__(&self) -> String {
        format!("{:?}", self.0.lock().defaults.event)
    }

    #[getter]
    /// Defines if the request buffer of the `Service` safely overflows.
    pub fn enable_safe_overflow_for_requests(&self) -> bool {
        self.0
            .lock()
            .defaults
            .request_response
            .enable_safe_overflow_for_requests
    }

    #[setter]
    /// Enables/disables safe overflow for the request buffer.
    pub fn set_enable_safe_overflow_for_requests(&self, value: bool) {
        self.0
            .lock()
            .defaults
            .request_response
            .enable_safe_overflow_for_requests = value
    }

    #[getter]
    /// Defines if the response buffer of the `Service` safely overflows.
    pub fn enable_safe_overflow_for_responses(&self) -> bool {
        self.0
            .lock()
            .defaults
            .request_response
            .enable_safe_overflow_for_responses
    }

    #[setter]
    /// Enables/disables safe overflow for the response buffer.
    pub fn set_enable_safe_overflow_for_responses(&self, value: bool) {
        self.0
            .lock()
            .defaults
            .request_response
            .enable_safe_overflow_for_responses = value
    }

    #[getter]
    /// The maximum of `ActiveRequest`s a `Server` can hold in
    /// parallel per `Client`.
    pub fn max_active_requests_per_client(&self) -> usize {
        self.0
            .lock()
            .defaults
            .request_response
            .max_active_requests_per_client
    }

    #[setter]
    /// Set the maximum of `ActiveRequest`s a `Server` can hold in
    /// parallel per `Client`.
    pub fn set_max_active_requests_per_client(&self, value: usize) {
        self.0
            .lock()
            .defaults
            .request_response
            .max_active_requests_per_client = value
    }

    #[getter]
    /// The maximum buffer size for `Response`s for a
    /// `PendingResponse`.
    pub fn max_response_buffer_size(&self) -> usize {
        self.0
            .lock()
            .defaults
            .request_response
            .max_response_buffer_size
    }

    #[setter]
    /// Set the maximum buffer size for `Response`s for a
    /// `PendingResponse`.
    pub fn set_max_response_buffer_size(&self, value: usize) {
        self.0
            .lock()
            .defaults
            .request_response
            .max_response_buffer_size = value
    }

    #[getter]
    /// The maximum amount of supported `Server`
    pub fn max_servers(&self) -> usize {
        self.0.lock().defaults.request_response.max_servers
    }

    #[setter]
    /// Set the maximum amount of supported `Server`
    pub fn set_max_servers(&self, value: usize) {
        self.0.lock().defaults.request_response.max_servers = value
    }

    #[getter]
    /// The maximum amount of supported `Client`
    pub fn max_clients(&self) -> usize {
        self.0.lock().defaults.request_response.max_clients
    }

    #[setter]
    /// Set the maximum amount of supported `Client`
    pub fn set_max_clients(&self, value: usize) {
        self.0.lock().defaults.request_response.max_clients = value
    }

    #[getter]
    /// The maximum amount of supported `Node`s. Defines
    /// indirectly how many processes can open the service at the same time.
    pub fn max_nodes(&self) -> usize {
        self.0.lock().defaults.request_response.max_nodes
    }

    #[setter]
    /// Set the maximum amount of supported `Node`s. Defines
    /// indirectly how many processes can open the service at the same time.
    pub fn set_max_nodes(&self, value: usize) {
        self.0.lock().defaults.request_response.max_nodes = value
    }

    #[getter]
    /// The maximum amount of borrowed `Response` per
    /// `PendingResponse` on the `Client` side.
    pub fn max_borrowed_responses_per_pending_response(&self) -> usize {
        self.0
            .lock()
            .defaults
            .request_response
            .max_borrowed_responses_per_pending_response
    }

    #[setter]
    /// Set the maximum amount of borrowed `Response` per
    /// `PendingResponse` on the `Client` side.
    pub fn set_max_borrowed_responses_per_pending_response(&self, value: usize) {
        self.0
            .lock()
            .defaults
            .request_response
            .max_borrowed_responses_per_pending_response = value
    }

    #[getter]
    /// Defines how many `RequestMut` a
    /// `Client` can loan in parallel.
    pub fn max_loaned_requests(&self) -> usize {
        self.0.lock().defaults.request_response.max_loaned_requests
    }

    #[setter]
    /// Set how many `RequestMut` a
    /// `Client` can loan in parallel.
    pub fn set_max_loaned_requests(&self, value: usize) {
        self.0.lock().defaults.request_response.max_loaned_requests = value
    }

    #[getter]
    /// Defines how many `ResponseMut` a `Server` can loan in
    /// parallel per `ActiveRequest`.
    pub fn server_max_loaned_responses_per_request(&self) -> usize {
        self.0
            .lock()
            .defaults
            .request_response
            .server_max_loaned_responses_per_request
    }

    #[setter]
    /// Set how many `ResponseMut` a `Server` can loan in
    /// parallel per `ActiveRequest`.
    pub fn set_server_max_loaned_responses_per_request(&self, value: usize) {
        self.0
            .lock()
            .defaults
            .request_response
            .server_max_loaned_responses_per_request = value
    }

    #[getter]
    /// Defines the `UnableToDeliverStrategy` when a `Client`
    /// could not deliver the request to the `Server`.
    pub fn client_unable_to_deliver_strategy(&self) -> UnableToDeliverStrategy {
        self.0
            .lock()
            .defaults
            .request_response
            .client_unable_to_deliver_strategy
            .into()
    }

    #[setter]
    /// Set the `UnableToDeliverStrategy` when a `Client`
    /// could not deliver the request to the `Server`.
    pub fn set_client_unable_to_deliver_strategy(&self, value: &UnableToDeliverStrategy) {
        self.0
            .lock()
            .defaults
            .request_response
            .client_unable_to_deliver_strategy = (value.clone()).into()
    }

    #[getter]
    /// Defines the `UnableToDeliverStrategy` when a `Server`
    /// could not deliver the response to the `Client`.
    pub fn server_unable_to_deliver_strategy(&self) -> UnableToDeliverStrategy {
        self.0
            .lock()
            .defaults
            .request_response
            .server_unable_to_deliver_strategy
            .into()
    }

    #[setter]
    /// Set the `UnableToDeliverStrategy` when a `Server`
    /// could not deliver the response to the `Client`.
    pub fn set_server_unable_to_deliver_strategy(&self, value: &UnableToDeliverStrategy) {
        self.0
            .lock()
            .defaults
            .request_response
            .server_unable_to_deliver_strategy = (value.clone()).into()
    }

    #[getter]
    /// Defines the size of the internal `Client`
    /// buffer that contains expired connections. An
    /// connection is expired when the `Server`
    /// disconnected from a service and the connection
    /// still contains unconsumed `Response`s.
    pub fn client_expired_connection_buffer(&self) -> usize {
        self.0
            .lock()
            .defaults
            .request_response
            .client_expired_connection_buffer
    }

    #[setter]
    /// Set the size of the internal `Client`
    /// buffer that contains expired connections. An
    /// connection is expired when the `Server`
    /// disconnected from a service and the connection
    /// still contains unconsumed `Response`s.
    pub fn set_client_expired_connection_buffer(&self, value: usize) {
        self.0
            .lock()
            .defaults
            .request_response
            .client_expired_connection_buffer = value
    }

    #[getter]
    /// Defines the size of the internal `Server`
    /// buffer that contains expired connections. An
    /// connection is expired when the `Client`
    /// disconnected from a service and the connection
    /// still contains unconsumed `ActiveRequest`s.
    pub fn server_expired_connection_buffer(&self) -> usize {
        self.0
            .lock()
            .defaults
            .request_response
            .server_expired_connection_buffer
    }

    #[setter]
    /// Set the size of the internal `Server`
    /// buffer that contains expired connections. An
    /// connection is expired when the `Client`
    /// disconnected from a service and the connection
    /// still contains unconsumed `ActiveRequest`s.
    pub fn set_server_expired_connection_buffer(&self, value: usize) {
        self.0
            .lock()
            .defaults
            .request_response
            .server_expired_connection_buffer = value
    }

    #[getter]
    /// Allows the `Server` to receive `RequestMut`s of `Client`s that are not interested in a
    /// `Response`, meaning that the `Server` will receive the `RequestMut` despite the
    /// corresponding `PendingResponse` already went out-of-scope. So any `Response` sent by the
    /// `Server` would not be received by the corresponding `Client`s `PendingResponse`.
    ///
    /// Consider enabling this feature if you do not want to loose any `RequestMut`.
    pub fn enable_fire_and_forget_requests(&self) -> bool {
        self.0
            .lock()
            .defaults
            .request_response
            .enable_fire_and_forget_requests
    }

    #[setter]
    /// Set if fire-and-forget feature is enabled
    pub fn set_enable_fire_and_forget_requests(&self, value: bool) {
        self.0
            .lock()
            .defaults
            .request_response
            .enable_fire_and_forget_requests = value
    }
}

#[pyclass]
/// The global settings
pub struct Global(Parc<iceoryx2::config::Config>);

#[pymethods]
impl Global {
    pub fn __str__(&self) -> String {
        format!("{:?}", self.0.lock().global)
    }

    #[getter]
    /// Returns the service part of the global configuration
    pub fn service(&self) -> Service {
        Service(self.0.clone())
    }

    #[getter]
    /// Returns the node part of the global configuration
    pub fn node(&self) -> Node {
        Node(self.0.clone())
    }

    #[getter]
    /// Returns the directory under which service files are stored.
    pub fn service_dir(&self) -> Path {
        Path(self.0.lock().global.service_dir().clone())
    }

    #[getter]
    /// Returns the directory under which node files are stored.
    pub fn node_dir(&self) -> Path {
        Path(self.0.lock().global.node_dir().clone())
    }

    #[getter]
    /// The path under which all other directories or files will be created
    pub fn root_path(&self) -> Path {
        Path(self.0.lock().global.root_path().clone())
    }

    #[setter]
    /// Defines the path under which all other directories or files will be created
    pub fn set_root_path(&self, value: &Path) {
        self.0.lock().global.set_root_path(&value.0.clone())
    }

    #[getter]
    /// Prefix used for all files created during runtime
    pub fn prefix(&self) -> FileName {
        FileName(self.0.lock().global.prefix.clone())
    }

    #[setter]
    /// Set the prefix used for all files created during runtime
    pub fn set_prefix(&self, value: &FileName) {
        self.0.lock().global.prefix = value.0.clone()
    }
}

#[pyclass]
/// Default settings. These values are used when the user in the code does not specify anything
/// else.
pub struct Defaults(Parc<iceoryx2::config::Config>);

#[pymethods]
impl Defaults {
    pub fn __str__(&self) -> String {
        format!("{:?}", self.0.lock().global)
    }

    #[getter]
    /// Returns the publish_subscribe part of the default settings
    pub fn publish_subscribe(&self) -> PublishSubscribe {
        PublishSubscribe(self.0.clone())
    }

    #[getter]
    /// Returns the event part of the default settings
    pub fn event(&self) -> Event {
        Event(self.0.clone())
    }

    #[getter]
    /// Returns the request_response part of the default settings
    pub fn request_response(&self) -> RequestResponse {
        RequestResponse(self.0.clone())
    }
}

#[pyclass]
/// Represents the configuration that iceoryx2 will utilize. It is divided into two sections:
/// the [Global] settings, which must align with the iceoryx2 instance the application intends to
/// join, and the [Defaults] for communication within that iceoryx2 instance. The user has the
/// flexibility to override both sections.
pub struct Config(pub(crate) Parc<iceoryx2::config::Config>);

impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        *self.0.lock() == *other.0.lock()
    }
}

#[pymethods]
impl Config {
    pub fn __eq__(&self, other: &Self) -> bool {
        self == other
    }

    pub fn __str__(&self) -> String {
        format!("{:?}", self.0.lock())
    }

    #[getter]
    /// Returns the `Global` part of the config
    pub fn global_cfg(&self) -> Global {
        Global(self.0.clone())
    }

    #[getter]
    /// Returns the `Defaults` part of the config
    pub fn defaults(&self) -> Defaults {
        Defaults(self.0.clone())
    }
}

#[pyfunction]
pub fn default() -> Config {
    Config(Parc::new(iceoryx2::config::Config::default()))
}

#[pyfunction]
pub fn global_config() -> Config {
    Config(Parc::new(iceoryx2::config::Config::global_config().clone()))
}

#[pyfunction]
pub fn setup_global_config_from_file(config_file: &FilePath) -> PyResult<Config> {
    Ok(Config(Parc::new(
        iceoryx2::config::Config::setup_global_config_from_file(&config_file.0.clone())
            .map_err(|e| ConfigCreationError::new_err(format!("{e:?}")))?
            .clone(),
    )))
}

#[pyfunction]
/// Loads a configuration from a file. On success it returns a `Config` object otherwise a
/// `ConfigCreationError` describing the failure.
pub fn from_file(config_file: &FilePath) -> PyResult<Config> {
    Ok(Config(Parc::new(
        iceoryx2::config::Config::from_file(&config_file.0.clone())
            .map_err(|e| ConfigCreationError::new_err(format!("{e:?}")))?
            .clone(),
    )))
}

#[pyfunction]
/// Path to the default user config file
pub fn default_user_config_file_path() -> FilePath {
    FilePath(iceoryx2::config::Config::default_user_config_file_path())
}

#[pyfunction]
/// Relative path to the config file
pub fn relative_config_path() -> Path {
    Path(iceoryx2::config::Config::relative_config_path())
}

#[pyfunction]
/// Path to the default config file
pub fn default_config_file_path() -> FilePath {
    FilePath(iceoryx2::config::Config::default_config_file_path())
}

#[pyfunction]
/// The name of the default iceoryx2 config file
pub fn default_config_file_name() -> FileName {
    FileName(iceoryx2::config::Config::default_config_file_name())
}

#[pymodule]
pub fn config(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<crate::config::Global>()?;
    m.add_class::<crate::config::Config>()?;
    m.add_class::<crate::config::Defaults>()?;
    m.add_class::<crate::config::Event>()?;
    m.add_class::<crate::config::PublishSubscribe>()?;
    m.add_class::<crate::config::RequestResponse>()?;
    m.add_class::<crate::config::Node>()?;
    m.add_class::<crate::config::Service>()?;
    m.add_wrapped(wrap_pyfunction!(default_config_file_name))?;
    m.add_wrapped(wrap_pyfunction!(default_config_file_path))?;
    m.add_wrapped(wrap_pyfunction!(relative_config_path))?;
    m.add_wrapped(wrap_pyfunction!(default_user_config_file_path))?;
    m.add_wrapped(wrap_pyfunction!(from_file))?;
    m.add_wrapped(wrap_pyfunction!(setup_global_config_from_file))?;
    m.add_wrapped(wrap_pyfunction!(global_config))?;
    m.add_wrapped(wrap_pyfunction!(default))?;
    Ok(())
}
