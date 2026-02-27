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

"""Event-based communication publisher example."""

import os

import iceoryx2 as iox2
from pubsub_event import PubSubEvent, from_event_id, to_event_id
from transmission_data import TransmissionData

SERVICE_NAME = os.environ.get("IOX2_SERVICE_NAME", "My/Funk/ServiceName")
CYCLE_TIME = iox2.Duration.from_secs(1)
HISTORY_SIZE = 20


class CustomPublisher:
    """High-level publisher with additional event channels."""

    def __init__(self, node: iox2.Node, service_name: str):
        pubsub = (
            node.service_builder(iox2.ServiceName.new(service_name))
            .publish_subscribe(TransmissionData)
            .history_size(HISTORY_SIZE)
            .subscriber_max_buffer_size(HISTORY_SIZE)
            .open_or_create()
        )
        event = (
            node.service_builder(iox2.ServiceName.new(service_name))
            .event()
            .open_or_create()
        )

        self._publisher = pubsub.publisher_builder().create()
        self.listener = event.listener_builder().create()
        self._notifier = event.notifier_builder().create()
        self._notifier.notify_with_custom_event_id(
            to_event_id(PubSubEvent.PublisherConnected)
        )

    def handle_event(self):
        """Handles incoming feedback events from subscribers."""
        for event_id in self.listener.try_wait_all():
            event = from_event_id(event_id)
            if event == PubSubEvent.SubscriberConnected:
                print("new subscriber connected - delivering history")
                # Python binding currently has no Publisher::update_connections().
                # Still emit SentHistory for protocol compatibility with Rust/C++.
                self._notifier.notify_with_custom_event_id(
                    to_event_id(PubSubEvent.SentHistory)
                )
            elif event == PubSubEvent.SubscriberDisconnected:
                print("subscriber disconnected")
            elif event == PubSubEvent.ReceivedSample:
                print("subscriber consumed sample")

    def send(self, counter: int):
        """Sends a sample and emits SentSample."""
        sample = self._publisher.loan_uninit()
        sample = sample.write_payload(
            TransmissionData(x=counter, y=counter * 3, funky=counter * 812.12)
        )
        sample.send()
        self._notifier.notify_with_custom_event_id(to_event_id(PubSubEvent.SentSample))

    def shutdown(self):
        """Emits PublisherDisconnected."""
        self._notifier.notify_with_custom_event_id(
            to_event_id(PubSubEvent.PublisherDisconnected)
        )


iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)
publisher = CustomPublisher(node, SERVICE_NAME)

waitset = iox2.WaitSetBuilder.new().create(iox2.ServiceType.Ipc)
publisher_guard = waitset.attach_notification(publisher.listener)
cyclic_guard = waitset.attach_interval(CYCLE_TIME)

counter = 0
print("Publisher ready to send and process events!")

try:
    while True:
        (notifications, result) = waitset.wait_and_process()
        if result in (
            iox2.WaitSetRunResult.TerminationRequest,
            iox2.WaitSetRunResult.Interrupt,
        ):
            break

        for attachment in notifications:
            if attachment.has_event_from(cyclic_guard):
                counter += 1
                print("send:", counter)
                publisher.send(counter)
            elif attachment.has_event_from(publisher_guard):
                publisher.handle_event()
except iox2.WaitSetRunError:
    print("waitset error")
finally:
    try:
        publisher.shutdown()
    except Exception:
        pass

print("exit")
