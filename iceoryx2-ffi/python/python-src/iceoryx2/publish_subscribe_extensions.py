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

from typing import Type, TypeVar
import ctypes
import iceoryx2 as iox2

T = TypeVar('T', bound=ctypes.Structure)

def payload(self, t: Type[T]):
    return ctypes.cast(self.payload_ptr, ctypes.POINTER(t))

def user_header(self, t: Type[T]):
    return ctypes.cast(self.user_header_ptr, ctypes.POINTER(t))

def publish_subscribe(self, t: Type[T]) -> iox2.ServiceBuilderPublishSubscribe:
    type_name = t.__name__
    if hasattr(t, "type_name"):
        type_name = t.type_name()
    result = self.__publish_subscribe()
    return result.__payload_type_details(
        iox2.TypeDetail.new()
        .type_variant(iox2.TypeVariant.FixedSize)
        .type_name(iox2.TypeName.new(type_name))
        .size(ctypes.sizeof(t))
        .alignment(ctypes.alignment(t))
    ).__user_header_type_details(
        iox2.TypeDetail.new()
        .type_variant(iox2.TypeVariant.FixedSize)
        .type_name(iox2.TypeName.new("()"))
        .size(0)
        .alignment(1)
    )

def set_user_header(self, t: Type[T]) -> iox2.ServiceBuilderPublishSubscribe:
    type_name = t.__name__
    if hasattr(t, "type_name"):
        type_name = t.type_name()
    return self.__user_header_type_details(
        iox2.TypeDetail.new()
        .type_variant(iox2.TypeVariant.FixedSize)
        .type_name(iox2.TypeName.new(type_name))
        .size(ctypes.sizeof(t))
        .alignment(ctypes.alignment(t))
    )

def send_copy(self, t: Type[T]) -> int:
    sample_uninit = self.loan_uninit()
    ctypes.memmove(sample_uninit.payload_ptr, ctypes.byref(t), ctypes.sizeof(t))
    sample = sample_uninit.assume_init()
    return sample.send()

def write_payload(self, t: Type[T]) -> iox2.SampleMut:
    ctypes.memmove(self.payload_ptr, ctypes.byref(t), ctypes.sizeof(t))
    return self.assume_init()


iox2.Publisher.send_copy = send_copy

iox2.Sample.payload = payload
iox2.Sample.user_header = user_header

iox2.SampleMut.payload = payload
iox2.SampleMut.user_header = user_header

iox2.SampleMutUninit.write_payload = write_payload
iox2.SampleMutUninit.payload = payload
iox2.SampleMutUninit.user_header = user_header

iox2.ServiceBuilder.publish_subscribe = publish_subscribe
iox2.ServiceBuilderPublishSubscribe.user_header = set_user_header
