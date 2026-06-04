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

#ifndef IOX2_SERVICE_BUILDER_PUBLISH_SUBSCRIBE_HPP
#define IOX2_SERVICE_BUILDER_PUBLISH_SUBSCRIBE_HPP

#include "iox2/attribute_specifier.hpp"
#include "iox2/attribute_verifier.hpp"
#include "iox2/bb/detail/builder.hpp"
#include "iox2/bb/expected.hpp"
#include "iox2/bb/layout.hpp"
#include "iox2/bb/slice.hpp"
#include "iox2/custom_header_marker.hpp"
#include "iox2/custom_payload_marker.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/internal/service_builder_internal.hpp"
#include "iox2/message_type_details.hpp"
#include "iox2/payload_info.hpp"
#include "iox2/port_factory_publish_subscribe.hpp"
#include "iox2/service_builder_publish_subscribe_error.hpp"
#include "iox2/service_type.hpp"
#include "iox2/type_name.hpp"
#include "iox2/type_variant.hpp"

#include <cstring>
#include <type_traits>

namespace iox2 {
/// Builder to create new [`MessagingPattern::PublishSubscribe`] based [`Service`]s
template <typename Payload, typename UserHeader, ServiceType S>
class ServiceBuilderPublishSubscribe {
  public:
    /// If the [`Service`] is created, it defines the [`Alignment`] of the payload for the service. If
    /// an existing [`Service`] is opened it requires the service to have at least the defined
    /// [`Alignment`]. If the Payload [`Alignment`] is greater than the provided [`Alignment`]
    /// then the Payload [`Alignment`] is used.
#ifdef DOXYGEN_MACRO_FIX
    auto payload_alignment(const uint64_t value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(uint64_t, payload_alignment);
#endif

    /// If the [`Service`] is created, defines the overflow behavior of the service. If an existing
    /// [`Service`] is opened it requires the service to have the defined overflow behavior.
#ifdef DOXYGEN_MACRO_FIX
    auto enable_safe_overflow(const bool value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(bool, enable_safe_overflow);
#endif

    /// If the [`Service`] is created it defines how many [`Sample`]s a
    /// [`Subscriber`] can borrow at most in parallel. If an existing
    /// [`Service`] is opened it defines the minimum required.
#ifdef DOXYGEN_MACRO_FIX
    auto subscriber_max_borrowed_samples(const uint64_t value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(uint64_t, subscriber_max_borrowed_samples);
#endif

    /// If the [`Service`] is created it defines the maximum history size a
    /// [`Subscriber`] can request on connection. If an existing
    /// [`Service`] is opened it defines the minimum required.
#ifdef DOXYGEN_MACRO_FIX
    auto history_size(const uint64_t value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(uint64_t, history_size);
#endif

    /// If the [`Service`] is created it defines how many [`Sample`] a `Subscriber` can store in
    /// its internal buffer. If an existing [`Service`] is opened it defines the minimum required.
#ifdef DOXYGEN_MACRO_FIX
    auto subscriber_max_buffer_size(const uint64_t value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(uint64_t, subscriber_max_buffer_size);
#endif

    /// If the [`Service`] is created it defines how many [`Subscriber`] shall be supported at
    /// most. If an existing [`Service`] is opened it defines how many [`Subscriber`] must be at
    /// least supported.
#ifdef DOXYGEN_MACRO_FIX
    auto max_subscribers(const uint64_t value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(uint64_t, max_subscribers);
#endif

    /// If the [`Service`] is created it defines how many [`Publisher`] shall be supported at
    /// most. If an existing [`Service`] is opened it defines how many [`Publisher`] must be at
    /// least supported.
#ifdef DOXYGEN_MACRO_FIX
    auto max_publishers(const uint64_t value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(uint64_t, max_publishers);
#endif

    /// If the [`Service`] is created it defines how many [`Node`]s shall be able to open it in
    /// parallel. If an existing [`Service`] is opened it defines how many [`Node`]s must be at
    /// least supported.
#ifdef DOXYGEN_MACRO_FIX
    auto max_nodes(const uint64_t value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(uint64_t, max_nodes);
#endif

  public:
    /// Sets the user header type of the [`Service`].
    template <typename NewHeader>
    auto user_header() && -> ServiceBuilderPublishSubscribe<Payload, NewHeader, S>&&;

    /// Overrides the user header type details with values provided at runtime instead of derived
    /// from the compile-time `UserHeader`. Only available for `CustomHeaderMarker`.
    template <typename H = UserHeader, typename = std::enable_if_t<std::is_same<H, CustomHeaderMarker>::value>>
    auto set_user_header_type_details(
        const TypeDetail& value) && -> ServiceBuilderPublishSubscribe<Payload, UserHeader, S>&&;

    /// Overrides the payload type details with values provided at runtime instead of derived
    /// from the compile-time `Payload`. Only available for `bb::Slice<CustomPayloadMarker>`.
    template <typename P = Payload, typename = std::enable_if_t<std::is_same<P, bb::Slice<CustomPayloadMarker>>::value>>
    auto
    set_payload_type_details(const TypeDetail& value) && -> ServiceBuilderPublishSubscribe<Payload, UserHeader, S>&&;

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created.
    auto open_or_create() && -> bb::Expected<PortFactoryPublishSubscribe<S, Payload, UserHeader>,
                                             PublishSubscribeOpenOrCreateError>;

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created. It defines a set of attributes. If the [`Service`] already exists all attribute
    /// requirements must be satisfied otherwise the open process will fail. If the [`Service`]
    /// does not exist the required attributes will be defined in the [`Service`].
    auto open_or_create_with_attributes(const AttributeVerifier& required_attributes) && -> bb::
        Expected<PortFactoryPublishSubscribe<S, Payload, UserHeader>, PublishSubscribeOpenOrCreateError>;

    /// Opens an existing [`Service`].
    auto open() && -> bb::Expected<PortFactoryPublishSubscribe<S, Payload, UserHeader>, PublishSubscribeOpenError>;

    /// Opens an existing [`Service`] with attribute requirements. If the defined attribute
    /// requirements are not satisfied the open process will fail.
    auto open_with_attributes(const AttributeVerifier& required_attributes) && -> bb::
        Expected<PortFactoryPublishSubscribe<S, Payload, UserHeader>, PublishSubscribeOpenError>;

    /// Creates a new [`Service`].
    auto create() && -> bb::Expected<PortFactoryPublishSubscribe<S, Payload, UserHeader>, PublishSubscribeCreateError>;

    /// Creates a new [`Service`] with a set of attributes.
    auto create_with_attributes(
        const AttributeSpecifier& attributes) && -> bb::Expected<PortFactoryPublishSubscribe<S, Payload, UserHeader>,
                                                                 PublishSubscribeCreateError>;

  private:
    template <ServiceType>
    friend class ServiceBuilder;

    explicit ServiceBuilderPublishSubscribe(iox2_service_builder_h handle);

    void set_parameters();

    iox2_service_builder_pub_sub_h m_handle = nullptr;
    bb::Optional<TypeDetail> m_user_header_type_details_override;
    bb::Optional<TypeDetail> m_payload_type_details_override;
};

template <typename Payload, typename UserHeader, ServiceType S>
inline ServiceBuilderPublishSubscribe<Payload, UserHeader, S>::ServiceBuilderPublishSubscribe(
    iox2_service_builder_h handle)
    : m_handle { iox2_service_builder_pub_sub(handle) } {
}

template <typename Payload, typename UserHeader, ServiceType S>
inline void ServiceBuilderPublishSubscribe<Payload, UserHeader, S>::set_parameters() {
    if (m_enable_safe_overflow.has_value()) {
        iox2_service_builder_pub_sub_set_enable_safe_overflow(&m_handle, m_enable_safe_overflow.value());
    }
    if (m_subscriber_max_borrowed_samples.has_value()) {
        iox2_service_builder_pub_sub_set_subscriber_max_borrowed_samples(&m_handle,
                                                                         m_subscriber_max_borrowed_samples.value());
    }
    if (m_history_size.has_value()) {
        iox2_service_builder_pub_sub_set_history_size(&m_handle, m_history_size.value());
    }
    if (m_subscriber_max_buffer_size.has_value()) {
        iox2_service_builder_pub_sub_set_subscriber_max_buffer_size(&m_handle, m_subscriber_max_buffer_size.value());
    }
    if (m_max_subscribers.has_value()) {
        iox2_service_builder_pub_sub_set_max_subscribers(&m_handle, m_max_subscribers.value());
    }
    if (m_max_publishers.has_value()) {
        iox2_service_builder_pub_sub_set_max_publishers(&m_handle, m_max_publishers.value());
    }
    if (m_payload_alignment.has_value()) {
        iox2_service_builder_pub_sub_set_payload_alignment(&m_handle, m_payload_alignment.value());
    }
    if (m_max_nodes.has_value()) {
        iox2_service_builder_pub_sub_set_max_nodes(&m_handle, m_max_nodes.value());
    }

    using ValueType = typename PayloadInfo<Payload>::ValueType;

    // user header type details, derived from the compile-time UserHeader unless overridden at runtime
    const auto header_layout = bb::Layout::from<UserHeader>();
    const auto derived_user_header_type_name = internal::get_type_name<UserHeader>();
    auto user_header_type_variant = iox2_type_variant_e_FIXED_SIZE;
    const char* user_header_type_name = derived_user_header_type_name.unchecked_access().c_str();
    uint64_t user_header_type_name_len = derived_user_header_type_name.size();
    uint64_t user_header_type_size = header_layout.size();
    uint64_t user_header_type_align = header_layout.alignment();

    if (m_user_header_type_details_override.has_value()) {
        const auto& header_details_override = m_user_header_type_details_override.value();
        user_header_type_variant = header_details_override.variant() == TypeVariant::FixedSize
                                       ? iox2_type_variant_e_FIXED_SIZE
                                       : iox2_type_variant_e_DYNAMIC;
        user_header_type_name = header_details_override.type_name();
        user_header_type_name_len = std::strlen(header_details_override.type_name());
        user_header_type_size = header_details_override.size();
        user_header_type_align = header_details_override.alignment();
    }

    // payload type details, derived from the compile-time Payload unless overridden at runtime
    const auto derived_type_name = internal::get_type_name<Payload>();
    auto type_variant = bb::IsSlice<Payload>::VALUE ? iox2_type_variant_e_DYNAMIC : iox2_type_variant_e_FIXED_SIZE;
    const char* payload_type_name = derived_type_name.unchecked_access().c_str();
    uint64_t payload_type_name_len = derived_type_name.size();
    uint64_t payload_type_size = sizeof(ValueType);
    uint64_t payload_type_align = alignof(ValueType);

    if (m_payload_type_details_override.has_value()) {
        const auto& payload_details_override = m_payload_type_details_override.value();
        type_variant = payload_details_override.variant() == TypeVariant::FixedSize ? iox2_type_variant_e_FIXED_SIZE
                                                                                    : iox2_type_variant_e_DYNAMIC;
        payload_type_name = payload_details_override.type_name();
        payload_type_name_len = std::strlen(payload_details_override.type_name());
        payload_type_size = payload_details_override.size();
        payload_type_align = payload_details_override.alignment();
    }

    const auto payload_result = iox2_service_builder_pub_sub_set_payload_type_details(
        &m_handle, type_variant, payload_type_name, payload_type_name_len, payload_type_size, payload_type_align);

    if (payload_result != IOX2_OK) {
        IOX2_PANIC("This should never happen! Implementation failure while setting the Payload-Type.");
    }

    const auto user_header_result = iox2_service_builder_pub_sub_set_user_header_type_details(&m_handle,
                                                                                              user_header_type_variant,
                                                                                              user_header_type_name,
                                                                                              user_header_type_name_len,
                                                                                              user_header_type_size,
                                                                                              user_header_type_align);

    if (user_header_result != IOX2_OK) {
        IOX2_PANIC("This should never happen! Implementation failure while setting the User-Header-Type.");
    }
}

template <typename Payload, typename UserHeader, ServiceType S>
template <typename NewHeader>
inline auto ServiceBuilderPublishSubscribe<Payload, UserHeader, S>::
    user_header() && -> ServiceBuilderPublishSubscribe<Payload, NewHeader, S>&& {
    // required here since we just change the template header type but the builder structure stays the same
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
    return std::move(*reinterpret_cast<ServiceBuilderPublishSubscribe<Payload, NewHeader, S>*>(this));
}

template <typename Payload, typename UserHeader, ServiceType S>
template <typename, typename>
inline auto ServiceBuilderPublishSubscribe<Payload, UserHeader, S>::set_user_header_type_details(
    const TypeDetail& value) && -> ServiceBuilderPublishSubscribe<Payload, UserHeader, S>&& {
    m_user_header_type_details_override = bb::Optional<TypeDetail>(value);
    return std::move(*this);
}

template <typename Payload, typename UserHeader, ServiceType S>
template <typename, typename>
inline auto ServiceBuilderPublishSubscribe<Payload, UserHeader, S>::set_payload_type_details(
    const TypeDetail& value) && -> ServiceBuilderPublishSubscribe<Payload, UserHeader, S>&& {
    m_payload_type_details_override = bb::Optional<TypeDetail>(value);
    return std::move(*this);
}

template <typename Payload, typename UserHeader, ServiceType S>
inline auto ServiceBuilderPublishSubscribe<Payload, UserHeader, S>::open_or_create() && -> bb::
    Expected<PortFactoryPublishSubscribe<S, Payload, UserHeader>, PublishSubscribeOpenOrCreateError> {
    set_parameters();

    iox2_port_factory_pub_sub_h port_factory_handle {};
    auto result = iox2_service_builder_pub_sub_open_or_create(m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return PortFactoryPublishSubscribe<S, Payload, UserHeader>(port_factory_handle);
    }

    return bb::err(bb::into<PublishSubscribeOpenOrCreateError>(result));
}

template <typename Payload, typename UserHeader, ServiceType S>
inline auto ServiceBuilderPublishSubscribe<Payload, UserHeader, S>::open() && -> bb::
    Expected<PortFactoryPublishSubscribe<S, Payload, UserHeader>, PublishSubscribeOpenError> {
    set_parameters();

    iox2_port_factory_pub_sub_h port_factory_handle {};
    auto result = iox2_service_builder_pub_sub_open(m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return PortFactoryPublishSubscribe<S, Payload, UserHeader>(port_factory_handle);
    }

    return bb::err(bb::into<PublishSubscribeOpenError>(result));
}

template <typename Payload, typename UserHeader, ServiceType S>
inline auto ServiceBuilderPublishSubscribe<Payload, UserHeader, S>::create() && -> bb::
    Expected<PortFactoryPublishSubscribe<S, Payload, UserHeader>, PublishSubscribeCreateError> {
    set_parameters();

    iox2_port_factory_pub_sub_h port_factory_handle {};
    auto result = iox2_service_builder_pub_sub_create(m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return PortFactoryPublishSubscribe<S, Payload, UserHeader>(port_factory_handle);
    }

    return bb::err(bb::into<PublishSubscribeCreateError>(result));
}

template <typename Payload, typename UserHeader, ServiceType S>
inline auto ServiceBuilderPublishSubscribe<Payload, UserHeader, S>::open_or_create_with_attributes(
    const AttributeVerifier&
        required_attributes) && -> bb::Expected<PortFactoryPublishSubscribe<S, Payload, UserHeader>,
                                                PublishSubscribeOpenOrCreateError> {
    set_parameters();

    iox2_port_factory_pub_sub_h port_factory_handle {};
    auto result = iox2_service_builder_pub_sub_open_or_create_with_attributes(
        m_handle, &required_attributes.m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return PortFactoryPublishSubscribe<S, Payload, UserHeader>(port_factory_handle);
    }

    return bb::err(bb::into<PublishSubscribeOpenOrCreateError>(result));
}

template <typename Payload, typename UserHeader, ServiceType S>
inline auto ServiceBuilderPublishSubscribe<Payload, UserHeader, S>::open_with_attributes(
    const AttributeVerifier&
        required_attributes) && -> bb::Expected<PortFactoryPublishSubscribe<S, Payload, UserHeader>,
                                                PublishSubscribeOpenError> {
    set_parameters();

    iox2_port_factory_pub_sub_h port_factory_handle {};
    auto result = iox2_service_builder_pub_sub_open_with_attributes(
        m_handle, &required_attributes.m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return PortFactoryPublishSubscribe<S, Payload, UserHeader>(port_factory_handle);
    }

    return bb::err(bb::into<PublishSubscribeOpenError>(result));
}

template <typename Payload, typename UserHeader, ServiceType S>
inline auto ServiceBuilderPublishSubscribe<Payload, UserHeader, S>::create_with_attributes(
    const AttributeSpecifier& attributes) && -> bb::Expected<PortFactoryPublishSubscribe<S, Payload, UserHeader>,
                                                             PublishSubscribeCreateError> {
    set_parameters();

    iox2_port_factory_pub_sub_h port_factory_handle {};
    auto result = iox2_service_builder_pub_sub_create_with_attributes(
        m_handle, &attributes.m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return PortFactoryPublishSubscribe<S, Payload, UserHeader>(port_factory_handle);
    }

    return bb::err(bb::into<PublishSubscribeCreateError>(result));
}
} // namespace iox2

#endif
