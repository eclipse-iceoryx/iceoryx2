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
def test_creating_node_works(service_type: iox2.ServiceType) -> None:
    sut = iox2.NodeBuilder.new().create(service_type)
    assert sut.name.as_str() == ""


@pytest.mark.parametrize("service_type", service_types)
def test_creating_node_with_properties_works(
    service_type: iox2.ServiceType,
) -> None:
    node_name = iox2.testing.generate_node_name()
    signal_handling_mode = iox2.SignalHandlingMode.Disabled
    config = iox2.testing.generate_isolated_config()

    sut = (
        iox2.NodeBuilder.new()
        .name(node_name)
        .signal_handling_mode(signal_handling_mode)
        .config(config)
        .create(service_type)
    )
    assert sut.name == node_name
    assert sut.signal_handling_mode == signal_handling_mode
    assert sut.config == config


@pytest.mark.parametrize("service_type", service_types)
def test_cleanup_dead_nodes_can_be_called(
    service_type: iox2.ServiceType,
) -> None:
    try:
        iox2.Node.cleanup_dead_nodes(
            service_type, iox2.testing.generate_isolated_config()
        )
    except iox2.NodeCleanupFailure:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_created_nodes_can_be_listed(service_type: iox2.ServiceType) -> None:
    config = iox2.testing.generate_isolated_config()
    sut_1 = (
        iox2.NodeBuilder.new()
        .name(iox2.NodeName.new("behind you, there is"))
        .config(config)
        .create(service_type)
    )
    sut_2 = (
        iox2.NodeBuilder.new()
        .name(iox2.NodeName.new("a 3 headed monkey"))
        .config(config)
        .create(service_type)
    )

    node_list = iox2.Node.list(service_type, config)
    assert len(node_list) == 2
    for node in node_list:
        match node:
            case node.Alive():
                assert isinstance(node, iox2.NodeState.Alive)
                assert node[0].id in (sut_1.id, sut_2.id)
                assert node[0].details.name in (sut_1.name, sut_2.name)
            case node.Dead():
                assert False
            case node.Inaccessible():
                assert False
            case node.Undefined():
                assert False


@pytest.mark.parametrize("service_type", service_types)
def test_wait_can_be_called(service_type: iox2.ServiceType) -> None:
    config = iox2.testing.generate_isolated_config()
    sut = iox2.NodeBuilder.new().config(config).create(service_type)
    try:
        sut.wait(iox2.Duration.from_millis(1))
    except iox2.NodeWaitFailure:
        assert False
