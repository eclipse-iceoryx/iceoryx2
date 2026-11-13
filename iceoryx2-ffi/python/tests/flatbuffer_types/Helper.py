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

import os
from pathlib import Path

# fmt: off
import iceoryx2 as iox2
from Testing.BoundedData import (BoundedData, BoundedDataAddData,
                                 BoundedDataEnd, BoundedDataStart)
from Testing.Entry import EntryAddData1, EntryEnd, EntryStart
from Testing.UnboundedData import (UnboundedData, UnboundedDataAddEntries,
                                   UnboundedDataAddTitle, UnboundedDataEnd,
                                   UnboundedDataStart,
                                   UnboundedDataStartEntriesVector)

# fmt: on


def create_bounded_data(builder, data):
    BoundedDataStart(builder)
    BoundedDataAddData(builder, data)
    return BoundedDataEnd(builder)


def create_unbounded_data(builder, data):
    title = builder.CreateString("hypnotoad")
    EntryStart(builder)
    EntryAddData1(builder, data)
    entry = EntryEnd(builder)
    UnboundedDataStartEntriesVector(builder, 1)
    builder.PrependUOffsetTRelative(entry)
    entries = builder.EndVector()
    UnboundedDataStart(builder)
    UnboundedDataAddTitle(builder, title)
    UnboundedDataAddEntries(builder, entries)
    return UnboundedDataEnd(builder)


SCHEMA_DIRECTORY = (
    Path(__file__).resolve().parents[4]
    / "iceoryx2-bb"
    / "flatbuffers-generated"
    / "schema"
)


def flatbuffer_tests_schema_path() -> iox2.Path:
    return iox2.Path.new(str(SCHEMA_DIRECTORY))


def get_schema_file(file_name: str) -> iox2.FilePath:
    full_path = os.path.join(SCHEMA_DIRECTORY, file_name)

    return iox2.FilePath.new(full_path)
