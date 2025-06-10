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
use crate::file_name::FileName;
use crate::path::Path;
use pyo3::prelude::*;

#[pyclass(str = "{value:?}")]
pub struct Node {
    value: iceoryx2::config::Node,
}

#[pymethods]
impl Node {
    #[getter]
    pub fn directory(&self) -> Path {
        Path {
            value: self.value.directory.clone(),
        }
    }

    #[setter]
    pub fn set_directory(&mut self, value: &Path) {
        self.value.directory = value.value.clone()
    }

    #[getter]
    pub fn monitor_suffix(&self) -> FileName {
        FileName {
            value: self.value.monitor_suffix.clone(),
        }
    }

    #[setter]
    pub fn set_monitor_suffix(&mut self, value: &FileName) {
        self.value.monitor_suffix = value.value.clone()
    }

    #[getter]
    pub fn static_config_suffix(&self) -> FileName {
        FileName {
            value: self.value.static_config_suffix.clone(),
        }
    }

    #[setter]
    pub fn set_static_config_suffix(&mut self, value: &FileName) {
        self.value.static_config_suffix = value.value.clone()
    }

    #[getter]
    pub fn service_tag_suffix(&self) -> FileName {
        FileName {
            value: self.value.service_tag_suffix.clone(),
        }
    }

    #[setter]
    pub fn set_service_tag_suffix(&mut self, value: &FileName) {
        self.value.service_tag_suffix = value.value.clone()
    }

    #[getter]
    pub fn cleanup_dead_nodes_on_creation(&self) -> bool {
        self.value.cleanup_dead_nodes_on_creation
    }

    #[setter]
    pub fn set_cleanup_dead_nodes_on_creation(&mut self, value: bool) {
        self.value.cleanup_dead_nodes_on_creation = value
    }

    #[getter]
    pub fn cleanup_dead_nodes_on_destruction(&self) -> bool {
        self.value.cleanup_dead_nodes_on_destruction
    }

    #[setter]
    pub fn set_cleanup_dead_nodes_on_destruction(&mut self, value: bool) {
        self.value.cleanup_dead_nodes_on_destruction = value
    }
}

#[pyclass(str = "{value:?}")]
pub struct Service {
    value: iceoryx2::config::Service,
}

#[pymethods]
impl Service {
    #[getter]
    pub fn directory(&self) -> Path {
        Path {
            value: self.value.directory.clone(),
        }
    }

    #[setter]
    pub fn set_directory(&mut self, value: &Path) {
        self.value.directory = value.value.clone()
    }

    #[getter]
    pub fn data_segment_suffix(&self) -> FileName {
        FileName {
            value: self.value.data_segment_suffix.clone(),
        }
    }

    #[setter]
    pub fn set_data_segment_suffix(&mut self, value: &FileName) {
        self.value.data_segment_suffix = value.value.clone()
    }

    #[getter]
    pub fn static_config_storage_suffix(&self) -> FileName {
        FileName {
            value: self.value.static_config_storage_suffix.clone(),
        }
    }

    #[setter]
    pub fn set_static_config_storage_suffix(&mut self, value: &FileName) {
        self.value.static_config_storage_suffix = value.value.clone()
    }

    #[getter]
    pub fn dynamic_config_storage_suffix(&self) -> FileName {
        FileName {
            value: self.value.dynamic_config_storage_suffix.clone(),
        }
    }

    #[setter]
    pub fn set_dynamic_config_storage_suffix(&mut self, value: &FileName) {
        self.value.dynamic_config_storage_suffix = value.value.clone()
    }

    #[getter]
    pub fn creation_timeout(&self) -> Duration {
        Duration {
            value: self.value.creation_timeout,
        }
    }

    #[setter]
    pub fn set_creation_timeout(&mut self, value: &Duration) {
        self.value.creation_timeout = value.value.clone()
    }

    #[getter]
    pub fn connection_suffix(&self) -> FileName {
        FileName {
            value: self.value.connection_suffix.clone(),
        }
    }

    #[setter]
    pub fn set_connection_suffix(&mut self, value: &FileName) {
        self.value.connection_suffix = value.value.clone()
    }

    #[getter]
    pub fn event_connection_suffix(&self) -> FileName {
        FileName {
            value: self.value.event_connection_suffix.clone(),
        }
    }

    #[setter]
    pub fn set_event_connection_suffix(&mut self, value: &FileName) {
        self.value.event_connection_suffix = value.value.clone()
    }
}

#[pyclass(str = "{value:?}")]
pub struct Global {
    value: iceoryx2::config::Global,
}

#[pymethods]
impl Global {
    #[getter]
    pub fn service_dir(&self) -> Path {
        Path {
            value: self.value.service_dir().clone(),
        }
    }

    #[getter]
    pub fn node_dir(&self) -> Path {
        Path {
            value: self.value.node_dir().clone(),
        }
    }

    #[getter]
    pub fn root_path(&self) -> Path {
        Path {
            value: self.value.root_path().clone(),
        }
    }

    #[setter]
    pub fn set_root_path(&mut self, value: &Path) {
        self.value.set_root_path(&value.value.clone())
    }

    #[getter]
    pub fn prefix(&self) -> FileName {
        FileName {
            value: self.value.prefix.clone(),
        }
    }

    #[setter]
    pub fn set_prefix(&mut self, value: &FileName) {
        self.value.prefix = value.value.clone()
    }
}
