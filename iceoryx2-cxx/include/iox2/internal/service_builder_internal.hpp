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

#include "iox2/bb/slice.hpp"
#include "iox2/bb/static_string.hpp"
#include "iox2/bb/static_vector.hpp"
#include "iox2/payload_info.hpp"
#include "iox2/type_name.hpp"

#include <cstdio>
#include <typeinfo>

namespace iox2 {
namespace internal {

template <typename>
auto get_type_name() -> TypeName;

template <typename Payload, typename = void>
struct HasPayloadTypeNameMember : std::false_type { };

template <typename Payload>
struct HasPayloadTypeNameMember<Payload, decltype((void) Payload::IOX2_TYPE_NAME)> : std::true_type { };

template <typename Payload>
using FromCustomizedPayloadTypeName = std::enable_if_t<HasPayloadTypeNameMember<Payload>::value, TypeName>;

template <typename Payload>
using FromNonSlice =
    std::enable_if_t<!HasPayloadTypeNameMember<Payload>::value && !bb::IsSlice<Payload>::VALUE
                         && !iox2::bb::IsStaticVector<Payload>::value && !iox2::bb::IsStaticString<Payload>::value,
                     TypeName>;

template <typename Payload>
using FromStaticVector = std::enable_if_t<iox2::bb::IsStaticVector<Payload>::value, TypeName>;

template <typename Payload>
using FromStaticString = std::enable_if_t<iox2::bb::IsStaticString<Payload>::value, TypeName>;

template <typename Payload>
using FromSliceWithCustomizedInnerPayloadTypeName =
    std::enable_if_t<!HasPayloadTypeNameMember<Payload>::value && bb::IsSlice<Payload>::VALUE
                         && HasPayloadTypeNameMember<typename Payload::ValueType>::value,
                     TypeName>;

template <typename Payload>
using FromSliceWithoutCustomizedInnerPayloadTypeName =
    std::enable_if_t<!HasPayloadTypeNameMember<Payload>::value && bb::IsSlice<Payload>::VALUE
                         && !HasPayloadTypeNameMember<typename Payload::ValueType>::value,
                     TypeName>;

template <typename PayloadType>
auto get_type_name_impl() -> internal::FromCustomizedPayloadTypeName<PayloadType> {
    return *TypeName::from_utf8_null_terminated_unchecked(PayloadType::IOX2_TYPE_NAME);
}

template <typename PayloadType>
auto get_type_name_impl() -> internal::FromStaticString<PayloadType> {
    // std::array is not available in this safety-critical context
    // NOLINTNEXTLINE
    char type_name[TypeName::capacity()] { 0 };
    // std::to_string() is not available in this safety-critical context
    // NOLINTNEXTLINE
    snprintf(&type_name[0],
             TypeName::capacity(),
             "iceoryx2_bb_container::string::static_string::StaticString<%llu>",
             static_cast<long long unsigned int>(PayloadType::capacity()));
    return *TypeName::from_utf8_null_terminated_unchecked(&type_name[0]);
}

template <typename PayloadType>
auto get_type_name_impl() -> internal::FromStaticVector<PayloadType> {
    // std::array is not available in this safety-critical context
    // NOLINTNEXTLINE
    char type_name[TypeName::capacity()] { 0 };
    // std::to_string() is not available in this safety-critical context
    // NOLINTNEXTLINE
    snprintf(&type_name[0],
             TypeName::capacity(),
             "iceoryx2_bb_container::vector::static_vec::StaticVec<%s, %llu>",
             get_type_name<typename PayloadType::ValueType>().unchecked_access().c_str(),
             static_cast<long long unsigned int>(PayloadType::capacity()));
    return *TypeName::from_utf8_null_terminated_unchecked(&type_name[0]);
}

// NOLINTBEGIN(readability-function-size) : template alternative is less readable
template <typename PayloadType>
auto get_type_name_impl() -> internal::FromNonSlice<PayloadType> {
    if (std::is_same<PayloadType, void>::value) {
        return *TypeName::from_utf8("()");
    }
    if (std::is_same<PayloadType, uint8_t>::value) {
        return *TypeName::from_utf8("u8");
    }
    if (std::is_same<PayloadType, uint16_t>::value) {
        return *TypeName::from_utf8("u16");
    }
    if (std::is_same<PayloadType, uint32_t>::value) {
        return *TypeName::from_utf8("u32");
    }
    if (std::is_same<PayloadType, uint64_t>::value) {
        return *TypeName::from_utf8("u64");
    }
    if (std::is_same<PayloadType, int8_t>::value) {
        return *TypeName::from_utf8("i8");
    }
    if (std::is_same<PayloadType, int16_t>::value) {
        return *TypeName::from_utf8("i16");
    }
    if (std::is_same<PayloadType, int32_t>::value) {
        return *TypeName::from_utf8("i32");
    }
    if (std::is_same<PayloadType, int64_t>::value) {
        return *TypeName::from_utf8("i64");
    }
    if (std::is_same<PayloadType, float>::value) {
        return *TypeName::from_utf8("f32");
    }
    if (std::is_same<PayloadType, double>::value) {
        return *TypeName::from_utf8("f64");
    }

    // std::array is not available in this safety-critical context
    // NOLINTNEXTLINE
    char type_name[TypeName::capacity()] { 0 };
    // std::to_string() is not available in this safety-critical context
    // NOLINTNEXTLINE
    snprintf(&type_name[0],
             TypeName::capacity(),
             "__cxx__abi__%s",
             typeid(typename PayloadInfo<PayloadType>::ValueType).name());

    return *TypeName::from_utf8_null_terminated_unchecked(&type_name[0]);
}
// NOLINTEND(readability-function-size)

template <typename PayloadType>
auto get_type_name_impl() -> internal::FromSliceWithCustomizedInnerPayloadTypeName<PayloadType> {
    return *TypeName::from_utf8_null_terminated_unchecked(PayloadType::ValueType::IOX2_TYPE_NAME);
}

template <typename PayloadType>
auto get_type_name_impl() -> internal::FromSliceWithoutCustomizedInnerPayloadTypeName<PayloadType> {
    return get_type_name_impl<typename PayloadType::ValueType>();
}

template <typename PayloadType>
auto get_type_name() -> TypeName {
    return get_type_name_impl<PayloadType>();
}

} // namespace internal
} // namespace iox2

#endif
