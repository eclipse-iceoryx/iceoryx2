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
import os

import iceoryx2 as iox2
from Example.Entry import EntryAddData1, EntryAddData2, EntryEnd, EntryStart
from Example.UnboundedData import (
    UnboundedData,
    UnboundedDataAddEntries,
    UnboundedDataAddTitle,
    UnboundedDataEnd,
    UnboundedDataStart,
    UnboundedDataStartEntriesVector,
)


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
    .config(config).create(iox2.ServiceType.Ipc)
)

service = (
    node.service_builder(iox2.ServiceName.new("My/Flatbuffer/Service"))
    .publish_subscribe(iox2.Flatbuffer[UnboundedData])
    .user_header(ctypes.c_uint64)
    # This method allows us to use a custom schema file path when no schema lookup path was
    # defined or when a custom file is required (maybe outside of the lookup path or
    # or the type_name() was not defined).
    #
    # .flatbuffer_schema_path(iox2.FilePath.new("unbounded_data.fbs"))
    .open_or_create()
)

publisher = (
    service.publisher_builder()
    # We start with 1024 bytes. The more accurate the initial_reserved_memory
    # estimate is, the fewer reallocations will be required. Reallocations occur
    # only at the beginning of communication. Once the publisher's data segment
    # has been resized appropriately, all subsequent samples will use that size.
    .initial_reserved_memory(1024)
    # By default, the allocation strategy is Static, which does not allow
    # reallocations when initial_reserved_memory is exhausted. Set it to
    # PowerOfTwo or BestFit to enable reallocations.
    #
    # The maximum number of reallocations is 256. BestFit allocates only the
    # explicitly requested amount of memory, so this limit can be reached
    # quickly. Increasing initial_reserved_memory reduces the number of
    # reallocations.
    .allocation_strategy(iox2.AllocationStrategy.PowerOfTwo).create()
)

COUNTER = 0
try:
    while True:
        COUNTER += 1
        node.wait(cycle_time)
        sample = publisher.loan_flatbuffer()
        sample.user_header().contents.value = COUNTER
        builder = sample.flatbuffer_builder()

        # BEGIN: standard flatbuffer API
        entry_offsets = []
        for i in range(0, COUNTER % 15):
            EntryStart(builder)
            EntryAddData1(builder, 6 * i + 5)
            EntryAddData2(builder, 6 * i + 7)
            entry_offsets.append(EntryEnd(builder))

        UnboundedDataStartEntriesVector(builder, len(entry_offsets))
        for offset in reversed(entry_offsets):
            builder.PrependUOffsetTRelative(offset)
        entries_vector = builder.EndVector()

        title = builder.CreateString("Hello World!")

        UnboundedDataStart(builder)
        UnboundedDataAddTitle(builder, title)
        UnboundedDataAddEntries(builder, entries_vector)
        unbounded_data = UnboundedDataEnd(builder)
        # END: standard flatbuffer API

        sample = sample.assume_init(unbounded_data)
        sample.send()

        print("Send sample", COUNTER, "...")

except iox2.NodeWaitFailure:
    print("exit")
