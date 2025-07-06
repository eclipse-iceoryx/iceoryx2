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

"""Publisher example."""

import ctypes

import iceoryx2 as iox2


class TransmissionData(ctypes.Structure):
    """The strongly typed payload type."""

    _fields_ = [
        ("x", ctypes.c_int),
        ("y", ctypes.c_int),
        ("funky", ctypes.c_double),
    ]

    @staticmethod
    def type_name():
        return "TransmissionData"

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

service = (
    node.service_builder(iox2.ServiceName.new("My/Funk/ServiceName"))
    .publish_subscribe(TransmissionData)
    .open_or_create()
)

publisher = service.publisher_builder().create()

cycle_time = iox2.Duration.from_secs(1)
COUNTER = 0

try:
    while True:
        COUNTER += 1
        node.wait(cycle_time)
        sample_uninit = publisher.loan_slice_uninit(1)
        data = TransmissionData(
            x=COUNTER, y=COUNTER * 3, funky=COUNTER * 812.12
        )
        ctypes.memmove(sample_uninit.payload_ptr, ctypes.byref(data), 16)
        sample = sample_uninit.assume_init()
        sample.send()
        print("send sample")
except iox2.NodeWaitFailure:
    print("exit")
