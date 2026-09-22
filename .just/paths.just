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

# The colcon workspaces, the message crates generated from the ROS 2 message
# definitions and the examples overlaying them. Build artifacts are placed in
# `target/`.
INTEGRATIONS_ROS2_COLCON_DIR := INTEGRATIONS_ROS2_DIR / "colcon"
INTEGRATIONS_ROS2_MESSAGES_WORKSPACE := INTEGRATIONS_ROS2_COLCON_DIR / "messages"
INTEGRATIONS_ROS2_MESSAGES_BUILD_BASE := justfile_directory() / "target" / "ros2" / "messages" / "build"
INTEGRATIONS_ROS2_MESSAGES_INSTALL_BASE := justfile_directory() / "target" / "ros2" / "messages" / "install"
INTEGRATIONS_ROS2_EXAMPLES_WORKSPACE := INTEGRATIONS_ROS2_COLCON_DIR / "examples"
INTEGRATIONS_ROS2_EXAMPLES_BUILD_BASE := justfile_directory() / "target" / "ros2" / "examples" / "build"
INTEGRATIONS_ROS2_EXAMPLES_INSTALL_BASE := justfile_directory() / "target" / "ros2" / "examples" / "install"
