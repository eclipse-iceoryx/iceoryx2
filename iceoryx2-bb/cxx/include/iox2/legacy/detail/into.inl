// Copyright (c) 2021 - 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_UTILITY_INTO_INL
#define IOX2_BB_UTILITY_INTO_INL

#include "iox2/legacy/into.hpp"
#include "iox2/legacy/type_traits.hpp"

namespace iox2 {
namespace legacy {
template <typename SourceType, typename DestinationType>
inline constexpr typename detail::extract_into_type<DestinationType>::type_t from(const SourceType value) noexcept {
    return FromImpl<SourceType, DestinationType>::fromImpl(value);
}

// AXIVION Next Construct AutosarC++19_03-A7.1.5 : 'auto' is only used for the generic implementation which will always result in a compile error
template <typename SourceType, typename DestinationType>
inline auto FromImpl<SourceType, DestinationType>::fromImpl(const SourceType&) noexcept {
    static_assert(always_false_v<SourceType> && always_false_v<DestinationType>, "\n \
        Conversion for the specified types is not implemented!\n \
        Please specialize 'FromImpl::fromImpl'!\n \
        -------------------------------------------------------------------------\n \
        template <typename SourceType, typename DestinationType>\n \
        constexpr DestinationType FromImpl::fromImpl(const SourceType&) noexcept;\n \
        -------------------------------------------------------------------------");
}

// AXIVION Next Construct AutosarC++19_03-A15.5.3, AutosarC++19_03-A15.4.2, FaultDetection-NoexceptViolations : Intentional behavior. The library itself does not throw and on the implementation side a try-catch block can be used
template <typename DestinationType, typename SourceType>
inline constexpr typename detail::extract_into_type<DestinationType>::type_t into(const SourceType value) noexcept {
    return from<SourceType, DestinationType>(value);
}
} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_UTILITY_INTO_INL
