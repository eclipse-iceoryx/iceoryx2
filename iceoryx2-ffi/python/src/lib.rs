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

pub mod cleanup_state;
pub mod config;
pub mod duration;
pub mod error;
pub mod file_name;
pub mod file_path;
pub mod node;
pub mod node_builder;
pub mod node_id;
pub mod node_name;
pub mod node_state;
pub mod parc;
pub mod path;
pub mod service_type;
pub mod signal_handling_mode;
pub mod unable_to_deliver_strategy;

use pyo3::prelude::*;
use pyo3::wrap_pymodule;

/// iceoryx2 Python language bindings
#[pymodule]
fn iceoryx2_ffi_python(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(crate::config::config))?;
    m.add_class::<crate::node_id::NodeId>()?;
    m.add_class::<crate::node::Node>()?;
    m.add_class::<crate::node_state::NodeState>()?;
    m.add_class::<crate::node_state::AliveNodeView>()?;
    m.add_class::<crate::node_state::DeadNodeView>()?;
    m.add_class::<crate::node_state::NodeDetails>()?;
    m.add_class::<crate::node_name::NodeName>()?;
    m.add_class::<crate::node_builder::NodeBuilder>()?;
    m.add_class::<crate::file_name::FileName>()?;
    m.add_class::<crate::file_path::FilePath>()?;
    m.add_class::<crate::signal_handling_mode::SignalHandlingMode>()?;
    m.add_class::<crate::path::Path>()?;
    m.add_class::<crate::duration::Duration>()?;
    m.add_class::<crate::unable_to_deliver_strategy::UnableToDeliverStrategy>()?;
    m.add_class::<crate::service_type::ServiceType>()?;
    m.add(
        "ConfigCreationError",
        py.get_type::<crate::error::ConfigCreationError>(),
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
        "NodeWaitFailure",
        py.get_type::<crate::error::NodeWaitFailure>(),
    )?;
    m.add(
        "SemanticStringError",
        py.get_type::<crate::error::SemanticStringError>(),
    )?;

    Ok(())
}
