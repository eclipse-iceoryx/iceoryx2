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
from typing import Any, Type, TypeVar, get_args, get_origin, overload

import flatbuffers

from ._iceoryx2 import *
from .flatbuffer import Flatbuffer
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
    request_type_name = ""
    request_type_size = 0
    request_type_align = 0
    request_type_variant = TypeVariant.FixedSize
    response_type_name = ""
    response_type_size = 0
    response_type_align = 0
    response_type_variant = TypeVariant.FixedSize

    if get_origin(request) is Flatbuffer:
        (contained_type,) = get_args(request)
        request_type_name = "iox2::Flatbuffer"
        request_type_variant = TypeVariant.FixedSize
        request_type_size = 1
        request_type_align = 1
    elif get_origin(request) is Slice:
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

    if get_origin(response) is Flatbuffer:
        (contained_type,) = get_args(response)
        response_type_name = "iox2::Flatbuffer"
        response_type_variant = TypeVariant.FixedSize
        response_type_size = 1
        response_type_align = 1
    elif get_origin(response) is Slice:
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

    if get_origin(request) is Flatbuffer:
        (PayloadType,) = get_args(request)
        result = result.__internal_request_type_definition_name_hint(
            get_type_name(PayloadType), ""
        )

    if get_origin(response) is Flatbuffer:
        (PayloadType,) = get_args(response)
        result = result.__internal_response_type_definition_name_hint(
            get_type_name(PayloadType), ""
        )

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
        return Slice(self.payload_ptr, self.__slice_len, contained_type, owner=self)

    ptr = ctypes.cast(
        self.payload_ptr, ctypes.POINTER(self.__request_payload_type_details)
    )
    ptr._iox2_owner = self
    return ptr


def response_payload(self: Any) -> Any:
    """Returns a `ctypes.POINTER` to the responses payload."""
    assert self.__response_payload_type_details is not None
    if get_origin(self.__response_payload_type_details) is Slice:
        (contained_type,) = get_args(self.__response_payload_type_details)
        return Slice(self.payload_ptr, self.__slice_len, contained_type, owner=self)

    ptr = ctypes.cast(
        self.payload_ptr, ctypes.POINTER(self.__response_payload_type_details)
    )
    ptr._iox2_owner = self
    return ptr


def request_header(self: Any) -> Any:
    """Returns a `ctypes.POINTER` to the request header."""
    assert self.__request_header_type_details is not None
    ptr = ctypes.cast(
        self.user_header_ptr, ctypes.POINTER(self.__request_header_type_details)
    )
    ptr._iox2_owner = self
    return ptr


def response_header(self: Any) -> Any:
    """Returns a `ctypes.POINTER` to the response header."""
    assert self.__response_header_type_details is not None
    ptr = ctypes.cast(
        self.user_header_ptr,
        ctypes.POINTER(self.__response_header_type_details),
    )
    ptr._iox2_owner = self
    return ptr


def request_payload_bytes(self: Any) -> Slice[ctypes.c_uint8]:
    """Returns the serialized flatbuffer data as bytes."""
    assert self.__request_payload_type_details is not None
    assert get_origin(self.__request_payload_type_details) is Flatbuffer

    number_of_elements = self.header.number_of_elements
    offset = self.header.payload_offset
    return Slice(
        self.payload_ptr + offset,
        number_of_elements - offset,
        ctypes.c_uint8,
        owner=self,
    )


def response_payload_bytes(self: Any) -> Slice[ctypes.c_uint8]:
    """Returns the serialized flatbuffer data as bytes."""
    assert self.__response_payload_type_details is not None
    assert get_origin(self.__response_payload_type_details) is Flatbuffer

    number_of_elements = self.header.number_of_elements
    offset = self.header.payload_offset
    return Slice(
        self.payload_ptr + offset,
        number_of_elements - offset,
        ctypes.c_uint8,
        owner=self,
    )


def request_payload_root(self: Any) -> Any:
    """Returns the root of the flatbuffer."""
    assert self.__request_payload_type_details is not None
    assert get_origin(self.__request_payload_type_details) is Flatbuffer

    (PayloadType,) = get_args(self.__request_payload_type_details)

    payload_bytes = self.payload_bytes()
    view = payload_bytes.as_memory_view()
    n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, view, 0)

    data = PayloadType()
    data.Init(view, n)
    return data


def response_payload_root(self: Any) -> Any:
    """Returns the root of the flatbuffer."""
    assert self.__response_payload_type_details is not None
    assert get_origin(self.__response_payload_type_details) is Flatbuffer

    (PayloadType,) = get_args(self.__response_payload_type_details)

    payload_bytes = self.payload_bytes()
    view = payload_bytes.as_memory_view()
    n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, view, 0)

    data = PayloadType()
    data.Init(view, n)
    return data


