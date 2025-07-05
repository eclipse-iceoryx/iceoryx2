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

import pytest

import iceoryx2 as iox2
import ctypes

service_types = [iox2.ServiceType.Ipc, iox2.ServiceType.Local]

class Payload(ctypes.Structure):
    _fields_ = [("data", ctypes.c_ubyte)]

class LargePayload(ctypes.Structure):
    _fields_ = [("data", ctypes.c_ulonglong)]


@pytest.mark.parametrize("service_type", service_types)
def test_send_and_receive_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    number_of_samples = 5

    service_name = iox2.testing.generate_service_name()
    service = node.service_builder(service_name).publish_subscribe().subscriber_max_buffer_size(number_of_samples).create()

    publisher = service.publisher_builder().create()
    subscriber = service.subscriber_builder().create()
    assert subscriber.has_samples() == False

    for i in range(0, number_of_samples):
        send_payload = Payload(data = 82 + i)
        sample_uninit = publisher.loan_slice_uninit(1)
        ctypes.memmove(sample_uninit.payload_ptr, ctypes.byref(send_payload), 1)
        sample = sample_uninit.assume_init()
        sample.send()

    assert subscriber.has_samples() == True

    for i in range(0, number_of_samples):
        received_sample = subscriber.receive()
        assert received_sample is not None
        received_payload = Payload(data = 0)
        ctypes.memmove(ctypes.byref(received_payload), received_sample.payload_ptr, 1)
        assert received_payload.data == 82 + i

    assert subscriber.has_samples() == False


@pytest.mark.parametrize("service_type", service_types)
def test_send_large_payload_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)

    service_name = iox2.testing.generate_service_name()
    service = node.service_builder(service_name).publish_subscribe().create()

    publisher = service.publisher_builder().initial_max_slice_len(8).create()
    subscriber = service.subscriber_builder().create()

    send_payload = LargePayload(data = 19203182930990147)
    sample_uninit = publisher.loan_slice_uninit(8)
    ctypes.memmove(sample_uninit.payload_ptr, ctypes.byref(send_payload), 8)
    sample = sample_uninit.assume_init()
    sample.send()

    received_sample = subscriber.receive()
    assert received_sample is not None
    received_payload = LargePayload(data = 0)
    ctypes.memmove(ctypes.byref(received_payload), received_sample.payload_ptr, 8)
    assert received_payload.data == send_payload.data

