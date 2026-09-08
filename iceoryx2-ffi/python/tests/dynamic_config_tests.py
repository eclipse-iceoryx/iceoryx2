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
from typing import Any, Tuple

import iceoryx2 as iox2
import pytest

service_patterns = [
    ("publish_subscribe", "publisher", "subscriber"),
    ("event", "notifier", "listener"),
    ("request_response", "client", "server"),
    ("blackboard", "writer", "reader"),
]


def create_service(node: iox2.Node, pattern: str) -> Any:
    builder = node.service_builder(iox2.testing.generate_service_name())
    if pattern == "publish_subscribe":
        return builder.publish_subscribe(ctypes.c_uint64).create()
    if pattern == "event":
        return builder.event().create()
    if pattern == "request_response":
        return builder.request_response(ctypes.c_uint64, ctypes.c_uint64).create()
    return (
        builder.blackboard_creator(ctypes.c_uint64)
        .add(ctypes.c_uint64(0), ctypes.c_uint64(0))
        .create()
    )


@pytest.fixture(params=[iox2.ServiceType.Ipc, iox2.ServiceType.Local])
def node(request: pytest.FixtureRequest) -> iox2.Node:
    config = iox2.testing.generate_isolated_config()
    return iox2.NodeBuilder.new().config(config).create(request.param)


@pytest.fixture(params=service_patterns)
def service_and_ports(
    request: pytest.FixtureRequest, node: iox2.Node
) -> Tuple[Any, str, str]:
    pattern, first_kind, second_kind = request.param
    return create_service(node, pattern), first_kind, second_kind


def test_dynamic_config_initially_has_no_ports(
    service_and_ports: Tuple[Any, str, str],
) -> None:
    service, first_kind, second_kind = service_and_ports

    sut = service.dynamic_config

    assert getattr(sut, f"number_of_{first_kind}s") == 0
    assert getattr(sut, f"number_of_{second_kind}s") == 0


@pytest.mark.parametrize("selected", [0, 1])
def test_dynamic_config_counts_new_port(
    service_and_ports: Tuple[Any, str, str], selected: int
) -> None:
    service, *kinds = service_and_ports
    sut = service.dynamic_config

    port = getattr(service, f"{kinds[selected]}_builder")().create()

    assert getattr(sut, f"number_of_{kinds[selected]}s") == 1
    assert getattr(sut, f"number_of_{kinds[1 - selected]}s") == 0
    port.delete()


@pytest.mark.parametrize("selected", [0, 1])
def test_dynamic_config_updates_count_when_port_is_deleted(
    service_and_ports: Tuple[Any, str, str], selected: int
) -> None:
    service, *kinds = service_and_ports
    sut = service.dynamic_config
    ports = [getattr(service, f"{kind}_builder")().create() for kind in kinds]

    ports[selected].delete()

    assert getattr(sut, f"number_of_{kinds[selected]}s") == 0
    assert getattr(sut, f"number_of_{kinds[1 - selected]}s") == 1
    ports[1 - selected].delete()


@pytest.mark.parametrize("pattern,first_kind,second_kind", service_patterns)
def test_dynamic_config_retains_temporary_service(
    node: iox2.Node, pattern: str, first_kind: str, second_kind: str
) -> None:
    first_count = f"number_of_{first_kind}s"
    second_count = f"number_of_{second_kind}s"

    sut = create_service(node, pattern).dynamic_config

    assert getattr(sut, first_count) == 0
    assert getattr(sut, second_count) == 0


def test_blackboard_dynamic_config_reports_deleted_service(node: iox2.Node) -> None:
    service = create_service(node, "blackboard")
    sut = service.dynamic_config

    service.delete()

    with pytest.raises(RuntimeError, match="service has been deleted"):
        _ = sut.number_of_writers
    with pytest.raises(RuntimeError, match="service has been deleted"):
        _ = sut.number_of_readers
