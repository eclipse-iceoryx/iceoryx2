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

use crate::{
    config::Config, file_name::FileName, file_path::FilePath, node_name::NodeName, parc::Parc,
    path::Path, service_name::ServiceName,
};

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

#[pyfunction]
/// generate a random and unique file name
pub fn generate_file_name() -> FileName {
    FileName(iceoryx2_bb_posix::testing::generate_file_name())
}

#[pyfunction]
/// generate a random and unique file path located inside the test directory
pub fn generate_file_path() -> FilePath {
    FilePath(iceoryx2_bb_posix::testing::generate_file_path())
}

#[pyfunction]
/// creates a test directory to store anything
pub fn create_test_directory() {
    iceoryx2_bb_posix::testing::create_test_directory();
}

#[pyfunction]
/// returns the path to the test directory
pub fn test_directory() -> Path {
    Path(iceoryx2_bb_posix::config::TEST_DIRECTORY)
}

#[pymodule]
pub fn testing(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(generate_service_name))?;
    m.add_wrapped(wrap_pyfunction!(generate_node_name))?;
    m.add_wrapped(wrap_pyfunction!(generate_isolated_config))?;
    m.add_wrapped(wrap_pyfunction!(generate_file_name))?;
    m.add_wrapped(wrap_pyfunction!(generate_file_path))?;
    m.add_wrapped(wrap_pyfunction!(create_test_directory))?;
    m.add_wrapped(wrap_pyfunction!(test_directory))?;

    Ok(())
}
