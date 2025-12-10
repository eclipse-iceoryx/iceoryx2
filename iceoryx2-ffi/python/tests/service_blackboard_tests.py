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

import ctypes

import iceoryx2 as iox2
import pytest

service_types = [iox2.ServiceType.Ipc, iox2.ServiceType.Local]


@pytest.mark.parametrize("service_type", service_types)
def test_same_entry_id_for_same_key(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key_0 = ctypes.c_uint64(0)
    key_1 = ctypes.c_uint64(1)
    value = ctypes.c_uint64(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key_0, value)
        .add(key_1, value)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key_0, ctypes.c_uint64)
    reader = service.reader_builder().create()
    entry_handle_0 = reader.entry(key_0, ctypes.c_uint64)
    entry_handle_1 = reader.entry(key_1, ctypes.c_uint64)

    assert entry_handle_mut.entry_id.as_value == entry_handle_0.entry_id.as_value
    assert entry_handle_0.entry_id.as_value != entry_handle_1.entry_id.as_value


@pytest.mark.parametrize("service_type", service_types)
def test_simple_communication_works_reader_created_first(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = ctypes.c_uint64(0)
    value = ctypes.c_uint16(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
        .create()
    )

    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, ctypes.c_uint16)
    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, ctypes.c_uint16)

    new_value = 1234
    entry_handle_mut.update_with_copy(ctypes.c_uint16(new_value))
    assert entry_handle.get().decode_as(ctypes.c_uint16).value == new_value

    new_value = 4567
    entry_handle_mut.update_with_copy(ctypes.c_uint16(new_value))
    assert entry_handle.get().decode_as(ctypes.c_uint16).value == new_value


@pytest.mark.parametrize("service_type", service_types)
def test_simple_communication_works_writer_created_first(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = ctypes.c_uint64(3)
    value = ctypes.c_int32(-3)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
        .create()
    )

    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, ctypes.c_int32)
    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, ctypes.c_int32)

    new_value = 50
    entry_handle_mut.update_with_copy(ctypes.c_int32(new_value))
    assert entry_handle.get().decode_as(ctypes.c_int32).value == new_value
    new_value = -12
    entry_handle_mut.update_with_copy(ctypes.c_int32(new_value))
    assert entry_handle.get().decode_as(ctypes.c_int32).value == new_value


@pytest.mark.parametrize("service_type", service_types)
def test_communication_with_max_readers(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    number_of_iterations = 128
    max_readers = 6
    key = ctypes.c_uint64(0)
    value = ctypes.c_uint64(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
        .max_readers(max_readers)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, ctypes.c_uint64)

    readers = []
    i = 0
    while i < max_readers:
        readers.append(service.reader_builder().create())
        i += 1

    counter = 0
    while counter < number_of_iterations:
        entry_handle_mut.update_with_copy(ctypes.c_uint64(counter))

        i = 0
        while i < max_readers:
            entry_handle = readers[i].entry(key, ctypes.c_uint64)
            assert entry_handle.get().decode_as(ctypes.c_uint64).value == counter
            i += 1

        counter += 1


@pytest.mark.parametrize("service_type", service_types)
def test_communication_with_several_entry_handles_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    max_handles = 6
    key_0 = ctypes.c_uint64(0)
    key_1 = ctypes.c_uint64(1)
    key_2 = ctypes.c_uint64(2)
    key_3 = ctypes.c_uint64(3)
    key_4 = ctypes.c_uint64(4)
    key_5 = ctypes.c_uint64(5)
    key_6 = ctypes.c_uint64(6)
    keys = [key_0, key_1, key_2, key_3, key_4, key_5, key_6]

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key_0, key_0)
        .add(key_1, key_1)
        .add(key_2, key_2)
        .add(key_3, key_3)
        .add(key_4, key_4)
        .add(key_5, key_5)
        .add(key_6, key_6)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_muts = []

    reader = service.reader_builder().create()
    entry_handles = []

    i = 0
    while i < max_handles:
        entry_handle_muts.append(writer.entry(keys[i], ctypes.c_uint64))
        entry_handles.append(reader.entry(keys[i], ctypes.c_uint64))
        i += 1

    i = 0
    while i < max_handles:
        entry_handle_muts[i].update_with_copy(ctypes.c_uint64(7))

        j = 0
        while j < i + 1:
            assert entry_handles[j].get().decode_as(ctypes.c_uint64).value == 7
            j += 1

        j = i + 1
        while j < max_handles:
            assert entry_handles[j].get().decode_as(ctypes.c_uint64).value == j
            j += 1

        i += 1


