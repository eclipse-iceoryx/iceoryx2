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
def test_non_existing_service_can_be_created(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    try:
        service_name = iox2.testing.generate_service_name()
        sut = (
            node.service_builder(service_name)
            .blackboard_creator(c_uint32)
            .add(c_uint32(0), c_uint32, c_uint32(0))
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
    _existing_service = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint32)
        .add(c_uint32(0), c_uint32, c_uint32(0))
        .create()
    )

    with pytest.raises(iox2.BlackboardCreateError):
        node.service_builder(service_name).blackboard_creator(c_uint32).add(
            c_uint32(0), c_uint32, c_uint32(0)
        ).create()


@pytest.mark.parametrize("service_type", service_types)
def test_node_listing_works(service_type: iox2.ServiceType) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    sut = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint32)
        .add(c_uint32(0), c_uint32, c_uint32(0))
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
    sut = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint32)
        .add(c_uint32(0), c_uint32, c_uint32(0))
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
    sut = (
        node.service_builder(service_name)
        .blackboard_creator(c_uint32)
        .add(c_uint32(0), c_uint32, c_uint32(0))
        .create()
    )

    static_config = sut.static_config
    assert static_config.max_nodes == max_nodes
