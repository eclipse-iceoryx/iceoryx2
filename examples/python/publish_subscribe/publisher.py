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

import iceoryx2 as iox2
import ctypes

class TransmissionData(ctypes.Structure):
    _fields_ = [("x", ctypes.c_int),
                ("y", ctypes.c_int),
                ("funky", ctypes.c_double)]

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

service = (
    node.service_builder(iox2.ServiceName.new("My/Funk/ServiceName"))
    .publish_subscribe()
    .payload_type_details(iox2.TypeDetail.new().type_variant(iox2.TypeVariant.FixedSize).type_name(iox2.TypeName.new("TransmissionData")).size(16).alignment(8))
    .user_header_type_details(iox2.TypeDetail.new().type_variant(iox2.TypeVariant.FixedSize).type_name(iox2.TypeName.new("()")).size(0).alignment(1))
    .open_or_create()
)

publisher = service.publisher_builder().create()

cycle_time = iox2.Duration.from_secs(1)
counter = 0

try:
    while True:
        counter += 1
        node.wait(cycle_time)
        sample_uninit = publisher.loan_slice_uninit(1)
        data = TransmissionData(x = counter, y = counter * 3, funky = counter * 812.12)
        ctypes.memmove(sample_uninit.payload_ptr, ctypes.byref(data), 16)
        sample = sample_uninit.assume_init()
        sample.send()
        print("send sample")
except iox2.NodeWaitFailure:
    print("exit")
