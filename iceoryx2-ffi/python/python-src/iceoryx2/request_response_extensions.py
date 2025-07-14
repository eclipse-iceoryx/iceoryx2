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

# """Strong type safe extensions for the request-response messaging pattern."""

import ctypes
from typing import Any, Type, TypeVar, get_args, get_origin

from ._iceoryx2 import *
from .slice import Slice
from .type_name import get_type_name

ReqT = TypeVar("ReqT", bound=ctypes.Structure)
ResT = TypeVar("ResT", bound=ctypes.Structure)


def request_response(
    self: ServiceBuilder, request: Type[ReqT], response: Type[ResT]
) -> ServiceBuilderPublishSubscribe:
    """Returns the `ServiceBuilderRequestResponse` to create a new request-response service. The request/response payload ctype must be provided as argument."""
    request_type_name = request.__name__
    request_type_size = 0
    request_type_align = 0
    request_type_variant = TypeVariant.FixedSize
    response_type_name = response.__name__
    response_type_size = 0
    response_type_align = 0
    response_type_variant = TypeVariant.FixedSize

    if get_origin(request) is Slice:
        (contained_type,) = get_args(request)
        request_type_name = get_type_name(contained_type)
        request_type_variant = TypeVariant.Dynamic
        request_type_size = ctypes.sizeof(contained_type)
        request_type_align = ctypes.alignment(contained_type)
    else:
        request_type_name = get_type_name(request)
        request_type_size = ctypes.sizeof(request)
        request_type_align = ctypes.alignment(request)
        request_type_variant = TypeVariant.FixedSize

    if get_origin(response) is Slice:
        (contained_type,) = get_args(response)
        response_type_name = get_type_name(contained_type)
        response_type_variant = TypeVariant.Dynamic
        response_type_size = ctypes.sizeof(contained_type)
        response_type_align = ctypes.alignment(contained_type)
    else:
        response_type_name = get_type_name(request)
        response_type_size = ctypes.sizeof(request)
        response_type_align = ctypes.alignment(request)
        response_type_variant = TypeVariant.FixedSize

    result = self.__request_response().__set_request_payload_type(request).__set_response_payload_type(response)

    return result.__request_payload_type_details(
        TypeDetail.new()
        .type_variant(request_type_variant)
        .type_name(TypeName.new(request_type_name))
        .size(request_type_size)
        .alignment(request_type_align)
    ).__request_header_type_details(
        TypeDetail.new()
        .type_variant(TypeVariant.FixedSize)
        .type_name(TypeName.new("()"))
        .size(0)
        .alignment(1)
    ).__response_payload_type_details(
        TypeDetail.new()
        .type_variant(response_type_variant)
        .type_name(TypeName.new(response_type_name))
        .size(response_type_size)
        .alignment(response_type_align)
    ).__response_header_type_details(
        TypeDetail.new()
        .type_variant(TypeVariant.FixedSize)
        .type_name(TypeName.new("()"))
        .size(0)
        .alignment(1)
    )


ServiceBuilder.request_response = request_response
