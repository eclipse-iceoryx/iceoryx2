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

from Example.Entry import EntryAddData1, EntryAddData2, EntryEnd, EntryStart
from Example.UnboundedData import (
    UnboundedData,
    UnboundedDataAddEntries,
    UnboundedDataAddTitle,
    UnboundedDataEnd,
    UnboundedDataStart,
    UnboundedDataStartEntriesVector,
)

import iceoryx2 as iox2

cycle_time = iox2.Duration.from_secs(1)

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)

config = iox2.config.global_config()

try:
    config.global_cfg.service.flatbuffer_schema_path = iox2.Path.new(
        os.environ["IOX2_FLATBUFFER_SCHEMA_PATH"]
    )
except KeyError:
    raise RuntimeError("Please define IOX2_FLATBUFFER_SCHEMA_PATH!")

node = iox2.NodeBuilder.new().config(config).create(iox2.ServiceType.Ipc)

service = (
    node.service_builder(iox2.ServiceName.new("My/Flatbuffer/Service"))
    .publish_subscribe(iox2.Flatbuffer[UnboundedData])
    .user_header(ctypes.c_uint64)
    .flatbuffer_schema_path(iox2.FilePath.new("unbounded_data.fbs"))
    .open_or_create()
)

publisher = (
    service.publisher_builder()
    .initial_reserved_memory(1024)
    .allocation_strategy(iox2.AllocationStrategy.PowerOfTwo)
    .create()
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
        for i in range(0, 15):
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
