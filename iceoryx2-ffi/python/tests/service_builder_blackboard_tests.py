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
def test_open_with_attributes_fails_when_key_types_differ(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    attribute_spec = iox2.AttributeSpecifier.new()
    attribute_verifier = iox2.AttributeVerifier.new().require(
        iox2.AttributeKey.new("knock"), iox2.AttributeValue.new("knock")
    )
    service_name = iox2.testing.generate_service_name()

    key = 0
    key = key.to_bytes(8, "little")
    sut = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint8, c_uint8(0))
        .create_with_attributes(attribute_spec)
    )
    with pytest.raises(iox2.BlackboardOpenError):
        node.service_builder(service_name).blackboard_opener(
            c_int64
        ).open_with_attributes(attribute_verifier)


@pytest.mark.parametrize("service_type", service_types)
def test_open_fails_when_key_types_differ(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    sut = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint8, c_uint8(0))
        .create()
    )
    with pytest.raises(iox2.BlackboardOpenError):
        node.service_builder(service_name).blackboard_opener(c_int64).open()


@pytest.mark.parametrize("service_type", service_types)
def test_non_existing_service_can_be_created(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    try:
        service_name = iox2.testing.generate_service_name()
        key = 0
        key = key.to_bytes(4, "little")
        sut = (
            node.service_builder(service_name)
            .blackboard_creator(c_uint32)
            .add(key, c_uint32, c_uint32(0))
            .create()
        )
        assert sut.name == service_name
    except iox2.BlackboardCreateError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_existing_service_cannot_be_created(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    key = 0
    key = key.to_bytes(4, "little")

    _existing_service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint32)
        .add(key, c_uint32, c_uint32(0))
        .create()
    )

    with pytest.raises(iox2.BlackboardCreateError):
        node.service_builder(service_name).blackboard_creator(c_uint32).add(
            key, c_uint32, c_uint32(0)
        ).create()


@pytest.mark.parametrize("service_type", service_types)
def test_create_fails_when_no_key_value_pairs_are_provided(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()

    with pytest.raises(iox2.BlackboardCreateError):
        node.service_builder(service_name).blackboard_creator(c_uint64).create()


@pytest.mark.parametrize("service_type", service_types)
def test_create_fails_when_key_is_provided_twice(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    key = 0
    key = key.to_bytes(8, "little")

    with pytest.raises(iox2.BlackboardCreateError):
        node.service_builder(service_name).blackboard_creator(c_uint64).add(
            key, c_uint8, c_uint8(0)
        ).add(key, c_uint8, c_uint8(0)).create()


@pytest.mark.parametrize("service_type", service_types)
def test_existing_service_can_be_opened(service_type: iox2.ServiceType) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    key = 0
    key = key.to_bytes(4, "little")
    _existing_service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint32)
        .add(key, c_uint32, c_uint32(0))
        .create()
    )
    try:
        sut = node.service_builder(service_name).blackboard_opener(c_uint32).open()
        assert sut.name == service_name
    except iox2.BlackboardOpenError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_service_can_be_created_with_different_keys(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    key_1 = 0
    key_2 = 1
    key_3 = 11
    key_1_bytes = key_1.to_bytes(4, "little")
    key_2_bytes = key_2.to_bytes(4, "little")
    key_3_bytes = key_3.to_bytes(4, "little")
    service_name = iox2.testing.generate_service_name()
    _existing_service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint32)
        .add(key_1_bytes, c_uint32, c_uint32(0))
        .add(key_2_bytes, c_uint32, c_uint32(0))
        .add(key_3_bytes, c_uint32, c_uint32(0))
        .create()
    )
    try:
        sut = node.service_builder(service_name).blackboard_opener(c_uint32).open()
        assert sut.name == service_name
    except iox2.BlackboardOpenError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_non_existing_service_cannot_be_opened(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    with pytest.raises(iox2.BlackboardOpenError):
        node.service_builder(service_name).blackboard_opener(c_uint32).open()


@pytest.mark.parametrize("service_type", service_types)
def test_create_service_with_attributes_work(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    attribute_spec = iox2.AttributeSpecifier.new().define(
        iox2.AttributeKey.new("fuu"), iox2.AttributeValue.new("bar")
    )

    key = 0
    key = key.to_bytes(4, "little")
    sut_create = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint32)
        .add(key, c_uint32, c_uint32(0))
        .create_with_attributes(attribute_spec)
    )

    sut_open = node.service_builder(service_name).blackboard_opener(c_uint32).open()

    assert sut_create.attributes == attribute_spec.attributes
    assert sut_create.attributes == sut_open.attributes


@pytest.mark.parametrize("service_type", service_types)
def test_open_service_with_attributes_work(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    attribute_spec = iox2.AttributeSpecifier.new().define(
        iox2.AttributeKey.new("knock"), iox2.AttributeValue.new("knock")
    )
    attribute_verifier = iox2.AttributeVerifier.new().require(
        iox2.AttributeKey.new("knock"), iox2.AttributeValue.new("knock")
    )

    key = 0
    key = key.to_bytes(4, "little")
    sut_create = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint32)
        .add(key, c_uint32, c_uint32(0))
        .create_with_attributes(attribute_spec)
    )

    sut_open = (
        node.service_builder(service_name)
        .blackboard_opener(c_uint32)
        .open_with_attributes(attribute_verifier)
    )

    assert sut_create.attributes == attribute_spec.attributes
    assert sut_create.attributes == sut_open.attributes


