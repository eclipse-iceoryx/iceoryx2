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
from typing import Generic, Type, TypeVar

T = TypeVar("T", bound=ctypes.Structure)

class Slice(Generic[T]):
    def __init__(self, data_ptr, number_of_elements) -> None:
        self.data_ptr = data_ptr
        self.number_of_elements = number_of_elements

    def __str__(self) -> str:
        return f"Slice {{ data_ptr: {self.data_ptr}, number_of_elements: {self.number_of_elements} }}"
