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


class TransmissionData(ctypes.Structure):
    """The strongly typed payload type."""

    _fields_ = [
        ("x", ctypes.c_int),
        ("y", ctypes.c_int),
        ("funky", ctypes.c_double),
    ]

    def __str__(self):
        return f'TransmissionData {{ x: {self.x}, y: {self.y}, funky: {self.funky} }}'

    @staticmethod
    def type_name():
        return "TransmissionData"
