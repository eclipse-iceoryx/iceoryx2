# Copyright (c) 2025 Contributors to the Eclipse Foundation
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

"""Open a service that needs to have some predefined attributes."""

import ctypes

import iceoryx2 as iox2

cycle_time = iox2.Duration.from_secs(1)

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

try:
    service = (
        node.service_builder(iox2.ServiceName.new("My/Funk/ServiceName"))
        .publish_subscribe(ctypes.c_uint64)
        .open_with_attributes(
            iox2.AttributeVerifier.new()
            # the opening of the service will fail since the
            # `camera_resolution` attribute is `1920x1080` and not `3840x2160`
            .require(
                iox2.AttributeKey.new("camera_resolution"),
                iox2.AttributeValue.new("3840x2160"),
            )
        )
    )
except iox2.PublishSubscribeOpenError as e:
    print(f"camera_resolution: 3840x2160 -> {e}")


try:
    service = (
        node.service_builder(iox2.ServiceName.new("My/Funk/ServiceName"))
        .publish_subscribe(ctypes.c_uint64)
        .open_with_attributes(
            iox2.AttributeVerifier.new()
            # the opening of the service will fail since the key is not defined.
            .require_key(iox2.AttributeKey.new("camera_type"))
        )
    )
except iox2.PublishSubscribeOpenError as e:
    print(f"camera_type -> {e}")
