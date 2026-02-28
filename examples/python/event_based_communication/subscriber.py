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

"""Event-based communication subscriber example."""

import iceoryx2 as iox2
from pubsub_event import PubSubEvent, from_event_id, to_event_id
from transmission_data import TransmissionData

SERVICE_NAME = "My/Funk/ServiceName"
DEADLINE = iox2.Duration.from_secs(2)
HISTORY_SIZE = 20


class CustomSubscriber:
    """High-level subscriber with additional event channels."""

    def __init__(self, node: iox2.Node, service_name: str):
        """Initializes the subscriber with publish-subscribe and event services."""
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

        self._subscriber = pubsub.subscriber_builder().create()
        self.listener = event.listener_builder().create()
        self._notifier = event.notifier_builder().create()
        self._notifier.notify_with_custom_event_id(
            to_event_id(PubSubEvent.SubscriberConnected)
        )

    def _receive(self):
        sample = self._subscriber.receive()
        if sample is None:
            return None

        self._notifier.notify_with_custom_event_id(
            to_event_id(PubSubEvent.ReceivedSample)
        )
        return sample.payload().contents

    def _drain_samples(self):
        values = []
        while True:
            payload = self._receive()
            if payload is None:
                break
            values.append(payload)
        return values

    def handle_event(self):
        """Handles incoming events from publishers."""
        for event_id in self.listener.try_wait_all():
            event = from_event_id(event_id)
            if event == PubSubEvent.SentHistory:
                print("History delivered")
                for sample in self._drain_samples():
                    print("  history:", sample)
            elif event == PubSubEvent.SentSample:
                for sample in self._drain_samples():
                    print("received:", sample)
            elif event == PubSubEvent.PublisherConnected:
                print("new publisher connected")
            elif event == PubSubEvent.PublisherDisconnected:
                print("publisher disconnected")

    def shutdown(self):
        """Emits SubscriberDisconnected."""
        self._notifier.notify_with_custom_event_id(
            to_event_id(PubSubEvent.SubscriberDisconnected)
        )


iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)
subscriber = CustomSubscriber(node, SERVICE_NAME)

waitset = iox2.WaitSetBuilder.new().create(iox2.ServiceType.Ipc)
subscriber_guard = waitset.attach_deadline(subscriber.listener, DEADLINE)

print("Subscriber ready to receive data!")

try:
    while True:
        (notifications, result) = waitset.wait_and_process()
        if result in (
            iox2.WaitSetRunResult.TerminationRequest,
            iox2.WaitSetRunResult.Interrupt,
        ):
            break

        for attachment in notifications:
            if attachment.has_event_from(subscriber_guard):
                subscriber.handle_event()
            elif attachment.has_missed_deadline(subscriber_guard):
                print(
                    "Contract violation! "
                    f"The subscriber did not receive a message for {DEADLINE}."
                )
except iox2.WaitSetRunError:
    print("waitset error")
finally:
    subscriber.shutdown()

print("exit")