@pytest.mark.parametrize("service_type", service_types)
def test_write_and_read_different_value_types_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    class Groovy(ctypes.Structure):
        _fields_ = [
            ("a", ctypes.c_bool),
            ("b", ctypes.c_uint32),
            ("c", ctypes.c_int64),
        ]

    key_0 = ctypes.c_uint64(0)
    key_1 = ctypes.c_uint64(1)
    key_2 = ctypes.c_uint64(100)
    key_3 = ctypes.c_uint64(13)
    value_0 = ctypes.c_uint64(0)
    value_1 = ctypes.c_int8(-5)
    value_2 = ctypes.c_bool(False)
    value_3 = Groovy(
        a=ctypes.c_bool(True), b=ctypes.c_uint32(7127), c=ctypes.c_int64(609)
    )

    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key_0, value_0)
        .add(key_1, value_1)
        .add(key_2, value_2)
        .add(key_3, value_3)
        .create()
    )

    writer = service.writer_builder().create()
    writer.entry(key_0, ctypes.c_uint64).update_with_copy(ctypes.c_uint64(2008))
    writer.entry(key_1, ctypes.c_int8).update_with_copy(ctypes.c_int8(11))
    writer.entry(key_2, ctypes.c_bool).update_with_copy(ctypes.c_bool(True))
    writer.entry(key_3, Groovy).update_with_copy(
        Groovy(a=ctypes.c_bool(False), b=ctypes.c_uint32(888), c=ctypes.c_int64(906))
    )

    reader = service.reader_builder().create()
    assert (
        reader.entry(key_0, ctypes.c_uint64).get().decode_as(ctypes.c_uint64).value
        == 2008
    )
    assert reader.entry(key_1, ctypes.c_int8).get().decode_as(ctypes.c_int8).value == 11
    assert reader.entry(key_2, ctypes.c_bool).get().decode_as(ctypes.c_bool).value
    entry_value_key_3 = reader.entry(key_3, Groovy).get().decode_as(Groovy)
    assert not entry_value_key_3.a
    assert entry_value_key_3.b == 888
    assert entry_value_key_3.c == 906


