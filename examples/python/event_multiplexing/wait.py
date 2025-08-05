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

"""Event Multiplexing listener example."""

import sys

import iceoryx2 as iox2

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

listeners = []
for i in range(1, len(sys.argv)):
    event = (
        node.service_builder(iox2.ServiceName.new(sys.argv[i])).event().open_or_create()
    )

    listener = event.listener_builder().create()
    listeners.append((sys.argv[i], listener))

waitset = iox2.WaitSetBuilder.new().create(iox2.ServiceType.Ipc)
listener_attachments = {}
guards = []

for service, listener in listeners:
    guard = waitset.attach_notification(listener)
    print("attachment: ", iox2.WaitSetAttachmentId.from_guard(guard))
    listener_attachments[iox2.WaitSetAttachmentId.from_guard(guard)] = (
        service,
        listener,
    )
    guards.append(guard)

print("Waiting on the following services: ", sys.argv[1:None])

try:
    while True:
        (notifications, result) = waitset.wait_and_process()
        if result in (
            iox2.WaitSetRunResult.TerminationRequest,
            iox2.WaitSetRunResult.Interrupt,
        ):
            break

        for attachment in notifications:
            (service_name, listener) = listener_attachments[attachment]
            print('Received trigger from "', service_name, '"')

            event_ids = listener.try_wait_all()
            for event_id in event_ids:
                print(event_id, " ")

except iox2.WaitSetRunError:
    print("exception raised")

print("exit")
