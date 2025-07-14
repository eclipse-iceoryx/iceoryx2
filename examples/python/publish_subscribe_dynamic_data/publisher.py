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

cycle_time = iox2.Duration.from_secs(1)

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

service = (
    node.service_builder(iox2.ServiceName.new("Service With Dynamic Data"))
    .publish_subscribe(iox2.Slice[ctypes.c_uint8])
    .open_or_create()
)

publisher = (
    service.publisher_builder()
    .initial_max_slice_len(16)
    .allocation_strategy(iox2.AllocationStrategy.PowerOfTwo)
    .create()
)

COUNTER = 1
try:
    while True:
        node.wait(cycle_time)
        required_memory_size = min(COUNTER * COUNTER, 1000000)
        sample = publisher.loan_slice_uninit(required_memory_size)
        for byte_idx in range(0, required_memory_size):
            sample.payload()[byte_idx] = (byte_idx + COUNTER) % 255

        sample = sample.assume_init()
        sample.send()

        print("send sample", COUNTER, "with", required_memory_size, "bytes...")
        COUNTER += 1

except iox2.NodeWaitFailure:
    print("exit")