@pytest.mark.parametrize("service_type", service_types)
def test_creating_max_supported_amount_of_ports_work(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    max_readers = 8
    readers = []
    key = ctypes.c_uint64(0)
    value = ctypes.c_uint8(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
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
    key = ctypes.c_uint64(0)
    value = ctypes.c_uint32(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, ctypes.c_uint32)
    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, ctypes.c_uint32)

    service.delete()

    new_value = 981293
    entry_handle_mut.update_with_copy(ctypes.c_uint32(new_value))
    assert entry_handle.get().decode_as(ctypes.c_uint32).value == new_value


@pytest.mark.parametrize("service_type", service_types)
def test_ports_of_dropped_service_block_new_service_creation(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = ctypes.c_uint64(0)
    value = ctypes.c_uint8(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
        .create()
    )

    reader = service.reader_builder().create()
    writer = service.writer_builder().create()

    service.delete()

    with pytest.raises(iox2.BlackboardCreateError):
        node.service_builder(service_name).blackboard_creator(ctypes.c_uint64).add(
            key, value
        ).create()

    reader.delete()

    with pytest.raises(iox2.BlackboardCreateError):
        node.service_builder(service_name).blackboard_creator(ctypes.c_uint64).add(
            key, value
        ).create()

    writer.delete()

    try:
        node.service_builder(service_name).blackboard_creator(ctypes.c_uint64).add(
            key, value
        ).create()
    except iox2.BlackboardCreateError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_service_can_be_opened_when_a_writer_exists(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = ctypes.c_uint64(0)
    value = ctypes.c_uint64(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
        .create()
    )
    _writer = service.writer_builder().create()

    service.delete()

    with pytest.raises(iox2.BlackboardCreateError):
        node.service_builder(service_name).blackboard_creator(ctypes.c_uint64).add(
            key, value
        ).create()

    try:
        node.service_builder(service_name).blackboard_opener(ctypes.c_uint64).open()
    except iox2.BlackboardOpenError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_service_can_be_opened_when_a_reader_exists(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = ctypes.c_uint64(0)
    value = ctypes.c_uint64(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
        .create()
    )
    _reader = service.reader_builder().create()

    service.delete()

    with pytest.raises(iox2.BlackboardCreateError):
        node.service_builder(service_name).blackboard_creator(ctypes.c_uint64).add(
            key, value
        ).create()

    try:
        node.service_builder(service_name).blackboard_opener(ctypes.c_uint64).open()
    except iox2.BlackboardOpenError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_reader_can_still_read_value_when_writer_was_disconnected(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = ctypes.c_uint64(0)
    value = ctypes.c_uint8(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, ctypes.c_uint8)
    entry_handle_mut.update_with_copy(ctypes.c_uint8(5))
    entry_handle_mut.delete()
    writer.delete()

    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, ctypes.c_uint8)
    assert entry_handle.get().decode_as(ctypes.c_uint8).value == 5


@pytest.mark.parametrize("service_type", service_types)
def test_reconnected_reader_sees_current_blackboard_status(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key_0 = ctypes.c_uint64(0)
    key_1 = ctypes.c_uint64(6)
    value_0 = ctypes.c_uint8(0)
    value_1 = ctypes.c_int32(-9)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key_0, value_0)
        .add(key_1, value_1)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key_0, ctypes.c_uint8)
    entry_handle_mut.update_with_copy(ctypes.c_uint8(5))

    reader = service.reader_builder().create()
    entry_handle_0 = reader.entry(key_0, ctypes.c_uint8)
    assert entry_handle_0.get().decode_as(ctypes.c_uint8).value == 5
    entry_handle_1 = reader.entry(key_1, ctypes.c_int32)
    assert entry_handle_1.get().decode_as(ctypes.c_int32).value == -9

    reader.delete()

    entry_handle_mut = writer.entry(key_1, ctypes.c_int32)
    entry_handle_mut.update_with_copy(ctypes.c_int32(-567))

    reader = service.reader_builder().create()
    entry_handle_0 = reader.entry(key_0, ctypes.c_uint8)
    assert entry_handle_0.get().decode_as(ctypes.c_uint8).value == 5
    entry_handle_1 = reader.entry(key_1, ctypes.c_int32)
    assert entry_handle_1.get().decode_as(ctypes.c_int32).value == -567


@pytest.mark.parametrize("service_type", service_types)
def test_entry_handle_mut_can_still_write_after_writer_was_dropped(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = ctypes.c_uint64(0)
    value = ctypes.c_uint8(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, ctypes.c_uint8)

    writer.delete()
    entry_handle_mut.update_with_copy(ctypes.c_uint8(1))

    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, ctypes.c_uint8)
    assert entry_handle.get().decode_as(ctypes.c_uint8).value == 1


@pytest.mark.parametrize("service_type", service_types)
def test_entry_handle_can_still_read_after_reader_was_dropped(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = ctypes.c_uint64(0)
    value = ctypes.c_uint8(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
        .create()
    )

    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, ctypes.c_uint8)

    reader.delete()
    assert entry_handle.get().decode_as(ctypes.c_uint8).value == 0

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, ctypes.c_uint8)
    entry_handle_mut.update_with_copy(ctypes.c_uint8(1))

    assert entry_handle.get().decode_as(ctypes.c_uint8).value == 1


@pytest.mark.parametrize("service_type", service_types)
def test_loan_and_write_entry_value_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = ctypes.c_uint64(0)
    value = ctypes.c_uint32(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, ctypes.c_uint32)
    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, ctypes.c_uint32)

    entry_value_uninit = entry_handle_mut.loan_uninit()
    entry_handle_mut = entry_value_uninit.update_with_copy(ctypes.c_uint32(333))
    assert entry_handle.get().decode_as(ctypes.c_uint32).value == 333


@pytest.mark.parametrize("service_type", service_types)
def test_entry_handle_mut_can_be_reused_after_entry_value_was_updated(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = ctypes.c_uint64(0)
    value = ctypes.c_uint32(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, ctypes.c_uint32)
    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, ctypes.c_uint32)

    entry_value_uninit = entry_handle_mut.loan_uninit()
    entry_handle_mut = entry_value_uninit.update_with_copy(ctypes.c_uint32(333))
    assert entry_handle.get().decode_as(ctypes.c_uint32).value == 333

    entry_handle_mut.update_with_copy(ctypes.c_uint32(999))
    assert entry_handle.get().decode_as(ctypes.c_uint32).value == 999


@pytest.mark.parametrize("service_type", service_types)
def test_entry_value_can_still_be_used_after_writer_was_dropped(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = ctypes.c_uint64(0)
    value = ctypes.c_uint32(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
        .create()
    )

    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, ctypes.c_uint32)
    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, ctypes.c_uint32)
    entry_value_uninit = entry_handle_mut.loan_uninit()

    writer.delete()

    entry_handle_mut = entry_value_uninit.update_with_copy(ctypes.c_uint32(333))

    assert entry_handle.get().decode_as(ctypes.c_uint32).value == 333


