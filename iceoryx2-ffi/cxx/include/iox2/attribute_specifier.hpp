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

#ifndef IOX2_ATTRIBUTE_SPECIFIER_HPP
#define IOX2_ATTRIBUTE_SPECIFIER_HPP

#include "attribute_set.hpp"

namespace iox2 {

/// Represents the set of [`Attribute`]s that are defined when the [`Service`]
/// is created.
class AttributeSpecifier {
  public:
    /// Creates a new empty set of [`Attribute`]s
    AttributeSpecifier();
    AttributeSpecifier(const AttributeSpecifier&) = delete;
    AttributeSpecifier(AttributeSpecifier&&) noexcept;
    ~AttributeSpecifier();

    auto operator=(const AttributeSpecifier&) -> AttributeSpecifier& = delete;
    auto operator=(AttributeSpecifier&&) noexcept -> AttributeSpecifier&;

    /// Defines a value for a specific key. A key is allowed to have multiple values.
    auto define(const Attribute::Key& key, const Attribute::Value& value) -> AttributeSpecifier&&;

    /// Returns the underlying [`AttributeSetView`]
    auto attributes() const -> AttributeSetView;

  private:
    template <ServiceType>
    friend class ServiceBuilderEvent;
    template <typename, typename, ServiceType>
    friend class ServiceBuilderPublishSubscribe;
    template <typename, typename, typename, typename, ServiceType>
    friend class ServiceBuilderRequestResponse;

    void drop();

    iox2_attribute_specifier_h m_handle = nullptr;
};
} // namespace iox2

#endif
