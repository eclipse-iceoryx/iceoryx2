// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
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

#ifndef IOX2_BB_INTO_HPP
#define IOX2_BB_INTO_HPP

#include "iox2/bb/detail/attributes.hpp"
#include "iox2/legacy/type_traits.hpp"

namespace iox2 {
namespace bb {

/// @brief Helper struct to indicate a lossy conversion, e.g. from an unbounded type into a bounded type
template <typename D>
struct Lossy { };

namespace detail {
/// @brief Helper struct to get the actual destination type 'T' for 'into' with an additional indirection like
/// 'into<Lossy<T>>'
template <typename T>
struct ExtractIntoType {
    using TargetType = T;
};

/// @brief Helper struct to get the actual destination type 'T' for 'into<Lossy<T>>'
template <typename T>
struct ExtractIntoType<Lossy<T>> {
    using TargetType = T;
};
} // namespace detail

// Using a struct as impl, as free functions do not support partially specialized templates
template <typename SourceType, typename DestinationType>
struct From {
    // AXIVION Next Construct AutosarC++19_03-A7.1.5 : 'auto' is only used for the generic implementation which will always result in a compile error
    static constexpr auto from(const SourceType& value IOX2_MAYBE_UNUSED) noexcept {
        static_assert(legacy::always_false_v<SourceType> && legacy::always_false_v<DestinationType>, "\n \
Conversion for the specified types is not implemented!\n \
Please specialize 'From::from'!\n \
-------------------------------------------------------------------------\n \
template <typename SourceType, typename DestinationType>\n \
constexpr DestinationType From::from(const SourceType&) noexcept;\n \
-------------------------------------------------------------------------");
    }
};

/// @brief Converts a value of type SourceType to a corresponding value of type DestinationType. This function needs to
/// be specialized by the user for the types to be converted. If a partial specialization is needed, please have a look
/// at 'From'.
/// @note If the conversion is potentially lossy 'Destination from<Source, Destination>(...)' should not be used but
/// instead either one or both of:
///   - 'Destination from<Source, Lossy<Destination>>(...)'
///   - 'optional<Destination> from<Source, optional<Destination>>(...)'
/// The 'Destination from<Source, Destination>(...)' implementation should have a 'static_assert' with a hint of the
/// reason, e.g. lossy conversion and a hint to use 'Destination into<Lossy<Destination>>(...)' or
/// 'optional<Destination> into<optional<Destination>>(...)'. The 'std_string_support.hpp' can be used as a source of
/// inspiration for an implementation and error message.
/// @code
/// enum class LowLevel
/// {
///     FileDescriptorInvalid,
///     FileDescriptorCorrupt,
///     Timeout
/// };
///
/// enum class HighLevel
/// {
///     FileDescriptorError,
///     Timeout
/// };
///
/// namespace iox
/// {
/// template <>
/// constexpr HighLevel from<LowLevel, HighLevel>(LowLevel e) noexcept
/// {
///     switch (e)
///     {
///     case LowLevel::FileDescriptorCorrupt:
///         return HighLevel::FileDescriptorError;
///     case LowLevel::FileDescriptorInvalid:
///         return HighLevel::FileDescriptorError;
///     case LowLevel::Timeout:
///         return HighLevel::Timeout;
///     }
/// }
/// } } // namespace iox
/// @endcode
/// @tparam SourceType is the 'from' type
/// @tparam DestinationType is the 'to' type
/// @param[in] value of type SourceType to convert to DestinationType
/// @return converted value of SourceType to corresponding value of DestinationType
template <typename SourceType, typename DestinationType>
constexpr auto from(const SourceType value) noexcept -> typename detail::ExtractIntoType<DestinationType>::TargetType {
    return From<SourceType, DestinationType>::from(value);
}

/// @brief Converts a value of type SourceType to a corresponding value of type DestinationType. This is a convenience
/// function which is automatically available when 'from' is implemented. This function shall therefore not be
/// specialized but always the 'from' function.
/// @code
/// Bar b = iox2::bb::into<Bar>(Foo::ENUM_VALUE);
/// @endcode
/// @tparam DestinationType is the 'to' type
/// @tparam SourceType is the 'from' type
/// @param[in] value of type SourceType to convert to DestinationType
/// @return converted value of SourceType to corresponding value of DestinationType
template <typename DestinationType, typename SourceType>
constexpr auto into(const SourceType value) noexcept -> typename detail::ExtractIntoType<DestinationType>::TargetType {
    return from<SourceType, DestinationType>(value);
}

} // namespace bb
} // namespace iox2

#endif // IOX2_BB_INTO_HPP
