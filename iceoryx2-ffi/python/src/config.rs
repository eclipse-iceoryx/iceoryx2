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

use std::sync::{Arc, Mutex, MutexGuard};

use crate::duration::Duration;
use crate::error::{ConfigCreationError, UnableToDeliverStrategy};
use crate::file_name::FileName;
use crate::file_path::FilePath;
use crate::path::Path;
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

    #[staticmethod]
    pub fn default() -> Config {
        Config {
            value: Arc::new(Mutex::new(iceoryx2::config::Config::default())),
        }
    }

    #[staticmethod]
    pub fn global_config() -> Config {
        Config {
            value: Arc::new(Mutex::new(
                iceoryx2::config::Config::global_config().clone(),
            )),
        }
    }

    #[staticmethod]
    pub fn setup_global_config_from_file(config_file: &FilePath) -> PyResult<Config> {
        Ok(Config {
            value: Arc::new(Mutex::new(
                iceoryx2::config::Config::setup_global_config_from_file(&config_file.value.clone())
                    .map_err(|e| ConfigCreationError::new_err(format!("{:?}", e)))?
                    .clone(),
            )),
        })
    }

    #[staticmethod]
    pub fn from_file(config_file: &FilePath) -> PyResult<Config> {
        Ok(Config {
            value: Arc::new(Mutex::new(
                iceoryx2::config::Config::from_file(&config_file.value.clone())
                    .map_err(|e| ConfigCreationError::new_err(format!("{:?}", e)))?
                    .clone(),
            )),
        })
    }

    #[staticmethod]
    pub fn default_user_config_file_path() -> FilePath {
        FilePath {
            value: iceoryx2::config::Config::default_user_config_file_path(),
        }
    }

    #[staticmethod]
    pub fn relative_config_path() -> Path {
        Path {
            value: iceoryx2::config::Config::relative_config_path(),
        }
    }

    #[staticmethod]
    pub fn default_config_file_path() -> FilePath {
        FilePath {
            value: iceoryx2::config::Config::default_config_file_path(),
        }
    }

    #[staticmethod]
    pub fn default_config_file_name() -> FileName {
        FileName {
            value: iceoryx2::config::Config::default_config_file_name(),
        }
    }
}
