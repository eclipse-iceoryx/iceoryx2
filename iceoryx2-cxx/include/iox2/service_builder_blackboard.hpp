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

#include "iox/builder_addendum.hpp"
#include "iox/expected.hpp"
#include "iox2/attribute_specifier.hpp"
#include "iox2/attribute_verifier.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/port_factory_blackboard.hpp"
#include "iox2/service_builder_blackboard_error.hpp"
#include "iox2/service_type.hpp"

#include <cstdint>
#include <type_traits>

namespace iox2 {
/// Builder to create new [`MessagingPattern::Blackboard`] based [`Service`]s
template <typename KeyType, ServiceType S>
class ServiceBuilderBlackboardCreator {
  public:
    static_assert(std::is_trivially_copyable_v<KeyType>, "The blackboard supports only trivially copyable key types.");
    static_assert(std::alignment_of<KeyType>() <= IOX2_MAX_BLACKBOARD_KEY_ALIGNMENT,
                  "The blackboard supports only key types with an alignment <= IOX2_MAX_BLACKBOARD_KEY_ALIGNMENT.");
    static_assert(sizeof(KeyType) <= IOX2_MAX_BLACKBOARD_KEY_SIZE,
                  "The blackboard supports only key types with a size <= IOX2_MAX_BLACKBOARD_KEY_SIZE.");


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

    iox2_service_builder_blackboard_creator_h m_handle = nullptr;
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

    iox2_service_builder_blackboard_opener_h m_handle = nullptr;
};

namespace internal {
template <typename T>
auto default_key_eq_cmp_func(const void* lhs, const void* rhs) -> bool {
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast): C API requires to pass void* instead of T*
    return (*reinterpret_cast<const T*>(lhs)) == (*reinterpret_cast<const T*>(rhs));
}
} // namespace internal

template <typename KeyType, ServiceType S>
inline ServiceBuilderBlackboardCreator<KeyType, S>::ServiceBuilderBlackboardCreator(iox2_service_builder_h handle)
    : m_handle { iox2_service_builder_blackboard_creator(handle) } {
    // set key type details so that these are available in add()
    const auto type_name = internal::get_type_name<KeyType>();
    const auto key_type_result = iox2_service_builder_blackboard_creator_set_key_type_details(
        &m_handle, type_name.unchecked_access().c_str(), type_name.size(), sizeof(KeyType), alignof(KeyType));
    if (key_type_result != IOX2_OK) {
        IOX_PANIC("This should never happen! Implementation failure while setting the key type.");
    }
}

template <typename KeyType, ServiceType S>
inline void ServiceBuilderBlackboardCreator<KeyType, S>::set_parameters() {
    m_max_readers.and_then(
        [&](auto value) { iox2_service_builder_blackboard_creator_set_max_readers(&m_handle, value); });
    m_max_nodes.and_then([&](auto value) { iox2_service_builder_blackboard_creator_set_max_nodes(&m_handle, value); });

    // key eq comparison function
    iox2_service_builder_blackboard_creator_set_key_eq_comparison_function(&m_handle,
                                                                           internal::default_key_eq_cmp_func<KeyType>);
}

template <typename KeyType, ServiceType S>
template <typename ValueType>
inline auto ServiceBuilderBlackboardCreator<KeyType, S>::add(KeyType key, ValueType value)
    -> ServiceBuilderBlackboardCreator&& {
    // NOLINTNEXTLINE(cppcoreguidelines-owning-memory): required by C API
    auto value_ptr = new ValueType(value);
    const auto type_name = internal::get_type_name<ValueType>();

    iox2_service_builder_blackboard_creator_add(
        &m_handle,
        &key,
        value_ptr,
        [](void* value) {
            auto* value_ptr = static_cast<ValueType*>(value);
            if (value_ptr != nullptr) {
                // NOLINTNEXTLINE(cppcoreguidelines-owning-memory): required by C API
                delete value_ptr;
                value_ptr = nullptr;
            }
        },
        type_name.unchecked_access().c_str(),
        type_name.size(),
        sizeof(ValueType),
        alignof(ValueType));

    return std::move(*this);
}

