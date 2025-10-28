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
from typing import Type, TypeVar

from ._iceoryx2 import *
from .type_name import get_type_name

T = TypeVar("T", bound=ctypes.Structure)


def blackboard_creator(
    self: ServiceBuilder, t: Type[T]
) -> ServiceBuilderBlackboardCreator:
    """Returns the `ServiceBuilderBlackboardCreator` to create a new blackboard service. The key ctype must be provided as argument."""

    type_name = get_type_name(t)
    type_size = ctypes.sizeof(t)
    type_align = ctypes.alignment(t)
    type_variant = TypeVariant.FixedSize

    result = self.__blackboard_creator()
    result.__set_key_type(t)

    return result.__set_key_type_details(
        TypeDetail.new()
        .type_variant(type_variant)
        .type_name(TypeName.new(type_name))
        .size(type_size)
        .alignment(type_align)
    )


def blackboard_opener(
    self: ServiceBuilder, t: Type[T]
) -> ServiceBuilderBlackboardOpener:
    """Returns the `ServiceBuilderBlackboardOpener` to open a blackboard service. The key ctype must be provided as argument."""

    type_name = get_type_name(t)
    type_size = ctypes.sizeof(t)
    type_align = ctypes.alignment(t)
    type_variant = TypeVariant.FixedSize

    result = self.__blackboard_opener()
    result.__set_key_type(t)

    return result.__set_key_type_details(
        TypeDetail.new()
        .type_variant(type_variant)
        .type_name(TypeName.new(type_name))
        .size(type_size)
        .alignment(type_align)
    )


def add(
    self: ServiceBuilderBlackboardCreator,
    key: Type[T],
    value_type: Type[T],
    value: Type[T],
) -> ServiceBuilderBlackboardCreator:
    """Adds a key-value pair to the blackboard."""
    assert self.__key_type_details is not None
    assert ctypes.sizeof(key) == ctypes.sizeof(self.__key_type_details)
    assert ctypes.alignment(key) == ctypes.alignment(self.__key_type_details)

    type_name = get_type_name(value_type)
    type_size = ctypes.sizeof(value_type)
    type_align = ctypes.alignment(value_type)
    type_variant = TypeVariant.FixedSize

    # TODO: can value_type and value argument be combined?
    assert ctypes.sizeof(value) == type_size
    assert ctypes.alignment(value) == type_align

    return self.__add(
        key,
        value,
        TypeDetail.new()
        .type_variant(type_variant)
        .type_name(TypeName.new(type_name))
        .size(type_size)
        .alignment(type_align),
    )


ServiceBuilder.blackboard_creator = blackboard_creator
ServiceBuilder.blackboard_opener = blackboard_opener
ServiceBuilderBlackboardCreator.add = add
