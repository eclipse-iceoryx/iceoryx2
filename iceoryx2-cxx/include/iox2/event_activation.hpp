// Copyright (c) 2026 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#ifndef IOX2_EVENT_ACTIVATION_HPP
#define IOX2_EVENT_ACTIVATION_HPP

#include "iox2/event_id.hpp"
#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {
/// Represents a single event activation with its identifier and activation count.
///
/// This class is returned by event listening operations and carries the event's
/// unique identifier along with the number of pending activations for that event.
class EventActivation {
  public:
    EventActivation(const EventActivation&) = default;
    EventActivation(EventActivation&&) = default;
    auto operator=(const EventActivation&) -> EventActivation& = default;
    auto operator=(EventActivation&&) -> EventActivation& = default;
    ~EventActivation() = default;

    /// Returns the event identifier of this activation.
    auto id() const -> EventId;
    /// Returns the number of event activations associated with this identifier.
    auto count() const -> uint64_t;

  private:
    friend auto operator<<(std::ostream& stream, const EventActivation& value) -> std::ostream&;
    EventActivation(iox2_event_id_t value, uint64_t count);
    friend void wait_callback(const iox2_event_id_t*, uint64_t, iox2_callback_context);

    EventId m_id;
    uint64_t m_count;
};
} // namespace iox2

#endif
