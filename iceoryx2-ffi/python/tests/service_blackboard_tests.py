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
def test_communication_with_max_readers(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    number_of_iterations = 128
    max_readers = 6
    key = 0
    key = key.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(8, "little")

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint64, value)
        .max_readers(max_readers)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, c_uint64)

    readers = []
    i = 0
    while i < max_readers:
        readers.append(service.reader_builder().create())
        i += 1

    counter = 0
    while counter < number_of_iterations:
        entry_handle_mut.update_with_copy(c_uint64(counter))

        i = 0
        while i < max_readers:
            entry_handle = readers[i].entry(key, c_uint64)
            int.from_bytes(
                entry_handle.get(), byteorder="little", signed=False
            ) == counter
            i += 1

        counter += 1


@pytest.mark.parametrize("service_type", service_types)
def test_communication_with_max_readers_and_entry_handle_muts(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    max_handles = 6
    keys = []
    i = 0
    while i < max_handles + 1:
        key = i
        keys.append(key.to_bytes(8, "little"))
        i += 1

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(keys[0], c_uint64, keys[0])
        .add(keys[1], c_uint64, keys[1])
        .add(keys[2], c_uint64, keys[2])
        .add(keys[3], c_uint64, keys[3])
        .add(keys[4], c_uint64, keys[4])
        .add(keys[5], c_uint64, keys[5])
        .add(keys[6], c_uint64, keys[6])
        .max_readers(max_handles)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_muts = []

    reader = service.reader_builder().create()
    entry_handles = []

    i = 0
    while i < max_handles:
        entry_handle_muts.append(writer.entry(keys[i], c_uint64))
        entry_handles.append(reader.entry(keys[i], c_uint64))
        i += 1

    i = 0
    while i < max_handles:
        entry_handle_muts[i].update_with_copy(c_uint64(7))

        j = 0
        while j < i + 1:
            assert (
                int.from_bytes(entry_handles[j].get(), byteorder="little", signed=False)
                == 7
            )
            j += 1

        j = i + 1
        while j < max_handles:
            assert (
                int.from_bytes(entry_handles[j].get(), byteorder="little", signed=False)
                == j
            )
            j += 1

        i += 1


@pytest.mark.parametrize("service_type", service_types)
def test_write_and_read_different_value_types_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    keys = [0, 1, 23, 100, 13]
    value_0 = 0
    value_0 = value_0.to_bytes(8, "little")
    value_1 = -5
    value_1 = value_1.to_bytes(1, "little", signed=True)
    value_23 = "Nala"
    value_23 = value_23.encode("utf-8")
    value_100 = False
    value_100 = value_100.to_bytes(1, "little")
    # TODO: add Groovy struct

    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(keys[0].to_bytes(8, "little"), c_uint64, value_0)
        .add(keys[1].to_bytes(8, "little"), c_int8, value_1)
        .add(keys[2].to_bytes(8, "little"), c_uint8 * 4, value_23)
        .add(keys[3].to_bytes(8, "little"), c_bool, value_100)
        .create()
    )

    writer = service.writer_builder().create()
    writer.entry(keys[0].to_bytes(8, "little"), c_uint64).update_with_copy(
        c_uint64(2008)
    )
    writer.entry(keys[1].to_bytes(8, "little"), c_int8).update_with_copy(c_int8(11))
    # TODO: how to pass string? check Payload in service_builder_publish_subscribe_tests.py
    # B = c_uint8 * 4
    # x = "Wolf"
    # # x = x.encode("utf-8")
    # x = bytearray(x, "utf-8")
    # writer.entry(keys[2].to_bytes(8, "little"), c_uint8 * 4).update_with_copy(B(x))
    writer.entry(keys[3].to_bytes(8, "little"), c_bool).update_with_copy(c_bool(True))

    reader = service.reader_builder().create()
    assert (
        int.from_bytes(
            reader.entry(keys[0].to_bytes(8, "little"), c_uint64).get(),
            byteorder="little",
            signed=False,
        )
        == 2008
    )
    assert (
        int.from_bytes(
            reader.entry(keys[1].to_bytes(8, "little"), c_int8).get(),
            byteorder="little",
            signed=False,
        )
        == 11
    )
    # TODO: how to convert other types than integers?
    # assert (
    #     int.from_bytes(
    #         reader.entry(keys[3].to_bytes(8, "little"), c_bool).get(),
    #         byteorder="little",
    #         signed=False,
    #     )
    #     == True
    # )


