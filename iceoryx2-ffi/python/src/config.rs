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

use core::usize;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::duration::Duration;
use crate::error::ConfigCreationError;
use crate::file_name::FileName;
use crate::file_path::FilePath;
use crate::path::Path;
use crate::unable_to_deliver_strategy::UnableToDeliverStrategy;
use pyo3::prelude::*;

#[pyclass]
pub struct Node {
    value: Arc<Mutex<iceoryx2::config::Config>>,
}

impl Node {
    fn value(&self) -> MutexGuard<iceoryx2::config::Config> {
        self.value.lock().unwrap()
    }
}

#[pymethods]
impl Node {
    pub fn __str__(&self) -> String {
        format!("{:?}", self.value().global.node)
    }

    #[getter]
    pub fn directory(&self) -> Path {
        Path {
            value: self.value().global.node.directory.clone(),
        }
    }

    #[setter]
    pub fn set_directory(&mut self, value: &Path) {
        self.value().global.node.directory = value.value.clone()
    }

    #[getter]
    pub fn monitor_suffix(&self) -> FileName {
        FileName {
            value: self.value().global.node.monitor_suffix.clone(),
        }
    }

    #[setter]
    pub fn set_monitor_suffix(&mut self, value: &FileName) {
        self.value().global.node.monitor_suffix = value.value.clone()
    }

    #[getter]
    pub fn static_config_suffix(&self) -> FileName {
        FileName {
            value: self.value().global.node.static_config_suffix.clone(),
        }
    }

    #[setter]
    pub fn set_static_config_suffix(&mut self, value: &FileName) {
        self.value().global.node.static_config_suffix = value.value.clone()
    }

    #[getter]
    pub fn service_tag_suffix(&self) -> FileName {
        FileName {
            value: self.value().global.node.service_tag_suffix.clone(),
        }
    }

    #[setter]
    pub fn set_service_tag_suffix(&mut self, value: &FileName) {
        self.value().global.node.service_tag_suffix = value.value.clone()
    }

    #[getter]
    pub fn cleanup_dead_nodes_on_creation(&self) -> bool {
        self.value().global.node.cleanup_dead_nodes_on_creation
    }

    #[setter]
    pub fn set_cleanup_dead_nodes_on_creation(&mut self, value: bool) {
        self.value().global.node.cleanup_dead_nodes_on_creation = value
    }

    #[getter]
    pub fn cleanup_dead_nodes_on_destruction(&self) -> bool {
        self.value().global.node.cleanup_dead_nodes_on_destruction
    }

    #[setter]
    pub fn set_cleanup_dead_nodes_on_destruction(&mut self, value: bool) {
        self.value().global.node.cleanup_dead_nodes_on_destruction = value
    }
}

#[pyclass]
pub struct Service {
    value: Arc<Mutex<iceoryx2::config::Config>>,
}

impl Service {
    fn value(&self) -> MutexGuard<iceoryx2::config::Config> {
        self.value.lock().unwrap()
    }
}

#[pymethods]
impl Service {
    pub fn __str__(&self) -> String {
        format!("{:?}", self.value().global.service)
    }

    #[getter]
    pub fn directory(&self) -> Path {
        Path {
            value: self.value().global.service.directory.clone(),
        }
    }

    #[setter]
    pub fn set_directory(&self, value: &Path) {
        self.value().global.service.directory = value.value.clone()
    }

    #[getter]
    pub fn data_segment_suffix(&self) -> FileName {
        FileName {
            value: self.value().global.service.data_segment_suffix.clone(),
        }
    }

    #[setter]
    pub fn set_data_segment_suffix(&self, value: &FileName) {
        self.value().global.service.data_segment_suffix = value.value.clone()
    }

    #[getter]
    pub fn static_config_storage_suffix(&self) -> FileName {
        FileName {
            value: self
                .value()
                .global
                .service
                .static_config_storage_suffix
                .clone(),
        }
    }

