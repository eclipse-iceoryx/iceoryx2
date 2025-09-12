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


class LargePayload(ctypes.Structure):
    _fields_ = [("data", ctypes.c_ulonglong)]


@pytest.mark.parametrize("service_type", service_types)
def test_send_and_receive_request_with_memmove_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    number_of_requests = 5

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(Payload, Payload)
        .max_active_requests_per_client(number_of_requests)
        .enable_fire_and_forget_requests(True)
        .create()
    )

    client = service.client_builder().create()
    server = service.server_builder().create()

    for i in range(0, number_of_requests):
        request_payload = Payload(data=19 + i)
        request_uninit = client.loan_uninit()
        ctypes.memmove(
            request_uninit.payload_ptr,
            ctypes.byref(request_payload),
            ctypes.sizeof(Payload),
        )
        request = request_uninit.assume_init()
        request.send()

    for i in range(0, number_of_requests):
        assert server.has_requests
        request = server.receive()
        assert request is not None
        request_payload = Payload(data=0)
        ctypes.memmove(
            ctypes.byref(request_payload),
            request.payload_ptr,
            ctypes.sizeof(Payload),
        )
        assert request_payload.data == 19 + i

    assert not server.has_requests


@pytest.mark.parametrize("service_type", service_types)
def test_send_and_receive_responses_with_memmove_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    number_of_responses = 5

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(Payload, Payload)
        .max_response_buffer_size(number_of_responses)
        .enable_fire_and_forget_requests(True)
        .create()
    )

    client = service.client_builder().create()
    server = service.server_builder().create()
    pending_response = client.send_copy(Payload(data=12))

    active_request = server.receive()

    for i in range(0, number_of_responses):
        response_payload = Payload(data=3 + 2 * i)
        response_uninit = active_request.loan_uninit()
        ctypes.memmove(
            response_uninit.payload_ptr,
            ctypes.byref(response_payload),
            ctypes.sizeof(Payload),
        )
        response = response_uninit.assume_init()
        response.send()

    for i in range(0, number_of_responses):
        assert pending_response.has_response
        response = pending_response.receive()
        payload = Payload(data=0)
        ctypes.memmove(
            ctypes.byref(payload), response.payload_ptr, ctypes.sizeof(Payload)
        )
        assert payload.data == 3 + 2 * i

    assert not pending_response.has_response


@pytest.mark.parametrize("service_type", service_types)
def test_send_and_receive_request_with_sendcopy_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name).request_response(Payload, Payload).create()
    )

    client = service.client_builder().create()
    server = service.server_builder().create()

    pending_response = client.send_copy(Payload(data=87))
    active_request = server.receive()

    assert active_request.payload().contents.data == 87
    active_request.send_copy(Payload(data=33))

    response = pending_response.receive()
    assert response.payload().contents.data == 33


@pytest.mark.parametrize("service_type", service_types)
def test_send_with_request_user_header_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(Payload, Payload)
        .request_header(ctypes.c_uint64)
        .create()
    )

    client = service.client_builder().create()
    server = service.server_builder().create()

    request_uninit = client.loan_uninit()
    ctypes.memmove(request_uninit.user_header_ptr, ctypes.byref(ctypes.c_uint64(89)), 8)
    assert request_uninit.user_header().contents.value == 89

    request = request_uninit.assume_init()
    assert request.user_header().contents.value == 89
    _pending_response = request.send()

    active_request = server.receive()
    assert active_request.user_header().contents.value == 89


@pytest.mark.parametrize("service_type", service_types)
def test_send_with_response_user_header_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(Payload, Payload)
        .response_header(ctypes.c_uint64)
        .create()
    )

    client = service.client_builder().create()
    server = service.server_builder().create()

    request_uninit = client.loan_uninit()
    request = request_uninit.assume_init()
    pending_response = request.send()

    active_request = server.receive()
    response_uninit = active_request.loan_uninit()
    ctypes.memmove(
        response_uninit.user_header_ptr, ctypes.byref(ctypes.c_uint64(44)), 8
    )
    assert response_uninit.user_header().contents.value == 44
    response = response_uninit.assume_init()
    assert response.user_header().contents.value == 44
    response.send()

    response = pending_response.receive()
    assert response.user_header().contents.value == 44


@pytest.mark.parametrize("service_type", service_types)
def test_send_with_request_system_header_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name).request_response(Payload, Payload).create()
    )

    client = service.client_builder().create()
    server = service.server_builder().create()

    request_uninit = client.loan_uninit()

    assert request_uninit.header.client_id == client.id
    assert request_uninit.header.number_of_elements == 1

    request = request_uninit.assume_init()

    assert request.header.client_id == client.id
    assert request.header.number_of_elements == 1

    _pending_response = request.send()

    active_request = server.receive()

    assert active_request.header.client_id == client.id
    assert active_request.header.number_of_elements == 1


