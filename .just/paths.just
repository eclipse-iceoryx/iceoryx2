# Copyright (c) 2026 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache Software License 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
# which is available at https://opensource.org/licenses/MIT.
#
# SPDX-License-Identifier: Apache-2.0 OR MIT

# =============================================================================
# Shared Paths
# =============================================================================

INTEGRATIONS_DIR := justfile_directory() / "integrations"
INTEGRATIONS_ZENOH_DIR := INTEGRATIONS_DIR / "zenoh"
INTEGRATIONS_ROS2_DIR := INTEGRATIONS_DIR / "ros2"

# Colcon workspace the ROS 2 examples live in, alongside the generated
# message crates they depend on. Build artifacts are placed in `target/`.
INTEGRATIONS_ROS2_EXAMPLE_WORKSPACE := INTEGRATIONS_ROS2_DIR / "workspace"
INTEGRATIONS_ROS2_EXAMPLE_BUILD_BASE := justfile_directory() / "target" / "ros2" / "build"
INTEGRATIONS_ROS2_EXAMPLE_INSTALL_BASE := justfile_directory() / "target" / "ros2" / "install"
