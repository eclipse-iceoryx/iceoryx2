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

import iceoryx2_ffi_python as iceoryx2
import pytest


def test_node_name_can_be_constructed() -> None:
    sut = iceoryx2.NodeName.new("hello world")
    assert sut.as_str() == "hello world"

def test_node_name_construction_fails_when_max_len_is_exceeded() -> None:
    sut_value = "x" * (iceoryx2.NodeName.max_len() + 1)
    with pytest.raises(iceoryx2.SemanticStringError):
        sut = iceoryx2.NodeName.new(sut_value)