    #[setter]
    pub fn set_static_config_storage_suffix(&self, value: &FileName) {
        self.value().global.service.static_config_storage_suffix = value.value.clone()
    }

    #[getter]
    pub fn dynamic_config_storage_suffix(&self) -> FileName {
        FileName {
            value: self
                .value()
                .global
                .service
                .dynamic_config_storage_suffix
                .clone(),
        }
    }

    #[setter]
    pub fn set_dynamic_config_storage_suffix(&self, value: &FileName) {
        self.value().global.service.dynamic_config_storage_suffix = value.value.clone()
    }

    #[getter]
    pub fn creation_timeout(&self) -> Duration {
        Duration {
            value: self.value().global.service.creation_timeout,
        }
    }

    #[setter]
    pub fn set_creation_timeout(&self, value: &Duration) {
        self.value().global.service.creation_timeout = value.value.clone()
    }

    #[getter]
    pub fn connection_suffix(&self) -> FileName {
        FileName {
            value: self.value().global.service.connection_suffix.clone(),
        }
    }

    #[setter]
    pub fn set_connection_suffix(&self, value: &FileName) {
        self.value().global.service.connection_suffix = value.value.clone()
    }

    #[getter]
    pub fn event_connection_suffix(&self) -> FileName {
        FileName {
            value: self.value().global.service.event_connection_suffix.clone(),
        }
    }

    #[setter]
    pub fn set_event_connection_suffix(&self, value: &FileName) {
        self.value().global.service.event_connection_suffix = value.value.clone()
    }
}

#[pyclass]
pub struct PublishSubscribe {
    value: Arc<Mutex<iceoryx2::config::Config>>,
}

impl PublishSubscribe {
    fn value(&self) -> MutexGuard<iceoryx2::config::Config> {
        self.value.lock().unwrap()
    }
}

#[pymethods]
impl PublishSubscribe {
    pub fn __str__(&self) -> String {
        format!("{:?}", self.value().defaults.publish_subscribe)
    }

    #[getter]
    pub fn max_subscribers(&self) -> usize {
        self.value().defaults.publish_subscribe.max_subscribers
    }

    #[setter]
    pub fn set_max_subscribers(&self, value: usize) {
        self.value().defaults.publish_subscribe.max_subscribers = value
    }

    #[getter]
    pub fn max_publishers(&self) -> usize {
        self.value().defaults.publish_subscribe.max_publishers
    }

    #[setter]
    pub fn set_max_publishers(&self, value: usize) {
        self.value().defaults.publish_subscribe.max_publishers = value
    }

    #[getter]
    pub fn max_nodes(&self) -> usize {
        self.value().defaults.publish_subscribe.max_nodes
    }

    #[setter]
    pub fn set_max_nodes(&self, value: usize) {
        self.value().defaults.publish_subscribe.max_nodes = value
    }

    #[getter]
    pub fn subscriber_max_buffer_size(&self) -> usize {
        self.value()
            .defaults
            .publish_subscribe
            .subscriber_max_buffer_size
    }

    #[setter]
    pub fn set_subscriber_max_buffer_size(&self, value: usize) {
        self.value()
            .defaults
            .publish_subscribe
            .subscriber_max_buffer_size = value
    }

    #[getter]
    pub fn subscriber_max_borrowed_samples(&self) -> usize {
        self.value()
            .defaults
            .publish_subscribe
            .subscriber_max_borrowed_samples
    }

    #[setter]
    pub fn set_subscriber_max_borrowed_samples(&self, value: usize) {
        self.value()
            .defaults
            .publish_subscribe
            .subscriber_max_borrowed_samples = value
    }

    #[getter]
    pub fn publisher_max_loaned_samples(&self) -> usize {
        self.value()
            .defaults
            .publish_subscribe
            .publisher_max_loaned_samples
    }

