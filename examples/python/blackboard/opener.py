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

"""Blackboard opener example."""

from ctypes import c_double, c_int32

import iceoryx2 as iox2
from blackboard_complex_key import BlackboardKey

cycle_time = iox2.Duration.from_secs(1)

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

key_0 = BlackboardKey(x=0, y=-4, z=4)
key_1 = BlackboardKey(x=1, y=-4, z=4)
service = (
    node.service_builder(iox2.ServiceName.new("My/Funk/ServiceName"))
    .blackboard_opener(BlackboardKey)
    .open()
)

reader = service.reader_builder().create()

entry_handle_0 = reader.entry(key_0, c_int32)
entry_handle_1 = reader.entry(key_1, c_double)

try:
    while True:
        node.wait(cycle_time)
        print("read values:")

        print("key: 0, value:", entry_handle_0.get().decode_as(c_int32).value)
        print("key: 1, value:", entry_handle_1.get().decode_as(c_double).value, "\n")

except iox2.NodeWaitFailure:
    print("exit")
