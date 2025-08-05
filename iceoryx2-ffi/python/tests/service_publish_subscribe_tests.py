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
def test_send_and_receive_with_memmove_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    number_of_samples = 5

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .publish_subscribe(Payload)
        .subscriber_max_buffer_size(number_of_samples)
        .create()
    )

    publisher = service.publisher_builder().create()
    subscriber = service.subscriber_builder().create()
    assert not subscriber.has_samples()

    for i in range(0, number_of_samples):
        send_payload = Payload(data=82 + i)
        sample_uninit = publisher.loan_uninit()
        ctypes.memmove(
            sample_uninit.payload_ptr,
            ctypes.byref(send_payload),
            ctypes.sizeof(Payload),
        )
        sample = sample_uninit.assume_init()
        sample.send()

    assert subscriber.has_samples()

    for i in range(0, number_of_samples):
        assert subscriber.has_samples()
        received_sample = subscriber.receive()
        assert received_sample is not None
        received_payload = Payload(data=0)
        ctypes.memmove(
            ctypes.byref(received_payload),
            received_sample.payload_ptr,
            ctypes.sizeof(Payload),
        )
        assert received_payload.data == 82 + i

    assert not subscriber.has_samples()


@pytest.mark.parametrize("service_type", service_types)
def test_send_copy_and_receive_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    number_of_samples = 6

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .publish_subscribe(Payload)
        .subscriber_max_buffer_size(number_of_samples)
        .create()
    )

    publisher = service.publisher_builder().create()
    subscriber = service.subscriber_builder().create()
    assert not subscriber.has_samples()

    for i in range(0, number_of_samples):
        publisher.send_copy(Payload(data=85 + i))

    assert subscriber.has_samples()

    for i in range(0, number_of_samples):
        received_sample = subscriber.receive()
        assert received_sample.payload().contents.data == 85 + i

    assert not subscriber.has_samples()


@pytest.mark.parametrize("service_type", service_types)
def test_send_with_write_payload_and_receive_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    number_of_samples = 6

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .publish_subscribe(Payload)
        .subscriber_max_buffer_size(number_of_samples)
        .create()
    )

    publisher = service.publisher_builder().create()
    subscriber = service.subscriber_builder().create()
    assert not subscriber.has_samples()

    for i in range(0, number_of_samples):
        sample_uninit = publisher.loan_uninit()
        sample = sample_uninit.write_payload(Payload(data=89 + i))
        sample.send()

    assert subscriber.has_samples()

    for i in range(0, number_of_samples):
        received_sample = subscriber.receive()
        assert received_sample.payload().contents.data == 89 + i

    assert not subscriber.has_samples()


@pytest.mark.parametrize("service_type", service_types)
def test_send_large_payload_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name).publish_subscribe(LargePayload).create()
    )

    publisher = service.publisher_builder().create()
    subscriber = service.subscriber_builder().create()

    send_payload = LargePayload(data=19203182930990147)
    sample_uninit = publisher.loan_uninit()
    ctypes.memmove(sample_uninit.payload_ptr, ctypes.byref(send_payload), 8)
    sample = sample_uninit.assume_init()
    sample.send()

    received_sample = subscriber.receive()
    assert received_sample is not None
    received_payload = LargePayload(data=0)
    ctypes.memmove(ctypes.byref(received_payload), received_sample.payload_ptr, 8)
    assert received_payload.data == send_payload.data


@pytest.mark.parametrize("service_type", service_types)
def test_published_header_is_the_same_as_received_header(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = node.service_builder(service_name).publish_subscribe(Payload).create()

    publisher = service.publisher_builder().create()
    subscriber = service.subscriber_builder().create()

    sample_uninit = publisher.loan_uninit()
    sample = sample_uninit.assume_init()
    send_header = sample.header
    assert send_header.node_id == node.id
    assert send_header.publisher_id == publisher.id
    assert send_header.number_of_elements == 1

    sample.send()

    received_sample = subscriber.receive()
    assert received_sample is not None
    assert received_sample.header == send_header


@pytest.mark.parametrize("service_type", service_types)
def test_custom_user_header_can_be_used(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .publish_subscribe(Payload)
        .user_header(Payload)
        .create()
    )

    publisher = service.publisher_builder().create()
    subscriber = service.subscriber_builder().create()

    sample_uninit = publisher.loan_uninit()
    send_user_header_payload = Payload(data=37)
    ctypes.memmove(
        sample_uninit.user_header_ptr, ctypes.byref(send_user_header_payload), 1
    )
    sample = sample_uninit.assume_init()
    sample.send()

    received_sample = subscriber.receive()
    assert received_sample is not None
    assert received_sample.user_header().contents.data == send_user_header_payload.data


@pytest.mark.parametrize("service_type", service_types)
def test_reallocation_fails_when_allocation_strategy_is_static(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .publish_subscribe(iox2.Slice[ctypes.c_uint8])
        .create()
    )

    publisher = (
        service.publisher_builder()
        .initial_max_slice_len(8)
        .allocation_strategy(iox2.AllocationStrategy.Static)
        .create()
    )

    try:
        publisher.loan_slice_uninit(8)
    except iox2.LoanError:
        assert False

    with pytest.raises(iox2.LoanError):
        publisher.loan_slice_uninit(9)


@pytest.mark.parametrize("service_type", service_types)
def test_reallocation_works_when_allocation_strategy_is_not_static(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .publish_subscribe(iox2.Slice[ctypes.c_uint8])
        .create()
    )

    publisher = (
        service.publisher_builder()
        .initial_max_slice_len(8)
        .allocation_strategy(iox2.AllocationStrategy.PowerOfTwo)
        .create()
    )

    try:
        publisher.loan_slice_uninit(12)
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
        .publish_subscribe(iox2.Slice[ctypes.c_uint8])
        .create()
    )

    publisher = service.publisher_builder().create()

    with pytest.raises(AssertionError):
        publisher.loan_uninit()


@pytest.mark.parametrize("service_type", service_types)
def test_non_slice_type_forbids_use_of_slice_api(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = node.service_builder(service_name).publish_subscribe(Payload).create()

    with pytest.raises(AssertionError):
        publisher = service.publisher_builder().initial_max_slice_len(8).create()

    with pytest.raises(AssertionError):
        publisher = (
            service.publisher_builder()
            .allocation_strategy(iox2.AllocationStrategy.PowerOfTwo)
            .create()
        )

    publisher = service.publisher_builder().create()

    with pytest.raises(AssertionError):
        publisher.loan_slice_uninit(1)
