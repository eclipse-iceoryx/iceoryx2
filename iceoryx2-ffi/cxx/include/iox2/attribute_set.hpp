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

#ifndef IOX2_ATTRIBUTE_SET_HPP
#define IOX2_ATTRIBUTE_SET_HPP

#include "iox/assertions_addendum.hpp"
#include "iox/vector.hpp"
#include "iox2/attribute.hpp"
#include "iox2/internal/iceoryx2.hpp"

#include <iostream>

namespace iox2 {
class AttributeSet {
  public:
    auto get(const Attribute::Key& key) const -> iox::vector<Attribute::Value, IOX2_MAX_VALUES_PER_ATTRIBUTE_KEY> {
        IOX_TODO();
    }
};

inline auto operator<<(std::ostream& stream, const AttributeSet& value) -> std::ostream& {
    stream << "AttributeSet { }";
    return stream;
}

} // namespace iox2

#endif
