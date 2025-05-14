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

#include "iox/function.hpp"
#include "iox2/attribute.hpp"
#include "iox2/internal/iceoryx2.hpp"

#include <iostream>

namespace iox2 {
class AttributeSet;

/// Represents all service attributes. They can be set when the service is created.
///
/// @attention The parent from which the view was extracted MUST live longer than the
///            [`AttributeSetView`].
class AttributeSetView {
  public:
    /// Returns the number of [`Attribute`]s stored inside the [`AttributeSet`].
    auto number_of_attributes() const -> uint64_t;

    /// Returns a [`AttributeView`] at a specific index. The number of indices is returned via
    /// [`AttributeSetView::number_of_attributes()`].
    auto operator[](uint64_t index) const -> AttributeView;

    /// Returns the number of values stored under a specific key. If the key does not exist it
    /// returns 0.
    auto number_of_key_values(const Attribute::Key& key) const -> uint64_t;

    /// Returns a value of a key at a specific index. The index enumerates the values of the key
    /// if the key has multiple values. The values are always stored at the same position during
    /// the lifetime of the service but they can change when the process is recreated by another
    /// process when the system restarts.
    /// If the key does not exist or it does not have a value at the specified index, it returns
    /// [`None`].
    auto key_value(const Attribute::Key& key, uint64_t idx) -> iox::optional<Attribute::Value>;

    /// Returns all values to a specific key
    void iter_key_values(const Attribute::Key& key,
                         const iox::function<CallbackProgression(const Attribute::Value&)>& callback) const;

    /// Creates a copy of the [`AttributeSetView`] that owns the attributes.
    auto to_owned() const -> AttributeSet;

  private:
    template <ServiceType, typename, typename>
    friend class PortFactoryPublishSubscribe;
    template <ServiceType, typename, typename, typename, typename>
    friend class PortFactoryRequestResponse;
    template <ServiceType>
    friend class PortFactoryEvent;
    friend class AttributeVerifier;
    friend class AttributeSpecifier;
    friend class StaticConfig;
    friend class AttributeSet;

    explicit AttributeSetView(iox2_attribute_set_ptr handle);

    iox2_attribute_set_ptr m_handle = nullptr;
};

/// Represents all service attributes. They can be set when the service is created.
class AttributeSet {
  public:
    AttributeSet(const AttributeSet&) = delete;
    AttributeSet(AttributeSet&& rhs) noexcept;
    auto operator=(const AttributeSet&) -> AttributeSet& = delete;
    auto operator=(AttributeSet&& rhs) noexcept -> AttributeSet&;
    ~AttributeSet();

    /// Returns the number of [`Attribute`]s stored inside the [`AttributeSet`].
    auto number_of_attributes() const -> uint64_t;

    /// Returns a [`AttributeView`] at a specific index. The number of indices is returned via
    /// [`AttributeSet::len()`].
    auto operator[](uint64_t index) const -> AttributeView;

    /// Returns the number of values stored under a specific key. If the key does not exist it
    /// returns 0.
    auto number_of_key_values(const Attribute::Key& key) const -> uint64_t;

    /// Returns a value of a key at a specific index. The index enumerates the values of the key
    /// if the key has multiple values. The values are always stored at the same position during
    /// the lifetime of the service but they can change when the process is recreated by another
    /// process when the system restarts.
    /// If the key does not exist or it does not have a value at the specified index, it returns
    /// [`None`].
    auto key_value(const Attribute::Key& key, uint64_t idx) -> iox::optional<Attribute::Value>;

    /// Returns all values to a specific key
    void iter_key_values(const Attribute::Key& key,
                         const iox::function<CallbackProgression(const Attribute::Value&)>& callback) const;

  private:
    friend class AttributeSetView;

    explicit AttributeSet(iox2_attribute_set_h handle);
    void drop();

    iox2_attribute_set_h m_handle = nullptr;
    AttributeSetView m_view;
};

} // namespace iox2

auto operator<<(std::ostream& stream, const iox2::AttributeSetView& value) -> std::ostream&;
auto operator<<(std::ostream& stream, const iox2::AttributeSet& value) -> std::ostream&;

#endif
