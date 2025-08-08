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

import iceoryx2 as iox2
import pytest


def test_path_can_be_constructed() -> None:
    sut = iox2.Path.new("/path/to/some/file")
    assert sut.to_string() == "/path/to/some/file"


def test_path_with_invalid_content_cannot_be_constructed() -> None:
    invalid_content = "/wildcards/***/are/*/forbidden"
    with pytest.raises(iox2.SemanticStringError):
        iox2.Path.new(invalid_content)
