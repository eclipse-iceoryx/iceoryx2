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

"""Client waiting for notifications addressed to its own listener."""

import ctypes

from request_header import RequestHeader
from transmission_data import TransmissionData

import iceoryx2 as iox2

cycle_time = iox2.Duration.from_secs(1)
timeout = iox2.Duration.from_millis(100)

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

server_event_service = (
    node.service_builder(
        iox2.ServiceName.new("example/request_response_with_events/cxx_python/server")
    )
    .event()
    .open_or_create()
)
server_notifier = server_event_service.notifier_builder().create()

client_event_service = (
    node.service_builder(
        iox2.ServiceName.new("example/request_response_with_events/cxx_python/client")
    )
    .event()
    .open_or_create()
)
client_listener = client_event_service.listener_builder().create()
listener_id = client_listener.id.value

service = (
    node.service_builder(
        iox2.ServiceName.new("example/request_response_with_events/cxx_python")
    )
    .request_response(ctypes.c_uint64, TransmissionData)
    .request_header(RequestHeader)
    .open_or_create()
)
client = service.client_builder().create()

REQUEST_COUNTER = 0
RESPONSE_COUNTER = 0
print("Client ready to send requests!", flush=True)

try:
    while True:
        node.wait(cycle_time)
        print("send request", REQUEST_COUNTER, "...")
        request = client.loan_uninit().write_payload(ctypes.c_uint64(REQUEST_COUNTER))
        request.user_header().contents.set_listener_id(listener_id)
        # Keep this owner alive while waiting for the notification and responses.
        pending_response = request.send()
        server_notifier.notify()

        events_received = sum(
            event.count for event in client_listener.timed_wait(timeout)
        )
        if events_received == 0:
            print("Timeout while waiting for response from server")
            pending_response.delete()
            continue
        print("  number of received notifications from server:", events_received)

        while True:
            response = pending_response.receive()
            if response is None:
                break
            print(
                "  received response",
                RESPONSE_COUNTER,
                "for request",
                REQUEST_COUNTER,
                ":",
                response.payload().contents,
            )
            RESPONSE_COUNTER += 1
            response.delete()

        pending_response.delete()
        REQUEST_COUNTER += 1

except iox2.NodeWaitFailure:
    print("exit")

except iox2.ListenerWaitError as error:
    if str(error) != "InterruptSignal":
        raise
    print("exit")
