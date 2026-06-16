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
def test_backpressure_strategy_can_be_configured(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(Payload, Payload)
        .enable_safe_overflow_for_requests(False)
        .max_clients(2)
        .create()
    )

    sut_1 = (
        service.client_builder()
        .backpressure_strategy(iox2.BackpressureStrategy.RetryUntilDelivered)
        .create()
    )
    sut_2 = (
        service.client_builder()
        .backpressure_strategy(iox2.BackpressureStrategy.DiscardData)
        .create()
    )

    assert sut_1.backpressure_strategy == iox2.BackpressureStrategy.RetryUntilDelivered
    assert sut_2.backpressure_strategy == iox2.BackpressureStrategy.DiscardData


@pytest.mark.parametrize("service_type", service_types)
def test_deleting_client_removes_it_from_the_service(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(Payload, Payload)
        .max_clients(1)
        .create()
    )

    sut = service.client_builder().create()

    with pytest.raises(iox2.ClientCreateError):
        sut = service.client_builder().create()

    sut.delete()

    try:
        sut = service.client_builder().create()
    except iox2.ClientCreateError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_deleting_request_mut_uninit_removes_it_from_the_service(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(Payload, Payload)
        .max_loaned_requests(1)
        .create()
    )

    sut = service.client_builder().create()
    request_uninit = sut.loan_uninit()

    with pytest.raises(iox2.LoanError):
        request_uninit = sut.loan_uninit()

    request_uninit.delete()

    try:
        request_uninit = sut.loan_uninit()
    except iox2.LoanError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_deleting_request_mut_removes_it_from_the_service(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(Payload, Payload)
        .max_loaned_requests(1)
        .create()
    )

    sut = service.client_builder().create()
    request_uninit = sut.loan_uninit()
    request = request_uninit.assume_init()

    with pytest.raises(iox2.LoanError):
        request_uninit = sut.loan_uninit()

    request.delete()

    try:
        request_uninit = sut.loan_uninit()
    except iox2.LoanError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_client_applies_max_active_requests(
    service_type: iox2.ServiceType,
) -> None:
    MAX_ACTIVE_REQUESTS = 6
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(Payload, Payload)
        .max_active_requests_per_client(2 * MAX_ACTIVE_REQUESTS)
        .create()
    )

    client = service.client_builder().max_active_requests(MAX_ACTIVE_REQUESTS).create()

    assert client.max_active_requests == MAX_ACTIVE_REQUESTS


@pytest.mark.parametrize("service_type", service_types)
def test_client_creation_fails_when_max_active_requests_exceeds_service_max(
    service_type: iox2.ServiceType,
) -> None:
    MAX_ACTIVE_REQUESTS = 13
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(Payload, Payload)
        .max_active_requests_per_client(MAX_ACTIVE_REQUESTS)
        .create()
    )

    with pytest.raises(iox2.ClientCreateError):
        service.client_builder().max_active_requests(MAX_ACTIVE_REQUESTS + 1).create()


@pytest.mark.parametrize("service_type", service_types)
def test_client_max_active_requests_is_at_least_one(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name).request_response(Payload, Payload).create()
    )

    client = service.client_builder().max_active_requests(0).create()

    assert client.max_active_requests == 1
