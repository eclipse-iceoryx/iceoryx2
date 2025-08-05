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

"""Client example."""

import ctypes

import iceoryx2 as iox2

cycle_time = iox2.Duration.from_secs(1)

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

service = (
    node.service_builder(iox2.ServiceName.new("example//dynamic_request_response"))
    .request_response(iox2.Slice[ctypes.c_uint8], iox2.Slice[ctypes.c_uint8])
    .open_or_create()
)

client = (
    service.client_builder()
    # We guess that the samples are at most 16 bytes in size.
    # This is just a hint to the underlying allocator and is purely optional
    # The better the guess is the less reallocations will be performed
    .initial_max_slice_len(16)
    # The underlying sample size will be increased with a power of two strategy
    # when `Client::loan_slice()` or `Client::loan_slice_uninit()` requires more
    # memory than available.
    .allocation_strategy(iox2.AllocationStrategy.PowerOfTwo).create()
)

COUNTER = 1

try:
    while True:
        required_memory_size = min(1000000, COUNTER * COUNTER)
        request = client.loan_slice_uninit(required_memory_size)
        for byte_idx in range(0, required_memory_size):
            request.payload()[byte_idx] = (byte_idx + COUNTER) % 255
        request = request.assume_init()
        pending_response = request.send()

        print("send request", COUNTER, "with", required_memory_size, "bytes ...")

        node.wait(cycle_time)

        while True:
            response = pending_response.receive()
            if response is not None:
                data = response.payload()
                print("received response with", response.payload().len(), "bytes")
            else:
                break

        COUNTER += 1

except iox2.NodeWaitFailure:
    print("exit")
