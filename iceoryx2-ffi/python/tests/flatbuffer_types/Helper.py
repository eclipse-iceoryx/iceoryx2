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

from .BoundedData import (BoundedData, BoundedDataAddData, BoundedDataEnd,
                          BoundedDataStart)
from .UnboundedData import (UnboundedData, UnboundedDataAddData,
                            UnboundedDataAddText, UnboundedDataEnd,
                            UnboundedDataStart)

# fmt: on


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


def flatbuffer_tests_schema_path() -> iox2.Path:
    dir_path = Path(__file__).resolve().parent

    return iox2.Path.new(str(dir_path))


def get_schema_file(file_name: str) -> iox2.FilePath:
    dir_path = Path(__file__).resolve().parent
    full_path = os.path.join(dir_path, file_name)

    return iox2.FilePath.new(full_path)
