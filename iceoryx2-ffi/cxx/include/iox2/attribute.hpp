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

#ifndef IOX2_ATTRIBUTE_HPP
#define IOX2_ATTRIBUTE_HPP

#include "iox/assertions_addendum.hpp"
#include "iox/string.hpp"
#include "iox2/internal/iceoryx2.hpp"

#include <string>

namespace iox2 {
class Attribute {
  public:
    using Key = iox::string<IOX2_ATTRIBUTE_KEY_LENGTH>;
    using Value = iox::string<IOX2_ATTRIBUTE_VALUE_LENGTH>;

    auto key() const -> Key {
        IOX_TODO();
    }
    auto value() const -> Value {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
