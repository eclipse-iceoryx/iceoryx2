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

"""Strong type safe extensions for the request-response messaging pattern."""

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
    """
    Returns the `ServiceBuilderRequestResponse` to create a new request-response service.

    The request/response payload ctype must be provided as argument.
    """
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
        response_type_name = get_type_name(response)
        response_type_size = ctypes.sizeof(response)
        response_type_align = ctypes.alignment(response)
        response_type_variant = TypeVariant.FixedSize

    result = self.__request_response()
    result.__set_request_payload_type(request)
    result.__set_response_payload_type(response)

    return (
        result.__request_payload_type_details(
            TypeDetail.new()
            .type_variant(request_type_variant)
            .type_name(TypeName.new(request_type_name))
            .size(request_type_size)
            .alignment(request_type_align)
        )
        .__request_header_type_details(
            TypeDetail.new()
            .type_variant(TypeVariant.FixedSize)
            .type_name(TypeName.new("()"))
            .size(0)
            .alignment(1)
        )
        .__response_payload_type_details(
            TypeDetail.new()
            .type_variant(response_type_variant)
            .type_name(TypeName.new(response_type_name))
            .size(response_type_size)
            .alignment(response_type_align)
        )
        .__response_header_type_details(
            TypeDetail.new()
            .type_variant(TypeVariant.FixedSize)
            .type_name(TypeName.new("()"))
            .size(0)
            .alignment(1)
        )
    )


def set_request_header(
    self: ServiceBuilderPublishSubscribe, request: Type[ReqT]
) -> ServiceBuilderPublishSubscribe:
    """Sets the request header type for the service."""
    type_name = get_type_name(request)
    result = self.__request_header_type_details(
        TypeDetail.new()
        .type_variant(TypeVariant.FixedSize)
        .type_name(TypeName.new(type_name))
        .size(ctypes.sizeof(request))
        .alignment(ctypes.alignment(request))
    )
    result.__set_request_header_type(request)
    return result


def set_response_header(
    self: ServiceBuilderPublishSubscribe, response: Type[ResT]
) -> ServiceBuilderPublishSubscribe:
    """Sets the response header type for the service."""
    type_name = get_type_name(response)
    result = self.__response_header_type_details(
        TypeDetail.new()
        .type_variant(TypeVariant.FixedSize)
        .type_name(TypeName.new(type_name))
        .size(ctypes.sizeof(response))
        .alignment(ctypes.alignment(response))
    )
    result.__set_response_header_type(response)
    return result


def request_payload(self: Any) -> Any:
    """Returns a `ctypes.POINTER` to the requests payload."""
    assert self.__request_payload_type_details is not None
    if get_origin(self.__request_payload_type_details) is Slice:
        (contained_type,) = get_args(self.__request_payload_type_details)
        return Slice(self.payload_ptr, self.__slice_len, contained_type)

    return ctypes.cast(
        self.payload_ptr, ctypes.POINTER(self.__request_payload_type_details)
    )


def response_payload(self: Any) -> Any:
    """Returns a `ctypes.POINTER` to the responses payload."""
    assert self.__response_payload_type_details is not None
    if get_origin(self.__response_payload_type_details) is Slice:
        (contained_type,) = get_args(self.__response_payload_type_details)
        return Slice(self.payload_ptr, self.__slice_len, contained_type)

    return ctypes.cast(
        self.payload_ptr, ctypes.POINTER(self.__response_payload_type_details)
    )


def request_header(self: Any) -> Any:
    """Returns a `ctypes.POINTER` to the request header."""
    assert self.__request_header_type_details is not None
    return ctypes.cast(
        self.user_header_ptr, ctypes.POINTER(self.__request_header_type_details)
    )


def response_header(self: Any) -> Any:
    """Returns a `ctypes.POINTER` to the response header."""
    assert self.__response_header_type_details is not None
    return ctypes.cast(
        self.user_header_ptr,
        ctypes.POINTER(self.__response_header_type_details),
    )


def write_request_payload(self: RequestMutUninit, t: Type[ReqT]) -> RequestMut:
    """Writes the provided payload into the request."""
    assert self.__request_payload_type_details is not None
    assert ctypes.sizeof(t) == ctypes.sizeof(self.__request_payload_type_details)
    assert ctypes.alignment(t) == ctypes.alignment(self.__request_payload_type_details)

    ctypes.memmove(self.payload_ptr, ctypes.byref(t), ctypes.sizeof(t))
    return self.assume_init()


def write_response_payload(self: ResponseMutUninit, t: Type[ReqT]) -> ResponseMut:
    """Writes the provided payload into the response."""
    assert self.__response_payload_type_details is not None
    assert ctypes.sizeof(t) == ctypes.sizeof(self.__response_payload_type_details)
    assert ctypes.alignment(t) == ctypes.alignment(self.__response_payload_type_details)

    ctypes.memmove(self.payload_ptr, ctypes.byref(t), ctypes.sizeof(t))
    return self.assume_init()


def loan_uninit_request(self: Client) -> RequestMutUninit:
    """
    Loans/allocates memory from the underlying data segment.

    The user has to initialize the payload before it can be sent. On failure it returns
    `LoanError` describing the failure.
    """
    assert not get_origin(self.__request_payload_type_details) is Slice

    return self.__loan_uninit()


