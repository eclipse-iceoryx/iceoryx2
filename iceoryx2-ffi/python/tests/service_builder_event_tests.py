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

import iceoryx2 as iox2
import pytest

service_types = [iox2.ServiceType.Ipc, iox2.ServiceType.Local]


@pytest.mark.parametrize("service_type", service_types)
def test_non_existing_service_can_be_created(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    try:
        service_name = iox2.testing.generate_service_name()
        sut = node.service_builder(service_name).event().create()
        assert sut.name == service_name
    except iox2.EventCreateError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_existing_service_cannot_be_created(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    _existing_service = node.service_builder(service_name).event().create()

    with pytest.raises(iox2.EventCreateError):
        node.service_builder(service_name).event().create()


@pytest.mark.parametrize("service_type", service_types)
def test_existing_service_can_be_opened(service_type: iox2.ServiceType) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    _existing_service = node.service_builder(service_name).event().create()
    try:
        sut = node.service_builder(service_name).event().open()
        assert sut.name == service_name
    except iox2.EventOpenError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_non_existing_service_cannot_be_opened(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    with pytest.raises(iox2.EventOpenError):
        node.service_builder(service_name).event().open()


@pytest.mark.parametrize("service_type", service_types)
def test_non_existing_service_is_created_with_open_or_create(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    try:
        sut = node.service_builder(service_name).event().open_or_create()
        assert sut.name == service_name
    except iox2.EventOpenOrCreateError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_existing_service_is_opened_with_open_or_create(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    _existing_service = node.service_builder(service_name).event().create()

    try:
        sut = node.service_builder(service_name).event().open_or_create()
        assert sut.name == service_name
    except iox2.EventOpenOrCreateError:
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
        .event()
        .create_with_attributes(attribute_spec)
    )

    sut_open = node.service_builder(service_name).event().open()

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
        .event()
        .open_or_create_with_attributes(attribute_verifier)
    )

    sut_open = node.service_builder(service_name).event().open()

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


@pytest.mark.parametrize("service_type", service_types)
def test_node_listing_works(service_type: iox2.ServiceType) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    sut = node.service_builder(service_name).event().create()

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
    deadline = iox2.Duration.from_millis(123)
    max_nodes = 456
    event_id_max = 78
    max_notifiers = 9
    max_listeners = 10
    notifier_created = iox2.EventId.new(321)
    notifier_dead = iox2.EventId.new(654)
    notifier_dropped = iox2.EventId.new(987)
    sut = (
        node.service_builder(service_name)
        .event()
        .deadline(deadline)
        .max_nodes(max_nodes)
        .event_id_max_value(event_id_max)
        .max_notifiers(max_notifiers)
        .max_listeners(max_listeners)
        .notifier_created_event(notifier_created)
        .notifier_dropped_event(notifier_dropped)
        .notifier_dead_event(notifier_dead)
        .create()
    )

    static_config = sut.static_config
    assert static_config.deadline == deadline
    assert static_config.max_nodes == max_nodes
    assert static_config.max_listeners == max_listeners
    assert static_config.max_notifiers == max_notifiers
    assert static_config.event_id_max_value == event_id_max
    assert static_config.notifier_created_event == notifier_created
    assert static_config.notifier_dead_event == notifier_dead
    assert static_config.notifier_dropped_event == notifier_dropped


@pytest.mark.parametrize("service_type", service_types)
def test_service_builder_based_on_custom_config_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    max_nodes = 112
    config.defaults.event.max_nodes = max_nodes
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    sut = node.service_builder(service_name).event().create()

    static_config = sut.static_config
    assert static_config.max_nodes == max_nodes
