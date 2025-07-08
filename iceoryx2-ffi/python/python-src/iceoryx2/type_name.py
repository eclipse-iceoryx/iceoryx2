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


def get_type_name(t: Type[T]) -> Any:
    """Generates a human readable type name from a given type."""
    if hasattr(t, "type_name"):
        return t.type_name()

    if t.__name__ == "c_ubyte" and ctypes.sizeof(t) == 1:
        return "u8"
    if t.__name__ == "c_ushort" and ctypes.sizeof(t) == 2:
        return "u16"
    if t.__name__ == "c_uint" and ctypes.sizeof(t) == 4:
        return "u32"
    if t.__name__ == "c_ulong" and ctypes.sizeof(t) == 8:
        return "u64"
    if t.__name__ == "c_byte" and ctypes.sizeof(t) == 1:
        return "i8"
    if t.__name__ == "c_short" and ctypes.sizeof(t) == 2:
        return "i16"
    if t.__name__ == "c_int" and ctypes.sizeof(t) == 4:
        return "i32"
    if t.__name__ == "c_long" and ctypes.sizeof(t) == 8:
        return "i64"
    if t.__name__ == "c_bool":
        return "bool"
    if t.__name__ == "c_float" and ctypes.sizeof(t) == 4:
        return "f32"
    if t.__name__ == "c_double" and ctypes.sizeof(t) == 8:
        return "f64"

    return t.__name__
