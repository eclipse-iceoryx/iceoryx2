// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

#ifndef IOX2_SERVICE_BLACKBOARD_BUILDER_HPP
#define IOX2_SERVICE_BLACKBOARD_BUILDER_HPP

#include "iox/assertions_addendum.hpp"
#include "iox/builder_addendum.hpp"
#include "iox/expected.hpp"
#include "iox2/attribute_specifier.hpp"
#include "iox2/attribute_verifier.hpp"
#include "iox2/port_factory_blackboard.hpp"
#include "iox2/service_builder_blackboard_error.hpp"
#include "iox2/service_type.hpp"

#include <cstdint>

namespace iox2 {
/// Builder to create new [`MessagingPattern::Blackboard`] based [`Service`]s
template <typename KeyType, ServiceType S>
class ServiceBuilderBlackboardCreator {
  public:
    /// Defines how many [`Reader`]s shall be supported at most.
#ifdef DOXYGEN_MACRO_FIX
    auto max_readers(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, max_readers);
#endif

    /// Defines how many [`Node`]s shall be able to open it in parallel.
#ifdef DOXYGEN_MACRO_FIX
    auto max_nodes(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, max_nodes);
#endif

  public:
    /// Adds key-value pairs to the blackboard.
    template <typename ValueType>
    auto add(KeyType key, ValueType value) -> ServiceBuilderBlackboardCreator&&;

    /// Adds key-value pairs to the blackboard where value is a default value.
    template <typename ValueType>
    auto add_with_default(KeyType key) -> ServiceBuilderBlackboardCreator&&;

    /// Creates a new [`Service`].
    auto create() && -> iox::expected<PortFactoryBlackboard<S, KeyType>, BlackboardCreateError>;

    /// Creates a new [`Service`] with a set of attributes.
    auto
    create_with_attributes(const AttributeSpecifier& attributes) && -> iox::expected<PortFactoryBlackboard<S, KeyType>,
                                                                                     BlackboardCreateError>;

  private:
    template <ServiceType>
    friend class ServiceBuilder;

    explicit ServiceBuilderBlackboardCreator(iox2_service_builder_h handle);

    void set_parameters();

    // iox2_service_builder_blackboard_creator_h m_handle = nullptr;
};

template <typename KeyType, ServiceType S>
class ServiceBuilderBlackboardOpener {
  public:
    /// Defines how many [`Reader`]s must be at least supported.
#ifdef DOXYGEN_MACRO_FIX
    auto max_readers(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, max_readers);
#endif

    /// Defines how many [`Node`]s must be at least supported.
#ifdef DOXYGEN_MACRO_FIX
    auto max_nodes(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, max_nodes);
#endif

  public:
    /// Opens an existing [`Service`].
    auto open() && -> iox::expected<PortFactoryBlackboard<S, KeyType>, BlackboardOpenError>;

    /// Opens an existing [`Service`] with attribute requirements. If the defined attribute
    /// requirements are not satisfied the open process will fail.
    auto open_with_attributes(
        const AttributeVerifier&
            required_attributes) && -> iox::expected<PortFactoryBlackboard<S, KeyType>, BlackboardOpenError>;

  private:
    template <ServiceType>
    friend class ServiceBuilder;

    explicit ServiceBuilderBlackboardOpener(iox2_service_builder_h handle);

    void set_parameters();

    // iox2_service_builder_blackboard_opener_h m_handle = nullptr;
};

template <typename KeyType, ServiceType S>
template <typename ValueType>
inline auto ServiceBuilderBlackboardCreator<KeyType, S>::add([[maybe_unused]] KeyType key,
                                                             [[maybe_unused]] ValueType value)
    -> ServiceBuilderBlackboardCreator&& {
    IOX_TODO();
}

template <typename KeyType, ServiceType S>
template <typename ValueType>
inline auto ServiceBuilderBlackboardCreator<KeyType, S>::add_with_default([[maybe_unused]] KeyType key)
    -> ServiceBuilderBlackboardCreator&& {
    IOX_TODO();
}

template <typename KeyType, ServiceType S>
inline auto ServiceBuilderBlackboardCreator<KeyType, S>::create() && -> iox::expected<PortFactoryBlackboard<S, KeyType>,
                                                                                      BlackboardCreateError> {
    IOX_TODO();
}

template <typename KeyType, ServiceType S>
inline auto ServiceBuilderBlackboardCreator<KeyType, S>::create_with_attributes(
    [[maybe_unused]] const AttributeSpecifier&
        attributes) && -> iox::expected<PortFactoryBlackboard<S, KeyType>, BlackboardCreateError> {
    IOX_TODO();
}

template <typename KeyType, ServiceType S>
inline auto ServiceBuilderBlackboardOpener<KeyType, S>::open() && -> iox::expected<PortFactoryBlackboard<S, KeyType>,
                                                                                   BlackboardOpenError> {
    IOX_TODO();
}

template <typename KeyType, ServiceType S>
inline auto ServiceBuilderBlackboardOpener<KeyType, S>::open_with_attributes(
    [[maybe_unused]] const AttributeVerifier&
        required_attributes) && -> iox::expected<PortFactoryBlackboard<S, KeyType>, BlackboardOpenError> {
    IOX_TODO();
}
} // namespace iox2

#endif