def write_request_payload(self: RequestMutUninit, t: Type[ReqT]) -> RequestMut:
    """Writes the provided payload into the request."""
    assert (
        self.__request_payload_type_details is not None
        and get_origin(self.__request_payload_type_details) != Flatbuffer
    )
    assert ctypes.sizeof(t) == ctypes.sizeof(self.__request_payload_type_details)
    assert ctypes.alignment(t) == ctypes.alignment(self.__request_payload_type_details)

    ctypes.memmove(self.payload_ptr, ctypes.byref(t), ctypes.sizeof(t))
    return self.__assume_init()


def write_response_payload(self: ResponseMutUninit, t: Type[ReqT]) -> ResponseMut:
    """Writes the provided payload into the response."""
    assert (
        self.__response_payload_type_details is not None
        and get_origin(self.__response_payload_type_details) != Flatbuffer
    )
    assert ctypes.sizeof(t) == ctypes.sizeof(self.__response_payload_type_details)
    assert ctypes.alignment(t) == ctypes.alignment(self.__response_payload_type_details)

    ctypes.memmove(self.payload_ptr, ctypes.byref(t), ctypes.sizeof(t))
    return self.__assume_init()


def loan_uninit_request(self: Client) -> RequestMutUninit:
    """
    Loans/allocates memory from the underlying data segment.

    The user has to initialize the payload before it can be sent. On failure it returns
    `LoanError` describing the failure.
    """
    origin = get_origin(self.__request_payload_type_details)
    assert origin not in (Slice, Flatbuffer)

    return self.__loan_uninit()


def loan_uninit_response(self: ActiveRequest) -> ResponseMutUninit:
    """
    Loans/allocates memory from the underlying data segment.

    The user has to initialize the payload before it can be sent. On failure it returns
    `LoanError` describing the failure.
    """
    origin = get_origin(self.__response_payload_type_details)
    assert origin not in (Slice, Flatbuffer)

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


def loan_flatbuffer_request(self: Client) -> RequestMutUninit:
    """Loans/allocates a `RequestMutUninit` from the underlying data segment of the `Client` with an integrated `FlatbufferBuilder`."""
    assert get_origin(self.__request_payload_type_details) is Flatbuffer

    # Loaning a slice of 1 byte is exactly what we need here. The flatbuffer builder is
    # in python not zero-copy and completely resides on the heap since they do not have
    # an API to provide a custom allocator.
    #
    # When the data production is finished the request payload is grown/resized and the
    # serialized flatbuffer content is copied into the request.
    return self.__loan_slice_uninit(1)


def loan_flatbuffer_response(self: Client) -> ResponseMutUninit:
    """Loans/allocates a `ResponseMutUninit` from the underlying data segment of the `Client` with an integrated `FlatbufferBuilder`."""
    assert get_origin(self.__response_payload_type_details) is Flatbuffer

    # Loaning a slice of 1 byte is exactly what we need here. The flatbuffer builder is
    # in python not zero-copy and completely resides on the heap since they do not have
    # an API to provide a custom allocator.
    #
    # When the data production is finished the response payload is grown/resized and the
    # serialized flatbuffer content is copied into the response.
    return self.__loan_slice_uninit(1)


_request_mut_uninit_dict: dict[int, flatbuffers.Builder] = {}


def flatbuffer_builder_request(self: RequestMutUninit) -> flatbuffers.Builder:
    """Returns the flatbuffers.Builder to produce the data that shall be sent."""
    key = id(self)
    builder = _request_mut_uninit_dict.get(key)
    if builder is None:
        builder = flatbuffers.Builder(self.__available_payload_memory)
        _request_mut_uninit_dict[key] = builder
    return builder


_response_mut_uninit_dict: dict[int, flatbuffers.Builder] = {}


def flatbuffer_builder_response(self: ResponseMutUninit) -> flatbuffers.Builder:
    """Returns the flatbuffers.Builder to produce the data that shall be sent."""
    key = id(self)
    builder = _response_mut_uninit_dict.get(key)
    if builder is None:
        builder = flatbuffers.Builder(self.__available_payload_memory)
        _response_mut_uninit_dict[key] = builder
    return builder


@overload
def assume_init_request(self: RequestMutUninit) -> RequestMut: ...  # noqa: E704
@overload
def assume_init_request(  # noqa: E704
    self: RequestMutUninit, root: int
) -> RequestMut: ...