    #[setter]
    pub fn set_publisher_max_loaned_samples(&self, value: usize) {
        self.value()
            .defaults
            .publish_subscribe
            .publisher_max_loaned_samples = value
    }

    #[getter]
    pub fn publisher_history_size(&self) -> usize {
        self.value()
            .defaults
            .publish_subscribe
            .publisher_history_size
    }

    #[setter]
    pub fn set_publisher_history_size(&self, value: usize) {
        self.value()
            .defaults
            .publish_subscribe
            .publisher_history_size = value
    }

    #[getter]
    pub fn enable_safe_overflow(&self) -> bool {
        self.value().defaults.publish_subscribe.enable_safe_overflow
    }

    #[setter]
    pub fn set_enable_safe_overflow(&self, value: bool) {
        self.value().defaults.publish_subscribe.enable_safe_overflow = value
    }

    #[getter]
    pub fn unable_to_deliver_strategy(&self) -> UnableToDeliverStrategy {
        self.value()
            .defaults
            .publish_subscribe
            .unable_to_deliver_strategy
            .into()
    }

    #[setter]
    pub fn set_unable_to_deliver_strategy(&self, value: &UnableToDeliverStrategy) {
        self.value()
            .defaults
            .publish_subscribe
            .unable_to_deliver_strategy = (value.clone()).into()
    }

    #[getter]
    pub fn subscriber_expired_connection_buffer(&self) -> usize {
        self.value()
            .defaults
            .publish_subscribe
            .subscriber_expired_connection_buffer
    }

    #[setter]
    pub fn set_subscriber_expired_connection_buffer(&self, value: usize) {
        self.value()
            .defaults
            .publish_subscribe
            .subscriber_expired_connection_buffer = value
    }
}

#[pyclass]
pub struct Event {
    value: Arc<Mutex<iceoryx2::config::Config>>,
}

impl Event {
    fn value(&self) -> MutexGuard<iceoryx2::config::Config> {
        self.value.lock().unwrap()
    }
}

#[pymethods]
impl Event {
    pub fn __str__(&self) -> String {
        format!("{:?}", self.value().defaults.event)
    }

    #[getter]
    pub fn max_listeners(&self) -> usize {
        self.value().defaults.event.max_listeners
    }

    #[setter]
    pub fn set_max_listeners(&self, value: usize) {
        self.value().defaults.event.max_listeners = value
    }

    #[getter]
    pub fn max_notifiers(&self) -> usize {
        self.value().defaults.event.max_notifiers
    }

    #[setter]
    pub fn set_max_notifiers(&self, value: usize) {
        self.value().defaults.event.max_notifiers = value
    }

    #[getter]
    pub fn max_nodes(&self) -> usize {
        self.value().defaults.event.max_nodes
    }

    #[setter]
    pub fn set_max_nodes(&self, value: usize) {
        self.value().defaults.event.max_nodes = value
    }

    #[getter]
    pub fn event_id_max_value(&self) -> usize {
        self.value().defaults.event.event_id_max_value
    }

    #[setter]
    pub fn set_event_id_max_value(&self, value: usize) {
        self.value().defaults.event.event_id_max_value = value
    }

    #[getter]
    pub fn deadline(&self) -> Duration {
        Duration {
            value: self
                .value()
                .defaults
                .event
                .deadline
                .unwrap_or(core::time::Duration::ZERO),
        }
    }

    #[setter]
    pub fn set_deadline(&self, value: &Duration) {
        if value.value.is_zero() {
            self.value().defaults.event.deadline = None
        } else {
            self.value().defaults.event.deadline = Some(value.value)
        }
    }

    #[getter]
    pub fn get_notifier_created_event(&self) -> usize {
        self.value()
            .defaults
            .event
            .notifier_created_event
            .unwrap_or(usize::MAX)
    }

    #[getter]
    pub fn has_notifier_created_event(&self) -> bool {
        self.value().defaults.event.notifier_created_event.is_some()
    }

