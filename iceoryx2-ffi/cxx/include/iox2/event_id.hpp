// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

#ifndef IOX2_EVENT_ID_HPP
#define IOX2_EVENT_ID_HPP

#include "iox2/internal/iceoryx2.hpp"

#include <cstdint>
#include <iostream>

namespace iox2 {
/// Type that allows to identify an event uniquely.
class EventId {
  public:
    EventId(const EventId&) = default;
    EventId(EventId&&) = default;
    auto operator=(const EventId&) -> EventId& = default;
    auto operator=(EventId&&) -> EventId& = default;
    ~EventId() = default;

    /// Creates a new uint128_t [`EventId`] from the high bit and low bit part.
    EventId(uint64_t low, uint64_t high);

    /// Returns the high part of the [`EventId`]
    auto as_value_high() const -> uint64_t;

    /// Returns the low part of the [`EventId`]
    auto as_value_low() const -> uint64_t;

  private:
    friend auto operator<<(std::ostream& stream, const EventId& value) -> std::ostream&;
    iox2_event_id_t m_value;
};

auto operator<<(std::ostream& stream, const EventId& value) -> std::ostream&;
} // namespace iox2

#endif
