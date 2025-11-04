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

from ctypes import *

import iceoryx2 as iox2
import pytest

service_types = [iox2.ServiceType.Ipc, iox2.ServiceType.Local]


@pytest.mark.parametrize("service_type", service_types)
def test_same_entry_id_for_same_key(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key_0 = 0
    key_0 = key_0.to_bytes(8, "little")
    key_1 = 1
    key_1 = key_1.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(8, "little")

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key_0, c_uint64, value)
        .add(key_1, c_uint64, value)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key_0, c_uint64)
    reader = service.reader_builder().create()
    entry_handle_0 = reader.entry(key_0, c_uint64)
    entry_handle_1 = reader.entry(key_1, c_uint64)

    assert entry_handle_mut.entry_id().as_value == entry_handle_0.entry_id().as_value
    assert entry_handle_0.entry_id().as_value != entry_handle_1.entry_id().as_value


@pytest.mark.parametrize("service_type", service_types)
def test_simple_communication_works_reader_created_first(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(2, "little")

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint16, value)
        .create()
    )

    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, c_uint16)
    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, c_uint16)

    new_value = 1234
    entry_handle_mut.update_with_copy(c_uint16(new_value))
    assert (
        int.from_bytes(entry_handle.get(), byteorder="little", signed=False)
        == new_value
    )
    new_value = 4567
    entry_handle_mut.update_with_copy(c_uint16(new_value))
    assert (
        int.from_bytes(entry_handle.get(), byteorder="little", signed=False)
        == new_value
    )


@pytest.mark.parametrize("service_type", service_types)
def test_simple_communication_works_writer_created_first(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = 3
    key = key.to_bytes(8, "little")
    value = -3
    value = value.to_bytes(4, "little", signed=True)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_int32, value)
        .create()
    )

    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, c_int32)
    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, c_int32)

    new_value = 50
    entry_handle_mut.update_with_copy(c_int32(new_value))
    assert (
        int.from_bytes(entry_handle.get(), byteorder="little", signed=True) == new_value
    )
    new_value = -12
    entry_handle_mut.update_with_copy(c_int32(new_value))
    assert (
        int.from_bytes(entry_handle.get(), byteorder="little", signed=True) == new_value
    )


@pytest.mark.parametrize("service_type", service_types)
def test_loan_and_write_entry_value_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(4, "little")

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint32, value)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, c_uint32)
    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, c_uint32)

    entry_value_uninit = entry_handle_mut.loan_uninit()
    entry_value = entry_value_uninit.write(c_uint32(333))
    entry_handle_mut = entry_value.update()
    assert int.from_bytes(entry_handle.get(), byteorder="little", signed=False) == 333


@pytest.mark.parametrize("service_type", service_types)
def test_entry_handle_mut_can_be_reused_after_entry_value_was_updated(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(4, "little")

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint32, value)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, c_uint32)
    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, c_uint32)

    entry_value_uninit = entry_handle_mut.loan_uninit()
    entry_value = entry_value_uninit.write(c_uint32(333))
    entry_handle_mut = entry_value.update()
    assert int.from_bytes(entry_handle.get(), byteorder="little", signed=False) == 333

    entry_handle_mut.update_with_copy(c_uint32(999))
    assert int.from_bytes(entry_handle.get(), byteorder="little", signed=False) == 999


@pytest.mark.parametrize("service_type", service_types)
def test_entry_value_can_still_be_used_after_writer_was_dropped(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(4, "little")

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint32, value)
        .create()
    )

    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, c_uint32)
    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, c_uint32)
    entry_value_uninit = entry_handle_mut.loan_uninit()

    writer.delete()

    entry_value = entry_value_uninit.write(c_uint32(333))
    entry_handle_mut = entry_value.update()

    assert int.from_bytes(entry_handle.get(), byteorder="little", signed=False) == 333


@pytest.mark.parametrize("service_type", service_types)
def test_entry_handle_mut_can_be_reused_after_entry_value_uninit_was_discarded(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(4, "little")

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint32, value)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, c_uint32)
    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, c_uint32)

    entry_value_uninit = entry_handle_mut.loan_uninit()
    entry_handle_mut = entry_value_uninit.discard()

    entry_handle_mut.update_with_copy(c_uint32(333))
    assert int.from_bytes(entry_handle.get(), byteorder="little", signed=False) == 333


@pytest.mark.parametrize("service_type", service_types)
def test_entry_handle_mut_can_be_reused_after_entry_value_was_discarded(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(4, "little")

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint32, value)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, c_uint32)
    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, c_uint32)

    entry_value_uninit = entry_handle_mut.loan_uninit()
    entry_value = entry_value_uninit.write(c_uint32(999))
    entry_handle_mut = entry_value.discard()
    entry_handle_mut.update_with_copy(c_uint32(333))

    assert int.from_bytes(entry_handle.get(), byteorder="little", signed=False) == 333


# TODO: "drop" tests
# TODO: test different key types, e.g. str, key struct