    #[setter]
    pub fn set_notifier_created_event(&self, value: usize) {
        self.value().defaults.event.notifier_created_event = Some(value)
    }

    pub fn disable_notifier_created_event(&self) {
        self.value().defaults.event.notifier_created_event = None
    }

    #[getter]
    pub fn get_notifier_dropped_event(&self) -> usize {
        self.value()
            .defaults
            .event
            .notifier_dropped_event
            .unwrap_or(usize::MAX)
    }

    #[getter]
    pub fn has_notifier_dropped_event(&self) -> bool {
        self.value().defaults.event.notifier_dropped_event.is_some()
    }

    #[setter]
    pub fn set_notifier_dropped_event(&self, value: usize) {
        self.value().defaults.event.notifier_dropped_event = Some(value)
    }

    pub fn disable_notifier_dropped_event(&self) {
        self.value().defaults.event.notifier_dropped_event = None
    }

    #[getter]
    pub fn get_notifier_dead_event(&self) -> usize {
        self.value()
            .defaults
            .event
            .notifier_dead_event
            .unwrap_or(usize::MAX)
    }

    #[getter]
    pub fn has_notifier_dead_event(&self) -> bool {
        self.value().defaults.event.notifier_dead_event.is_some()
    }

    #[setter]
    pub fn set_notifier_dead_event(&self, value: usize) {
        self.value().defaults.event.notifier_dead_event = Some(value)
    }

    pub fn disable_notifier_dead_event(&self) {
        self.value().defaults.event.notifier_dead_event = None
    }
}

#[pyclass]
pub struct RequestResponse {
    value: Arc<Mutex<iceoryx2::config::Config>>,
}

impl RequestResponse {
    fn value(&self) -> MutexGuard<iceoryx2::config::Config> {
        self.value.lock().unwrap()
    }
}

#[pymethods]
impl RequestResponse {
    pub fn __str__(&self) -> String {
        format!("{:?}", self.value().defaults.event)
    }

    #[getter]
    pub fn enable_safe_overflow_for_requests(&self) -> bool {
        self.value()
            .defaults
            .request_response
            .enable_safe_overflow_for_requests
    }

    #[setter]
    pub fn set_enable_safe_overflow_for_requests(&self, value: bool) {
        self.value()
            .defaults
            .request_response
            .enable_safe_overflow_for_requests = value
    }

    #[getter]
    pub fn enable_safe_overflow_for_responses(&self) -> bool {
        self.value()
            .defaults
            .request_response
            .enable_safe_overflow_for_responses
    }

    #[setter]
    pub fn set_enable_safe_overflow_for_responses(&self, value: bool) {
        self.value()
            .defaults
            .request_response
            .enable_safe_overflow_for_responses = value
    }

    #[getter]
    pub fn max_active_requests_per_client(&self) -> usize {
        self.value()
            .defaults
            .request_response
            .max_active_requests_per_client
    }

    #[setter]
    pub fn set_max_active_requests_per_client(&self, value: usize) {
        self.value()
            .defaults
            .request_response
            .max_active_requests_per_client = value
    }

    #[getter]
    pub fn max_response_buffer_size(&self) -> usize {
        self.value()
            .defaults
            .request_response
            .max_response_buffer_size
    }

    #[setter]
    pub fn set_max_response_buffer_size(&self, value: usize) {
        self.value()
            .defaults
            .request_response
            .max_response_buffer_size = value
    }

    #[getter]
    pub fn max_servers(&self) -> usize {
        self.value().defaults.request_response.max_servers
    }

    #[setter]
    pub fn set_max_servers(&self, value: usize) {
        self.value().defaults.request_response.max_servers = value
    }

    #[getter]
    pub fn max_clients(&self) -> usize {
        self.value().defaults.request_response.max_clients
    }

    #[setter]
    pub fn set_max_clients(&self, value: usize) {
        self.value().defaults.request_response.max_clients = value
    }

