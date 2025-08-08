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
        .publish_subscribe(Payload)
        .enable_safe_overflow(False)
        .max_publishers(2)
        .create()
    )

    sut_1 = (
        service.publisher_builder()
        .unable_to_deliver_strategy(iox2.UnableToDeliverStrategy.Block)
        .create()
    )
    sut_2 = (
        service.publisher_builder()
        .unable_to_deliver_strategy(iox2.UnableToDeliverStrategy.DiscardSample)
        .create()
    )

    assert sut_1.unable_to_deliver_strategy == iox2.UnableToDeliverStrategy.Block
    assert (
        sut_2.unable_to_deliver_strategy == iox2.UnableToDeliverStrategy.DiscardSample
    )


@pytest.mark.parametrize("service_type", service_types)
def test_max_loans_can_be_set_up(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = node.service_builder(service_name).publish_subscribe(Payload).create()
    max_loans = 8

    sut = service.publisher_builder().max_loaned_samples(max_loans).create()

    samples = []
    for _ in range(0, max_loans):
        sample = sut.loan_uninit()
        samples.append(sample)

    with pytest.raises(iox2.LoanError):
        sut.loan_uninit()


@pytest.mark.parametrize("service_type", service_types)
def test_deleting_publisher_removes_it_from_the_service(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .publish_subscribe(Payload)
        .max_publishers(1)
        .create()
    )

    sut = service.publisher_builder().create()

    with pytest.raises(iox2.PublisherCreateError):
        sut = service.publisher_builder().create()

    sut.delete()

    try:
        sut = service.publisher_builder().create()
    except iox2.PublisherCreateError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_deleting_sample_mut_uninit_releases_it(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .publish_subscribe(Payload)
        .max_publishers(1)
        .create()
    )

    sut = service.publisher_builder().max_loaned_samples(1).create()
    sample_uninit = sut.loan_uninit()

    with pytest.raises(iox2.LoanError):
        sample_uninit = sut.loan_uninit()

    sample_uninit.delete()

    try:
        sample_uninit = sut.loan_uninit()
    except iox2.LoanError:
        assert False


@pytest.mark.parametrize("service_type", service_types)
def test_deleting_sample_mut_releases_it(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .publish_subscribe(Payload)
        .max_publishers(1)
        .create()
    )

    sut = service.publisher_builder().max_loaned_samples(1).create()
    sample_uninit = sut.loan_uninit()
    sample = sample_uninit.assume_init()

    with pytest.raises(iox2.LoanError):
        sample_uninit = sut.loan_uninit()

    sample.delete()

    try:
        sample_uninit = sut.loan_uninit()
    except iox2.LoanError:
        assert False
