# Copyright (c) 2026 Contributors to the Eclipse Foundation
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

import ctypes
import os

import flatbuffers
import iceoryx2 as iox2
import pytest
from flatbuffers.compat import import_numpy

np = import_numpy()

service_types = [iox2.ServiceType.Ipc, iox2.ServiceType.Local]


class UnboundedData(object):
    __slots__ = ["_tab"]

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = UnboundedData()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsUnboundedData(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)

    # UnboundedData
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # UnboundedData
    def Data1(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0


def UnboundedDataStart(builder):
    builder.StartObject(1)


def Start(builder):
    UnboundedDataStart(builder)


def UnboundedDataAddData1(builder, data1):
    builder.PrependInt32Slot(0, data1, 0)


def AddData1(builder, data1):
    UnboundedDataAddData1(builder, data1)


def UnboundedDataEnd(builder):
    return builder.EndObject()


def End(builder):
    return UnboundedDataEnd(builder)


schema = """
table UnboundedData {
    data_1: int32;
}

root_type UnboundedData;
"""

alt_schema = """
table BoundedData {
    data_1: int32;
}

root_type BoundedData;
"""


def create_schema_file(content) -> iox2.FilePath:
    iox2.testing.create_test_directory()
    file_path = iox2.testing.generate_file_path()
    with open(file_path.to_string(), "w") as file:
        file.write(content)

    return file_path


@pytest.mark.parametrize("service_type", service_types)
def test_create_fails_when_no_schema_file_is_available(
    service_type: iox2.ServiceType,
) -> None:

    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    with pytest.raises(iox2.PublishSubscribeCreateError) as exc_info:
        node.service_builder(service_name).publish_subscribe(
            iox2.Flatbuffer[UnboundedData]
        ).create()

    assert str(exc_info.value) == "UnableToAcquireTypeDefinition"


@pytest.mark.parametrize("service_type", service_types)
def test_create_succeeds_with_schema_file(
    service_type: iox2.ServiceType,
) -> None:
    schema_file_path = create_schema_file(schema)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    try:
        node.service_builder(service_name).publish_subscribe(
            iox2.Flatbuffer[UnboundedData]
        ).flatbuffer_schema_path(schema_file_path).create()
    except iox2.PublishSubscribeCreateError:
        assert False

    os.remove(schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_open_fails_when_no_schema_file_is_available(
    service_type: iox2.ServiceType,
) -> None:
    schema_file_path = create_schema_file(schema)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    try:
        sut = (
            node.service_builder(service_name)
            .publish_subscribe(iox2.Flatbuffer[UnboundedData])
            .flatbuffer_schema_path(schema_file_path)
            .create()
        )
    except iox2.PublishSubscribeCreateError:
        assert False

    with pytest.raises(iox2.PublishSubscribeOpenError) as exc_info:
        node.service_builder(service_name).publish_subscribe(
            iox2.Flatbuffer[UnboundedData]
        ).open()

    assert str(exc_info.value) == "UnableToAcquireTypeDefinition"

    os.remove(schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_open_fails_schema_is_not_the_same(
    service_type: iox2.ServiceType,
) -> None:
    schema_file_path = create_schema_file(schema)
    alt_schema_file_path = create_schema_file(alt_schema)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    try:
        sut = (
            node.service_builder(service_name)
            .publish_subscribe(iox2.Flatbuffer[UnboundedData])
            .flatbuffer_schema_path(schema_file_path)
            .create()
        )
    except iox2.PublishSubscribeCreateError:
        assert False

    with pytest.raises(iox2.PublishSubscribeOpenError) as exc_info:
        node.service_builder(service_name).publish_subscribe(
            iox2.Flatbuffer[UnboundedData]
        ).flatbuffer_schema_path(alt_schema_file_path).open()

    assert str(exc_info.value) == "IncompatibleTypes"

    os.remove(schema_file_path.to_string())
    os.remove(alt_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_open_succeeds_when_schema_content_is_identical(
    service_type: iox2.ServiceType,
) -> None:
    schema_file_path = create_schema_file(schema)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    try:
        sut = (
            node.service_builder(service_name)
            .publish_subscribe(iox2.Flatbuffer[UnboundedData])
            .flatbuffer_schema_path(schema_file_path)
            .create()
        )
    except iox2.PublishSubscribeCreateError:
        assert False

    try:
        node.service_builder(service_name).publish_subscribe(
            iox2.Flatbuffer[UnboundedData]
        ).flatbuffer_schema_path(schema_file_path).open()
    except iox2.PublishSubscribeOpenError:
        assert False

    os.remove(schema_file_path.to_string())
