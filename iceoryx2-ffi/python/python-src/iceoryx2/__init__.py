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

from ._iceoryx2 import *
from .publisher import PublisherFoo
from typing import Type, TypeVar
import ctypes
import types
import inspect

T = TypeVar('T', bound=ctypes.Structure)

original_service_builder_publish_subscribe = object.__getattribute__(ServiceBuilder, "publish_subscribe")

def publish_subscribe(self, T: Type[T]):
    type_name = T.__name__
    if hasattr(T, "type_name"):
        type_name = T.type_name()
    result = original_service_builder_publish_subscribe(self)
    return result.payload_type_details(
        TypeDetail.new()
        .type_variant(TypeVariant.FixedSize)
        .type_name(TypeName.new(type_name))
        .size(ctypes.sizeof(T))
        .alignment(ctypes.alignment(T))
    ).user_header_type_details(
        TypeDetail.new()
        .type_variant(TypeVariant.FixedSize)
        .type_name(TypeName.new("()"))
        .size(0)
        .alignment(1)
    )

ServiceBuilder.publish_subscribe = publish_subscribe
