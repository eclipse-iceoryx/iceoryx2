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

import pytest

import iceoryx2 as iox2

service_types = [iox2.ServiceType.Ipc, iox2.ServiceType.Local]


@pytest.mark.parametrize("service_type", service_types)
def test_non_existing_service_can_be_created(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    try:
        service_name = iox2.testing.generate_service_name()
        sut = node.service_builder(service_name).request_response().create()
        assert sut.name == service_name
    except iox2.RequestResponseCreateError:
        raise pytest.fail("DID RAISE EXCEPTION")


@pytest.mark.parametrize("service_type", service_types)
def test_existing_service_cannot_be_created(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    existing_service = (
        node.service_builder(service_name).request_response().create()
    )

    with pytest.raises(iox2.RequestResponseCreateError):
        sut = node.service_builder(service_name).request_response().create()


@pytest.mark.parametrize("service_type", service_types)
def test_existing_service_can_be_opened(service_type: iox2.ServiceType) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    existing_service = (
        node.service_builder(service_name).request_response().create()
    )
    try:
        sut = node.service_builder(service_name).request_response().open()
        assert sut.name == service_name
    except iox2.RequestResponseOpenError:
        raise pytest.fail("DID RAISE EXCEPTION")


@pytest.mark.parametrize("service_type", service_types)
def test_non_existing_service_cannot_be_opened(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    with pytest.raises(iox2.RequestResponseOpenError):
        node.service_builder(service_name).request_response().open()


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
            .request_response()
            .open_or_create()
        )
        assert sut.name == service_name
    except iox2.RequestResponseOpenOrCreateError:
        raise pytest.fail("DID RAISE EXCEPTION")


@pytest.mark.parametrize("service_type", service_types)
def test_existing_service_is_opened_with_open_or_create(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    existing_service = (
        node.service_builder(service_name).request_response().create()
    )

    try:
        sut = (
            node.service_builder(service_name)
            .request_response()
            .open_or_create()
        )
        assert sut.name == service_name
    except iox2.RequestResponseOpenOrCreateError:
        raise pytest.fail("DID RAISE EXCEPTION")


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
        .request_response()
        .create_with_attributes(attribute_spec)
    )

    sut_open = node.service_builder(service_name).request_response().open()

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
        .request_response()
        .open_or_create_with_attributes(attribute_verifier)
    )

    sut_open = node.service_builder(service_name).request_response().open()

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
        .request_response()
        .create_with_attributes(attribute_spec)
    )

    sut_open = (
        node.service_builder(service_name)
        .request_response()
        .open_with_attributes(attribute_verifier)
    )

    assert sut_create.attributes == attribute_spec.attributes
    assert sut_create.attributes == sut_open.attributes


@pytest.mark.parametrize("service_type", service_types)
def test_node_listing_works(service_type: iox2.ServiceType) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    sut = node.service_builder(service_name).request_response().create()

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
    safe_overflow_requests = False
    safe_overflow_responses = False
    fire_and_forget = False
    max_active_requests_per_client = 99
    max_loaned_requests = 88
    max_response_buffer_size = 77
    max_servers = 66
    max_clients = 55
    max_nodes = 44
    max_borrowed_responses_per_pending_response = 33
    sut = (
        node.service_builder(service_name)
        .request_response()
        .enable_safe_overflow_for_requests(safe_overflow_requests)
        .enable_safe_overflow_for_responses(safe_overflow_responses)
        .enable_fire_and_forget_requests(fire_and_forget)
        .max_active_requests_per_client(max_active_requests_per_client)
        .max_loaned_requests(max_loaned_requests)
        .max_response_buffer_size(max_response_buffer_size)
        .max_servers(max_servers)
        .max_clients(max_clients)
        .max_nodes(max_nodes)
        .max_borrowed_responses_per_pending_response(
            max_borrowed_responses_per_pending_response
        )
        .create()
    )

    static_config = sut.static_config
    assert (
        static_config.has_safe_overflow_for_requests == safe_overflow_requests
    )
    assert (
        static_config.has_safe_overflow_for_responses == safe_overflow_responses
    )
    assert (
        static_config.does_support_fire_and_forget_requests == fire_and_forget
    )
    assert (
        static_config.max_borrowed_responses_per_pending_response
        == max_borrowed_responses_per_pending_response
    )
    assert (
        static_config.max_active_requests_per_client
        == max_active_requests_per_client
    )
    assert static_config.max_response_buffer_size == max_response_buffer_size
    assert static_config.max_loaned_requests == max_loaned_requests
    assert static_config.max_servers == max_servers
    assert static_config.max_clients == max_clients
    assert static_config.max_nodes == max_nodes


@pytest.mark.parametrize("service_type", service_types)
def test_custom_request_payload_works(service_type: iox2.ServiceType) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    payload = (
        iox2.TypeDetail.new()
        .type_variant(iox2.TypeVariant.Dynamic)
        .type_name(iox2.TypeName.new("froooogs"))
        .size(512)
        .alignment(128)
    )
    sut = (
        node.service_builder(service_name)
        .request_response()
        .request_payload_type_details(payload)
        .create()
    )

    assert sut.static_config.request_message_type_details.payload == payload


@pytest.mark.parametrize("service_type", service_types)
def test_custom_request_header_works(service_type: iox2.ServiceType) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    user_header = (
        iox2.TypeDetail.new()
        .type_variant(iox2.TypeVariant.FixedSize)
        .type_name(iox2.TypeName.new("touch the stone, but gently"))
        .size(8192)
        .alignment(4096)
    )
    sut = (
        node.service_builder(service_name)
        .request_response()
        .request_header_type_details(user_header)
        .create()
    )

    assert (
        sut.static_config.request_message_type_details.user_header
        == user_header
    )


@pytest.mark.parametrize("service_type", service_types)
def test_custom_response_payload_works(service_type: iox2.ServiceType) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    payload = (
        iox2.TypeDetail.new()
        .type_variant(iox2.TypeVariant.Dynamic)
        .type_name(iox2.TypeName.new("you crinkled the candle"))
        .size(8)
        .alignment(8)
    )
    sut = (
        node.service_builder(service_name)
        .request_response()
        .response_payload_type_details(payload)
        .create()
    )

    assert sut.static_config.response_message_type_details.payload == payload


@pytest.mark.parametrize("service_type", service_types)
def test_custom_response_header_works(service_type: iox2.ServiceType) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    user_header = (
        iox2.TypeDetail.new()
        .type_variant(iox2.TypeVariant.FixedSize)
        .type_name(iox2.TypeName.new("now the candle bends you"))
        .size(16)
        .alignment(1)
    )
    sut = (
        node.service_builder(service_name)
        .request_response()
        .response_header_type_details(user_header)
        .create()
    )

    assert (
        sut.static_config.response_message_type_details.user_header
        == user_header
    )