template <typename KeyType, ServiceType S>
template <typename ValueType>
inline auto ServiceBuilderBlackboardCreator<KeyType, S>::add_with_default(KeyType key)
    -> ServiceBuilderBlackboardCreator&& {
    return add(key, ValueType());
}

template <typename KeyType, ServiceType S>
inline auto ServiceBuilderBlackboardCreator<KeyType, S>::create() && -> iox::expected<PortFactoryBlackboard<S, KeyType>,
                                                                                      BlackboardCreateError> {
    set_parameters();

    iox2_port_factory_blackboard_h port_factory_handle {};
    auto result = iox2_service_builder_blackboard_create(m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return iox::ok(PortFactoryBlackboard<S, KeyType>(port_factory_handle));
    }

    return iox::err(iox::into<BlackboardCreateError>(result));
}

template <typename KeyType, ServiceType S>
inline auto ServiceBuilderBlackboardCreator<KeyType, S>::create_with_attributes(
    const AttributeSpecifier&
        attributes) && -> iox::expected<PortFactoryBlackboard<S, KeyType>, BlackboardCreateError> {
    set_parameters();

    iox2_port_factory_blackboard_h port_factory_handle {};
    auto result = iox2_service_builder_blackboard_create_with_attributes(
        m_handle, &attributes.m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return iox::ok(PortFactoryBlackboard<S, KeyType>(port_factory_handle));
    }

    return iox::err(iox::into<BlackboardCreateError>(result));
}

template <typename KeyType, ServiceType S>
inline ServiceBuilderBlackboardOpener<KeyType, S>::ServiceBuilderBlackboardOpener(iox2_service_builder_h handle)
    : m_handle { iox2_service_builder_blackboard_opener(handle) } {
}

template <typename KeyType, ServiceType S>
inline void ServiceBuilderBlackboardOpener<KeyType, S>::set_parameters() {
    m_max_readers.and_then(
        [&](auto value) { iox2_service_builder_blackboard_opener_set_max_readers(&m_handle, value); });
    m_max_nodes.and_then([&](auto value) { iox2_service_builder_blackboard_opener_set_max_nodes(&m_handle, value); });

    // key type details
    const auto type_name = internal::get_type_name<KeyType>();
    const auto key_type_result = iox2_service_builder_blackboard_opener_set_key_type_details(
        &m_handle, type_name.unchecked_access().c_str(), type_name.size(), sizeof(KeyType), alignof(KeyType));
    if (key_type_result != IOX2_OK) {
        IOX_PANIC("This should never happen! Implementation failure while setting the Key-Type.");
    }
}

template <typename KeyType, ServiceType S>
inline auto ServiceBuilderBlackboardOpener<KeyType, S>::open() && -> iox::expected<PortFactoryBlackboard<S, KeyType>,
                                                                                   BlackboardOpenError> {
    set_parameters();

    iox2_port_factory_blackboard_h port_factory_handle {};
    auto result = iox2_service_builder_blackboard_open(m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return iox::ok(PortFactoryBlackboard<S, KeyType>(port_factory_handle));
    }

    return iox::err(iox::into<BlackboardOpenError>(result));
}

template <typename KeyType, ServiceType S>
inline auto ServiceBuilderBlackboardOpener<KeyType, S>::open_with_attributes(
    const AttributeVerifier&
        required_attributes) && -> iox::expected<PortFactoryBlackboard<S, KeyType>, BlackboardOpenError> {
    set_parameters();

    iox2_port_factory_blackboard_h port_factory_handle {};
    auto result = iox2_service_builder_blackboard_open_with_attributes(
        m_handle, &required_attributes.m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return iox::ok(PortFactoryBlackboard<S, KeyType>(port_factory_handle));
    }

    return iox::err(iox::into<BlackboardOpenError>(result));
}
} // namespace iox2

#endif
