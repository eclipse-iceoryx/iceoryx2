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

#ifndef IOX2_ATTRIBUTE_SPECIFIER_HPP_
#define IOX2_ATTRIBUTE_SPECIFIER_HPP_

#include <string>

#include "attribute_set.hpp"
#include "iox/assertions_addendum.hpp"

namespace iox2 {
class AttributeSpecifier {
  public:
    AttributeSpecifier() = default;
    AttributeSpecifier& define(const std::string& key, const std::string& value) {
        IOX_TODO();
    }
    AttributeSet& attributes() const {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