@pytest.mark.parametrize("service_type", service_types)
def test_send_with_response_system_header_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name).request_response(Payload, Payload).create()
    )

    client = service.client_builder().create()
    server = service.server_builder().create()

    request_uninit = client.loan_uninit()
    request = request_uninit.assume_init()
    pending_response = request.send()

    active_request = server.receive()
    response_uninit = active_request.loan_uninit()

    assert response_uninit.header.server_id == server.id
    assert response_uninit.header.number_of_elements == 1

    response = response_uninit.assume_init()

    assert response.header.server_id == server.id
    assert response.header.number_of_elements == 1

    response.send()

    response = pending_response.receive()

    assert response.header.server_id == server.id
    assert response.header.number_of_elements == 1


@pytest.mark.parametrize("service_type", service_types)
def test_client_reallocation_fails_when_allocation_strategy_is_static(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(iox2.Slice[ctypes.c_uint8], iox2.Slice[ctypes.c_uint8])
        .create()
    )

    client = (
        service.client_builder()
        .initial_max_slice_len(8)
        .allocation_strategy(iox2.AllocationStrategy.Static)
        .create()
    )

    try:
        client.loan_slice_uninit(8)
    except iox2.LoanError:
        assert False

    with pytest.raises(iox2.LoanError):
        client.loan_slice_uninit(9)


@pytest.mark.parametrize("service_type", service_types)
def test_client_reallocation_works_when_allocation_strategy_is_not_static(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(iox2.Slice[ctypes.c_uint8], iox2.Slice[ctypes.c_uint8])
        .create()
    )

    client = (
        service.client_builder()
        .initial_max_slice_len(8)
        .allocation_strategy(iox2.AllocationStrategy.PowerOfTwo)
        .create()
    )

    try:
        client.loan_slice_uninit(12)
    except iox2.LoanError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_server_reallocation_fails_when_allocation_strategy_is_static(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(Payload, iox2.Slice[ctypes.c_uint8])
        .create()
    )

    client = service.client_builder().create()
    server = (
        service.server_builder()
        .initial_max_slice_len(8)
        .allocation_strategy(iox2.AllocationStrategy.Static)
        .create()
    )

    _pending_response = client.send_copy(Payload(data=1))
    active_request = server.receive()

    with pytest.raises(iox2.LoanError):
        active_request.loan_slice_uninit(9)

    try:
        active_request.loan_slice_uninit(8)
    except iox2.LoanError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_server_reallocation_works_when_allocation_strategy_is_not_static(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(Payload, iox2.Slice[ctypes.c_uint8])
        .create()
    )

    client = service.client_builder().create()
    server = (
        service.server_builder()
        .initial_max_slice_len(8)
        .allocation_strategy(iox2.AllocationStrategy.PowerOfTwo)
        .create()
    )

    _pending_response = client.send_copy(Payload(data=1))
    active_request = server.receive()

    try:
        active_request.loan_slice_uninit(12)
    except iox2.LoanError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_slice_type_forbids_use_of_non_slice_api(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(iox2.Slice[ctypes.c_uint8], iox2.Slice[ctypes.c_uint8])
        .create()
    )

    server = service.server_builder().create()
    client = service.client_builder().create()

    with pytest.raises(AssertionError):
        client.loan_uninit()

    request_uninit = client.loan_slice_uninit(1)
    request = request_uninit.assume_init()
    request.send()

    active_request = server.receive()

    with pytest.raises(AssertionError):
        active_request.loan_uninit()


@pytest.mark.parametrize("service_type", service_types)
def test_non_slice_type_forbids_use_of_slice_api(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name).request_response(Payload, Payload).create()
    )

    server = service.server_builder().create()
    client = service.client_builder().create()

    with pytest.raises(AssertionError):
        client.loan_slice_uninit(1)

    request_uninit = client.loan_uninit()
    request = request_uninit.assume_init()
    request.send()

    active_request = server.receive()

    with pytest.raises(AssertionError):
        active_request.loan_slice_uninit(1)


@pytest.mark.parametrize("service_type", service_types)
def test_client_can_request_graceful_disconnect(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name).request_response(Payload, Payload).create()
    )

    client = service.client_builder().create()
    server = service.server_builder().create()

    pending_response = client.send_copy(Payload(data=0))
    active_request = server.receive()

    assert pending_response.is_connected is True
    assert active_request.is_connected is True
    assert active_request.has_disconnect_hint is False

    pending_response.set_disconnect_hint()

    assert pending_response.is_connected is True
    assert active_request.is_connected is True
    assert active_request.has_disconnect_hint is True

    pending_response.delete()

    assert active_request.is_connected is False
    assert active_request.has_disconnect_hint is False
