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
from custom_header import CustomHeader

cycle_time = iox2.Duration.from_secs(1)

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

service = (
    node.service_builder(iox2.ServiceName.new("My/Funk/ServiceName"))
    .publish_subscribe(ctypes.c_uint64)
    .user_header(CustomHeader)
    .open_or_create()
)

publisher = service.publisher_builder().create()

COUNTER = 0
try:
    while True:
        node.wait(cycle_time)
        COUNTER += 1
        sample = publisher.loan_uninit()

        sample.user_header().contents.version = 123
        sample.user_header().contents.timestamp = 80337 + COUNTER

        sample = sample.write_payload(ctypes.c_uint64(COUNTER))

        sample.send()
        print("Send sample", COUNTER, "...")

except iox2.NodeWaitFailure:
    print("exit")
