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

"""Shared event ids for event-based communication examples."""

from enum import IntEnum

import iceoryx2 as iox2


class PubSubEvent(IntEnum):
    """Event ids shared by publisher and subscriber."""

    PublisherConnected = 0
    PublisherDisconnected = 1
    SubscriberConnected = 2
    SubscriberDisconnected = 3
    SentSample = 4
    ReceivedSample = 5
    SentHistory = 6
    Unknown = 7


def to_event_id(event: PubSubEvent) -> iox2.EventId:
    """Converts enum value to iceoryx2 EventId."""
    return iox2.EventId.new(int(event))


def from_event_id(event_id: iox2.EventId) -> PubSubEvent:
    """Converts iceoryx2 EventId to enum value."""
    value = event_id.as_value
    for event in PubSubEvent:
        if int(event) == value:
            return event
    return PubSubEvent.Unknown
