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

"""Request header carrying a client's listener ID."""

import ctypes
import sys


class RequestHeader(ctypes.Structure):
    """Fixed 16-byte representation shared with the C++ example."""

    _fields_ = [("listener_id", ctypes.c_uint8 * 16)]

    @staticmethod
    def type_name() -> str:
        """Return the type name shared with the C++ example."""
        return "RequestResponseWithEventsHeader"

    def set_listener_id(self, value: int) -> None:
        """Store the native-endian bytes used by the C++ ID API."""
        self.listener_id[:] = value.to_bytes(16, sys.byteorder)

    def listener_id_value(self) -> int:
        """Return the integer ID used by the Python API."""
        return int.from_bytes(bytes(self.listener_id), sys.byteorder)