@pytest.mark.parametrize("service_type", service_types)
def test_open_fails_when_service_does_not_satisfy_max_nodes_requirement(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    key = 0
    key = key.to_bytes(8, "little")
    sut = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint8, c_uint8(0))
        .max_nodes(2)
        .create()
    )
    with pytest.raises(iox2.BlackboardOpenError):
        node.service_builder(service_name).blackboard_opener(c_uint64).max_nodes(
            3
        ).open()

    try:
        sut2 = (
            node.service_builder(service_name)
            .blackboard_opener(c_uint64)
            .max_nodes(1)
            .open()
        )
        assert sut2.name == service_name
    except iox2.BlackboardOpenError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_open_fails_when_service_does_not_satisfy_max_readers_requirement(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    key = 0
    key = key.to_bytes(8, "little")
    sut = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint8, c_uint8(0))
        .max_readers(2)
        .create()
    )
    with pytest.raises(iox2.BlackboardOpenError):
        node.service_builder(service_name).blackboard_opener(c_uint64).max_readers(
            3
        ).open()

    try:
        sut2 = (
            node.service_builder(service_name)
            .blackboard_opener(c_uint64)
            .max_readers(1)
            .open()
        )
        assert sut2.name == service_name
    except iox2.BlackboardOpenError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_node_listing_works(service_type: iox2.ServiceType) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    key = 0
    key = key.to_bytes(4, "little")
    sut = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint32)
        .add(key, c_uint32, c_uint32(0))
        .create()
    )

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

    max_readers = 56
    max_nodes = 65
    key = 0
    key = key.to_bytes(4, "little")
    sut = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint32)
        .add(key, c_uint32, c_uint32(0))
        .max_readers(56)
        .max_nodes(65)
        .create()
    )

    static_config = sut.static_config
    assert static_config.max_nodes == max_nodes
    assert static_config.max_readers == max_readers


@pytest.mark.parametrize("service_type", service_types)
def test_service_builder_based_on_custom_config_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    max_nodes = 112
    config.defaults.blackboard.max_nodes = max_nodes
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    key = 0
    key = key.to_bytes(4, "little")
    sut = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint32)
        .add(key, c_uint32, c_uint32(0))
        .create()
    )

    static_config = sut.static_config
    assert static_config.max_nodes == max_nodes


@pytest.mark.parametrize("service_type", service_types)
def test_max_readers_is_set_to_config_default(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    key = 0
    key = key.to_bytes(8, "little")
    sut = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint8, c_uint8(0))
        .create()
    )

    static_config = sut.static_config
    assert static_config.max_readers == config.defaults.blackboard.max_readers


@pytest.mark.parametrize("service_type", service_types)
def test_open_uses_predefined_settings_when_nothing_is_specified(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()

    key = 0
    key = key.to_bytes(8, "little")
    sut = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint8, c_uint8(0))
        .max_nodes(89)
        .max_readers(4)
        .create()
    )

    sut2 = node.service_builder(service_name).blackboard_opener(c_uint64).open()

    static_config = sut2.static_config
    assert static_config.max_nodes == 89
    assert static_config.max_readers == 4


@pytest.mark.parametrize("service_type", service_types)
def test_service_id_is_unique_per_service(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name_1 = iox2.testing.generate_service_name()
    service_name_2 = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")

    service_1_create = (
        node.service_builder(service_name_1)
        .blackboard_creator(c_uint64)
        .add(key, c_uint64, c_uint64(0))
        .create()
    )
    service_1_open = (
        node.service_builder(service_name_1).blackboard_opener(c_uint64).open()
    )
    service_2 = (
        node.service_builder(service_name_2)
        .blackboard_creator(c_uint64)
        .add(key, c_uint64, c_uint64(0))
        .create()
    )

    assert service_1_create.service_id.as_str == service_1_open.service_id.as_str
    assert service_1_create.service_id.as_str != service_2.service_id.as_str


@pytest.mark.parametrize("service_type", service_types)
def test_max_number_of_nodes_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    key = 0
    key = key.to_bytes(8, "little")
    max_nodes = 8
    nodes = []
    services = []

    node = iox2.NodeBuilder.new().config(config).create(service_type)
    creator = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint64)
        .add(key, c_uint64, c_uint64(0))
        .max_nodes(max_nodes)
        .create()
    )
    nodes.append(node)
    services.append(creator)

    i = 1
    while i < max_nodes:
        opener_node = iox2.NodeBuilder.new().config(config).create(service_type)
        nodes.append(opener_node)
        services.append(
            opener_node.service_builder(service_name).blackboard_opener(c_uint64).open()
        )
        i += 1

    assert len(services) == 8
    with pytest.raises(iox2.BlackboardOpenError):
        opener_node = iox2.NodeBuilder.new().config(config).create(service_type)
        services.append(
            opener_node.service_builder(service_name).blackboard_opener(c_uint64).open()
        )

    assert len(services) == 8


# TODO: "drop" tests
