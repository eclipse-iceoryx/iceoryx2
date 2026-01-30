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

"""Generates a human readable type name from a given type."""

import ctypes
from typing import Any, Type, TypeVar

T = TypeVar("T")


def get_unsigned_int_type_name(t: Type[T]) -> Any:
    """Generates a human readable type name from a given unsigned integer type."""
    if ctypes.sizeof(t) == 1:
        return "u8"
    if ctypes.sizeof(t) == 2:
        return "u16"
    if ctypes.sizeof(t) == 4:
        return "u32"
    if ctypes.sizeof(t) == 8:
        return "u64"

    return t.__name__


def get_signed_int_type_name(t: Type[T]) -> Any:
    """Generates a human readable type name from a given signed integer type."""
    if ctypes.sizeof(t) == 1:
        return "i8"
    if ctypes.sizeof(t) == 2:
        return "i16"
    if ctypes.sizeof(t) == 4:
        return "i32"
    if ctypes.sizeof(t) == 8:
        return "i64"

    return t.__name__


def get_float_type_name(t: Type[T]) -> Any:
    """Generates a human readable type name from a given float type."""
    if ctypes.sizeof(t) == 4:
        return "f32"
    if ctypes.sizeof(t) == 8:
        return "f64"

    return t.__name__


def get_type_name(t: Type[T]) -> Any:
    """Generates a human readable type name from a given type."""
    if hasattr(t, "type_name"):
        return t.type_name()

    if t.__name__ in ("c_ubyte", "c_ushort", "c_uint", "c_ulong"):
        return get_unsigned_int_type_name(t)
    if t.__name__ in ("c_byte", "c_short", "c_int", "c_long"):
        return get_signed_int_type_name(t)
    if t.__name__ in ("c_float", "c_double"):
        return get_float_type_name(t)
    if t.__name__ == "c_bool":
        return "bool"

    return t.__name__
