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

"""Create a service that is strictly restricted to the local process."""

import ctypes
import threading
import time

import iceoryx2 as iox2

cycle_time = iox2.Duration.from_secs(1)


class BackgroundThread(threading.Thread):
    """Background thread with internal node to receive data."""

    def __init__(self, stop_event):
        """Initializes the background thread."""
        super().__init__()
        self.stop_event = stop_event

    def run(self):
        """Runs the background thread and receives cyclically samples."""
        # Another node is created inside this thread to communicate with the main thread
        node = (
            iox2.NodeBuilder.new()
            # Optionally, a name can be provided to the node which helps identifying them later during
            # debugging or introspection
            .name(iox2.NodeName.new("threadnode")).create(iox2.ServiceType.Local)
        )

        service = (
            node.service_builder(iox2.ServiceName.new("Service-Variants-Example"))
            .publish_subscribe(ctypes.c_uint64)
            .open_or_create()
        )

        subscriber = service.subscriber_builder().create()

        while not self.stop_event.is_set():
            time.sleep(cycle_time.as_secs())
            while True:
                sample = subscriber.receive()
                if sample is not None:
                    data = sample.payload()
                    print("received:", data.contents.value)
                else:
                    break


iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
# When choosing `iox2.ServiceType.Local` the service does not use inter-process mechanisms
# like shared memory or unix domain sockets but mechanisms like socketpairs and heap.
#
# Those services can communicate only within a single process.
node = (
    iox2.NodeBuilder.new()
    # Optionally, a name can be provided to the node which helps identifying them later during
    # debugging or introspection
    .name(iox2.NodeName.new("mainnode")).create(iox2.ServiceType.Local)
)

service = (
    node.service_builder(iox2.ServiceName.new("Service-Variants-Example"))
    .publish_subscribe(ctypes.c_uint64)
    .open_or_create()
)

publisher = service.publisher_builder().create()

stop_event = threading.Event()
thread = BackgroundThread(stop_event)
thread.start()

COUNTER = 0
try:
    while True:
        node.wait(cycle_time)
        print("send:", COUNTER)
        publisher.send_copy(ctypes.c_uint64(COUNTER))
        COUNTER += 1

except iox2.NodeWaitFailure:
    stop_event.set()
    thread.join()
    print("exit")
