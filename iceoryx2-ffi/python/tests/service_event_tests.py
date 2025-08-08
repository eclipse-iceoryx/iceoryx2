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

import iceoryx2 as iox2
import pytest

service_types = [iox2.ServiceType.Ipc, iox2.ServiceType.Local]


@pytest.mark.parametrize("service_type", service_types)
def test_notifier_use_default_event_id(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    event_id = iox2.EventId.new(45)

    service_name = iox2.testing.generate_service_name()
    service = node.service_builder(service_name).event().create()

    notifier = service.notifier_builder().default_event_id(event_id).create()
    listener = service.listener_builder().create()

    notifier.notify()
    assert listener.try_wait_one() == event_id


@pytest.mark.parametrize("service_type", service_types)
def test_notification_with_custom_event_id_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    event_id = iox2.EventId.new(41)

    service_name = iox2.testing.generate_service_name()
    service = node.service_builder(service_name).event().create()

    notifier = service.notifier_builder().create()
    listener = service.listener_builder().create()

    notifier.notify_with_custom_event_id(event_id)
    assert listener.try_wait_one() == event_id


@pytest.mark.parametrize("service_type", service_types)
def test_deadline_can_be_acquired_via_ports(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    deadline = iox2.Duration.from_secs(123)

    service_name = iox2.testing.generate_service_name()
    service = node.service_builder(service_name).event().deadline(deadline).create()

    notifier = service.notifier_builder().create()
    listener = service.listener_builder().create()

    assert listener.deadline == deadline
    assert notifier.deadline == deadline


@pytest.mark.parametrize("service_type", service_types)
def test_listener_try_wait_one_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    event_id_1 = iox2.EventId.new(12)
    event_id_2 = iox2.EventId.new(21)

    service_name = iox2.testing.generate_service_name()
    service = node.service_builder(service_name).event().create()

    notifier = service.notifier_builder().create()
    listener = service.listener_builder().create()

    notifier.notify_with_custom_event_id(event_id_1)
    notifier.notify_with_custom_event_id(event_id_2)

    assert listener.try_wait_one() == event_id_1
    assert listener.try_wait_one() == event_id_2


@pytest.mark.parametrize("service_type", service_types)
def test_listener_timed_wait_one_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    timeout = iox2.Duration.from_secs(1)
    event_id_1 = iox2.EventId.new(13)
    event_id_2 = iox2.EventId.new(31)

    service_name = iox2.testing.generate_service_name()
    service = node.service_builder(service_name).event().create()

    notifier = service.notifier_builder().create()
    listener = service.listener_builder().create()

    notifier.notify_with_custom_event_id(event_id_1)
    notifier.notify_with_custom_event_id(event_id_2)

    assert listener.timed_wait_one(timeout) == event_id_1
    assert listener.timed_wait_one(timeout) == event_id_2


@pytest.mark.parametrize("service_type", service_types)
def test_listener_blocking_wait_one_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    event_id_1 = iox2.EventId.new(14)
    event_id_2 = iox2.EventId.new(41)

    service_name = iox2.testing.generate_service_name()
    service = node.service_builder(service_name).event().create()

    notifier = service.notifier_builder().create()
    listener = service.listener_builder().create()

    notifier.notify_with_custom_event_id(event_id_1)
    notifier.notify_with_custom_event_id(event_id_2)

    assert listener.blocking_wait_one() == event_id_1
    assert listener.blocking_wait_one() == event_id_2


@pytest.mark.parametrize("service_type", service_types)
def test_listener_try_wait_all_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    event_id_1 = iox2.EventId.new(15)
    event_id_2 = iox2.EventId.new(51)

    service_name = iox2.testing.generate_service_name()
    service = node.service_builder(service_name).event().create()

    notifier = service.notifier_builder().create()
    listener = service.listener_builder().create()

    notifier.notify_with_custom_event_id(event_id_1)
    notifier.notify_with_custom_event_id(event_id_2)

    events = listener.try_wait_all()

    assert len(events) == 2
    assert events[0] == event_id_1
    assert events[1] == event_id_2


@pytest.mark.parametrize("service_type", service_types)
def test_listener_timed_wait_all_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    event_id_1 = iox2.EventId.new(16)
    event_id_2 = iox2.EventId.new(61)
    timeout = iox2.Duration.from_secs(2)

    service_name = iox2.testing.generate_service_name()
    service = node.service_builder(service_name).event().create()

    notifier = service.notifier_builder().create()
    listener = service.listener_builder().create()

    notifier.notify_with_custom_event_id(event_id_1)
    notifier.notify_with_custom_event_id(event_id_2)

    events = listener.timed_wait_all(timeout)

    assert len(events) == 2
    assert events[0] == event_id_1
    assert events[1] == event_id_2


@pytest.mark.parametrize("service_type", service_types)
def test_listener_blocking_wait_all_works(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    event_id_1 = iox2.EventId.new(17)
    event_id_2 = iox2.EventId.new(71)

    service_name = iox2.testing.generate_service_name()
    service = node.service_builder(service_name).event().create()

    notifier = service.notifier_builder().create()
    listener = service.listener_builder().create()

    notifier.notify_with_custom_event_id(event_id_1)
    notifier.notify_with_custom_event_id(event_id_2)

    events = listener.blocking_wait_all()

    assert len(events) == 2
    assert events[0] == event_id_1
    assert events[1] == event_id_2
