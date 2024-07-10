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
#ifndef IOX2_EVENT_ID_HPP_
#define IOX2_EVENT_ID_HPP_

#include <cstdint>
#include <iostream>

namespace iox2 {
class EventId {
   public:
    EventId(const uint64_t value) : m_value{value} {}
    uint64_t as_value() const { return m_value; }

   private:
    friend std::ostream& operator<<(std::ostream&, const EventId&);
    uint64_t m_value;
};

inline std::ostream& operator<<(std::ostream& stream, const EventId& value) {
    std::cout << "EventId { m_value: " << value.m_value << "}";
    return stream;
}

}  // namespace iox2

#endif
