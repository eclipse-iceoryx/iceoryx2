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
# ROS 2 Environment
# =============================================================================

# Fails unless a ROS 2 distribution is sourced.
[private]
_ros2-require-sourced:
    #!/usr/bin/env bash
    if [ -z "${ROS_DISTRO:-}" ]; then
        echo "Error: no sourced ROS 2 environment" >&2
        echo "  Source your ROS 2 setup first, e.g.: source /opt/ros/jazzy/setup.bash" >&2
        exit 1
    fi
