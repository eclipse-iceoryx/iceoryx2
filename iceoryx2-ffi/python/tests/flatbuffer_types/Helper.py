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

# fmt: off
import iceoryx2 as iox2

from .BoundedData import (BoundedData, BoundedDataAddData, BoundedDataEnd,
                          BoundedDataStart)
from .UnboundedData import (UnboundedData, UnboundedDataAddData,
                            UnboundedDataAddText, UnboundedDataEnd,
                            UnboundedDataStart)

# fmt: on

schema_bounded = """
table BoundedData {
    data: int32;
}

root_type BoundedData;
"""

schema_unbounded = """
table UnboundedData {
    text: string;
    data: int32;
}

root_type UnboundedData;
"""

schema_incompatible = """
table IncompatibleData {
    data_1: int32;
    data_2: int32;
}

root_type UnboundedData;
"""


def create_bounded_data(builder, data):
    BoundedDataStart(builder)
    BoundedDataAddData(builder, data)
    return BoundedDataEnd(builder)


def create_unbounded_data(builder, data):
    text = builder.CreateString("hypnotoad")
    UnboundedDataStart(builder)
    UnboundedDataAddText(builder, text)
    UnboundedDataAddData(builder, data)
    return UnboundedDataEnd(builder)


def create_schema_file(content) -> iox2.FilePath:
    iox2.testing.create_test_directory()
    file_path = iox2.testing.generate_file_path()
    with open(file_path.to_string(), "w", encoding="utf-8") as file:
        file.write(content)

    return file_path


def create_schema_file_at(content: str, file_name: str) -> iox2.FilePath:
    iox2.testing.create_test_directory()
    lookup_path = iox2.testing.test_directory()
    dir_path = lookup_path.to_string()
    full_path = os.path.join(dir_path, file_name)
    with open(full_path, "w", encoding="utf-8") as file:
        file.write(content)

    return iox2.FilePath.new(full_path)
