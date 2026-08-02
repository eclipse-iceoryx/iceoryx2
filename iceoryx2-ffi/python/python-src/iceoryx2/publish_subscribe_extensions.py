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
from typing import Any, Type, TypeVar, get_args, get_origin, overload

import flatbuffers

from ._iceoryx2 import *
from .flatbuffer import Flatbuffer
from .slice import Slice
from .type_name import get_type_name

T = TypeVar("T", bound=ctypes.Structure)


def payload_bytes(self: Any) -> Slice[ctypes.c_int8]:
    assert self.__payload_type_details is not None
    assert get_origin(self.__payload_type_details) is Flatbuffer

    number_of_elements = self.header.number_of_elements
    offset = self.header.payload_offset
    return Slice(
        self.payload_ptr + offset,
        number_of_elements - offset,
        ctypes.c_uint8,
        owner=self,
    )


def payload_root(self: Any) -> Any:
    assert self.__payload_type_details is not None
    assert get_origin(self.__payload_type_details) is Flatbuffer

    (PayloadType,) = get_args(self.__payload_type_details)

    bytes = self.payload_bytes()
    view = bytes.as_memory_view()
    n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, view, 0)

    data = PayloadType()
    data.Init(view, n)
    return data


def payload(self: Any) -> Any:
    """Returns a `ctypes.POINTER` to the payload."""
    assert self.__payload_type_details is not None
    if get_origin(self.__payload_type_details) is Slice:
        (contained_type,) = get_args(self.__payload_type_details)
        return Slice(self.payload_ptr, self.__slice_len, contained_type, owner=self)

    ptr = ctypes.cast(self.payload_ptr, ctypes.POINTER(self.__payload_type_details))
    ptr._iox2_owner = self  # type: ignore[attr-defined]
    return ptr


def user_header(self: Any) -> Any:
    """Returns a `ctypes.POINTER` to the user header."""
    assert self.__user_header_type_details is not None
    ptr = ctypes.cast(
        self.user_header_ptr, ctypes.POINTER(self.__user_header_type_details)
    )
    ptr._iox2_owner = self  # type: ignore[attr-defined]
    return ptr


def publish_subscribe(
    self: ServiceBuilder, t: Type[T]
) -> ServiceBuilderPublishSubscribe:
    """Returns the `ServiceBuilderPublishSubscribe` to create a new publish-subscribe service. The payload ctype must be provided as argument."""
    type_name = ""
    type_size = 0
    type_align = 0
    type_variant = TypeVariant.FixedSize

    if get_origin(t) is Flatbuffer:
        (contained_type,) = get_args(t)
        type_name = "iox2::Flatbuffer"
        type_variant = TypeVariant.FixedSize
        type_size = 1
        type_align = 1
    elif get_origin(t) is Slice:
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
    assert (
        self.__payload_type_details is not None
        and get_origin(self.__payload_type_details) != Flatbuffer
    )
    sample_uninit = self.loan_uninit()

    assert ctypes.sizeof(t) == ctypes.sizeof(sample_uninit.__payload_type_details)
    assert ctypes.alignment(t) == ctypes.alignment(sample_uninit.__payload_type_details)

    ctypes.memmove(sample_uninit.payload_ptr, ctypes.byref(t), ctypes.sizeof(t))
    sample = sample_uninit.__assume_init()
    return sample.send()


def write_payload(self: SampleMutUninit, t: Type[T]) -> SampleMut:
    """Writes the provided payload into the sample."""
    assert (
        self.__payload_type_details is not None
        and get_origin(self.__payload_type_details) != Flatbuffer
    )
    assert ctypes.sizeof(t) == ctypes.sizeof(self.__payload_type_details)
    assert ctypes.alignment(t) == ctypes.alignment(self.__payload_type_details)

    ctypes.memmove(self.payload_ptr, ctypes.byref(t), ctypes.sizeof(t))
    return self.__assume_init()


def loan_uninit(self: Publisher) -> SampleMutUninit:
    """
    Loans/allocates a `SampleMutUninit` from the underlying data segment of the `Publisher`.

    The user has to initialize the payload before it can be sent. On failure it returns
    `LoanError` describing the failure.
    """
    origin = get_origin(self.__payload_type_details)
    assert origin != Slice and origin != Flatbuffer

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
    assert (
        get_origin(self.__payload_type_details) is Slice
        or get_origin(self.__payload_type_details) is Flatbuffer
    )

    return self.__allocation_strategy(value)


def flatbuffer_schema_path(
    self: ServiceBuilderPublishSubscribe, value: FilePath
) -> PortFactoryPublisher:
    """Sets the path to the flatbuffer schema file. If this is not explicitly defined, iceoryx2
    will try to find the best fitting schema file in the configured filebuffer schema paths
    defined in the config."""
    assert get_origin(self.__get_payload_type_details) is Flatbuffer

    return self.__flatbuffer_schema_path(value)