def assume_init_request(self: RequestMutUninit, root=None) -> RequestMut:
    """
    Extracts the value of the uninitialized payload and labels the `RequestMutUninit` as initialized `RequestMut`.

    After this call the `RequestMutUninit` is no longer usable!
    """
    origin = get_origin(self.__request_payload_type_details)
    assert (origin is Flatbuffer and root is not None) or (
        origin is not Flatbuffer and root is None
    )

    if root is None:
        return self.__assume_init()

    builder = self.flatbuffer_builder()
    builder.Finish(root)

    payload_offset = builder.Head()
    buffer_len = len(builder.Bytes)
    base_view = (ctypes.c_ubyte * buffer_len).from_buffer(builder.Bytes)
    buffer_ptr = ctypes.addressof(base_view)

    initialized_request = self.__assume_init_flatbuffer(
        buffer_ptr, buffer_len, payload_offset
    )
    _request_mut_uninit_dict.pop(id(self), None)
    return initialized_request


@overload
def assume_init_response(self: ResponseMutUninit) -> ResponseMut: ...  # noqa: E704
@overload
def assume_init_response(  # noqa: E704
    self: ResponseMutUninit, root: int
) -> ResponseMut: ...


def assume_init_response(self: ResponseMutUninit, root=None) -> ResponseMut:
    """
    Extracts the value of the uninitialized payload and labels the `ResponseMutUninit` as initialized `ResponseMut`.

    After this call the `ResponseMutUninit` is no longer usable!
    """
    origin = get_origin(self.__response_payload_type_details)
    assert (origin is Flatbuffer and root is not None) or (
        origin is not Flatbuffer and root is None
    )

    if root is None:
        return self.__assume_init()

    builder = self.flatbuffer_builder()
    builder.Finish(root)

    payload_offset = builder.Head()
    buffer_len = len(builder.Bytes)
    base_view = (ctypes.c_ubyte * buffer_len).from_buffer(builder.Bytes)
    buffer_ptr = ctypes.addressof(base_view)

    initialized_response = self.__assume_init_flatbuffer(
        buffer_ptr, buffer_len, payload_offset
    )
    _response_mut_uninit_dict.pop(id(self), None)
    return initialized_response


_request_mut_uninit_del_original = getattr(RequestMutUninit, "__del__", None)


def _request_mut_uninit_del(self: RequestMutUninit) -> None:
    _request_mut_uninit_dict.pop(id(self), None)
    if _request_mut_uninit_del_original is not None:
        _request_mut_uninit_del_original(self)


_response_mut_uninit_del_original = getattr(ResponseMutUninit, "__del__", None)


def _response_mut_uninit_del(self: ResponseMutUninit) -> None:
    _response_mut_uninit_dict.pop(id(self), None)
    if _response_mut_uninit_del_original is not None:
        _response_mut_uninit_del_original(self)


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
    assert (
        get_origin(self.__request_payload_type_details) is Slice
        or get_origin(self.__request_payload_type_details) is Flatbuffer
    )

    return self.__allocation_strategy(value)


def allocation_strategy_response(
    self: PortFactoryServer, value: AllocationStrategy
) -> PortFactoryServer:
    """Defines the allocation strategy that is used when the memory is exhausted."""
    assert (
        get_origin(self.__response_payload_type_details) is Slice
        or get_origin(self.__response_payload_type_details) is Flatbuffer
    )

    return self.__allocation_strategy(value)


def send_request_copy(self: Client, t: Type[ReqT]) -> PendingResponse:
    """Sends a copy of the provided type."""
    assert (
        self.__request_payload_type_details is not None
        and get_origin(self.__request_payload_type_details) != Flatbuffer
    )
    request_uninit = self.__loan_uninit()

    assert ctypes.sizeof(t) == ctypes.sizeof(
        request_uninit.__request_payload_type_details
    )
    assert ctypes.alignment(t) == ctypes.alignment(
        request_uninit.__request_payload_type_details
    )

    ctypes.memmove(request_uninit.payload_ptr, ctypes.byref(t), ctypes.sizeof(t))
    request = request_uninit.__assume_init()
    return request.send()


def send_response_copy(self: ActiveRequest, t: Type[ResT]) -> Any:
    """Sends a copy of the provided type."""
    assert (
        self.__response_payload_type_details is not None
        and get_origin(self.__response_payload_type_details) != Flatbuffer
    )
    response_uninit = self.__loan_uninit()

    assert ctypes.sizeof(t) == ctypes.sizeof(
        response_uninit.__response_payload_type_details
    )
    assert ctypes.alignment(t) == ctypes.alignment(
        response_uninit.__response_payload_type_details
    )

    ctypes.memmove(response_uninit.payload_ptr, ctypes.byref(t), ctypes.sizeof(t))
    response = response_uninit.__assume_init()
    return response.send()


