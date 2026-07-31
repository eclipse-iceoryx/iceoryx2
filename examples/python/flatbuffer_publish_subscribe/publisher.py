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

from Example import UnboundedData

import iceoryx2 as iox2

cycle_time = iox2.Duration.from_secs(1)

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

service = (
    node.service_builder(iox2.ServiceName.new("My/Flatbuffer/Service"))
    .publish_subscribe(iox2.Flatbuffer[UnboundedData])
    .user_header(ctypes.c_uint64)
    .flatbuffer_schema_path(
        iox2.FilePath.new(
            "/home/elchris/Development/ekxide/prime/iceoryx2/examples/python/flatbuffer_publish_subscribe/unbounded_data.fbs"
        )
    )
    .open_or_create()
)

publisher = (
    service.publisher_builder()
    .initial_reserved_memory(1024)
    .allocation_strategy(iox2.AllocationStrategy.PowerOfTwo)
    .create()
)

COUNTER = 0
try:
    while True:
        COUNTER += 1
        node.wait(cycle_time)
        # sample = publisher.loan_uninit()
        # sample = sample.write_payload(
        #     TransmissionData(x=COUNTER, y=COUNTER * 3, funky=COUNTER * 812.12)
        # )
        # sample.send()
        print("Send sample", COUNTER, "...")

except iox2.NodeWaitFailure:
    print("exit")
