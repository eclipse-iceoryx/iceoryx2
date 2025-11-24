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

"""Blackboard example key type."""

import ctypes


class BlackboardKey(ctypes.Structure):
    """The strongly typed key type."""

    _fields_ = [("x", ctypes.c_uint32), ("y", ctypes.c_int64), ("z", ctypes.c_uint16)]

    # To store and retrieve a key in the blackboard, a comparison function must be provided.
    def __eq__(self, other):
        """Key eq comparison function."""
        if not isinstance(other, BlackboardKey):
            return NotImplemented
        return (self.x, self.y, self.z) == (other.x, other.y, other.z)

    @staticmethod
    def type_name() -> str:
        """Returns the system-wide unique type name required for communication."""
        return "BlackboardKey"