def loan_uninit_response(self: ActiveRequest) -> ResponseMutUninit:
    """
    Loans/allocates memory from the underlying data segment.

    The user has to initialize the payload before it can be sent. On failure it returns
    `LoanError` describing the failure.
    """
    assert not get_origin(self.__response_payload_type_details) is Slice

    return self.__loan_uninit()


def loan_slice_uninit_request(
    self: Client, number_of_elements: int
) -> RequestMutUninit:
    """
    Loans/allocates memory from the underlying data segment.

    The user has to initialize the payload before it can be sent.
    Fails when it is called for data types which are not a slice.
    On failure it returns `LoanError` describing the failure.
    """
    assert get_origin(self.__request_payload_type_details) is Slice

    return self.__loan_slice_uninit(number_of_elements)


def loan_slice_uninit_response(
    self: ActiveRequest, number_of_elements: int
) -> ResponseMutUninit:
    """
    Loans/allocates memory from the underlying data segment.

    The user has to initialize the payload before it can be sent.
    Fails when it is called for data types which are not a slice.
    On failure it returns `LoanError` describing the failure.
    """
    assert get_origin(self.__response_payload_type_details) is Slice

    return self.__loan_slice_uninit(number_of_elements)


def initial_max_slice_len_request(
    self: PortFactoryClient, value: int
) -> PortFactoryClient:
    """Sets the maximum slice length that a user can allocate."""
    assert get_origin(self.__request_payload_type_details) is Slice

    return self.__initial_max_slice_len(value)


def initial_max_slice_len_response(
    self: PortFactoryServer, value: int
) -> PortFactoryServer:
    """Sets the maximum slice length that a user can allocate."""
    assert get_origin(self.__response_payload_type_details) is Slice

    return self.__initial_max_slice_len(value)


def allocation_strategy_request(
    self: PortFactoryClient, value: AllocationStrategy
) -> PortFactoryClient:
    """Defines the allocation strategy that is used when the memory is exhausted."""
    assert get_origin(self.__request_payload_type_details) is Slice

    return self.__allocation_strategy(value)


def allocation_strategy_response(
    self: PortFactoryServer, value: AllocationStrategy
) -> PortFactoryServer:
    """Defines the allocation strategy that is used when the memory is exhausted."""
    assert get_origin(self.__response_payload_type_details) is Slice

    return self.__allocation_strategy(value)


def send_request_copy(self: Client, t: Type[ReqT]) -> PendingResponse:
    """Sends a copy of the provided type."""
    assert self.__request_payload_type_details is not None
    request_uninit = self.__loan_uninit()

    assert ctypes.sizeof(t) == ctypes.sizeof(
        request_uninit.__request_payload_type_details
    )
    assert ctypes.alignment(t) == ctypes.alignment(
        request_uninit.__request_payload_type_details
    )

    ctypes.memmove(request_uninit.payload_ptr, ctypes.byref(t), ctypes.sizeof(t))
    request = request_uninit.assume_init()
    return request.send()


def send_response_copy(self: ActiveRequest, t: Type[ResT]) -> Any:
    """Sends a copy of the provided type."""
    assert self.__response_payload_type_details is not None
    response_uninit = self.__loan_uninit()

    assert ctypes.sizeof(t) == ctypes.sizeof(
        response_uninit.__response_payload_type_details
    )
    assert ctypes.alignment(t) == ctypes.alignment(
        response_uninit.__response_payload_type_details
    )

    ctypes.memmove(response_uninit.payload_ptr, ctypes.byref(t), ctypes.sizeof(t))
    response = response_uninit.assume_init()
    return response.send()


ServiceBuilder.request_response = request_response
ServiceBuilderRequestResponse.request_header = set_request_header
ServiceBuilderRequestResponse.response_header = set_response_header

ActiveRequest.send_copy = send_response_copy
ActiveRequest.payload = request_payload
ActiveRequest.user_header = request_header
ActiveRequest.loan_uninit = loan_uninit_response
ActiveRequest.loan_slice_uninit = loan_slice_uninit_response

PortFactoryClient.initial_max_slice_len = initial_max_slice_len_request
PortFactoryClient.allocation_strategy = allocation_strategy_request
PortFactoryServer.initial_max_slice_len = initial_max_slice_len_response
PortFactoryServer.allocation_strategy = allocation_strategy_response

PendingResponse.payload = request_payload
PendingResponse.user_header = request_header

RequestMut.payload = request_payload
RequestMut.user_header = request_header

RequestMutUninit.payload = request_payload
RequestMutUninit.user_header = request_header
RequestMutUninit.write_payload = write_request_payload

Response.payload = response_payload
Response.user_header = response_header

ResponseMut.payload = response_payload
ResponseMut.user_header = response_header

ResponseMutUninit.payload = response_payload
ResponseMutUninit.user_header = response_header
ResponseMutUninit.write_payload = write_response_payload

Client.loan_uninit = loan_uninit_request
Client.loan_slice_uninit = loan_slice_uninit_request
Client.send_copy = send_request_copy
