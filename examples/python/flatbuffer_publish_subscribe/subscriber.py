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

import ctypes
import os

from Example.UnboundedData import UnboundedData

import iceoryx2 as iox2


# Explicitly sets the type name of our generated type so that the auto path-lookup
# works.
def type_name() -> str:
    """Returns the system-wide unique type name required for communication."""
    return "UnboundedData"


UnboundedData.type_name = staticmethod(type_name)


cycle_time = iox2.Duration.from_secs(1)

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)

# export IOX2_FLATBUFFER_SCHEMA_PATH=${pwd}/examples/rust/flatbuffer_publish_subscribe
try:
    lookup_path = os.environ["IOX2_FLATBUFFER_SCHEMA_PATH"]
except KeyError:
    raise RuntimeError("Please define IOX2_FLATBUFFER_SCHEMA_PATH!")

config = iox2.config.global_config()
config.global_cfg.service.flatbuffer_schema_path = iox2.Path.new(lookup_path)

node = (
    iox2.NodeBuilder.new()
    # Use the config with the defined flatbuffer schema path to enable automatic flatbuffer
    # schema file lookup.
    .config(config)
    .create(iox2.ServiceType.Ipc)
)

service = (
    node.service_builder(iox2.ServiceName.new("My/Flatbuffer/Service"))
    .publish_subscribe(iox2.Flatbuffer[UnboundedData])
    .user_header(ctypes.c_uint64)
    # This method allows us to use a custom schema file path when no schema lookup path was
    # defined or when a custom file is required (maybe outside of the lookup path or
    # or the type_name() was not defined).
    #
    .flatbuffer_schema_path(iox2.FilePath.new("unbounded_data.fbs"))
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
                data = sample.payload_root()
                print("title:", data.Title().decode("utf-8"))
                print("user header:", sample.user_header().contents.value)
                for i in range(data.EntriesLength()):
                    entry = data.Entries(i)
                    print(f"Entry {i}: data_1={entry.Data1()}, data_2={entry.Data2()}")
                print(" ")
            else:
                break

except iox2.NodeWaitFailure:
    print("exit")
