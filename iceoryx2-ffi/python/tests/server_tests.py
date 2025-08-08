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

    assert sut_1.unable_to_deliver_strategy == iox2.UnableToDeliverStrategy.Block
    assert (
        sut_2.unable_to_deliver_strategy == iox2.UnableToDeliverStrategy.DiscardSample
    )


@pytest.mark.parametrize("service_type", service_types)
def test_deleting_server_removes_it_from_the_service(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(Payload, Payload)
        .max_servers(1)
        .create()
    )

    sut = service.server_builder().create()

    with pytest.raises(iox2.ServerCreateError):
        sut = service.server_builder().create()

    sut.delete()

    try:
        sut = service.server_builder().create()
    except iox2.ServerCreateError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_deleting_response_mut_uninit_removes_it(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(ctypes.c_uint64, ctypes.c_uint64)
        .create()
    )

    client = service.client_builder().create()
    sut = service.server_builder().max_loaned_responses_per_request(1).create()
    _pending_response = client.send_copy(ctypes.c_uint64(0))
    active_request = sut.receive()

    response_uninit = active_request.loan_uninit()
    with pytest.raises(iox2.LoanError):
        response_uninit = active_request.loan_uninit()

    response_uninit.delete()

    try:
        response_uninit = active_request.loan_uninit()
    except iox2.LoanError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_deleting_response_mut_removes_it(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(ctypes.c_uint64, ctypes.c_uint64)
        .create()
    )

    client = service.client_builder().create()
    sut = service.server_builder().max_loaned_responses_per_request(1).create()
    _pending_response = client.send_copy(ctypes.c_uint64(0))
    active_request = sut.receive()

    response_uninit = active_request.loan_uninit()
    response = response_uninit.assume_init()
    with pytest.raises(iox2.LoanError):
        response_uninit = active_request.loan_uninit()

    response.delete()

    try:
        response_uninit = active_request.loan_uninit()
    except iox2.LoanError:
        assert False
