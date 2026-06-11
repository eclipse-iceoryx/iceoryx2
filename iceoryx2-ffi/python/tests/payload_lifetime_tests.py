# Copyright (c) 2026 Contributors to the Eclipse Foundation
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
import gc

import iceoryx2 as iox2
import pytest

service_types = [iox2.ServiceType.Ipc, iox2.ServiceType.Local]


@pytest.mark.parametrize("service_type", service_types)
def test_saved_payload_value_persists_after_sample_drop(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = (
        node.service_builder(service_name)
        .publish_subscribe(iox2.Slice[ctypes.c_uint64])
        .open_or_create()
    )
    publisher = service.publisher_builder().initial_max_slice_len(8).create()
    subscriber = service.subscriber_builder().create()

    marker = 0xCAFEBABEDEADBEEF
    loan = publisher.loan_slice_uninit(4)
    for i in range(4):
        loan.payload()[i] = marker
    loan.assume_init().send()

    sample = subscriber.receive()
    assert sample is not None
    payload_ref = sample.payload()

    del sample
    gc.collect()

    overwrite = 0xDEADC0FFEE
    loan2 = publisher.loan_slice_uninit(4)
    for i in range(4):
        loan2.payload()[i] = overwrite
    loan2.assume_init().send()
    sample2 = subscriber.receive()
    assert sample2 is not None
    del sample2
    gc.collect()

    assert payload_ref[0] == marker, (
        f"iox2-1548: saved payload was overwritten "
        f"(got 0x{payload_ref[0]:x}, expected 0x{marker:x})"
    )


@pytest.mark.parametrize("service_type", service_types)
def test_accumulated_payload_references_each_keep_own_data(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    num_samples = 5
    service = (
        node.service_builder(service_name)
        .publish_subscribe(iox2.Slice[ctypes.c_uint64])
        .subscriber_max_borrowed_samples(num_samples)
        .subscriber_max_buffer_size(num_samples)
        .open_or_create()
    )
    publisher = (
        service.publisher_builder()
        .initial_max_slice_len(4)
        .max_loaned_samples(num_samples)
        .create()
    )
    subscriber = service.subscriber_builder().create()

    saved = []
    for i in range(num_samples):
        loan = publisher.loan_slice_uninit(4)
        for j in range(4):
            loan.payload()[j] = (i + 1) * 1_000_000 + j
        loan.assume_init().send()

        sample = subscriber.receive()
        assert sample is not None
        saved.append(sample.payload())

    gc.collect()

    for i, payload in enumerate(saved):
        for j in range(4):
            expected = (i + 1) * 1_000_000 + j
            actual = payload[j]
            assert (
                actual == expected
            ), f"iox2-1548: saved[{i}][{j}] = {actual}, expected {expected}"


@pytest.mark.parametrize("service_type", service_types)
def test_dropping_payload_returns_chunk_to_pool(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    max_borrowed = 2
    service = (
        node.service_builder(service_name)
        .publish_subscribe(iox2.Slice[ctypes.c_uint64])
        .subscriber_max_borrowed_samples(max_borrowed)
        .subscriber_max_buffer_size(5)
        .open_or_create()
    )
    publisher = (
        service.publisher_builder()
        .initial_max_slice_len(4)
        .max_loaned_samples(5)
        .create()
    )
    subscriber = service.subscriber_builder().create()

    for i in range(3):
        loan = publisher.loan_slice_uninit(4)
        loan.payload()[0] = i + 1
        loan.assume_init().send()

    s1 = subscriber.receive()
    assert s1 is not None
    p1 = s1.payload()
    del s1

    s2 = subscriber.receive()
    assert s2 is not None
    p2 = s2.payload()
    del s2

    with pytest.raises(Exception) as exc_info:
        _ = subscriber.receive()
    assert "ExceedsMaxBorrows" in str(
        exc_info.value
    ), f"expected ExceedsMaxBorrows, got {exc_info.value}"

    del p1
    gc.collect()

    s3 = subscriber.receive()
    assert s3 is not None, (
        "after releasing the last reference to a payload, the Sample's "
        "chunk should return to the pool and receive() should succeed"
    )
    assert p2[0] == 2


@pytest.mark.parametrize("service_type", service_types)
def test_payload_survives_slice_reallocation(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    num_samples = 5
    service = (
        node.service_builder(service_name)
        .publish_subscribe(iox2.Slice[ctypes.c_uint64])
        .subscriber_max_borrowed_samples(num_samples)
        .subscriber_max_buffer_size(num_samples)
        .open_or_create()
    )
    publisher = (
        service.publisher_builder()
        .initial_max_slice_len(4)
        .max_loaned_samples(num_samples)
        .allocation_strategy(iox2.AllocationStrategy.PowerOfTwo)
        .create()
    )
    subscriber = service.subscriber_builder().create()

    saved = []
    for i in range(1, num_samples + 1):
        size = i * 3
        loan = publisher.loan_slice_uninit(size)
        loan.payload()[0] = i * 10
        loan.assume_init().send()

        sample = subscriber.receive()
        assert sample is not None
        saved.append(sample.payload())

    gc.collect()

    for i, payload in enumerate(saved, start=1):
        assert (
            payload[0] == i * 10
        ), f"iox2-1548: payload[{i-1}][0] = {payload[0]}, expected {i*10}"
