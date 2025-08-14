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

"""Client example."""

import ctypes

import iceoryx2 as iox2
from transmission_data import TransmissionData

cycle_time = iox2.Duration.from_secs(1)

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

service = (
    node.service_builder(iox2.ServiceName.new("My/Funk/ServiceName"))
    .request_response(ctypes.c_uint64, TransmissionData)
    .open_or_create()
)

client = service.client_builder().create()

REQUEST_COUNTER = 0
RESPONSE_COUNTER = 0

# sending first request by using slower, inefficient copy API
print("send request", REQUEST_COUNTER, "...")
pending_response = client.send_copy(ctypes.c_uint64(REQUEST_COUNTER))

try:
    while True:
        node.wait(cycle_time)
        while True:
            response = pending_response.receive()
            if response is not None:
                data = response.payload()
                print("received response", RESPONSE_COUNTER, ":", data.contents)
                RESPONSE_COUNTER += 1
            else:
                break

        REQUEST_COUNTER += 1
        request = client.loan_uninit()
        request = request.write_payload(ctypes.c_uint64(REQUEST_COUNTER))

        pending_response = request.send()

        print("send request", REQUEST_COUNTER, "...")

except iox2.NodeWaitFailure:
    print("exit")
