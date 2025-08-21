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

#ifndef IOX2_SERVICE_BUILDER_INTERNAL_HPP
#define IOX2_SERVICE_BUILDER_INTERNAL_HPP

#include "iox/slice.hpp"
#include "iox2/payload_info.hpp"

#include <typeinfo>

namespace iox2::internal {
template <typename Payload, typename = void>
struct HasPayloadTypeNameMember : std::false_type { };

template <typename Payload>
struct HasPayloadTypeNameMember<Payload, decltype((void) Payload::IOX2_TYPE_NAME)> : std::true_type { };

template <typename Payload>
using FromCustomizedPayloadTypeName = std::enable_if_t<HasPayloadTypeNameMember<Payload>::value, const char*>;

template <typename Payload>
using FromNonSlice =
    std::enable_if_t<!HasPayloadTypeNameMember<Payload>::value && !iox::IsSlice<Payload>::VALUE, const char*>;

template <typename Payload>
using FromSliceWithCustomizedInnerPayloadTypeName =
    std::enable_if_t<!HasPayloadTypeNameMember<Payload>::value && iox::IsSlice<Payload>::VALUE
                         && HasPayloadTypeNameMember<typename Payload::ValueType>::value,
                     const char*>;

template <typename Payload>
using FromSliceWithoutCustomizedInnerPayloadTypeName =
    std::enable_if_t<!HasPayloadTypeNameMember<Payload>::value && iox::IsSlice<Payload>::VALUE
                         && !HasPayloadTypeNameMember<typename Payload::ValueType>::value,
                     const char*>;

template <typename UserHeader, typename = void>
struct HasUserHeaderTypeNameMember : std::false_type { };

template <typename UserHeader>
struct HasUserHeaderTypeNameMember<UserHeader, decltype((void) UserHeader::IOX2_TYPE_NAME)> : std::true_type { };

template <typename PayloadType>
auto get_payload_type_name() -> internal::FromCustomizedPayloadTypeName<PayloadType> {
    return PayloadType::IOX2_TYPE_NAME;
}

// NOLINTBEGIN(readability-function-size) : template alternative is less readable
template <typename PayloadType>
auto get_payload_type_name() -> internal::FromNonSlice<PayloadType> {
    if (std::is_same_v<PayloadType, uint8_t>) {
        return "u8";
    }
    if (std::is_same_v<PayloadType, uint16_t>) {
        return "u16";
    }
    if (std::is_same_v<PayloadType, uint32_t>) {
        return "u32";
    }
    if (std::is_same_v<PayloadType, uint64_t>) {
        return "u64";
    }
    if (std::is_same_v<PayloadType, int8_t>) {
        return "i8";
    }
    if (std::is_same_v<PayloadType, int16_t>) {
        return "i16";
    }
    if (std::is_same_v<PayloadType, int32_t>) {
        return "i32";
    }
    if (std::is_same_v<PayloadType, int64_t>) {
        return "i64";
    }
    if (std::is_same_v<PayloadType, float>) {
        return "f32";
    }
    if (std::is_same_v<PayloadType, double>) {
        return "f64";
    }
    if (std::is_same_v<PayloadType, bool>) {
        return "bool";
    }
    return typeid(typename PayloadInfo<PayloadType>::ValueType).name();
}
// NOLINTEND(readability-function-size)

template <typename PayloadType>
auto get_payload_type_name() -> internal::FromSliceWithCustomizedInnerPayloadTypeName<PayloadType> {
    return PayloadType::ValueType::IOX2_TYPE_NAME;
}

template <typename PayloadType>
auto get_payload_type_name() -> internal::FromSliceWithoutCustomizedInnerPayloadTypeName<PayloadType> {
    return get_payload_type_name<typename PayloadType::ValueType>();
}

template <typename UserHeaderType>
auto get_user_header_type_name() ->
    typename std::enable_if_t<internal::HasUserHeaderTypeNameMember<UserHeaderType>::value, const char*> {
    return UserHeaderType::IOX2_TYPE_NAME;
}

template <typename UserHeaderType>
auto get_user_header_type_name() ->
    typename std::enable_if_t<!internal::HasUserHeaderTypeNameMember<UserHeaderType>::value, const char*> {
    if (std::is_void_v<UserHeaderType>) {
        return "()"; // no user header provided
    }
    return typeid(UserHeaderType).name();
}
} // namespace iox2::internal

#endif
