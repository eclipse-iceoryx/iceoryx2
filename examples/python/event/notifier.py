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

"""Notifier example."""

import iceoryx2 as iox2

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

event = (
    node.service_builder(iox2.ServiceName.new("MyEventName")).event().open_or_create()
)
max_event_id = event.static_config.event_id_max_value
notifier = event.notifier_builder().create()

COUNTER = 0
cycle_time = iox2.Duration.from_secs(1)

try:
    while True:
        node.wait(cycle_time)
        COUNTER += 1
        notifier.notify_with_custom_event_id(iox2.EventId.new(COUNTER % max_event_id))

        print("Trigger event with id ", COUNTER, " ...")
except iox2.NodeWaitFailure:
    print("exit")
