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

import pytest

import iceoryx2 as iox2

service_types = [iox2.ServiceType.Ipc, iox2.ServiceType.Local]


class Payload(ctypes.Structure):
    _fields_ = [("data", ctypes.c_ubyte)]


@pytest.mark.parametrize("service_type", service_types)
def test_unable_to_deliver_strategy_can_be_configured(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(Payload, Payload)
        .enable_safe_overflow_for_responses(False)
        .max_servers(2)
        .create()
    )

    sut_1 = (
        service.server_builder()
        .unable_to_deliver_strategy(iox2.UnableToDeliverStrategy.Block)
        .create()
    )
    sut_2 = (
        service.server_builder()
        .unable_to_deliver_strategy(iox2.UnableToDeliverStrategy.DiscardSample)
        .create()
    )

    assert (
        sut_1.unable_to_deliver_strategy == iox2.UnableToDeliverStrategy.Block
    )
    assert (
        sut_2.unable_to_deliver_strategy
        == iox2.UnableToDeliverStrategy.DiscardSample
    )


