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
from flatbuffer_types.Helper import (create_schema_file, create_schema_file_at,
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