    #[getter]
    pub fn max_nodes(&self) -> usize {
        self.value().defaults.request_response.max_nodes
    }

    #[setter]
    pub fn set_max_nodes(&self, value: usize) {
        self.value().defaults.request_response.max_nodes = value
    }

    #[getter]
    pub fn max_borrowed_responses_per_pending_response(&self) -> usize {
        self.value()
            .defaults
            .request_response
            .max_borrowed_responses_per_pending_response
    }

    #[setter]
    pub fn set_max_borrowed_responses_per_pending_response(&self, value: usize) {
        self.value()
            .defaults
            .request_response
            .max_borrowed_responses_per_pending_response = value
    }

    #[getter]
    pub fn max_loaned_requests(&self) -> usize {
        self.value().defaults.request_response.max_loaned_requests
    }

    #[setter]
    pub fn set_max_loaned_requests(&self, value: usize) {
        self.value().defaults.request_response.max_loaned_requests = value
    }

    #[getter]
    pub fn server_max_loaned_responses_per_request(&self) -> usize {
        self.value()
            .defaults
            .request_response
            .server_max_loaned_responses_per_request
    }

    #[setter]
    pub fn set_server_max_loaned_responses_per_request(&self, value: usize) {
        self.value()
            .defaults
            .request_response
            .server_max_loaned_responses_per_request = value
    }

    #[getter]
    pub fn client_unable_to_deliver_strategy(&self) -> UnableToDeliverStrategy {
        self.value()
            .defaults
            .request_response
            .client_unable_to_deliver_strategy
            .into()
    }

    #[setter]
    pub fn set_client_unable_to_deliver_strategy(&self, value: &UnableToDeliverStrategy) {
        self.value()
            .defaults
            .request_response
            .client_unable_to_deliver_strategy = (value.clone()).into()
    }

    #[getter]
    pub fn server_unable_to_deliver_strategy(&self) -> UnableToDeliverStrategy {
        self.value()
            .defaults
            .request_response
            .server_unable_to_deliver_strategy
            .into()
    }

    #[setter]
    pub fn set_server_unable_to_deliver_strategy(&self, value: &UnableToDeliverStrategy) {
        self.value()
            .defaults
            .request_response
            .server_unable_to_deliver_strategy = (value.clone()).into()
    }

    #[getter]
    pub fn client_expired_connection_buffer(&self) -> usize {
        self.value()
            .defaults
            .request_response
            .client_expired_connection_buffer
    }

    #[setter]
    pub fn set_client_expired_connection_buffer(&self, value: usize) {
        self.value()
            .defaults
            .request_response
            .client_expired_connection_buffer = value
    }

    #[getter]
    pub fn server_expired_connection_buffer(&self) -> usize {
        self.value()
            .defaults
            .request_response
            .server_expired_connection_buffer
    }

    #[setter]
    pub fn set_server_expired_connection_buffer(&self, value: usize) {
        self.value()
            .defaults
            .request_response
            .server_expired_connection_buffer = value
    }

    #[getter]
    pub fn enable_fire_and_forget_requests(&self) -> bool {
        self.value()
            .defaults
            .request_response
            .enable_fire_and_forget_requests
    }

    #[setter]
    pub fn set_enable_fire_and_forget_requests(&self, value: bool) {
        self.value()
            .defaults
            .request_response
            .enable_fire_and_forget_requests = value
    }
}

#[pyclass]
pub struct Global {
    value: Arc<Mutex<iceoryx2::config::Config>>,
}

impl Global {
    fn value(&self) -> MutexGuard<iceoryx2::config::Config> {
        self.value.lock().unwrap()
    }
}

#[pymethods]
impl Global {
    pub fn __str__(&self) -> String {
        format!("{:?}", self.value().global)
    }

    #[getter]
    pub fn service_dir(&self) -> Path {
        Path {
            value: self.value().global.service_dir().clone(),
        }
    }

