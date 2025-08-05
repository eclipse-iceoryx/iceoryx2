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

"""Open a service that needs to have some predefined attributes."""

import ctypes
import threading
import time

import iceoryx2 as iox2

cycle_time = iox2.Duration.from_secs(1)

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
# In contrast to Rust, all service variants in python have threadsafe ports
# but at the cost of an additional mutex lock/unlock call.
#
# An `iox2.ServiceType.Ipc` service cannot communicate with an
# `iox2.ServiceType.Local` service.
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

service = (
    node.service_builder(iox2.ServiceName.new("Service-Variants-Example"))
    .publish_subscribe(ctypes.c_uint64)
    .open_or_create()
)

subscriber = service.subscriber_builder().create()


# All ports (like Subscriber, Publisher, Server, Client) are threadsafe
# by default so they can be shared between threads.
class BackgroundThread(threading.Thread):
    """Background thread that uses the subscriber to receive cyclically samples."""

    def __init__(self, stop_event):
        """Initializes the background thread."""
        super().__init__()
        self.stop_event = stop_event

    def run(self):
        """Runs the background thread and receives cyclically samples."""
        while not self.stop_event.is_set():
            time.sleep(cycle_time.as_secs())
            while True:
                sample = subscriber.receive()
                if sample is not None:
                    data = sample.payload()
                    print("[thread] received:", data.contents.value)
                else:
                    break


stop_event = threading.Event()
thread = BackgroundThread(stop_event)
thread.start()

try:
    while True:
        node.wait(cycle_time)
        while True:
            sample = subscriber.receive()
            if sample is not None:
                data = sample.payload()
                print("[main] received:", data.contents.value)
            else:
                break

except iox2.NodeWaitFailure:
    stop_event.set()
    thread.join()
    print("exit")