def request_flatbuffer_schema_path(
    self: ServiceBuilderRequestResponse, value: FilePath
) -> ServiceBuilderRequestResponse:
    """
    Sets the path to the flatbuffer schema file.

    If this is not explicitly defined, iceoryx2 will try to find the best fitting schema file
    in the configured filebuffer schema paths defined in the config.
    """
    assert get_origin(self.__get_request_payload_type_details) is Flatbuffer

    return self.__internal_request_flatbuffer_schema_path(value)


def response_flatbuffer_schema_path(
    self: ServiceBuilderRequestResponse, value: FilePath
) -> ServiceBuilderRequestResponse:
    """
    Sets the path to the flatbuffer schema file.

    If this is not explicitly defined, iceoryx2 will try to find the best fitting schema file
    in the configured filebuffer schema paths defined in the config.
    """
    assert get_origin(self.__get_response_payload_type_details) is Flatbuffer

    return self.__internal_response_flatbuffer_schema_path(value)


def initial_reserved_memory_request(
    self: PortFactoryClient, value: int
) -> PortFactoryClient:
    """Sets the maximum initial reserved memory that the underlying allocator reserves for the flatbuffer builder."""
    assert get_origin(self.__request_payload_type_details) is Flatbuffer

    return self.__initial_max_slice_len(value)


def initial_reserved_memory_response(
    self: PortFactoryServer, value: int
) -> PortFactoryServer:
    """Sets the maximum initial reserved memory that the underlying allocator reserves for the flatbuffer builder."""
    assert get_origin(self.__response_payload_type_details) is Flatbuffer

    return self.__initial_max_slice_len(value)


ServiceBuilder.request_response = request_response
ServiceBuilderRequestResponse.request_header = set_request_header
ServiceBuilderRequestResponse.response_header = set_response_header
ServiceBuilderRequestResponse.request_flatbuffer_schema_path = (
    request_flatbuffer_schema_path
)
ServiceBuilderRequestResponse.response_flatbuffer_schema_path = (
    response_flatbuffer_schema_path
)

ActiveRequest.send_copy = send_response_copy
ActiveRequest.payload = request_payload
ActiveRequest.user_header = request_header
ActiveRequest.payload_bytes = request_payload_bytes
ActiveRequest.payload_root = request_payload_root
ActiveRequest.loan_uninit = loan_uninit_response
ActiveRequest.loan_flatbuffer = loan_flatbuffer_response
ActiveRequest.loan_slice_uninit = loan_slice_uninit_response

PortFactoryClient.initial_max_slice_len = initial_max_slice_len_request
PortFactoryClient.allocation_strategy = allocation_strategy_request
PortFactoryClient.initial_reserved_memory = initial_reserved_memory_request
PortFactoryServer.initial_max_slice_len = initial_max_slice_len_response
PortFactoryServer.allocation_strategy = allocation_strategy_response
PortFactoryServer.initial_reserved_memory = initial_reserved_memory_response

PendingResponse.payload = request_payload
PendingResponse.user_header = request_header
PendingResponse.payload_bytes = request_payload_bytes
PendingResponse.payload_root = request_payload_root

RequestMut.payload = request_payload
RequestMut.user_header = request_header
RequestMut.payload_bytes = request_payload_bytes
RequestMut.payload_root = request_payload_root

RequestMutUninit.payload = request_payload
RequestMutUninit.user_header = request_header
RequestMutUninit.flatbuffer_builder = flatbuffer_builder_request
RequestMutUninit.assume_init = assume_init_request
RequestMutUninit.write_payload = write_request_payload
RequestMutUninit.__del__ = _request_mut_uninit_del

Response.payload = response_payload
Response.user_header = response_header
Response.payload_bytes = response_payload_bytes
Response.payload_root = response_payload_root

ResponseMut.payload = response_payload
ResponseMut.user_header = response_header
ResponseMut.payload_bytes = response_payload_bytes
ResponseMut.payload_root = response_payload_root

ResponseMutUninit.payload = response_payload
ResponseMutUninit.user_header = response_header
ResponseMutUninit.flatbuffer_builder = flatbuffer_builder_response
ResponseMutUninit.assume_init = assume_init_response
ResponseMutUninit.write_payload = write_response_payload
ResponseMutUninit.__del__ = _response_mut_uninit_del

Client.loan_uninit = loan_uninit_request
Client.loan_slice_uninit = loan_slice_uninit_request
Client.loan_flatbuffer = loan_flatbuffer_request
Client.send_copy = send_request_copy
