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

import iceoryx2_ffi_python as iceoryx2
import pytest


def test_creating_node_works() -> None:
    sut = iceoryx2.NodeBuilder.new().create(iceoryx2.ServiceType.Ipc)
    assert sut.name.as_str() == ""

def test_creating_node_with_properties_works() -> None:
    node_name = iceoryx2.NodeName.new("all glory to the hypnotoad")
    signal_handling_mode = iceoryx2.SignalHandlingMode.Disabled
    config = iceoryx2.config.default()
    config.global_cfg.prefix = iceoryx2.FileName.new("dont touch my spider")

    sut = iceoryx2.NodeBuilder.new().name(node_name).signal_handling_mode(signal_handling_mode).config(config).create(iceoryx2.ServiceType.Ipc)
    assert sut.name == node_name
    assert sut.signal_handling_mode == signal_handling_mode
    assert sut.config == config

def test_cleanup_dead_nodes_can_be_called() -> None:
    try:
        iceoryx2.Node.cleanup_dead_nodes(iceoryx2.ServiceType.Local, iceoryx2.config.default())
    except exception:
        raise pytest.fail("DID RAISE {0}".format(exception))

def test_created_nodes_can_be_listed() -> None:
    sut_1 = iceoryx2.NodeBuilder.new().name(iceoryx2.NodeName.new("behind you, there is")).create(iceoryx2.ServiceType.Ipc)
    sut_2 = iceoryx2.NodeBuilder.new().name(iceoryx2.NodeName.new("a 3 headed monkey")).create(iceoryx2.ServiceType.Ipc)

    node_list = iceoryx2.Node.list(iceoryx2.ServiceType.Ipc, iceoryx2.config.default())
    assert len(node_list) == 2
    for node in node_list:
        match node:
            case node.Alive():
                assert isinstance(node, iceoryx2.NodeState.Alive)
                assert (node[0].id == sut_1.id) or (node[0].id == sut_2.id)
                assert (node[0].details.name == sut_1.name) or (node[0].details.name == sut_2.name)
                assert True
            case node.Dead():
                assert False
            case node.Inaccessible():
                assert False
            case node.Undefined():
                assert False

def test_wait_can_be_called() -> None:
    sut = iceoryx2.NodeBuilder.new().create(iceoryx2.ServiceType.Ipc)
    try:
        sut.wait(iceoryx2.Duration.from_millis(1))
    except exception:
        raise pytest.fail("DID RAISE {0}".format(exception))
