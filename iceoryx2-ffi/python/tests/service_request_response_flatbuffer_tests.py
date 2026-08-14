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
from flatbuffer_types.BoundedData import BoundedData
from flatbuffer_types.Helper import (create_bounded_data, create_schema_file,
                                     create_schema_file_at,
                                     create_unbounded_data, schema_bounded,
                                     schema_incompatible, schema_unbounded)
from flatbuffer_types.UnboundedData import UnboundedData

# fmt: on

service_types = [iox2.ServiceType.Ipc, iox2.ServiceType.Local]


@pytest.mark.parametrize("service_type", service_types)
def test_create_fails_when_no_request_schema_file_is_available(
    service_type: iox2.ServiceType,
) -> None:
    response_schema_file_path = create_schema_file(schema_unbounded)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    with pytest.raises(iox2.RequestResponseCreateError) as exc_info:
        node.service_builder(service_name).request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        ).response_flatbuffer_schema_path(response_schema_file_path).create()

    assert str(exc_info.value) == "UnableToAcquireTypeDefinition"

    os.remove(response_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_create_fails_when_no_response_schema_file_is_available(
    service_type: iox2.ServiceType,
) -> None:
    request_schema_file_path = create_schema_file(schema_bounded)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    with pytest.raises(iox2.RequestResponseCreateError) as exc_info:
        node.service_builder(service_name).request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        ).request_flatbuffer_schema_path(request_schema_file_path).create()

    assert str(exc_info.value) == "UnableToAcquireTypeDefinition"

    os.remove(request_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_create_succeeds_with_schema_file(
    service_type: iox2.ServiceType,
) -> None:
    request_schema_file_path = create_schema_file(schema_bounded)
    response_schema_file_path = create_schema_file(schema_unbounded)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    try:
        node.service_builder(service_name).request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        ).request_flatbuffer_schema_path(
            request_schema_file_path
        ).response_flatbuffer_schema_path(
            response_schema_file_path
        ).create()
    except iox2.RequestResponseCreateError:
        assert False

    os.remove(request_schema_file_path.to_string())
    os.remove(response_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_open_fails_when_no_request_schema_file_is_available(
    service_type: iox2.ServiceType,
) -> None:
    request_schema_file_path = create_schema_file(schema_bounded)
    response_schema_file_path = create_schema_file(schema_unbounded)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    try:
        _sut = (
            node.service_builder(service_name)
            .request_response(
                iox2.Flatbuffer[BoundedData],
                iox2.Flatbuffer[UnboundedData],
            )
            .request_flatbuffer_schema_path(request_schema_file_path)
            .response_flatbuffer_schema_path(response_schema_file_path)
            .create()
        )
    except iox2.RequestResponseCreateError:
        assert False

    with pytest.raises(iox2.RequestResponseOpenError) as exc_info:
        node.service_builder(service_name).request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        ).response_flatbuffer_schema_path(response_schema_file_path).open()

    assert str(exc_info.value) == "UnableToAcquireTypeDefinition"

    os.remove(request_schema_file_path.to_string())
    os.remove(response_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_open_fails_when_no_response_schema_file_is_available(
    service_type: iox2.ServiceType,
) -> None:
    request_schema_file_path = create_schema_file(schema_bounded)
    response_schema_file_path = create_schema_file(schema_unbounded)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    try:
        _sut = (
            node.service_builder(service_name)
            .request_response(
                iox2.Flatbuffer[BoundedData],
                iox2.Flatbuffer[UnboundedData],
            )
            .request_flatbuffer_schema_path(request_schema_file_path)
            .response_flatbuffer_schema_path(response_schema_file_path)
            .create()
        )
    except iox2.RequestResponseCreateError:
        assert False

    with pytest.raises(iox2.RequestResponseOpenError) as exc_info:
        node.service_builder(service_name).request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        ).request_flatbuffer_schema_path(request_schema_file_path).open()

    assert str(exc_info.value) == "UnableToAcquireTypeDefinition"

    os.remove(request_schema_file_path.to_string())
    os.remove(response_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_open_fails_when_request_schema_is_not_the_same(
    service_type: iox2.ServiceType,
) -> None:
    request_schema_file_path = create_schema_file(schema_bounded)
    response_schema_file_path = create_schema_file(schema_unbounded)
    incompatible_schema_file_path = create_schema_file(schema_incompatible)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    try:
        _sut = (
            node.service_builder(service_name)
            .request_response(
                iox2.Flatbuffer[BoundedData],
                iox2.Flatbuffer[UnboundedData],
            )
            .request_flatbuffer_schema_path(request_schema_file_path)
            .response_flatbuffer_schema_path(response_schema_file_path)
            .create()
        )
    except iox2.RequestResponseCreateError:
        assert False

    # request scheme incompatible
    with pytest.raises(iox2.RequestResponseOpenError) as exc_info:
        node.service_builder(service_name).request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        ).request_flatbuffer_schema_path(
            incompatible_schema_file_path
        ).response_flatbuffer_schema_path(
            response_schema_file_path
        ).open()

    assert str(exc_info.value) == "IncompatibleRequestOrResponseType"

    os.remove(request_schema_file_path.to_string())
    os.remove(response_schema_file_path.to_string())
    os.remove(incompatible_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_open_fails_when_response_schema_is_not_the_same(
    service_type: iox2.ServiceType,
) -> None:
    request_schema_file_path = create_schema_file(schema_bounded)
    response_schema_file_path = create_schema_file(schema_unbounded)
    incompatible_schema_file_path = create_schema_file(schema_incompatible)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    try:
        _sut = (
            node.service_builder(service_name)
            .request_response(
                iox2.Flatbuffer[BoundedData],
                iox2.Flatbuffer[UnboundedData],
            )
            .request_flatbuffer_schema_path(request_schema_file_path)
            .response_flatbuffer_schema_path(response_schema_file_path)
            .create()
        )
    except iox2.RequestResponseCreateError:
        assert False

    # request scheme incompatible
    with pytest.raises(iox2.RequestResponseOpenError) as exc_info:
        node.service_builder(service_name).request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        ).request_flatbuffer_schema_path(
            request_schema_file_path
        ).response_flatbuffer_schema_path(
            incompatible_schema_file_path
        ).open()

    assert str(exc_info.value) == "IncompatibleRequestOrResponseType"

    os.remove(request_schema_file_path.to_string())
    os.remove(response_schema_file_path.to_string())
    os.remove(incompatible_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_open_succeeds_when_schema_content_is_identical(
    service_type: iox2.ServiceType,
) -> None:
    request_schema_file_path = create_schema_file(schema_bounded)
    response_schema_file_path = create_schema_file(schema_unbounded)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    try:
        _sut = (
            node.service_builder(service_name)
            .request_response(
                iox2.Flatbuffer[BoundedData],
                iox2.Flatbuffer[UnboundedData],
            )
            .request_flatbuffer_schema_path(request_schema_file_path)
            .response_flatbuffer_schema_path(response_schema_file_path)
            .create()
        )
    except iox2.RequestResponseCreateError:
        assert False

    try:
        node.service_builder(service_name).request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        ).request_flatbuffer_schema_path(
            request_schema_file_path
        ).response_flatbuffer_schema_path(
            response_schema_file_path
        ).open()
    except iox2.RequestResponseOpenError:
        assert False

    os.remove(request_schema_file_path.to_string())
    os.remove(response_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_schema_path_lookup_works_when_creating_a_service(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    config.global_cfg.service.flatbuffer_schema_path = iox2.testing.test_directory()

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    request_schema_file_path = create_schema_file_at(schema_bounded, "BoundedData.fbs")
    response_schema_file_path = create_schema_file_at(
        schema_unbounded, "UnboundedData.fbs"
    )

    try:
        node.service_builder(service_name).request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        ).create()
    except iox2.PublishSubscribeCreateError:
        assert False

    os.remove(request_schema_file_path.to_string())
    os.remove(response_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_schema_path_lookup_works_when_opening_a_service(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    config.global_cfg.service.flatbuffer_schema_path = iox2.testing.test_directory()

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    request_schema_file_path = create_schema_file_at(schema_bounded, "BoundedData.fbs")
    response_schema_file_path = create_schema_file_at(
        schema_unbounded, "UnboundedData.fbs"
    )

    try:
        _sut = (
            node.service_builder(service_name)
            .request_response(
                iox2.Flatbuffer[BoundedData],
                iox2.Flatbuffer[UnboundedData],
            )
            .create()
        )
    except iox2.PublishSubscribeCreateError:
        assert False

    try:
        node.service_builder(service_name).request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        ).open()
    except iox2.PublishSubscribeOpenError:
        assert False

    os.remove(request_schema_file_path.to_string())
    os.remove(response_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_request_response_works(
    service_type: iox2.ServiceType,
) -> None:
    request_schema_file_path = create_schema_file(schema_bounded)
    response_schema_file_path = create_schema_file(schema_unbounded)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    sut = (
        node.service_builder(service_name)
        .request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        )
        .request_flatbuffer_schema_path(request_schema_file_path)
        .response_flatbuffer_schema_path(response_schema_file_path)
        .create()
    )

    server = sut.server_builder().initial_reserved_memory(4096).create()
    client = sut.client_builder().initial_reserved_memory(4096).create()

    request = client.loan_flatbuffer()
    builder = request.flatbuffer_builder()
    request_data = create_bounded_data(builder, 123)
    request = request.assume_init(request_data)
    pending_response = request.send()

    active_request = server.receive()
    assert active_request is not None
    data = active_request.payload_root()
    assert data.Data() == 123

    response = active_request.loan_flatbuffer()
    builder = response.flatbuffer_builder()
    response_data = create_unbounded_data(builder, 321)
    response = response.assume_init(response_data)
    response.send()

    response_received = pending_response.receive()
    assert response_received is not None
    data = response_received.payload_root()
    assert data.Data() == 321

    os.remove(request_schema_file_path.to_string())
    os.remove(response_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_client_and_server_allocate_more_memory_when_initial_reserve_is_out_with_allocation_strategy_power_of_two(
    service_type: iox2.ServiceType,
) -> None:
    request_schema_file_path = create_schema_file(schema_bounded)
    response_schema_file_path = create_schema_file(schema_unbounded)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    sut = (
        node.service_builder(service_name)
        .request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        )
        .request_flatbuffer_schema_path(request_schema_file_path)
        .response_flatbuffer_schema_path(response_schema_file_path)
        .create()
    )

    server = (
        sut.server_builder()
        .initial_reserved_memory(1)
        .allocation_strategy(iox2.AllocationStrategy.PowerOfTwo)
        .create()
    )
    client = (
        sut.client_builder()
        .initial_reserved_memory(1)
        .allocation_strategy(iox2.AllocationStrategy.PowerOfTwo)
        .create()
    )

    request = client.loan_flatbuffer()
    builder = request.flatbuffer_builder()
    request_data = create_bounded_data(builder, 78)
    request = request.assume_init(request_data)
    pending_response = request.send()

    active_request = server.receive()
    assert active_request is not None
    data = active_request.payload_root()
    assert data.Data() == 78

    response = active_request.loan_flatbuffer()
    builder = response.flatbuffer_builder()
    response_data = create_unbounded_data(builder, 45)
    response = response.assume_init(response_data)
    response.send()

    response_received = pending_response.receive()
    assert response_received is not None
    data = response_received.payload_root()
    assert data.Data() == 45

    os.remove(request_schema_file_path.to_string())
    os.remove(response_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_client_and_server_allocate_more_memory_when_initial_reserve_is_out_with_allocation_strategy_best_fit(
    service_type: iox2.ServiceType,
) -> None:
    request_schema_file_path = create_schema_file(schema_bounded)
    response_schema_file_path = create_schema_file(schema_unbounded)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    sut = (
        node.service_builder(service_name)
        .request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        )
        .request_flatbuffer_schema_path(request_schema_file_path)
        .response_flatbuffer_schema_path(response_schema_file_path)
        .create()
    )

    server = (
        sut.server_builder()
        .initial_reserved_memory(1)
        .allocation_strategy(iox2.AllocationStrategy.BestFit)
        .create()
    )
    client = (
        sut.client_builder()
        .initial_reserved_memory(1)
        .allocation_strategy(iox2.AllocationStrategy.BestFit)
        .create()
    )

    request = client.loan_flatbuffer()
    builder = request.flatbuffer_builder()
    request_data = create_bounded_data(builder, 991)
    request = request.assume_init(request_data)
    pending_response = request.send()

    active_request = server.receive()
    assert active_request is not None
    data = active_request.payload_root()
    assert data.Data() == 991

    response = active_request.loan_flatbuffer()
    builder = response.flatbuffer_builder()
    response_data = create_unbounded_data(builder, 119)
    response = response.assume_init(response_data)
    response.send()

    response_received = pending_response.receive()
    assert response_received is not None
    data = response_received.payload_root()
    assert data.Data() == 119

    os.remove(request_schema_file_path.to_string())
    os.remove(response_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_server_does_not_allocate_when_allocation_strategy_is_static(
    service_type: iox2.ServiceType,
) -> None:
    request_schema_file_path = create_schema_file(schema_bounded)
    response_schema_file_path = create_schema_file(schema_unbounded)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    sut = (
        node.service_builder(service_name)
        .request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        )
        .request_flatbuffer_schema_path(request_schema_file_path)
        .response_flatbuffer_schema_path(response_schema_file_path)
        .create()
    )

    server = (
        sut.server_builder()
        .initial_reserved_memory(1)
        .allocation_strategy(iox2.AllocationStrategy.Static)
        .create()
    )
    client = sut.client_builder().initial_reserved_memory(4096).create()

    request = client.loan_flatbuffer()
    builder = request.flatbuffer_builder()
    request_data = create_bounded_data(builder, 151)
    request = request.assume_init(request_data)
    _pending_response = request.send()

    active_request = server.receive()
    assert active_request is not None
    data = active_request.payload_root()
    assert data.Data() == 151

    response = active_request.loan_flatbuffer()
    builder = response.flatbuffer_builder()
    response_data = create_unbounded_data(builder, 515)

    with pytest.raises(iox2.AllocationGrowError):
        response = response.assume_init(response_data)

    os.remove(request_schema_file_path.to_string())
    os.remove(response_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_client_does_not_allocate_when_allocation_strategy_is_static(
    service_type: iox2.ServiceType,
) -> None:
    request_schema_file_path = create_schema_file(schema_bounded)
    response_schema_file_path = create_schema_file(schema_unbounded)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    sut = (
        node.service_builder(service_name)
        .request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        )
        .request_flatbuffer_schema_path(request_schema_file_path)
        .response_flatbuffer_schema_path(response_schema_file_path)
        .create()
    )

    client = (
        sut.client_builder()
        .initial_reserved_memory(1)
        .allocation_strategy(iox2.AllocationStrategy.Static)
        .create()
    )

    request = client.loan_flatbuffer()
    builder = request.flatbuffer_builder()
    request_data = create_bounded_data(builder, 666)

    with pytest.raises(iox2.AllocationGrowError):
        request = request.assume_init(request_data)

    os.remove(request_schema_file_path.to_string())
    os.remove(response_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_data_can_be_reconstructed_from_payload_bytes(
    service_type: iox2.ServiceType,
) -> None:
    request_schema_file_path = create_schema_file(schema_bounded)
    response_schema_file_path = create_schema_file(schema_unbounded)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    sut = (
        node.service_builder(service_name)
        .request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        )
        .request_flatbuffer_schema_path(request_schema_file_path)
        .response_flatbuffer_schema_path(response_schema_file_path)
        .create()
    )

    server = sut.server_builder().initial_reserved_memory(4096).create()
    client = sut.client_builder().initial_reserved_memory(4096).create()

    request = client.loan_flatbuffer()
    builder = request.flatbuffer_builder()
    request_data = create_bounded_data(builder, 951)
    request = request.assume_init(request_data)
    pending_response = request.send()

    active_request = server.receive()
    assert active_request is not None
    data = BoundedData.GetRootAs(active_request.payload_bytes().as_memory_view(), 0)
    assert data.Data() == 951

    response = active_request.loan_flatbuffer()
    builder = response.flatbuffer_builder()
    response_data = create_unbounded_data(builder, 159)
    response = response.assume_init(response_data)
    response.send()

    response_received = pending_response.receive()
    assert response_received is not None
    data = UnboundedData.GetRootAs(
        response_received.payload_bytes().as_memory_view(), 0
    )
    assert data.Data() == 159

    os.remove(request_schema_file_path.to_string())
    os.remove(response_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_client_and_server_can_read_their_own_serialized_data(
    service_type: iox2.ServiceType,
) -> None:
    request_schema_file_path = create_schema_file(schema_bounded)
    response_schema_file_path = create_schema_file(schema_unbounded)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    sut = (
        node.service_builder(service_name)
        .request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        )
        .request_flatbuffer_schema_path(request_schema_file_path)
        .response_flatbuffer_schema_path(response_schema_file_path)
        .create()
    )

    server = sut.server_builder().initial_reserved_memory(4096).create()
    client = sut.client_builder().initial_reserved_memory(4096).create()

    request = client.loan_flatbuffer()
    builder = request.flatbuffer_builder()
    request_data = create_bounded_data(builder, 951)
    request = request.assume_init(request_data)

    data = BoundedData.GetRootAs(request.payload_bytes().as_memory_view(), 0)
    assert data.Data() == 951

    pending_response = request.send()

    data = BoundedData.GetRootAs(pending_response.payload_bytes().as_memory_view(), 0)
    assert data.Data() == 951

    active_request = server.receive()

    response = active_request.loan_flatbuffer()
    builder = response.flatbuffer_builder()
    response_data = create_unbounded_data(builder, 159)
    response = response.assume_init(response_data)

    data = UnboundedData.GetRootAs(response.payload_bytes().as_memory_view(), 0)
    assert data.Data() == 159

    os.remove(request_schema_file_path.to_string())
    os.remove(response_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_request_response_with_user_header_works(
    service_type: iox2.ServiceType,
) -> None:
    request_schema_file_path = create_schema_file(schema_bounded)
    response_schema_file_path = create_schema_file(schema_unbounded)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    sut = (
        node.service_builder(service_name)
        .request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        )
        .request_flatbuffer_schema_path(request_schema_file_path)
        .response_flatbuffer_schema_path(response_schema_file_path)
        .request_header(ctypes.c_uint64)
        .response_header(ctypes.c_uint64)
        .create()
    )

    server = sut.server_builder().initial_reserved_memory(4096).create()
    client = sut.client_builder().initial_reserved_memory(4096).create()

    request = client.loan_flatbuffer()
    builder = request.flatbuffer_builder()
    request_data = create_bounded_data(builder, 32123)
    request = request.assume_init(request_data)
    request.user_header().contents.value = 666
    pending_response = request.send()

    active_request = server.receive()
    assert active_request is not None
    assert active_request.user_header().contents.value == 666
    data = active_request.payload_root()
    assert data.Data() == 32123

    response = active_request.loan_flatbuffer()
    builder = response.flatbuffer_builder()
    response_data = create_unbounded_data(builder, 12321)
    response = response.assume_init(response_data)
    response.user_header().contents.value = 555
    response.send()

    response_received = pending_response.receive()
    assert response_received is not None
    assert response_received.user_header().contents.value == 555
    data = response_received.payload_root()
    assert data.Data() == 12321

    os.remove(request_schema_file_path.to_string())
    os.remove(response_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_builder_is_cleaned_up_when_request_is_initialized(
    service_type: iox2.ServiceType,
) -> None:
    request_schema_file_path = create_schema_file(schema_bounded)
    response_schema_file_path = create_schema_file(schema_unbounded)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    sut = (
        node.service_builder(service_name)
        .request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        )
        .request_flatbuffer_schema_path(request_schema_file_path)
        .response_flatbuffer_schema_path(response_schema_file_path)
        .create()
    )

    client = (
        sut.client_builder()
        .initial_reserved_memory(1)
        .allocation_strategy(iox2.AllocationStrategy.PowerOfTwo)
        .create()
    )

    for _ in range(1000):
        request = client.loan_flatbuffer()
        builder = request.flatbuffer_builder()
        request_data = create_bounded_data(builder, 31213)
        request = request.assume_init(request_data)
        assert len(builder.Bytes) <= 128

    os.remove(request_schema_file_path.to_string())
    os.remove(response_schema_file_path.to_string())


@pytest.mark.parametrize("service_type", service_types)
def test_builder_is_cleaned_up_when_response_is_initialized(
    service_type: iox2.ServiceType,
) -> None:
    request_schema_file_path = create_schema_file(schema_bounded)
    response_schema_file_path = create_schema_file(schema_unbounded)
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    sut = (
        node.service_builder(service_name)
        .request_response(
            iox2.Flatbuffer[BoundedData],
            iox2.Flatbuffer[UnboundedData],
        )
        .request_flatbuffer_schema_path(request_schema_file_path)
        .response_flatbuffer_schema_path(response_schema_file_path)
        .create()
    )

    server = (
        sut.server_builder()
        .initial_reserved_memory(1)
        .allocation_strategy(iox2.AllocationStrategy.PowerOfTwo)
        .create()
    )
    client = (
        sut.client_builder()
        .initial_reserved_memory(1)
        .allocation_strategy(iox2.AllocationStrategy.PowerOfTwo)
        .create()
    )

    request = client.loan_flatbuffer()
    builder = request.flatbuffer_builder()
    request_data = create_bounded_data(builder, 31213)
    request = request.assume_init(request_data)
    _pending_response = request.send()

    active_request = server.receive()
    assert active_request is not None

    for _ in range(1000):
        response = active_request.loan_flatbuffer()
        builder = response.flatbuffer_builder()
        response_data = create_unbounded_data(builder, 21312)
        response = response.assume_init(response_data)
        assert len(builder.Bytes) <= 128

    os.remove(request_schema_file_path.to_string())
    os.remove(response_schema_file_path.to_string())