@pytest.mark.parametrize("service_type", service_types)
def test_creating_max_supported_amount_of_ports_work(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    max_readers = 8
    readers = []
    key = 0
    key = key.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(1, "little")

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint8, value)
        .max_readers(max_readers)
        .create()
    )

    # acquire all possible ports
    writer = service.writer_builder().create()

    i = 0
    while i < max_readers:
        readers.append(service.reader_builder().create())
        i += 1

    # create additional ports and fail
    with pytest.raises(iox2.WriterCreateError):
        service.writer_builder().create()
    with pytest.raises(iox2.ReaderCreateError):
        service.reader_builder().create()

    # remove one reader and the writer
    writer.delete()
    readers[0].delete()

    # create additional ports shall work again
    try:
        service.writer_builder().create()
    except iox2.WriterCreateError:
        assert False
    try:
        service.reader_builder().create()
    except iox2.ReaderCreateError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_dropping_service_keeps_established_communication(
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

    service.delete()

    new_value = 981293
    entry_handle_mut.update_with_copy(c_uint32(new_value))
    assert (
        int.from_bytes(entry_handle.get(), byteorder="little", signed=False)
        == new_value
    )


@pytest.mark.parametrize("service_type", service_types)
def test_ports_of_dropped_service_block_new_service_creation(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(1, "little")

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint8, value)
        .create()
    )

    reader = service.reader_builder().create()
    writer = service.writer_builder().create()

    service.delete()

    with pytest.raises(iox2.BlackboardCreateError):
        node.service_builder(service_name).blackboard_creator(c_uint64).add(
            key, c_uint8, value
        ).create()

    reader.delete()

    with pytest.raises(iox2.BlackboardCreateError):
        node.service_builder(service_name).blackboard_creator(c_uint64).add(
            key, c_uint8, value
        ).create()

    writer.delete()

    try:
        node.service_builder(service_name).blackboard_creator(c_uint64).add(
            key, c_uint8, value
        ).create()
    except iox2.BlackboardCreateError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_service_can_be_opened_when_a_writer_exists(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(8, "little")

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint64, value)
        .create()
    )
    _writer = service.writer_builder().create()

    service.delete()

    with pytest.raises(iox2.BlackboardCreateError):
        node.service_builder(service_name).blackboard_creator(c_uint64).add(
            key, c_uint64, value
        ).create()

    try:
        node.service_builder(service_name).blackboard_opener(c_uint64).open()
    except iox2.BlackboardOpenError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_service_can_be_opened_when_a_reader_exists(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(8, "little")

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint64, value)
        .create()
    )
    _reader = service.reader_builder().create()

    service.delete()

    with pytest.raises(iox2.BlackboardCreateError):
        node.service_builder(service_name).blackboard_creator(c_uint64).add(
            key, c_uint64, value
        ).create()

    try:
        node.service_builder(service_name).blackboard_opener(c_uint64).open()
    except iox2.BlackboardOpenError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_reader_can_still_read_value_when_writer_was_disconnected(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(1, "little")

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint8, value)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, c_uint8)
    entry_handle_mut.update_with_copy(c_uint8(5))
    entry_handle_mut.delete()
    writer.delete()

    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, c_uint8)
    assert int.from_bytes(entry_handle.get(), byteorder="little", signed=False) == 5


@pytest.mark.parametrize("service_type", service_types)
def test_reconnected_reader_sees_current_blackboard_status(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key_0 = 0
    key_0 = key_0.to_bytes(8, "little")
    key_1 = 6
    key_1 = key_1.to_bytes(8, "little")
    value_0 = 0
    value_0 = value_0.to_bytes(1, "little")
    value_1 = -9
    value_1 = value_1.to_bytes(4, "little", signed=True)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key_0, c_uint8, value_0)
        .add(key_1, c_int32, value_1)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key_0, c_uint8)
    entry_handle_mut.update_with_copy(c_uint8(5))

    reader = service.reader_builder().create()
    entry_handle_0 = reader.entry(key_0, c_uint8)
    assert int.from_bytes(entry_handle_0.get(), byteorder="little", signed=False) == 5
    entry_handle_1 = reader.entry(key_1, c_int32)
    assert int.from_bytes(entry_handle_1.get(), byteorder="little", signed=True) == -9

    reader.delete()

    entry_handle_mut = writer.entry(key_1, c_int32)
    entry_handle_mut.update_with_copy(c_int32(-567))

    reader = service.reader_builder().create()
    entry_handle_0 = reader.entry(key_0, c_uint8)
    assert int.from_bytes(entry_handle_0.get(), byteorder="little", signed=False) == 5
    entry_handle_1 = reader.entry(key_1, c_int32)
    assert int.from_bytes(entry_handle_1.get(), byteorder="little", signed=True) == -567


@pytest.mark.parametrize("service_type", service_types)
def test_entry_handle_mut_can_still_write_after_writer_was_dropped(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(1, "little")

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint8, value)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, c_uint8)

    writer.delete()
    entry_handle_mut.update_with_copy(c_uint8(1))

    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, c_uint8)
    assert int.from_bytes(entry_handle.get(), byteorder="little", signed=False) == 1


@pytest.mark.parametrize("service_type", service_types)
def test_entry_handle_can_still_read_after_reader_was_dropped(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    value = 0
    value = value.to_bytes(1, "little")

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint8, value)
        .create()
    )

    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, c_uint8)

    reader.delete()
    assert int.from_bytes(entry_handle.get(), byteorder="little", signed=False) == 0

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, c_uint8)
    entry_handle_mut.update_with_copy(c_uint8(1))

    assert int.from_bytes(entry_handle.get(), byteorder="little", signed=False) == 1


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


@pytest.mark.parametrize("service_type", service_types)
def test_handle_can_still_be_used_after_every_previous_service_state_owner_was_dropped(
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

    writer.delete()
    service.delete()

    entry_handle_mut.update_with_copy(c_uint32(3))
    entry_handle_mut.delete()

    service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint32, value)
        .create()
    )

    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, c_uint32)

    reader.delete()
    service.delete()

    assert int.from_bytes(entry_handle.get(), byteorder="little", signed=False) == 0


# TODO: missing tests
# - concurrent_write_and_read_of_the_same_value_works
# - concurrent_write_of_different_values_works
# - simple_communication_with_key_struct_works
# - adding_key_struct_twice_fails
