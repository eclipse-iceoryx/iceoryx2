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

use pyo3::prelude::*;

use crate::{config::Config, node_name::NodeName, parc::Parc, service_name::ServiceName};

#[pyfunction]
/// generates a system-wide unique `ServiceName`
pub fn generate_service_name() -> ServiceName {
    ServiceName(iceoryx2::testing::generate_service_name())
}

#[pyfunction]
/// generates a system-wide unique `NodeName`
pub fn generate_node_name() -> NodeName {
    NodeName(iceoryx2::testing::generate_node_name())
}

#[pyfunction]
/// generates a iceoryx2 `Config` that does not overlap with any other configuration
pub fn generate_isolated_config() -> Config {
    Config(Parc::new(iceoryx2::testing::generate_isolated_config()))
}

#[pymodule]
pub fn testing(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(generate_service_name))?;
    m.add_wrapped(wrap_pyfunction!(generate_node_name))?;
    m.add_wrapped(wrap_pyfunction!(generate_isolated_config))?;
    Ok(())
}
