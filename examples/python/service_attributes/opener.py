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

service = (
    node.service_builder(iox2.ServiceName.new("My/Funk/ServiceName"))
    .publish_subscribe(ctypes.c_uint64)
    .open_with_attributes(
        iox2.AttributeVerifier.new()
        .require(
            iox2.AttributeKey.new("camera_resolution"),
            iox2.AttributeValue.new("1920x1080"),
        )
        .require_key(iox2.AttributeKey.new("dds_service_mapping"))
    )
)

subscriber = service.subscriber_builder().create()

print("defined service attributes:", service.attributes)

COUNTER = 0
try:
    while True:
        COUNTER += 1
        node.wait(cycle_time)
        while True:
            sample = subscriber.receive()
            if sample is not None:
                data = sample.payload()
                print("received:", data.contents)
            else:
                break

except iox2.NodeWaitFailure:
    print("exit")
