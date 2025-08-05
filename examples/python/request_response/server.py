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

"""Publisher example."""

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

server = service.server_builder().create()

print("Server ready to receive requests!")

COUNTER = 0
try:
    while True:
        node.wait(cycle_time)
        while True:
            active_request = server.receive()
            if active_request is not None:
                data = active_request.payload()
                print("received request:", data.contents.value)

                # send first response by using the slower, non-zero-copy API
                response = TransmissionData(x=5 + COUNTER, y=6 * COUNTER, funky=7.77)
                print("  send response:", response)
                active_request.send_copy(response)

                # use zero copy API, send out some responses to demonstrate the streaming API
                for n in range(0, data.contents.value % 2):
                    response = active_request.loan_uninit()
                    response = response.write_payload(
                        TransmissionData(
                            x=COUNTER * (n + 1),
                            y=COUNTER + n,
                            funky=COUNTER * 0.1234,
                        )
                    )
                    print("  send response:", response.payload().contents)
                    response.send()

                active_request.delete()
            else:
                break

        COUNTER += 1

except iox2.NodeWaitFailure:
    print("exit")
