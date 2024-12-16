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

#include "iox/expected.hpp"
#include "iox/vector.hpp"
#include "iox2/attribute.hpp"
#include "iox2/attribute_set.hpp"
#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {
class AttributeVerifier {
  public:
    AttributeVerifier();
    AttributeVerifier(const AttributeVerifier&) = delete;
    AttributeVerifier(AttributeVerifier&&) noexcept;
    ~AttributeVerifier();

    auto operator=(const AttributeVerifier&) -> AttributeVerifier& = delete;
    auto operator=(AttributeVerifier&&) noexcept -> AttributeVerifier&;

    auto require(const Attribute::Key& key, const Attribute::Value& value) -> AttributeVerifier&&;
    auto require_key(const Attribute::Key& key) -> AttributeVerifier&&;
    auto attributes() const -> AttributeSetView;
    auto keys() const -> iox::vector<Attribute::Key, IOX2_MAX_ATTRIBUTES_PER_SERVICE>;
    auto verify_requirements(const AttributeSetView& rhs) const -> iox::expected<void, Attribute::Key>;

  private:
    template <ServiceType>
    friend class ServiceBuilderEvent;
    template <typename, typename, ServiceType>
    friend class ServiceBuilderPublishSubscribe;

    void drop();

    iox2_attribute_verifier_h m_handle;
};
} // namespace iox2

#endif
