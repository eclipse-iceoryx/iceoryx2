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

"""Strong type safe extensions for the publish-subscribe messaging pattern."""

import ctypes
from typing import Any, Type, TypeVar, get_args, get_origin

from ._iceoryx2 import *
from .slice import Slice
from .type_name import get_type_name

T = TypeVar("T", bound=ctypes.Structure)


def payload(self: Any) -> Any:
    """Returns a `ctypes.POINTER` to the payload."""
    assert self.__payload_type_details is not None
    if get_origin(self.__payload_type_details) is Slice:
        (contained_type,) = get_args(self.__payload_type_details)
        return Slice(self.payload_ptr, self.__slice_len, contained_type)

    return ctypes.cast(self.payload_ptr, ctypes.POINTER(self.__payload_type_details))


def user_header(self: Any) -> Any:
    """Returns a `ctypes.POINTER` to the user header."""
    assert self.__user_header_type_details is not None
    return ctypes.cast(
        self.user_header_ptr, ctypes.POINTER(self.__user_header_type_details)
    )


def publish_subscribe(
    self: ServiceBuilder, t: Type[T]
) -> ServiceBuilderPublishSubscribe:
    """Returns the `ServiceBuilderPublishSusbcribe` to create a new publish-subscribe service. The payload ctype must be provided as argument."""
    type_name = t.__name__
    type_size = 0
    type_align = 0
    type_variant = TypeVariant.FixedSize

    if get_origin(t) is Slice:
        (contained_type,) = get_args(t)
        type_name = get_type_name(contained_type)
        type_variant = TypeVariant.Dynamic
        type_size = ctypes.sizeof(contained_type)
        type_align = ctypes.alignment(contained_type)
    else:
        type_name = get_type_name(t)
        type_size = ctypes.sizeof(t)
        type_align = ctypes.alignment(t)
        type_variant = TypeVariant.FixedSize

    result = self.__publish_subscribe()
    result.__set_payload_type(t)

    return result.__payload_type_details(
        TypeDetail.new()
        .type_variant(type_variant)
        .type_name(TypeName.new(type_name))
        .size(type_size)
        .alignment(type_align)
    ).__user_header_type_details(
        TypeDetail.new()
        .type_variant(TypeVariant.FixedSize)
        .type_name(TypeName.new("()"))
        .size(0)
        .alignment(1)
    )


def set_user_header(
    self: ServiceBuilderPublishSubscribe, t: Type[T]
) -> ServiceBuilderPublishSubscribe:
    """Sets the user header type for the service."""
    type_name = get_type_name(t)
    result = self.__user_header_type_details(
        TypeDetail.new()
        .type_variant(TypeVariant.FixedSize)
        .type_name(TypeName.new(type_name))
        .size(ctypes.sizeof(t))
        .alignment(ctypes.alignment(t))
    )
    result.__set_user_header_type(t)
    return result


def send_copy(self: Publisher, t: Type[T]) -> Any:
    """Sends a copy of the provided type."""
    assert self.__payload_type_details is not None
    sample_uninit = self.loan_uninit()

    assert ctypes.sizeof(t) == ctypes.sizeof(sample_uninit.__payload_type_details)
    assert ctypes.alignment(t) == ctypes.alignment(sample_uninit.__payload_type_details)

    ctypes.memmove(sample_uninit.payload_ptr, ctypes.byref(t), ctypes.sizeof(t))
    sample = sample_uninit.assume_init()
    return sample.send()


def write_payload(self: SampleMutUninit, t: Type[T]) -> SampleMut:
    """Writes the provided payload into the sample."""
    assert self.__payload_type_details is not None
    assert ctypes.sizeof(t) == ctypes.sizeof(self.__payload_type_details)
    assert ctypes.alignment(t) == ctypes.alignment(self.__payload_type_details)

    ctypes.memmove(self.payload_ptr, ctypes.byref(t), ctypes.sizeof(t))
    return self.assume_init()


def loan_uninit(self: Publisher) -> SampleMutUninit:
    """
    Loans/allocates a `SampleMutUninit` from the underlying data segment of the `Publisher`.

    The user has to initialize the payload before it can be sent. On failure it returns
    `LoanError` describing the failure.
    """
    assert not get_origin(self.__payload_type_details) is Slice

    return self.__loan_uninit()


def loan_slice_uninit(self: Publisher, number_of_elements: int) -> SampleMutUninit:
    """
    Loans/allocates a `SampleMutUninit` from the underlying data segment of the `Publisher`.

    The user has to initialize the payload before it can be sent.
    Fails when it is called for data types which are not a slice.
    On failure it returns `LoanError` describing the failure.
    """
    assert get_origin(self.__payload_type_details) is Slice

    return self.__loan_slice_uninit(number_of_elements)


def initial_max_slice_len(
    self: PortFactoryPublisher, value: int
) -> PortFactoryPublisher:
    """Sets the maximum slice length that a user can allocate."""
    assert get_origin(self.__payload_type_details) is Slice

    return self.__initial_max_slice_len(value)


def allocation_strategy(
    self: PortFactoryPublisher, value: AllocationStrategy
) -> PortFactoryPublisher:
    """Defines the allocation strategy that is used when the memory is exhausted."""
    assert get_origin(self.__payload_type_details) is Slice

    return self.__allocation_strategy(value)


PortFactoryPublisher.initial_max_slice_len = initial_max_slice_len
PortFactoryPublisher.allocation_strategy = allocation_strategy

Publisher.send_copy = send_copy
Publisher.loan_uninit = loan_uninit
Publisher.loan_slice_uninit = loan_slice_uninit

Sample.payload = payload
Sample.user_header = user_header

SampleMut.payload = payload
SampleMut.user_header = user_header

SampleMutUninit.write_payload = write_payload
SampleMutUninit.payload = payload
SampleMutUninit.user_header = user_header

ServiceBuilder.publish_subscribe = publish_subscribe
ServiceBuilderPublishSubscribe.user_header = set_user_header
