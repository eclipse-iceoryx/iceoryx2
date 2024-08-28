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

#ifndef IOX2_SERVICE_ID_HPP
#define IOX2_SERVICE_ID_HPP

#include "iox/assertions_addendum.hpp"

namespace iox2 {
/// Represents the unique if of a [`Service`].
class ServiceId {
  public:
    /// Returns the maximum string length of a [`ServiceId`]
    auto max_len() -> uint64_t {
        IOX_TODO();
    }

    /// Returns the string value of the [`ServiceId`]
    auto as_str() const -> const char* {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
