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

"""Blackboard creator example."""

from ctypes import c_double, c_int32

import iceoryx2 as iox2
from blackboard_complex_key import BlackboardKey

cycle_time = iox2.Duration.from_secs(1)

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

key_0 = BlackboardKey(x=0, y=-4, z=4)
key_1 = BlackboardKey(x=1, y=-4, z=4)
INITIAL_VALUE_1 = c_double(1.1)
service = (
    node.service_builder(iox2.ServiceName.new("My/Funk/ServiceName"))
    .blackboard_creator(BlackboardKey)
    .add(key_0, c_int32(3))
    .add(key_1, INITIAL_VALUE_1)
    .create()
)

print("Blackboard created.\n")

writer = service.writer_builder().create()

entry_handle_mut_0 = writer.entry(key_0, c_int32)
entry_handle_mut_1 = writer.entry(key_1, c_double)

COUNTER = 0
try:
    while True:
        COUNTER += 1
        node.wait(cycle_time)

        entry_handle_mut_0.update_with_copy(c_int32(COUNTER))
        print("Write new value for key 0:", COUNTER)

        entry_value_uninit = entry_handle_mut_1.loan_uninit()
        value = INITIAL_VALUE_1.value * COUNTER
        entry_value = entry_value_uninit.write(c_double(value))
        entry_handle_mut_1 = entry_value.update()
        print("Write new value for key 1:", value, "\n")

except iox2.NodeWaitFailure:
    print("exit")
