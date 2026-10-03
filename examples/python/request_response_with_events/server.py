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

"""Server sending a notification only to the client receiving its response."""

import ctypes
from functools import partial

import iceoryx2 as iox2
from request_header import RequestHeader
from transmission_data import TransmissionData

timeout = iox2.Duration.from_secs(2)

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

server_event_service = (
    node.service_builder(
        iox2.ServiceName.new("example/request_response_with_events/cxx_python/server")
    )
    .event()
    .open_or_create()
)
server_listener = server_event_service.listener_builder().create()

client_event_service = (
    node.service_builder(
        iox2.ServiceName.new("example/request_response_with_events/cxx_python/client")
    )
    .event()
    .open_or_create()
)
client_notifier = client_event_service.notifier_builder().create()

service = (
    node.service_builder(
        iox2.ServiceName.new("example/request_response_with_events/cxx_python")
    )
    .request_response(ctypes.c_uint64, TransmissionData)
    .request_header(RequestHeader)
    .open_or_create()
)
server = service.server_builder().create()

print("Server ready to receive requests!", flush=True)
COUNTER = 0


def notify_client(
    listener_id: int, monofier: iox2.Monofier, details: iox2.ListenerDetails
) -> iox2.CallbackProgression:
    """Use callback-scoped Monofier only for the request's listener."""
    if details.listener_id.value == listener_id:
        monofier.notify()
        return iox2.CallbackProgression.Stop
    return iox2.CallbackProgression.Continue


try:
    while True:
        node.wait(iox2.Duration.from_secs(0))
        if not server_listener.timed_wait(timeout):
            print("Timeout while waiting for clients")
            continue

        while True:
            active_request = server.receive()
            if active_request is None:
                break
            print("received request:", active_request.payload().contents.value)
            response = TransmissionData(x=5 + COUNTER, y=6 * COUNTER, funky=7.77)
            print("  send response:", response)
            active_request.send_copy(response)

            # Copy the ID before releasing the request. The callback runs synchronously.
            listener_id = active_request.user_header().contents.listener_id_value()
            client_notifier.for_each_listener(partial(notify_client, listener_id))
            active_request.delete()

        COUNTER += 1

except iox2.NodeWaitFailure:
    print("exit")

except iox2.ListenerWaitError as error:
    if str(error) != "InterruptSignal":
        raise
    print("exit")
