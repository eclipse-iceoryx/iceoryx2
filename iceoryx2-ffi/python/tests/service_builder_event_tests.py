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

import iceoryx2_ffi_python as iox2
import pytest

service_types = [
    iox2.ServiceType.Ipc,
    iox2.ServiceType.Local
]

@pytest.mark.parametrize("service_type", service_types)
def test_non_existing_service_can_be_created(service_type) -> None:
    config = iox2.testing.generate_isolated_config()
    node = (
        iox2.NodeBuilder.new()
        .config(config)
        .create(service_type)
    )
    try:
        service_name = iox2.testing.generate_service_name()
        sut = (
            node.service_builder(service_name)
                .event()
                .create()
        )
        assert sut.name == service_name
    except iox2.EventCreateError:
        raise pytest.fail("DID RAISE EXCEPTION")

@pytest.mark.parametrize("service_type", service_types)
def test_existing_service_cannot_be_created(service_type) -> None:
    config = iox2.testing.generate_isolated_config()
    node = (
        iox2.NodeBuilder.new()
        .config(config)
        .create(service_type)
    )

    service_name = iox2.testing.generate_service_name()
    existing_service = (
        node.service_builder(service_name)
            .event()
            .create()
    )

    with pytest.raises(iox2.EventCreateError):
        sut = node.service_builder(service_name).event().create()

@pytest.mark.parametrize("service_type", service_types)
def test_existing_service_can_be_opened(service_type) -> None:
    config = iox2.testing.generate_isolated_config()
    node = (
        iox2.NodeBuilder.new()
        .config(config)
        .create(service_type)
    )

    service_name = iox2.testing.generate_service_name()
    existing_service = (
        node.service_builder(service_name)
            .event()
            .create()
    )
    try:
        sut = (
            node.service_builder(service_name)
                .event()
                .open()
        )
        assert sut.name == service_name
    except iox2.EventOpenError:
        raise pytest.fail("DID RAISE EXCEPTION")

@pytest.mark.parametrize("service_type", service_types)
def test_non_existing_service_cannot_be_opened(service_type) -> None:
    config = iox2.testing.generate_isolated_config()
    node = (
        iox2.NodeBuilder.new()
        .config(config)
        .create(service_type)
    )

    service_name = iox2.testing.generate_service_name()
    with pytest.raises(iox2.EventOpenError):
        node.service_builder(service_name).event().open()

@pytest.mark.parametrize("service_type", service_types)
def test_non_existing_service_is_created_with_open_or_create(service_type) -> None:
    config = iox2.testing.generate_isolated_config()
    node = (
        iox2.NodeBuilder.new()
        .config(config)
        .create(service_type)
    )

    service_name = iox2.testing.generate_service_name()
    try:
        sut = (
            node.service_builder(service_name)
                .event()
                .open_or_create()
        )
        assert sut.name == service_name
    except iox2.EventOpenOrCreateError:
        raise pytest.fail("DID RAISE EXCEPTION")

@pytest.mark.parametrize("service_type", service_types)
def test_existing_service_is_opened_with_open_or_create(service_type) -> None:
    config = iox2.testing.generate_isolated_config()
    node = (
        iox2.NodeBuilder.new()
        .config(config)
        .create(service_type)
    )

    service_name = iox2.testing.generate_service_name()
    existing_service = (
        node.service_builder(service_name)
            .event()
            .create()
    )

    try:
        sut = (
            node.service_builder(service_name)
                .event()
                .open_or_create()
        )
        assert sut.name == service_name
    except iox2.EventOpenOrCreateError:
        raise pytest.fail("DID RAISE EXCEPTION")

@pytest.mark.parametrize("service_type", service_types)
def test_create_service_with_attributes_work(service_type) -> None:
    config = iox2.testing.generate_isolated_config()
    node = (
        iox2.NodeBuilder.new()
        .config(config)
        .create(service_type)
    )
    attribute_spec = (
        iox2.AttributeSpecifier.new()
            .define(iox2.AttributeKey.new("fuu"), iox2.AttributeValue.new("bar"))
    )

    service_name = iox2.testing.generate_service_name()
    sut_create = (
        node.service_builder(service_name)
            .event()
            .create_with_attributes(attribute_spec)
    )

    sut_open = (
        node.service_builder(service_name)
            .event()
            .open()
    )

    assert sut_create.attributes == attribute_spec.attributes
    assert sut_create.attributes == sut_open.attributes

@pytest.mark.parametrize("service_type", service_types)
def test_open_or_create_service_with_attributes_work(service_type) -> None:
    config = iox2.testing.generate_isolated_config()
    node = (
        iox2.NodeBuilder.new()
        .config(config)
        .create(service_type)
    )

    attribute_spec = (
        iox2.AttributeSpecifier.new()
            .define(iox2.AttributeKey.new("what"), iox2.AttributeValue.new("ever"))
    )
    attribute_verifier = (
        iox2.AttributeVerifier.new()
            .require(iox2.AttributeKey.new("what"), iox2.AttributeValue.new("ever"))
    )

    service_name = iox2.testing.generate_service_name()
    sut_create = (
        node.service_builder(service_name)
            .event()
            .open_or_create_with_attributes(attribute_verifier)
    )

    sut_open = (
        node.service_builder(service_name)
            .event()
            .open()
    )

    assert sut_create.attributes == attribute_spec.attributes
    assert sut_create.attributes == sut_open.attributes

@pytest.mark.parametrize("service_type", service_types)
def test_open_service_with_attributes_work(service_type) -> None:
    config = iox2.testing.generate_isolated_config()
    node = (
        iox2.NodeBuilder.new()
        .config(config)
        .create(service_type)
    )

    attribute_spec = (
        iox2.AttributeSpecifier.new()
            .define(iox2.AttributeKey.new("knock"), iox2.AttributeValue.new("knock"))
    )
    attribute_verifier = (
        iox2.AttributeVerifier.new()
            .require(iox2.AttributeKey.new("knock"), iox2.AttributeValue.new("knock"))
    )

    service_name = iox2.testing.generate_service_name()
    sut_create = (
        node.service_builder(service_name)
            .event()
            .create_with_attributes(attribute_spec)
    )

    sut_open = (
        node.service_builder(service_name)
            .event()
            .open_with_attributes(attribute_verifier)
    )

    assert sut_create.attributes == attribute_spec.attributes
    assert sut_create.attributes == sut_open.attributes
