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


class Payload(ctypes.Structure):
    _fields_ = [("data", ctypes.c_ubyte)]


class HeaderPayload(ctypes.Structure):
    _fields_ = [("data", ctypes.c_ubyte), ("fuu", ctypes.c_int)]


@pytest.mark.parametrize("service_type", service_types)
def test_non_existing_service_can_be_created(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    try:
        service_name = iox2.testing.generate_service_name()
        sut = node.service_builder(service_name).publish_subscribe(Payload).create()
        assert sut.name == service_name
    except iox2.PublishSubscribeCreateError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_existing_service_cannot_be_created(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    _existing_service = (
        node.service_builder(service_name).publish_subscribe(Payload).create()
    )

    with pytest.raises(iox2.PublishSubscribeCreateError):
        node.service_builder(service_name).publish_subscribe(Payload).create()


@pytest.mark.parametrize("service_type", service_types)
def test_existing_service_can_be_opened(service_type: iox2.ServiceType) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    _existing_service = (
        node.service_builder(service_name).publish_subscribe(Payload).create()
    )
    try:
        sut = node.service_builder(service_name).publish_subscribe(Payload).open()
        assert sut.name == service_name
    except iox2.PublishSubscribeOpenError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_non_existing_service_cannot_be_opened(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    with pytest.raises(iox2.PublishSubscribeOpenError):
        node.service_builder(service_name).publish_subscribe(Payload).open()


@pytest.mark.parametrize("service_type", service_types)
def test_non_existing_service_is_created_with_open_or_create(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    try:
        sut = (
            node.service_builder(service_name)
            .publish_subscribe(Payload)
            .open_or_create()
        )
        assert sut.name == service_name
    except iox2.PublishSubscribeOpenOrCreateError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_existing_service_is_opened_with_open_or_create(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    _existing_service = (
        node.service_builder(service_name).publish_subscribe(Payload).create()
    )

    try:
        sut = (
            node.service_builder(service_name)
            .publish_subscribe(Payload)
            .open_or_create()
        )
        assert sut.name == service_name
    except iox2.PublishSubscribeOpenOrCreateError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_create_service_with_attributes_work(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    attribute_spec = iox2.AttributeSpecifier.new().define(
        iox2.AttributeKey.new("fuu"), iox2.AttributeValue.new("bar")
    )

    service_name = iox2.testing.generate_service_name()
    sut_create = (
        node.service_builder(service_name)
        .publish_subscribe(Payload)
        .create_with_attributes(attribute_spec)
    )

    sut_open = node.service_builder(service_name).publish_subscribe(Payload).open()

    assert sut_create.attributes == attribute_spec.attributes
    assert sut_create.attributes == sut_open.attributes


@pytest.mark.parametrize("service_type", service_types)
def test_open_or_create_service_with_attributes_work(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    attribute_spec = iox2.AttributeSpecifier.new().define(
        iox2.AttributeKey.new("what"), iox2.AttributeValue.new("ever")
    )
    attribute_verifier = iox2.AttributeVerifier.new().require(
        iox2.AttributeKey.new("what"), iox2.AttributeValue.new("ever")
    )

    service_name = iox2.testing.generate_service_name()
    sut_create = (
        node.service_builder(service_name)
        .publish_subscribe(Payload)
        .open_or_create_with_attributes(attribute_verifier)
    )

    sut_open = node.service_builder(service_name).publish_subscribe(Payload).open()

    assert sut_create.attributes == attribute_spec.attributes
    assert sut_create.attributes == sut_open.attributes


@pytest.mark.parametrize("service_type", service_types)
def test_open_service_with_attributes_work(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    attribute_spec = iox2.AttributeSpecifier.new().define(
        iox2.AttributeKey.new("knock"), iox2.AttributeValue.new("knock")
    )
    attribute_verifier = iox2.AttributeVerifier.new().require(
        iox2.AttributeKey.new("knock"), iox2.AttributeValue.new("knock")
    )

    service_name = iox2.testing.generate_service_name()
    sut_create = (
        node.service_builder(service_name)
        .publish_subscribe(Payload)
        .create_with_attributes(attribute_spec)
    )

    sut_open = (
        node.service_builder(service_name)
        .publish_subscribe(Payload)
        .open_with_attributes(attribute_verifier)
    )

    assert sut_create.attributes == attribute_spec.attributes
    assert sut_create.attributes == sut_open.attributes


@pytest.mark.parametrize("service_type", service_types)
def test_node_listing_works(service_type: iox2.ServiceType) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    sut = node.service_builder(service_name).publish_subscribe(Payload).create()

    nodes = sut.nodes

    assert len(nodes) == 1
    for n in nodes:
        match n:
            case n.Alive():
                assert n[0].id == node.id
            case node.Dead():
                assert False
            case node.Inaccessible():
                assert False
            case node.Undefined():
                assert False


@pytest.mark.parametrize("service_type", service_types)
def test_service_builder_configuration_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    safe_overflow = False
    subscriber_max_borrowed_samples = 10
    history_size = 29
    subscriber_max_buffer_size = 38
    max_subscribers = 47
    max_publishers = 56
    max_nodes = 65
    sut = (
        node.service_builder(service_name)
        .publish_subscribe(Payload)
        .enable_safe_overflow(safe_overflow)
        .subscriber_max_borrowed_samples(subscriber_max_borrowed_samples)
        .history_size(history_size)
        .subscriber_max_buffer_size(subscriber_max_buffer_size)
        .max_subscribers(max_subscribers)
        .max_publishers(56)
        .max_nodes(65)
        .create()
    )

    static_config = sut.static_config
    assert static_config.max_nodes == max_nodes
    assert static_config.max_publishers == max_publishers
    assert static_config.max_subscribers == max_subscribers
    assert static_config.history_size == history_size
    assert static_config.subscriber_max_buffer_size == subscriber_max_buffer_size
    assert (
        static_config.subscriber_max_borrowed_samples == subscriber_max_borrowed_samples
    )
    assert static_config.has_safe_overflow == safe_overflow


@pytest.mark.parametrize("service_type", service_types)
def test_service_builder_based_on_custom_config_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    max_nodes = 112
    config.defaults.publish_subscribe.max_nodes = max_nodes
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    sut = node.service_builder(service_name).publish_subscribe(Payload).create()

    static_config = sut.static_config
    assert static_config.max_nodes == max_nodes


@pytest.mark.parametrize("service_type", service_types)
def test_custom_user_header_works(service_type: iox2.ServiceType) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    user_header = (
        iox2.TypeDetail.new()
        .type_variant(iox2.TypeVariant.FixedSize)
        .type_name(iox2.TypeName.new("HeaderPayload"))
        .size(ctypes.sizeof(HeaderPayload))
        .alignment(ctypes.alignment(HeaderPayload))
    )
    sut = (
        node.service_builder(service_name)
        .publish_subscribe(Payload)
        .user_header(HeaderPayload)
        .create()
    )

    assert sut.static_config.message_type_details.user_header == user_header


@pytest.mark.parametrize("service_type", service_types)
def test_custom_payload_works(service_type: iox2.ServiceType) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    payload = (
        iox2.TypeDetail.new()
        .type_variant(iox2.TypeVariant.FixedSize)
        .type_name(iox2.TypeName.new("Payload"))
        .size(ctypes.sizeof(Payload))
        .alignment(ctypes.alignment(Payload))
    )
    sut = node.service_builder(service_name).publish_subscribe(Payload).create()

    assert sut.static_config.message_type_details.payload == payload
