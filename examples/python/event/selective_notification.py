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

"""Notify a named listener and retain its key for later notifications."""

import iceoryx2 as iox2


def main() -> None:
    """Notify one listener by name, then reuse its saved key."""
    node = iox2.NodeBuilder.new().create(iox2.ServiceType.Local)
    service = (
        node.service_builder(iox2.ServiceName.new("Example/SelectiveNotification"))
        .event()
        .max_listeners(2)
        .create()
    )
    selected = service.listener_builder().name(iox2.PortName.new("selected")).create()
    other = service.listener_builder().name(iox2.PortName.new("other")).create()
    notifier = service.notifier_builder().default_event_id(iox2.EventId.new(7)).create()
    keys = []

    def visit(
        monofier: iox2.Monofier, details: iox2.ListenerDetails
    ) -> iox2.CallbackProgression:
        if details.listener_name.as_str() == "selected":
            monofier.notify()
            keys.append(monofier.listener_key())
        return iox2.CallbackProgression.Continue

    notifier.for_each_listener(visit)
    # Monofier expires when visit returns. Its independent key remains usable.
    notifier.notify_single_listener_with_custom_event_id(keys[0], iox2.EventId.new(8))
    print(
        "Selected listener:",
        [(event.id.as_value, event.count) for event in selected.try_wait()],
    )
    print(
        "Other listener:",
        [(event.id.as_value, event.count) for event in other.try_wait()],
    )


if __name__ == "__main__":
    main()
