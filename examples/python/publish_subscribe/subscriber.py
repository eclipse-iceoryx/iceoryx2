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

"""Subscriber example."""

import iceoryx2 as iox2
from transmission_data import TransmissionData

cycle_time = iox2.Duration.from_secs(1)

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

service = (
    node.service_builder(iox2.ServiceName.new("My/Funk/ServiceName"))
    .publish_subscribe(TransmissionData)
    .open_or_create()
)

subscriber = service.subscriber_builder().create()

print("Subscriber ready to receive data!")

try:
    while True:
        node.wait(cycle_time)
        while True:
            sample = subscriber.receive()
            if sample is not None:
                data = sample.payload()
                print("received:", data.contents)
            else:
                break

except iox2.NodeWaitFailure:
    print("exit")
