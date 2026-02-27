# Copyright (c) 2026 Contributors to the Eclipse Foundation
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

"""Minimal event-driven subscriber example."""

import os

import iceoryx2 as iox2
from transmission_data import TransmissionData

SERVICE_NAME = os.environ.get("IOX2_SERVICE_NAME", "My/Funk/ServiceName")
DEADLINE = iox2.Duration.from_secs(2)

# Keep the same IDs as the Rust/C++ event-based communication example.
EVENT_SENT_SAMPLE = iox2.EventId.new(4)
EVENT_RECEIVED_SAMPLE = iox2.EventId.new(5)

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

pubsub = (
    node.service_builder(iox2.ServiceName.new(SERVICE_NAME))
    .publish_subscribe(TransmissionData)
    .open_or_create()
)
subscriber = pubsub.subscriber_builder().create()

event = node.service_builder(iox2.ServiceName.new(SERVICE_NAME)).event().open_or_create()
listener = event.listener_builder().create()
notifier = event.notifier_builder().create()

waitset = iox2.WaitSetBuilder.new().create(iox2.ServiceType.Ipc)
listener_guard = waitset.attach_deadline(listener, DEADLINE)

print("Minimal event-driven subscriber waiting for events...")

try:
    while True:
        (notifications, result) = waitset.wait_and_process()
        if result in (
            iox2.WaitSetRunResult.TerminationRequest,
            iox2.WaitSetRunResult.Interrupt,
        ):
            break

        for attachment in notifications:
            if attachment.has_event_from(listener_guard):
                for event_id in listener.try_wait_all():
                    if event_id == EVENT_SENT_SAMPLE:
                        while True:
                            sample = subscriber.receive()
                            if sample is None:
                                break

                            print("received:", sample.payload().contents)
                            notifier.notify_with_custom_event_id(EVENT_RECEIVED_SAMPLE)
            elif attachment.has_missed_deadline(listener_guard):
                print(
                    "Contract violation! "
                    f"The subscriber did not receive a message for {DEADLINE}."
                )
except iox2.WaitSetRunError:
    print("waitset error")

print("exit")
