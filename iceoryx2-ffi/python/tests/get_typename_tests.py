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


def test_type_names_are_translated_correctly() -> None:
    assert iox2.get_type_name(ctypes.c_ubyte) == "u8"
    assert iox2.get_type_name(ctypes.c_uint8) == "u8"

    assert iox2.get_type_name(ctypes.c_ushort) == "u16"
    assert iox2.get_type_name(ctypes.c_uint16) == "u16"

    assert iox2.get_type_name(ctypes.c_uint) == "u32"
    assert iox2.get_type_name(ctypes.c_uint32) == "u32"

    assert iox2.get_type_name(ctypes.c_ulong) == "u64"
    assert iox2.get_type_name(ctypes.c_uint64) == "u64"

    assert iox2.get_type_name(ctypes.c_byte) == "i8"
    assert iox2.get_type_name(ctypes.c_int8) == "i8"

    assert iox2.get_type_name(ctypes.c_short) == "i16"
    assert iox2.get_type_name(ctypes.c_int16) == "i16"

    assert iox2.get_type_name(ctypes.c_int) == "i32"
    assert iox2.get_type_name(ctypes.c_int32) == "i32"

    assert iox2.get_type_name(ctypes.c_long) == "i64"
    assert iox2.get_type_name(ctypes.c_int64) == "i64"

    assert iox2.get_type_name(ctypes.c_bool) == "bool"
    assert iox2.get_type_name(ctypes.c_float) == "f32"
    assert iox2.get_type_name(ctypes.c_double) == "f64"
