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

"""Slice - A class representing a set of contiguous elements of type T."""

import ctypes
from typing import Any, Generic, Type, TypeVar

T = TypeVar("T", bound=ctypes.Structure)


class Slice(Generic[T]):
    """
    A class representing a slice of contiguous elements of type T.

    A Slice provides a view into a contiguous sequence of elements without owning the memory.
    It allows for efficient access and iteration over a portion of a contiguous data structure.

    T -  The type of elements in the slice. Can be const-qualified for read-only slices.
    """

    def __init__(self, data_ptr: int, number_of_elements: int, t: Type[T]) -> None:
        """Initializes a slice with a data_ptr, number_of_elements it contains and the type."""
        self.data_ptr = data_ptr
        self.number_of_elements = number_of_elements
        self.contained_type = t

    def __str__(self) -> str:
        """Returns human-readable string of the contents."""
        return f"Slice {{ data_ptr: {self.data_ptr}, number_of_elements: {self.number_of_elements} }}"

    def __getitem__(self, index: int) -> Any:
        """Acquires a pointer T to the element at the specified index."""
        if not 0 <= index < self.number_of_elements:
            raise IndexError("Slice index out of range")

        typed_ptr = ctypes.cast(self.data_ptr, ctypes.POINTER(self.contained_type))
        return typed_ptr[index]

    def __setitem__(self, index: int, value: T) -> None:
        """Sets the value at the specified index."""
        if not 0 <= index < self.number_of_elements:
            raise IndexError("Slice index out of range")

        typed_ptr = ctypes.cast(self.data_ptr, ctypes.POINTER(self.contained_type))
        typed_ptr[index] = value

    def len(self) -> int:
        """Returns the length / number of elements contained in the `Slice`."""
        return self.number_of_elements

    def as_ptr(self) -> int:
        """Returns a pointer to the first element of the `Slice`."""
        return self.data_ptr
