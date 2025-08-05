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
    _fields_ = [
        ("data", ctypes.c_int),
    ]


@pytest.mark.parametrize("service_type", service_types)
def test_can_be_configured(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .publish_subscribe(Payload)
        .subscriber_max_buffer_size(47112)
        .create()
    )

    sut = service.subscriber_builder().buffer_size(47112).create()

    assert sut.buffer_size == 47112


@pytest.mark.parametrize("service_type", service_types)
def test_deleting_subscriber_removes_if_from_the_service(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .publish_subscribe(Payload)
        .max_subscribers(1)
        .create()
    )

    sut = service.subscriber_builder().create()

    with pytest.raises(iox2.SubscriberCreateError):
        sut = service.subscriber_builder().create()

    sut.delete()

    try:
        sut = service.subscriber_builder().create()
    except iox2.SubscriberCreateError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_deleting_sample_releases_it(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .publish_subscribe(Payload)
        .subscriber_max_buffer_size(2)
        .subscriber_max_borrowed_samples(1)
        .create()
    )

    sut = service.subscriber_builder().create()
    publisher = service.publisher_builder().create()
    publisher.loan_uninit().assume_init().send()
    publisher.loan_uninit().assume_init().send()

    sample = sut.receive()
    with pytest.raises(iox2.ReceiveError):
        sample = sut.receive()

    sample.delete()

    try:
        sample = sut.receive()
    except iox2.ReceiveError:
        assert False
