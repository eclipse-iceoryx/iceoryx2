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
//! import.

use std::path::Path;

/// The message packages the examples use through `ros-env`.
const REQUIRED_PACKAGES: [&str; 2] = ["std_msgs", "geometry_msgs"];

fn main() {
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
             AMENT_PREFIX_PATH. Build the message workspace with `just setup \
             integrations-ros2-messages` and source \
             integrations/ros2/target/<distro>/colcon/messages/install/setup.bash before building."
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