@pytest.mark.parametrize("service_type", service_types)
def test_entry_handle_mut_can_be_reused_after_entry_value_uninit_was_discarded(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = ctypes.c_uint64(0)
    value = ctypes.c_uint32(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, ctypes.c_uint32)
    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, ctypes.c_uint32)

    entry_value_uninit = entry_handle_mut.loan_uninit()
    entry_handle_mut = entry_value_uninit.discard()

    entry_handle_mut.update_with_copy(ctypes.c_uint32(333))
    assert entry_handle.get().decode_as(ctypes.c_uint32).value == 333


@pytest.mark.parametrize("service_type", service_types)
def test_handle_can_still_be_used_after_every_previous_service_state_owner_was_dropped(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = ctypes.c_uint64(0)
    value = ctypes.c_uint32(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, ctypes.c_uint32)

    writer.delete()
    service.delete()

    entry_handle_mut.update_with_copy(ctypes.c_uint32(3))
    entry_handle_mut.delete()

    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
        .create()
    )

    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, ctypes.c_uint32)

    reader.delete()
    service.delete()

    assert entry_handle.get().decode_as(ctypes.c_uint32).value == 0


@pytest.mark.parametrize("service_type", service_types)
def test_entry_handle_is_up_to_date_works_correctly(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = ctypes.c_uint64(0)
    value = ctypes.c_uint16(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
        .create()
    )

    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, ctypes.c_uint16)
    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, ctypes.c_uint16)

    read_value = entry_handle.get()
    assert read_value.decode_as(ctypes.c_uint16).value == 0
    assert entry_handle.is_up_to_date(read_value)

    entry_handle_mut.update_with_copy(ctypes.c_uint16(1))
    assert not entry_handle.is_up_to_date(read_value)
    read_value = entry_handle.get()
    assert read_value.decode_as(ctypes.c_uint16).value == 1
    assert entry_handle.is_up_to_date(read_value)

    entry_handle_mut.update_with_copy(ctypes.c_uint16(4))
    read_value = entry_handle.get()
    assert read_value.decode_as(ctypes.c_uint16).value == 4
    assert entry_handle.is_up_to_date(read_value)


@pytest.mark.parametrize("service_type", service_types)
def test_list_keys_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    keys = [
        ctypes.c_uint64(0),
        ctypes.c_uint64(1),
        ctypes.c_uint64(2),
        ctypes.c_uint64(3),
        ctypes.c_uint64(4),
        ctypes.c_uint64(5),
        ctypes.c_uint64(6),
        ctypes.c_uint64(7),
    ]
    value = ctypes.c_uint16(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(keys[0], value)
        .add(keys[1], value)
        .add(keys[2], value)
        .add(keys[3], value)
        .add(keys[4], value)
        .add(keys[5], value)
        .add(keys[6], value)
        .add(keys[7], value)
        .create()
    )

    listed_keys = service.list_keys()
    assert len(listed_keys) == len(keys)

    found_key = False
    for _i, key in enumerate(keys):
        key_value = key.value
        for _j, listed_key in enumerate(listed_keys):
            listed_key_value = listed_key.decode_as(ctypes.c_uint64).value
            if listed_key_value == key_value:
                found_key = True
                break
        assert found_key
        found_key = False


class Foo(ctypes.Structure):
    _fields_ = [
        ("a", ctypes.c_uint8),
        ("b", ctypes.c_uint32),
        ("c", ctypes.c_double),
    ]

    def __eq__(self, other):
        """Key eq comparison function."""
        if not isinstance(other, Foo):
            return NotImplemented
        return (self.a, self.b, self.c) == (other.a, other.b, other.c)


