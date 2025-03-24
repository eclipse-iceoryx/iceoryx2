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

#ifndef IOX2_SERVICE_BUILDER_PUBLISH_SUBSCRIBE_INTERNAL_HPP
#define IOX2_SERVICE_BUILDER_PUBLISH_SUBSCRIBE_INTERNAL_HPP

#include "iox/slice.hpp"

namespace iox2::internal {
template <typename Payload, typename = void>
struct HasPayloadTypeNameMember : std::false_type { };

template <typename Payload>
struct HasPayloadTypeNameMember<Payload, decltype((void) Payload::TYPE_NAME)> : std::true_type { };

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
struct HasUserHeaderTypeNameMember<UserHeader, decltype((void) UserHeader::TYPE_NAME)> : std::true_type { };
} // namespace iox2::internal

#endif
