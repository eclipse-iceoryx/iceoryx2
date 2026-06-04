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

"""Listener example."""

import iceoryx2 as iox2

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

event = (
    node.service_builder(iox2.ServiceName.new("MyEventName")).event().open_or_create()
)

listener = event.listener_builder().create()

print("Listener ready to receive events!")

cycle_time = iox2.Duration.from_secs(1)

try:
    while True:
        events = listener.timed_wait(cycle_time)
        for event in listener.timed_wait(cycle_time):
            print("event was triggered with id: ", event.id, " ", event.count, " times")
except iox2.ListenerWaitError:
    print("exit")
