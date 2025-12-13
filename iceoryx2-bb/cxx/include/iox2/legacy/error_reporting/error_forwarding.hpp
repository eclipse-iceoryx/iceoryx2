// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_REPORTING_ERROR_REPORTING_ERROR_FORWARDING_HPP
#define IOX2_BB_REPORTING_ERROR_REPORTING_ERROR_FORWARDING_HPP

#include "iox2/legacy/error_reporting/error_kind.hpp"
#include "iox2/legacy/error_reporting/source_location.hpp"

// to establish connection to the custom implementation
#include "iox2/legacy/error_reporting/custom/error_reporting.hpp"

#include <type_traits>
#include <utility>

namespace iox2 {
namespace legacy {
namespace er {
// This is lightweight and only exists to hide some complexity that would otherwise be part of the
// macro API.

/// @brief Forwards that a panic state was encountered and does not return.
/// @param location the location of the panic invocation
/// @param msg the message to be forwarded
/// @note required to enforce no return
template <typename Message>
[[noreturn]] inline void forwardPanic(const SourceLocation& location, Message&& msg) {
    panic(location, std::forward<Message>(msg));
    abort();
}

namespace detail {
// workaround for gcc 8 bug
// see also https://github.com/eclipse-iceoryx/iceoryx2/issues/855
template <typename T1, typename T2, typename Message>
[[noreturn]] constexpr inline void unreachable_wrapped(const SourceLocation& location, Message&& msg) {
    if (std::is_same<T1, T2>::value) {
        forwardPanic(location, std::forward<Message>(msg));
    }
}
} // namespace detail

/// @brief Forwards a fatal error and does not return.
/// @param error the error
/// @param kind the kind of error (category)
/// @param location the location of the error
/// @param stringifiedCondition the condition as string if a macro with a condition was used; an empty string otherwise
template <typename Error, typename Kind>
[[noreturn]] inline void
forwardFatalError(Error&& error, Kind&& kind, const SourceLocation& location, const char* stringifiedCondition) {
    using K = typename std::remove_const<typename std::remove_reference<Kind>::type>::type;
    static_assert(IsFatal<K>::value, "Must forward a fatal error!");

    report(location, std::forward<Kind>(kind), std::forward<Error>(error), stringifiedCondition);
    panic(location);
    abort();
}

/// @brief Forwards a non-fatal error.
/// @param error the error
/// @param kind the kind of error (category)
/// @param location the location of the error
/// @param stringifiedCondition the condition as string if a macro with a condition was used; an empty string otherwise
template <typename Error, typename Kind>
inline void
forwardNonFatalError(Error&& error, Kind&& kind, const SourceLocation& location, const char* stringifiedCondition) {
    using K = typename std::remove_const<typename std::remove_reference<Kind>::type>::type;
    static_assert(!IsFatal<K>::value, "Must forward a non-fatal error!");

    report(location, std::forward<Kind>(kind), std::forward<Error>(error), stringifiedCondition);
}

/// @brief Forwards a fatal error and a message and does not return.
/// @param error the error
/// @param kind the kind of error (category)
/// @param location the location of the error
/// @param stringifiedCondition the condition as string if a macro with a condition was used; an empty string otherwise
/// @param msg the message to be forwarded
template <typename Error, typename Kind, typename Message>
// NOLINTNEXTLINE(readability-function-size) Not used directly but via a macro which hides the number of parameter away
[[noreturn]] inline void forwardFatalError(
    Error&& error, Kind&& kind, const SourceLocation& location, const char* stringifiedCondition, Message&& msg) {
    using K = typename std::remove_const<typename std::remove_reference<Kind>::type>::type;
    static_assert(IsFatal<K>::value, "Must forward a fatal error!");

    report(location,
           std::forward<Kind>(kind),
           std::forward<Error>(error),
           stringifiedCondition,
           std::forward<Message>(msg));
    panic(location);
    abort();
}

} // namespace er
} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_REPORTING_ERROR_REPORTING_ERROR_FORWARDING_HPP
