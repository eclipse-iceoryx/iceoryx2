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

#include "iox/assertions_addendum.hpp"
#include "iox/expected.hpp"
#include "iox/vector.hpp"
#include "iox2/attribute.hpp"
#include "iox2/attribute_set.hpp"
#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {
class AttributeVerifier {
  public:
    AttributeVerifier() = default;
    auto require(const Attribute::Key& key, const Attribute::Value& value) -> AttributeVerifier& {
        IOX_TODO();
    }
    auto require_key(const Attribute::Key& key) -> AttributeVerifier& {
        IOX_TODO();
    }
    auto attributes() const -> const AttributeSet& {
        IOX_TODO();
    }
    auto keys() const -> iox::vector<Attribute::Key, IOX2_MAX_ATTRIBUTES_PER_SERVICE> {
        IOX_TODO();
    }

    auto verify_requirements(const AttributeSet& rhs) const -> iox::expected<void, Attribute::Key> {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
