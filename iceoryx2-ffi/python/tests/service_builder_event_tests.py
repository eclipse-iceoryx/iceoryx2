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
def test_existing_service_not_be_opened(service_type) -> None:
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
