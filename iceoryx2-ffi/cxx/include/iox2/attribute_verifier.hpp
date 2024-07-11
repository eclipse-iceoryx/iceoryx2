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

#ifndef IOX2_ATTRIBUTE_VERIFIER_HPP
#define IOX2_ATTRIBUTE_VERIFIER_HPP

#include "attribute_set.hpp"
#include "iox/assertions_addendum.hpp"

#include <iox/expected.hpp>
#include <string>
#include <vector>

namespace iox2 {
class AttributeVerifier {
  public:
    AttributeVerifier() = default;
    auto require(const std::string& key, const std::string& value) -> AttributeVerifier& {
        IOX_TODO();
    }
    auto require_key(const std::string& key) -> AttributeVerifier& {
        IOX_TODO();
    }
    auto attributes() const -> const AttributeSet& {
        IOX_TODO();
    }
    auto keys() const -> std::vector<std::string> {
        IOX_TODO();
    }

    auto verify_requirements(const AttributeSet& rhs) const -> iox::expected<void, std::string> {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