def initial_reserved_memory(
    self: PortFactoryPublisher, value: int
) -> PortFactoryPublisher:
    """Sets the maximum initial reserved memory that the underlying allocator reserves
    for the flatbuffer builder."""
    assert get_origin(self.__payload_type_details) is Flatbuffer

    return self.__initial_max_slice_len(value)


def loan_slice_uninit(self: Publisher, number_of_elements: int) -> SampleMutUninit:
    """
    Loans/allocates a `SampleMutUninit` from the underlying data segment of the `Publisher`.

    The user has to initialize the payload before it can be sent.
    Fails when it is called for data types which are not a slice.
    On failure it returns `LoanError` describing the failure.
    """
    assert get_origin(self.__payload_type_details) is Slice

    return self.__loan_slice_uninit(number_of_elements)


def loan_flatbuffer(self: Publisher) -> SampleMutUninit:
    """
    Loans/allocates a `SampleMutUninit` from the underlying data segment of the `Publisher`
    with an integrated `FlatbufferBuilder`.
    """
    assert get_origin(self.__payload_type_details) is Flatbuffer

    # Loaning a slice of 1 byte is exactly what we need here. The flatbuffer builder is
    # in python not zero-copy and completely resides on the heap since they do not have
    # an API to provide a custom allocator.
    #
    # When the data production is finished the sample payload is grown/resized and the
    # serialized flatbuffer content is copied into the sample.
    return self.__loan_slice_uninit(1)


_sample_mut_uninit_dict: dict[int, flatbuffers.Builder] = {}


def flatbuffer_builder(self: SampleMutUninit) -> flatbuffers.Builder:
    """
    Returns the flatbuffers.Builder to produce the data that shall be sent.
    """
    key = id(self)
    builder = _sample_mut_uninit_dict.get(key)
    if builder is None:
        builder = flatbuffers.Builder(1024)
        _sample_mut_uninit_dict[key] = builder
    return builder


@overload
def assume_init(self: SampleMutUninit) -> SampleMut: ...
@overload
def assume_init(self: SampleMutUninit, root: int) -> SampleMut: ...


def assume_init(self: SampleMutUninit, root=None) -> SampleMut:
    """Extracts the value of the uninitialized payload and labels the `SampleMutUninit` as
    initialized `SampleMut`

    After this call the `SampleMutUninit` is no longer usable!"""

    origin = get_origin(self.__payload_type_details)
    assert (origin == Flatbuffer and root != None) or (
        origin != Flatbuffer and root == None
    )

    if root is None:
        return self.__assume_init()

    builder = self.flatbuffer_builder()
    builder.Finish(root)

    payload_offset = builder.Head()
    buffer_len = len(builder.Bytes)
    base_view = (ctypes.c_ubyte * buffer_len).from_buffer(builder.Bytes)
    buffer_ptr = ctypes.addressof(base_view)

    initialized_sample = self.__assume_init_flatbuffer(
        buffer_ptr, buffer_len, payload_offset
    )
    _sample_mut_uninit_dict.pop(id(self), None)
    return initialized_sample


_sample_mut_uninit_del_original = getattr(SampleMutUninit, "__del__", None)


def _sample_mut_uninit_del(self: SampleMutUninit) -> None:
    _sample_mut_uninit_dict.pop(id(self), None)
    if _sample_mut_uninit_del_original is not None:
        _sample_mut_uninit_del_original(self)


PortFactoryPublisher.initial_max_slice_len = initial_max_slice_len
PortFactoryPublisher.allocation_strategy = allocation_strategy
PortFactoryPublisher.initial_reserved_memory = initial_reserved_memory

Publisher.send_copy = send_copy
Publisher.loan_uninit = loan_uninit
Publisher.loan_slice_uninit = loan_slice_uninit
Publisher.loan_flatbuffer = loan_flatbuffer

Sample.payload = payload
Sample.user_header = user_header
Sample.payload_bytes = payload_bytes
Sample.payload_root = payload_root

SampleMut.payload = payload
SampleMut.user_header = user_header
SampleMut.payload_bytes = payload_bytes
SampleMut.payload_root = payload_root

SampleMutUninit.write_payload = write_payload
SampleMutUninit.payload = payload
SampleMutUninit.user_header = user_header
SampleMutUninit.flatbuffer_builder = flatbuffer_builder
SampleMutUninit.assume_init = assume_init
SampleMutUninit.__del__ = _sample_mut_uninit_del

ServiceBuilder.publish_subscribe = publish_subscribe
ServiceBuilderPublishSubscribe.user_header = set_user_header
ServiceBuilderPublishSubscribe.flatbuffer_schema_path = flatbuffer_schema_path
