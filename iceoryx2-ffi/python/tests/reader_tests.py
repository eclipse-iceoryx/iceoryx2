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

from ctypes import c_uint64, c_uint8, c_int64, c_uint32

import iceoryx2 as iox2
import pytest

service_types = [iox2.ServiceType.Ipc, iox2.ServiceType.Local]


@pytest.mark.parametrize("service_type", service_types)
def test_reader_is_is_unique(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(1, "little")
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(c_uint64(0), c_uint8, value)
        .create()
    )

    max_readers = 8
    readers = []
    reader_ids = {0}
    assert len(reader_ids) == 1

    i = 0
    while i < max_readers:
        reader = service.reader_builder().create()
        reader_ids.add(reader.id.value)
        readers.append(reader)
        i += 1

    assert len(reader_ids) == max_readers + 1


@pytest.mark.parametrize("service_type", service_types)
def test_handle_can_be_acquired_for_existing_key_value_pair(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    value = 7
    value = value.to_bytes(8, "little")
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(c_uint64(0), c_uint64, value)
        .create()
    )

    reader = service.reader_builder().create()
    try:
        entry_handle = reader.entry(key, c_uint64)
        value = entry_handle.get()
        assert int.from_bytes(value, byteorder="little", signed=False) == 7

    except iox2.EntryHandleError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_handle_cannot_be_acquired_for_non_existing_key(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(8, "little")
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(c_uint64(0), c_uint64, value)
        .create()
    )

    reader = service.reader_builder().create()
    invalid_key = 9
    invalid_key = invalid_key.to_bytes(8, "little")
    with pytest.raises(iox2.EntryHandleError):
        reader.entry(invalid_key, c_uint64)


@pytest.mark.parametrize("service_type", service_types)
def test_handle_cannot_be_acquired_for_wrong_value_type(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(8, "little")
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(c_uint64(0), c_uint64, value)
        .create()
    )

    reader = service.reader_builder().create()
    invalid_key = 0
    invalid_key = invalid_key.to_bytes(8, "little")
    with pytest.raises(iox2.EntryHandleError):
        reader.entry(invalid_key, c_int64)


@pytest.mark.parametrize("service_type", service_types)
def test_deleting_reader_removes_it_from_the_service(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(4, "little")
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(c_uint64(0), c_uint32, value)
        .max_readers(1)
        .create()
    )

    sut = service.reader_builder().create()

    with pytest.raises(iox2.ReaderCreateError):
        sut = service.reader_builder().create()

    sut.delete()

    try:
        sut = service.reader_builder().create()
    except iox2.ReaderCreateError:
        assert False