    #[getter]
    pub fn node_dir(&self) -> Path {
        Path {
            value: self.value().global.node_dir().clone(),
        }
    }

    #[getter]
    pub fn root_path(&self) -> Path {
        Path {
            value: self.value().global.root_path().clone(),
        }
    }

    #[setter]
    pub fn set_root_path(&self, value: &Path) {
        self.value().global.set_root_path(&value.value.clone())
    }

    #[getter]
    pub fn prefix(&self) -> FileName {
        FileName {
            value: self.value().global.prefix.clone(),
        }
    }

    #[setter]
    pub fn set_prefix(&self, value: &FileName) {
        self.value().global.prefix = value.value.clone()
    }
}

#[pyclass]
pub struct Defaults {
    value: Arc<Mutex<iceoryx2::config::Config>>,
}

impl Defaults {
    fn value(&self) -> MutexGuard<iceoryx2::config::Config> {
        self.value.lock().unwrap()
    }
}

#[pymethods]
impl Defaults {
    pub fn __str__(&self) -> String {
        format!("{:?}", self.value().global)
    }

    #[getter]
    pub fn publish_subscribe(&self) -> PublishSubscribe {
        PublishSubscribe {
            value: self.value.clone(),
        }
    }

    #[getter]
    pub fn event(&self) -> Event {
        Event {
            value: self.value.clone(),
        }
    }

    #[getter]
    pub fn request_response(&self) -> RequestResponse {
        RequestResponse {
            value: self.value.clone(),
        }
    }
}

#[pyclass]
pub struct Config {
    value: Arc<Mutex<iceoryx2::config::Config>>,
}

impl Config {
    fn value(&self) -> MutexGuard<iceoryx2::config::Config> {
        self.value.lock().unwrap()
    }
}

#[pymethods]
impl Config {
    pub fn __str__(&self) -> String {
        format!("{:?}", self.value())
    }

    #[getter]
    pub fn global_cfg(&self) -> Global {
        Global {
            value: self.value.clone(),
        }
    }

    #[getter]
    pub fn defaults(&self) -> Defaults {
        Defaults {
            value: self.value.clone(),
        }
    }
}

#[pyfunction]
pub fn default() -> Config {
    Config {
        value: Arc::new(Mutex::new(iceoryx2::config::Config::default())),
    }
}

#[pyfunction]
pub fn global_config() -> Config {
    Config {
        value: Arc::new(Mutex::new(
            iceoryx2::config::Config::global_config().clone(),
        )),
    }
}

#[pyfunction]
pub fn setup_global_config_from_file(config_file: &FilePath) -> PyResult<Config> {
    Ok(Config {
        value: Arc::new(Mutex::new(
            iceoryx2::config::Config::setup_global_config_from_file(&config_file.value.clone())
                .map_err(|e| ConfigCreationError::new_err(format!("{:?}", e)))?
                .clone(),
        )),
    })
}

#[pyfunction]
pub fn from_file(config_file: &FilePath) -> PyResult<Config> {
    Ok(Config {
        value: Arc::new(Mutex::new(
            iceoryx2::config::Config::from_file(&config_file.value.clone())
                .map_err(|e| ConfigCreationError::new_err(format!("{:?}", e)))?
                .clone(),
        )),
    })
}

#[pyfunction]
pub fn default_user_config_file_path() -> FilePath {
    FilePath {
        value: iceoryx2::config::Config::default_user_config_file_path(),
    }
}

#[pyfunction]
pub fn relative_config_path() -> Path {
    Path {
        value: iceoryx2::config::Config::relative_config_path(),
    }
}

#[pyfunction]
pub fn default_config_file_path() -> FilePath {
    FilePath {
        value: iceoryx2::config::Config::default_config_file_path(),
    }
}

#[pyfunction]
pub fn default_config_file_name() -> FileName {
    FileName {
        value: iceoryx2::config::Config::default_config_file_name(),
    }
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
