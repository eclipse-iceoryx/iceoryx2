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

"""Example that highlights service variants for a publisher."""

import ctypes

import iceoryx2 as iox2

cycle_time = iox2.Duration.from_secs(1)

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)

# The create argument defines the service variant. Different variants can use
# different mechanisms. For instance the upcoming `iox2.ServiceType.Cuda` would use GPU memory
# or the `iox2.ServiceType.Local` would use mechanisms that are optimized
# for intra-process communication.
#
# All services which are created via this `Node` use the same service variant.
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

service = (
    node.service_builder(iox2.ServiceName.new("Service-Variants-Example"))
    .publish_subscribe(ctypes.c_uint64)
    .open_or_create()
)

publisher = service.publisher_builder().create()

COUNTER = 0
try:
    while True:
        node.wait(cycle_time)
        print("send:", COUNTER)
        publisher.send_copy(ctypes.c_uint64(COUNTER))
        COUNTER += 1

except iox2.NodeWaitFailure:
    print("exit")
