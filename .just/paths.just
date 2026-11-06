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

# One target directory per distribution.
INTEGRATIONS_ROS2_COLCON_DIR := INTEGRATIONS_ROS2_DIR / "colcon"
INTEGRATIONS_ROS2_TARGET_DIR := INTEGRATIONS_ROS2_DIR / "target" / "${ROS_DISTRO}"
INTEGRATIONS_ROS2_CARGO_TARGET_DIR := INTEGRATIONS_ROS2_TARGET_DIR / "cargo"
INTEGRATIONS_ROS2_MESSAGES_WORKSPACE := INTEGRATIONS_ROS2_COLCON_DIR / "messages"
INTEGRATIONS_ROS2_MESSAGES_TARGET_DIR := INTEGRATIONS_ROS2_TARGET_DIR / "colcon" / "messages"
INTEGRATIONS_ROS2_MESSAGES_SOURCES := INTEGRATIONS_ROS2_MESSAGES_TARGET_DIR / "src"
INTEGRATIONS_ROS2_MESSAGES_INSTALL_BASE := INTEGRATIONS_ROS2_MESSAGES_TARGET_DIR / "install"
INTEGRATIONS_ROS2_EXAMPLES_WORKSPACE := INTEGRATIONS_ROS2_COLCON_DIR / "examples"
INTEGRATIONS_ROS2_EXAMPLES_TARGET_DIR := INTEGRATIONS_ROS2_TARGET_DIR / "colcon" / "examples"
INTEGRATIONS_ROS2_EXAMPLES_INSTALL_BASE := INTEGRATIONS_ROS2_EXAMPLES_TARGET_DIR / "install"

ICEORYX2_BB_DIR := justfile_directory() / "iceoryx2-bb"
ICEORYX2_BB_FLATBUFFERS_GENERATED_DIR := ICEORYX2_BB_DIR / "flatbuffers-generated"

ICEORYX2_FFI_DIR := justfile_directory() / "iceoryx2-ffi"
ICEORYX2_FFI_PYTHON_DIR := ICEORYX2_FFI_DIR / "python"
ICEORYX2_FFI_PYTHON_FLATBUFFER_TYPES_DIR := ICEORYX2_FFI_PYTHON_DIR / "tests" / "flatbuffer_types"
FLATC_CACHE_DIR := justfile_directory() / "target" / "flatc"
