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

import ctypes

import iceoryx2 as iox2
import pytest


def test_slice_contains_correct_number_of_elements() -> None:
    data_ptr = 1234
    number_of_elements = 78
    sut = iox2.Slice[ctypes.c_uint8](data_ptr, number_of_elements, ctypes.c_uint8)

    assert sut.len() == number_of_elements


def test_slice_returns_correct_data_ptr() -> None:
    data_ptr = 123
    number_of_elements = 7809
    sut = iox2.Slice[ctypes.c_uint8](data_ptr, number_of_elements, ctypes.c_uint8)

    assert sut.as_ptr() == data_ptr


def test_get_set_item_raises_exception_when_out_of_bounds() -> None:
    data_ptr = 23
    number_of_elements = 5

    sut = iox2.Slice[ctypes.c_uint8](data_ptr, number_of_elements, ctypes.c_uint8)

    with pytest.raises(IndexError):
        _unused = sut[number_of_elements + 1]

    with pytest.raises(IndexError):
        sut[number_of_elements + 1] = ctypes.c_uint8(7)

    with pytest.raises(IndexError):
        _unused = sut[-1]

    with pytest.raises(IndexError):
        sut[-1] = ctypes.c_uint8(7)
