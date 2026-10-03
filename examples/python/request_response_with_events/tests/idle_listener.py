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

"""Test observer rejecting notifications when it has sent no requests."""

import sys

import iceoryx2 as iox2

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)
service = (
    node.service_builder(
        iox2.ServiceName.new("example/request_response_with_events/cxx_python/client")
    )
    .event()
    .open_or_create()
)
listener = service.listener_builder().create()
timeout = iox2.Duration.from_secs(2)
print("Idle listener ready!", flush=True)

try:
    while True:
        node.wait(iox2.Duration.from_secs(0))
        if listener.timed_wait(timeout):
            print("Unexpected notification for idle listener", flush=True)
            sys.exit(1)

except iox2.NodeWaitFailure:
    print("exit")

except iox2.ListenerWaitError as error:
    if str(error) != "InterruptSignal":
        raise
    print("exit")
