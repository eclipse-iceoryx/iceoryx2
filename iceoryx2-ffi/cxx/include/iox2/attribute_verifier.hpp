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

#ifndef IOX2_ATTRIBUTE_VERIFIER_HPP_
#define IOX2_ATTRIBUTE_VERIFIER_HPP_

#include <iox/expected.hpp>
#include <string>
#include <vector>

#include "attribute_set.hpp"

namespace iox2 {
class AttributeVerifier {
   public:
    AttributeVerifier() = default;
    AttributeVerifier& require(const std::string& key,
                               const std::string& value) {}
    AttributeVerifier& require_key(const std::string& key) {}
    const AttributeSet& attributes() const {}
    std::vector<std::string> keys() const {}

    iox::expected<void, std::string> verify_requirements(
        const AttributeSet& rhs) const {}
};
}  // namespace iox2

#endif
