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

"""Example that creates a service with defined attributes."""

import ctypes

import iceoryx2 as iox2

cycle_time = iox2.Duration.from_secs(1)

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

service = (
    node.service_builder(iox2.ServiceName.new("My/Funk/ServiceName"))
    .publish_subscribe(ctypes.c_uint64)
    .create_with_attributes(
        iox2.AttributeSpecifier.new()
        .define(
            iox2.AttributeKey.new("dds_service_mapping"),
            iox2.AttributeValue.new("my_funky_service_name"),
        )
        .define(
            iox2.AttributeKey.new("tcp_serialization_format"),
            iox2.AttributeValue.new("cdr"),
        )
        .define(
            iox2.AttributeKey.new("someip_service_mapping"),
            iox2.AttributeValue.new("1/2/3"),
        )
        .define(
            iox2.AttributeKey.new("camera_resolution"),
            iox2.AttributeValue.new("1920x1080"),
        )
    )
)

publisher = service.publisher_builder().create()

print("defined service attributes:", service.attributes)

COUNTER = 0
try:
    while True:
        COUNTER += 1
        node.wait(cycle_time)
        sample = publisher.loan_uninit()
        sample = sample.write_payload(ctypes.c_uint64(0))
        sample.send()

except iox2.NodeWaitFailure:
    print("exit")
