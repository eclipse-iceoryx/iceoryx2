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

# fmt: off
import iceoryx2 as iox2
import pytest
from flatbuffer_types.Helper import (create_schema_file, create_schema_file_at,
                                     create_unbounded_data,
                                     schema_incompatible, schema_unbounded)
from flatbuffer_types.UnboundedData import UnboundedData

# fmt: on

service_types = [iox2.ServiceType.Ipc, iox2.ServiceType.Local]


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
    schema_file_path = create_schema_file(schema_unbounded)
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
    schema_file_path = create_schema_file(schema_unbounded)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    try:
        _sut = (
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
    schema_file_path = create_schema_file(schema_unbounded)
    alt_schema_file_path = create_schema_file(schema_incompatible)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    try:
        _sut = (
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
    schema_file_path = create_schema_file(schema_unbounded)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    try:
        _sut = (
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


@pytest.mark.parametrize("service_type", service_types)
def test_schema_path_lookup_works_when_creating_a_service(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    config.global_cfg.service.flatbuffer_schema_path = iox2.testing.test_directory()

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    schema_file_path = create_schema_file_at(schema_unbounded, "UnboundedData.fbs")

    try:
        node.service_builder(service_name).publish_subscribe(
            iox2.Flatbuffer[UnboundedData]
        ).create()
    except iox2.PublishSubscribeCreateError:
        assert False

    os.remove(schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_schema_path_lookup_works_when_opening_a_service(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    config.global_cfg.service.flatbuffer_schema_path = iox2.testing.test_directory()

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    schema_file_path = create_schema_file_at(schema_unbounded, "UnboundedData.fbs")

    try:
        _sut = (
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
        ).open()
    except iox2.PublishSubscribeOpenError:
        assert False

    os.remove(schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_publish_subscribe_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    schema_file_path = create_schema_file(schema_unbounded)

    sut = (
        node.service_builder(service_name)
        .publish_subscribe(iox2.Flatbuffer[UnboundedData])
        .flatbuffer_schema_path(schema_file_path)
        .create()
    )

    publisher = sut.publisher_builder().initial_reserved_memory(4096).create()
    subscriber = sut.subscriber_builder().create()

    sample = publisher.loan_flatbuffer()
    builder = sample.flatbuffer_builder()
    unbounded_data = create_unbounded_data(builder, 123)
    sample = sample.assume_init(unbounded_data)
    sample.send()

    received = subscriber.receive()
    assert received is not None
    data = received.payload_root()
    assert data.Data() == 123

    os.remove(schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_publisher_allocates_more_memory_when_initial_reserve_is_out_with_allocation_strategy_power_of_two(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    schema_file_path = create_schema_file(schema_unbounded)

    sut = (
        node.service_builder(service_name)
        .publish_subscribe(iox2.Flatbuffer[UnboundedData])
        .flatbuffer_schema_path(schema_file_path)
        .create()
    )

    publisher = (
        sut.publisher_builder()
        .initial_reserved_memory(1)
        .allocation_strategy(iox2.AllocationStrategy.PowerOfTwo)
        .create()
    )
    subscriber = sut.subscriber_builder().create()

    sample = publisher.loan_flatbuffer()
    builder = sample.flatbuffer_builder()
    unbounded_data = create_unbounded_data(builder, 78)
    sample = sample.assume_init(unbounded_data)
    sample.send()

    received = subscriber.receive()
    assert received is not None
    data = received.payload_root()
    assert data.Data() == 78

    os.remove(schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_publisher_allocates_more_memory_when_initial_reserve_is_out_with_allocation_strategy_best_fit(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    schema_file_path = create_schema_file(schema_unbounded)

    sut = (
        node.service_builder(service_name)
        .publish_subscribe(iox2.Flatbuffer[UnboundedData])
        .flatbuffer_schema_path(schema_file_path)
        .create()
    )

    publisher = (
        sut.publisher_builder()
        .initial_reserved_memory(1)
        .allocation_strategy(iox2.AllocationStrategy.BestFit)
        .create()
    )
    subscriber = sut.subscriber_builder().create()

    sample = publisher.loan_flatbuffer()
    builder = sample.flatbuffer_builder()
    unbounded_data = create_unbounded_data(builder, 991)
    sample = sample.assume_init(unbounded_data)
    sample.send()

    received = subscriber.receive()
    assert received is not None
    data = received.payload_root()
    assert data.Data() == 991

    os.remove(schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_publisher_does_not_allocate_when_allocation_strategy_is_static(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    schema_file_path = create_schema_file(schema_unbounded)

    sut = (
        node.service_builder(service_name)
        .publish_subscribe(iox2.Flatbuffer[UnboundedData])
        .flatbuffer_schema_path(schema_file_path)
        .create()
    )

    publisher = (
        sut.publisher_builder()
        .initial_reserved_memory(1)
        .allocation_strategy(iox2.AllocationStrategy.Static)
        .create()
    )

    sample = publisher.loan_flatbuffer()
    builder = sample.flatbuffer_builder()
    unbounded_data = create_unbounded_data(builder, 991)

    with pytest.raises(iox2.AllocationGrowError):
        sample = sample.assume_init(unbounded_data)

    os.remove(schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_data_can_be_reconstructed_from_payload_bytes(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    schema_file_path = create_schema_file(schema_unbounded)

    sut = (
        node.service_builder(service_name)
        .publish_subscribe(iox2.Flatbuffer[UnboundedData])
        .flatbuffer_schema_path(schema_file_path)
        .create()
    )

    publisher = sut.publisher_builder().initial_reserved_memory(4096).create()
    subscriber = sut.subscriber_builder().create()

    sample = publisher.loan_flatbuffer()
    builder = sample.flatbuffer_builder()
    unbounded_data = create_unbounded_data(builder, 44)
    sample = sample.assume_init(unbounded_data)
    sample.send()

    received = subscriber.receive()
    assert received is not None
    data = UnboundedData.GetRootAs(received.payload_bytes().as_memory_view(), 0)
    assert data.Data() == 44

    os.remove(schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_publisher_can_read_its_own_serialized_data(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    schema_file_path = create_schema_file(schema_unbounded)

    sut = (
        node.service_builder(service_name)
        .publish_subscribe(iox2.Flatbuffer[UnboundedData])
        .flatbuffer_schema_path(schema_file_path)
        .create()
    )

    publisher = sut.publisher_builder().initial_reserved_memory(4096).create()

    sample = publisher.loan_flatbuffer()
    builder = sample.flatbuffer_builder()
    unbounded_data = create_unbounded_data(builder, 123)
    sample = sample.assume_init(unbounded_data)

    data = UnboundedData.GetRootAs(sample.payload_bytes().as_memory_view(), 0)
    assert data.Data() == 123

    os.remove(schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_publish_subscribe_with_user_header_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    schema_file_path = create_schema_file(schema_unbounded)

    sut = (
        node.service_builder(service_name)
        .publish_subscribe(iox2.Flatbuffer[UnboundedData])
        .user_header(ctypes.c_uint64)
        .flatbuffer_schema_path(schema_file_path)
        .create()
    )

    publisher = sut.publisher_builder().initial_reserved_memory(4096).create()
    subscriber = sut.subscriber_builder().create()

    sample = publisher.loan_flatbuffer()
    builder = sample.flatbuffer_builder()
    unbounded_data = create_unbounded_data(builder, 91912)
    sample = sample.assume_init(unbounded_data)
    sample.user_header().contents.value = 4456411
    sample.send()

    received = subscriber.receive()
    assert received is not None
    assert received.user_header().contents.value == 4456411

    data = received.payload_root()
    assert data.Data() == 91912

    os.remove(schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_builder_is_cleaned_up_when_sample_is_initialized(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    schema_file_path = create_schema_file(schema_unbounded)

    sut = (
        node.service_builder(service_name)
        .publish_subscribe(iox2.Flatbuffer[UnboundedData])
        .user_header(ctypes.c_uint64)
        .flatbuffer_schema_path(schema_file_path)
        .create()
    )

    publisher = (
        sut.publisher_builder()
        .initial_reserved_memory(1)
        .allocation_strategy(iox2.AllocationStrategy.PowerOfTwo)
        .create()
    )

    for _ in range(1000):
        sample = publisher.loan_flatbuffer()
        builder = sample.flatbuffer_builder()
        unbounded_data = create_unbounded_data(builder, 91912)
        sample = sample.assume_init(unbounded_data)
        assert len(builder.Bytes) <= 128

    os.remove(schema_file_path.to_string())
