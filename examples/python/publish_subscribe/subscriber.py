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

"""Subscriber example."""

import ctypes

import iceoryx2 as iox2


class TransmissionData(ctypes.Structure):
    """The strongly typed payload type."""

    _fields_ = [
        ("x", ctypes.c_int),
        ("y", ctypes.c_int),
        ("funky", ctypes.c_double),
    ]


iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

service = (
    node.service_builder(iox2.ServiceName.new("My/Funk/ServiceName"))
    .publish_subscribe()
    .payload_type_details(
        iox2.TypeDetail.new()
        .type_variant(iox2.TypeVariant.FixedSize)
        .type_name(iox2.TypeName.new("TransmissionData"))
        .size(16)
        .alignment(8)
    )
    .user_header_type_details(
        iox2.TypeDetail.new()
        .type_variant(iox2.TypeVariant.FixedSize)
        .type_name(iox2.TypeName.new("()"))
        .size(0)
        .alignment(1)
    )
    .open_or_create()
)

subscriber = service.subscriber_builder().create()

cycle_time = iox2.Duration.from_secs(1)

try:
    while True:
        node.wait(cycle_time)
        while True:
            sample = subscriber.receive()
            if sample is not None:
                data = TransmissionData()
                ctypes.memmove(ctypes.byref(data), sample.payload_ptr, 16)
                print("received data", data.x, " ", data.y)
            else:
                break

except iox2.NodeWaitFailure:
    print("exit")
