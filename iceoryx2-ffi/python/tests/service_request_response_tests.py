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
        ctypes.memmove(request_uninit.payload_ptr, ctypes.byref(request_payload), ctypes.sizeof(Payload))
        request = request_uninit.assume_init()
        request.send()

    for i in range(0, number_of_requests):
        assert server.has_requests
        request = server.receive()
        assert request is not None
        request_payload = Payload(data=0)
        ctypes.memmove(ctypes.byref(request_payload), request.payload_ptr, ctypes.sizeof(Payload))
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
        ctypes.memmove(response_uninit.payload_ptr, ctypes.byref(response_payload), ctypes.sizeof(Payload))
        response = response_uninit.assume_init()
        response.send()

    for i in range(0, number_of_responses):
        assert pending_response.has_response()
        response = pending_response.receive()
        payload = Payload(data=0)
        ctypes.memmove(ctypes.byref(payload), response.payload_ptr, ctypes.sizeof(Payload))
        assert payload.data == 3 + 2 * i

    assert not pending_response.has_response()


@pytest.mark.parametrize("service_type", service_types)
def test_send_and_receive_request_with_sendcopy_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .request_response(Payload, Payload)
        .create()
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
def test_send_with_request_header_works(
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
    ctypes.memmove(
        request_uninit.user_header_ptr, ctypes.byref(ctypes.c_uint64(89)), 8
    )

    request = request_uninit.assume_init()
    pending_response = request.send()

    active_request = server.receive()
    assert active_request.user_header().contents.value == 89


@pytest.mark.parametrize("service_type", service_types)
def test_send_with_response_header_works(
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
