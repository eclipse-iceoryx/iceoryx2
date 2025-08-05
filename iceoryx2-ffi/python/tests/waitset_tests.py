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
def test_newly_created_waitset_is_empty(
    service_type: iox2.ServiceType,
) -> None:
    sut = iox2.WaitSetBuilder.new().create(service_type)
    assert sut.len == 0
    assert sut.is_empty


@pytest.mark.parametrize("service_type", service_types)
def test_waitset_builder_set_signal_handling_mode_correctly(
    service_type: iox2.ServiceType,
) -> None:
    sut_1 = (
        iox2.WaitSetBuilder.new()
        .signal_handling_mode(iox2.SignalHandlingMode.Disabled)
        .create(service_type)
    )
    sut_2 = (
        iox2.WaitSetBuilder.new()
        .signal_handling_mode(iox2.SignalHandlingMode.HandleTerminationRequests)
        .create(service_type)
    )
    assert sut_1.signal_handling_mode == iox2.SignalHandlingMode.Disabled
    assert (
        sut_2.signal_handling_mode == iox2.SignalHandlingMode.HandleTerminationRequests
    )


@pytest.mark.parametrize("service_type", service_types)
def test_attaching_notifications_works(
    service_type: iox2.ServiceType,
) -> None:
    number_of_attachments = 15
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .event()
        .max_listeners(number_of_attachments)
        .create()
    )
    listeners = []
    for i in range(0, number_of_attachments):
        listener = service.listener_builder().create()
        listeners.append(listener)

    sut = iox2.WaitSetBuilder.new().create(service_type)
    waitset_guards = []
    for i in range(0, number_of_attachments):
        guard = sut.attach_notification(listeners[i])
        waitset_guards.append(guard)
        assert sut.len == i + 1
        assert not sut.is_empty


@pytest.mark.parametrize("service_type", service_types)
def test_attaching_deadlines_works(
    service_type: iox2.ServiceType,
) -> None:
    number_of_attachments = 15
    deadline = iox2.Duration.from_millis(123)
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .event()
        .max_listeners(number_of_attachments)
        .create()
    )
    listeners = []
    for i in range(0, number_of_attachments):
        listener = service.listener_builder().create()
        listeners.append(listener)

    sut = iox2.WaitSetBuilder.new().create(service_type)
    waitset_guards = []
    for i in range(0, number_of_attachments):
        guard = sut.attach_deadline(listeners[i], deadline)
        waitset_guards.append(guard)
        assert sut.len == i + 1
        assert not sut.is_empty


@pytest.mark.parametrize("service_type", service_types)
def test_attaching_interval_works(
    service_type: iox2.ServiceType,
) -> None:
    number_of_attachments = 15

    sut = iox2.WaitSetBuilder.new().create(service_type)
    waitset_guards = []
    for i in range(0, number_of_attachments):
        guard = sut.attach_interval(iox2.Duration.from_millis(i + 1))
        waitset_guards.append(guard)
        assert sut.len == i + 1
        assert not sut.is_empty


@pytest.mark.parametrize("service_type", service_types)
def test_wait_and_process_returns_when_timeout_has_passed(
    service_type: iox2.ServiceType,
) -> None:
    sut = iox2.WaitSetBuilder.new().create(service_type)
    _guard = sut.attach_interval(iox2.Duration.from_secs(123))
    (triggers, result) = sut.wait_and_process_with_timeout(iox2.Duration.from_millis(1))
    assert len(triggers) == 0
    assert result == iox2.WaitSetRunResult.AllEventsHandled


@pytest.mark.parametrize("service_type", service_types)
def test_wait_and_process_returns_triggered_listeners(
    service_type: iox2.ServiceType,
) -> None:
    number_of_attachments = 15
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    services = []
    listeners = []
    notifiers = []
    for i in range(0, number_of_attachments):
        service_name = iox2.testing.generate_service_name()
        service = (
            node.service_builder(service_name)
            .event()
            .max_listeners(number_of_attachments)
            .create()
        )
        listener = service.listener_builder().create()
        notifier = service.notifier_builder().create()
        listeners.append(listener)
        notifiers.append(notifier)
        services.append(service)

    sut = iox2.WaitSetBuilder.new().create(service_type)
    waitset_guards = []
    for i in range(0, number_of_attachments):
        guard = sut.attach_notification(listeners[i])
        waitset_guards.append(guard)
        assert sut.len == i + 1
        assert not sut.is_empty

    notifiers[0].notify()
    notifiers[1].notify()

    (triggers, result) = sut.wait_and_process()
    assert len(triggers) == 2
    assert result == iox2.WaitSetRunResult.AllEventsHandled

    for i in range(0, 1):
        assert triggers[i].has_event_from(waitset_guards[0]) or triggers[
            i
        ].has_event_from(waitset_guards[1])

    for k in range(2, number_of_attachments):
        for i in range(0, 1):
            assert not triggers[i].has_event_from(waitset_guards[k])


@pytest.mark.parametrize("service_type", service_types)
def test_reports_missed_deadline(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = node.service_builder(service_name).event().create()
    listener = service.listener_builder().create()

    sut = iox2.WaitSetBuilder.new().create(service_type)
    guard = sut.attach_deadline(listener, iox2.Duration.from_nanos(1))

    (triggers, result) = sut.wait_and_process()
    assert len(triggers) == 1
    assert result == iox2.WaitSetRunResult.AllEventsHandled

    assert triggers[0].has_missed_deadline(guard)


@pytest.mark.parametrize("service_type", service_types)
def test_create_attachment_id_from_guard(
    service_type: iox2.ServiceType,
) -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service_name = iox2.testing.generate_service_name()
    service = node.service_builder(service_name).event().create()
    listener = service.listener_builder().create()

    sut = iox2.WaitSetBuilder.new().create(service_type)
    guard = sut.attach_deadline(listener, iox2.Duration.from_nanos(1))
    attachment_id = iox2.WaitSetAttachmentId.from_guard(guard)

    (triggers, result) = sut.wait_and_process()
    assert len(triggers) == 1
    assert result == iox2.WaitSetRunResult.AllEventsHandled

    assert triggers[0] == attachment_id


@pytest.mark.parametrize("service_type", service_types)
def test_deleting_guard_explicitly_removes_attachment(
    service_type: iox2.ServiceType,
) -> None:
    number_of_attachments = 15
    config = iox2.testing.generate_isolated_config()
    service_name = iox2.testing.generate_service_name()
    node = iox2.NodeBuilder.new().config(config).create(service_type)
    service = (
        node.service_builder(service_name)
        .event()
        .max_listeners(number_of_attachments)
        .create()
    )
    listeners = []
    for i in range(0, number_of_attachments):
        listener = service.listener_builder().create()
        listeners.append(listener)

    sut = iox2.WaitSetBuilder.new().create(service_type)
    waitset_guards = []
    for i in range(0, number_of_attachments):
        guard = sut.attach_notification(listeners[i])
        waitset_guards.append(guard)

    for i in range(0, number_of_attachments):
        assert sut.len == number_of_attachments - i
        waitset_guards[i].delete()

    assert sut.len == 0
    assert sut.is_empty
