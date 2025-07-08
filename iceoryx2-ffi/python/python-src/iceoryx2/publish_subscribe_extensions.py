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
from typing import Any, Type, TypeVar

from ._iceoryx2 import *

T = TypeVar("T", bound=ctypes.Structure)


def payload(self: Any) -> Any:
    """Returns a `ctypes.POINTER` to the payload."""
    return ctypes.cast(
        self.payload_ptr, ctypes.POINTER(self.__payload_type_details)
    )


def user_header(self: Any) -> Any:
    """Returns a `ctypes.POINTER` to the user header."""
    return ctypes.cast(
        self.user_header_ptr, ctypes.POINTER(self.__user_header_type_details)
    )


def publish_subscribe(
    self: ServiceBuilder, t: Type[T]
) -> ServiceBuilderPublishSubscribe:
    """Returns the `ServiceBuilderPublishSusbcribe` to create a new publish-subscribe service. The payload ctype must be provided as argument."""
    type_name = t.__name__
    if hasattr(t, "type_name"):
        type_name = t.type_name()  # type: ignore[operator]
    result = self.__publish_subscribe()
    result.__set_payload_type(t)
    return result.__payload_type_details(
        TypeDetail.new()
        .type_variant(TypeVariant.FixedSize)
        .type_name(TypeName.new(type_name))
        .size(ctypes.sizeof(t))
        .alignment(ctypes.alignment(t))
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
    type_name = t.__name__
    if hasattr(t, "type_name"):
        type_name = t.type_name()  # type: ignore[operator]
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
    sample_uninit = self.loan_uninit()

    assert ctypes.sizeof(t) == ctypes.sizeof(
        sample_uninit.__payload_type_details
    )
    assert ctypes.alignment(t) == ctypes.alignment(
        sample_uninit.__payload_type_details
    )

    ctypes.memmove(sample_uninit.payload_ptr, ctypes.byref(t), ctypes.sizeof(t))  # type: ignore[arg-type]
    sample = sample_uninit.assume_init()
    return sample.send()


def write_payload(self: SampleMutUninit, t: Type[T]) -> SampleMut:
    """Sends a copy of the provided type."""
    assert ctypes.sizeof(t) == ctypes.sizeof(self.__payload_type_details)
    assert ctypes.alignment(t) == ctypes.alignment(self.__payload_type_details)

    ctypes.memmove(self.payload_ptr, ctypes.byref(t), ctypes.sizeof(t))  # type: ignore[arg-type]
    return self.assume_init()


Publisher.send_copy = send_copy

Sample.payload = payload
Sample.user_header = user_header

SampleMut.payload = payload
SampleMut.user_header = user_header

SampleMutUninit.write_payload = write_payload
SampleMutUninit.payload = payload
SampleMutUninit.user_header = user_header

ServiceBuilder.publish_subscribe = publish_subscribe
ServiceBuilderPublishSubscribe.user_header = set_user_header