@pytest.mark.parametrize("service_type", service_types)
def test_simple_communication_with_key_struct_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    key_1 = Foo(a=9, b=99, c=9.9)
    value_1 = ctypes.c_int32(-3)
    key_2 = Foo(a=9, b=999, c=9.9)
    value_2 = ctypes.c_uint32(3)

    service = (
        node.service_builder(service_name)
        .blackboard_creator(Foo)
        .add(key_1, value_1)
        .add(key_2, value_2)
        .create()
    )

    writer = service.writer_builder().create()
    entry_handle_mut_1 = writer.entry(key_1, ctypes.c_int32)
    entry_handle_mut_2 = writer.entry(key_2, ctypes.c_uint32)
    reader = service.reader_builder().create()
    entry_handle_1 = reader.entry(key_1, ctypes.c_int32)
    entry_handle_2 = reader.entry(key_2, ctypes.c_uint32)

    assert entry_handle_1.get().decode_as(ctypes.c_int32).value == -3
    assert entry_handle_2.get().decode_as(ctypes.c_uint32).value == 3

    entry_handle_mut_1.update_with_copy(ctypes.c_int32(50))
    assert entry_handle_1.get().decode_as(ctypes.c_int32).value == 50
    assert entry_handle_2.get().decode_as(ctypes.c_uint32).value == 3

    entry_handle_mut_2.update_with_copy(ctypes.c_uint32(12))
    assert entry_handle_1.get().decode_as(ctypes.c_int32).value == 50
    assert entry_handle_2.get().decode_as(ctypes.c_uint32).value == 12


@pytest.mark.parametrize("service_type", service_types)
def test_adding_key_struct_twice_fails(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    key = Foo(a=ctypes.c_uint8(9), b=ctypes.c_uint32(99), c=ctypes.c_double(9.9))
    value_0 = ctypes.c_int32(-3)
    value_1 = ctypes.c_uint32(3)

    with pytest.raises(iox2.BlackboardCreateError):
        (
            node.service_builder(service_name)
            .blackboard_creator(Foo)
            .add(key, value_0)
            .add(key, value_1)
            .create()
        )


@pytest.mark.parametrize("service_type", service_types)
def test_list_keys_with_key_struct_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    keys = [Foo(a=9, b=99, c=9.9), Foo(a=9, b=999, c=9.9)]
    value = ctypes.c_uint32(3)

    service = (
        node.service_builder(service_name)
        .blackboard_creator(Foo)
        .add(keys[0], value)
        .add(keys[1], value)
        .create()
    )

    listed_keys = service.list_keys()
    assert len(listed_keys) == len(keys)

    found_key = False
    for _i, key in enumerate(keys):
        for _j, listed_key in enumerate(listed_keys):
            listed_key_value = listed_key.decode_as(Foo)
            if listed_key_value == key:
                found_key = True
                break
        assert found_key
        found_key = False


@pytest.mark.parametrize("service_type", service_types)
def test_new_value_can_be_written_using_value_mut(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = ctypes.c_uint64(0)
    value = ctypes.c_uint16(0)

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .blackboard_creator(ctypes.c_uint64)
        .add(key, value)
        .create()
    )

    reader = service.reader_builder().create()
    entry_handle = reader.entry(key, ctypes.c_uint16)
    writer = service.writer_builder().create()
    entry_handle_mut = writer.entry(key, ctypes.c_uint16)
    entry_value_uninit = entry_handle_mut.loan_uninit()

    new_value_1 = ctypes.c_uint16(1234)
    ctypes.memmove(
        entry_value_uninit.value_mut(),
        ctypes.byref(new_value_1),
        ctypes.sizeof(ctypes.c_uint16),
    )
    entry_handle_mut = entry_value_uninit.assume_init_and_update()
    assert entry_handle.get().decode_as(ctypes.c_uint16).value == new_value_1.value

    entry_value_uninit = entry_handle_mut.loan_uninit()
    new_value_2 = ctypes.c_uint16(4321)
    ctypes.memmove(
        entry_value_uninit.value_mut(),
        ctypes.byref(new_value_2),
        ctypes.sizeof(ctypes.c_uint16),
    )
    # before calling assume_init_and_update(), the old value is read
    assert entry_handle.get().decode_as(ctypes.c_uint16).value == new_value_1.value
    entry_handle_mut = entry_value_uninit.discard()

    new_value_3 = ctypes.c_uint16(4567)
    entry_handle_mut.update_with_copy(new_value_3)
    assert entry_handle.get().decode_as(ctypes.c_uint16).value == new_value_3.value
