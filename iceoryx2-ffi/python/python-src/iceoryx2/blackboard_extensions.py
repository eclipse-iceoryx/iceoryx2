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

"""Strong type safe extensions for the blackboard messaging pattern."""

import ctypes
from typing import Any, Callable, Type, TypeVar

from ._iceoryx2 import *
from .type_name import get_type_name

K = TypeVar("K", bound=ctypes.Structure)
V = TypeVar("V", bound=ctypes.Structure)


def get_key_cmp_func(
    key_type_details: Any,
) -> Callable[[ctypes.c_uint64, ctypes.c_uint64], bool]:
    """Returns a callable for comparing keys."""

    def key_cmp(lhs: ctypes.c_uint64, rhs: ctypes.c_uint64) -> bool:
        lhs_key = ctypes.cast(lhs, ctypes.POINTER(key_type_details)).contents
        rhs_key = ctypes.cast(rhs, ctypes.POINTER(key_type_details)).contents
        if isinstance(lhs_key, ctypes.Structure):
            return lhs_key == rhs_key
        return lhs_key.value == rhs_key.value

    return key_cmp


def blackboard_creator(
    self: ServiceBuilder, key: Type[K]
) -> ServiceBuilderBlackboardCreator:
    """
    Returns the `ServiceBuilderBlackboardCreator` to create a new blackboard service.

    The key ctype must be provided as argument. If the key is of type ctypes.Structure,
    it must implement __eq__.
    """
    type_name = get_type_name(key)
    type_size = ctypes.sizeof(key)
    type_align = ctypes.alignment(key)
    type_variant = TypeVariant.FixedSize

    result = self.__blackboard_creator()
    result.__set_key_type(key)

    result = result.__set_key_type_details(
        TypeDetail.new()
        .type_variant(type_variant)
        .type_name(TypeName.new(type_name))
        .size(type_size)
        .alignment(type_align)
    )

    return result.__set_key_eq_cmp_func(get_key_cmp_func(result.__key_type_details))


def blackboard_opener(
    self: ServiceBuilder, key: Type[K]
) -> ServiceBuilderBlackboardOpener:
    """
    Returns the `ServiceBuilderBlackboardOpener` to open a blackboard service.

    The key ctype must be provided as argument.
    """
    type_name = get_type_name(key)
    type_size = ctypes.sizeof(key)
    type_align = ctypes.alignment(key)
    type_variant = TypeVariant.FixedSize

    result = self.__blackboard_opener()
    result.__set_key_type(key)

    return result.__set_key_type_details(
        TypeDetail.new()
        .type_variant(type_variant)
        .type_name(TypeName.new(type_name))
        .size(type_size)
        .alignment(type_align)
    )


def add(
    self: ServiceBuilderBlackboardCreator,
    key: Type[K],
    value: Type[V],
) -> ServiceBuilderBlackboardCreator:
    """Adds a key-value pair to the blackboard."""
    assert self.__key_type_details is not None
    assert ctypes.sizeof(key) == ctypes.sizeof(self.__key_type_details)
    assert ctypes.alignment(key) == ctypes.alignment(self.__key_type_details)

    type_name = get_type_name(type(value))
    type_size = ctypes.sizeof(value)
    type_align = ctypes.alignment(value)
    type_variant = TypeVariant.FixedSize

    return self.__add(
        ctypes.addressof(key),
        ctypes.addressof(value),
        TypeDetail.new()
        .type_variant(type_variant)
        .type_name(TypeName.new(type_name))
        .size(type_size)
        .alignment(type_align),
    )


def entry_handle(self: Reader, key: Type[K], value: Type[V]) -> EntryHandle:
    """
    Creates an EntryHandle for direct read access to the value.

    On failure it returns `EntryHandleError` describing the failure.
    """
    assert self.__key_type_details is not None
    assert ctypes.sizeof(key) == ctypes.sizeof(self.__key_type_details)
    assert ctypes.alignment(key) == ctypes.alignment(self.__key_type_details)

    type_name = get_type_name(value)
    type_size = ctypes.sizeof(value)
    type_align = ctypes.alignment(value)
    type_variant = TypeVariant.FixedSize

    entry_handle = self.__entry(
        ctypes.addressof(key),
        TypeDetail.new()
        .type_variant(type_variant)
        .type_name(TypeName.new(type_name))
        .size(type_size)
        .alignment(type_align),
    )
    entry_handle.__set_value_type(value)
    entry_handle.__set_value_ptr()
    return entry_handle


class RawValue:
    """A wrapper class for the value returned by `EntryHandle.get()`."""

    def __init__(self, data: bytes):
        """Initializes `RawValue` from bytes."""
        self.data = data
        self.size = len(data)

    def decode_as(self, ct_type):
        """Interpret the raw bytes as a ctypes type."""
        return ct_type.from_buffer_copy(self.data)


def get(self: EntryHandle) -> RawValue:
    """
    Returns a copy of the value as bytes.

    Use decode_as() to reinterpret the raw bytes as a ctypes type.
    """
    value_ptr = self.__get()
    value_size = ctypes.sizeof(self.__value_type)
    raw_bytes = ctypes.string_at(value_ptr, value_size)
    return RawValue(raw_bytes)


def entry_handle_mut(self: Writer, key: Type[K], value: Type[V]) -> EntryHandleMut:
    """
    Creates an EntryHandleMut for direct write access to the value.

    There can be only one EntryHandleMut per value. On failure it returns
    `EntryHandleMutError` describing the failure.
    """
    assert self.__key_type_details is not None
    assert ctypes.sizeof(key) == ctypes.sizeof(self.__key_type_details)
    assert ctypes.alignment(key) == ctypes.alignment(self.__key_type_details)

    type_name = get_type_name(value)
    type_size = ctypes.sizeof(value)
    type_align = ctypes.alignment(value)
    type_variant = TypeVariant.FixedSize

    entry_handle_mut = self.__entry(
        ctypes.addressof(key),
        TypeDetail.new()
        .type_variant(type_variant)
        .type_name(TypeName.new(type_name))
        .size(type_size)
        .alignment(type_align),
    )
    entry_handle_mut.__set_value_type(value)
    return entry_handle_mut


def update_with_copy(self: EntryHandleMut, value: Type[V]):
    """Updates the value by copying the passed value into it."""
    assert self.__value_type is not None
    type_size = ctypes.sizeof(value)
    type_align = ctypes.alignment(value)
    assert type_size == ctypes.sizeof(self.__value_type)
    assert type_align == ctypes.alignment(self.__value_type)

    data_cell_ptr = self.__get_data_ptr(type_size, type_align)
    ctypes.memmove(data_cell_ptr, ctypes.byref(value), type_size)
    self.__update_data_ptr()


def write(self: EntryValueUninit, value: Type[V]) -> EntryValue:
    """
    Writes to the entry value.

    Consumes the EntryValueUninit, writes values to the entry
    value and returns the initialized EntryValue.
    """
    assert self.__value_type is not None
    type_size = ctypes.sizeof(value)
    assert type_size == ctypes.sizeof(self.__value_type)
    assert ctypes.alignment(value) == ctypes.alignment(self.__value_type)

    write_cell = self.__get_write_cell()
    ctypes.memmove(write_cell, ctypes.byref(value), type_size)
    return self.__assume_init()


EntryHandle.get = get

EntryHandleMut.update_with_copy = update_with_copy

EntryValueUninit.write = write

Reader.entry = entry_handle

ServiceBuilder.blackboard_creator = blackboard_creator
ServiceBuilder.blackboard_opener = blackboard_opener
ServiceBuilderBlackboardCreator.add = add

Writer.entry = entry_handle_mut
