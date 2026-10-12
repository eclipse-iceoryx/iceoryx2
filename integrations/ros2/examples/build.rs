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

//! Fails the build with a clear message when a ROS 2 message package the
//! examples translate to has no generated Rust crate in the sourced
//! environment, which `ros-env` would otherwise report as an unresolved
//! import, or when a binary flatbuffer schema the examples embed has not
//! been generated yet.

use std::path::Path;

/// The message packages the examples use through `ros-env`.
const REQUIRED_PACKAGES: [&str; 4] = [
    "builtin_interfaces",
    "std_msgs",
    "geometry_msgs",
    "sensor_msgs",
];

/// The binary flatbuffer schemas the examples bake into the binary.
const BINARY_SCHEMAS: [&str; 2] = [
    "flatbuffer_translator/schemas/joint_readings.bfbs",
    "flatbuffer_translator/schemas/pose.bfbs",
];

fn main() {
    require_message_packages();
    require_binary_schemas();
}

/// Fails the build unless every package in [`REQUIRED_PACKAGES`] provides its
/// Rust crate under `AMENT_PREFIX_PATH`.
fn require_message_packages() {
    println!("cargo:rerun-if-env-changed=AMENT_PREFIX_PATH");

    let prefix_path = std::env::var("AMENT_PREFIX_PATH")
        .expect("AMENT_PREFIX_PATH not set - source the ROS 2 environment before building");
    let prefixes: Vec<&str> = prefix_path
        .split(':')
        .filter(|prefix| !prefix.is_empty())
        .collect();

    let missing: Vec<&str> = REQUIRED_PACKAGES
        .iter()
        .copied()
        .filter(|package| {
            !prefixes
                .iter()
                .any(|prefix| has_rust_crate(prefix, package))
        })
        .collect();
    if !missing.is_empty() {
        panic!(
            "No generated Rust crate found for the ROS 2 message packages {missing:?} under \
             AMENT_PREFIX_PATH. Ensure an environment providing them is sourced before building."
        );
    }
}

/// Fails the build unless every schema in [`BINARY_SCHEMAS`] is present.
fn require_binary_schemas() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("set by cargo");
    let manifest_dir = Path::new(&manifest_dir);

    let missing: Vec<&str> = BINARY_SCHEMAS
        .iter()
        .copied()
        .inspect(|bfbs| println!("cargo:rerun-if-changed={bfbs}"))
        .filter(|bfbs| !manifest_dir.join(bfbs).is_file())
        .collect();
    if !missing.is_empty() {
        panic!(
            "The binary flatbuffer schemas {missing:?} are missing. See \
             integrations/ros2/examples/flatbuffer_translator/README.md for
             instructions on how to generate them."
        );
    }
}

/// Whether `prefix` holds the generated Rust crate of `package` at
/// `share/<package>/rust`.
fn has_rust_crate(prefix: &str, package: &str) -> bool {
    Path::new(prefix)
        .join("share")
        .join(package)
        .join("rust")
        .join("Cargo.toml")
        .is_file()
}
