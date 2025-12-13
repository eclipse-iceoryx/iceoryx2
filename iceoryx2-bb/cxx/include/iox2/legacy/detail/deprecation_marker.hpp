// Copyright (c) 2023 by ekxide IO GmbH. All rights reserved.
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

#ifndef IOX2_BB_UTILITY_DEPRECATION_MARKER_HPP
#define IOX2_BB_UTILITY_DEPRECATION_MARKER_HPP

// #include "iceoryx_versions.hpp"
// FIXME introduce versions for iceoryx2
#define ICEORYX_VERSION_MAJOR 0

namespace iox2 {
namespace legacy {
namespace detail {
struct DeprecationMarker { };
} // namespace detail
}
} // namespace iox

// NOLINTJUSTIFICATION there is no other way to create the intended functionality for the deprecation marker
// NOLINTBEGIN(cppcoreguidelines-macro-usage)

#define IOX2_INTERNAL_NEXT_DEPRECATED_VERSION 3

#define IOX2_INTERNAL_DEPRECATED_STINGIFY_HELPER_EXPAMD(NUM) #NUM
#define IOX2_INTERNAL_DEPRECATED_STINGIFY_HELPER(NUM) IOX2_INTERNAL_DEPRECATED_STINGIFY_HELPER_EXPAMD(NUM)

// clang-format off
static_assert(ICEORYX_VERSION_MAJOR < IOX2_INTERNAL_NEXT_DEPRECATED_VERSION,
    "The iceoryx major version changed to v" IOX2_INTERNAL_DEPRECATED_STINGIFY_HELPER(ICEORYX_VERSION_MAJOR) "!\n"
    "The following steps need to be done to fix this error: \n"
    " - increment 'IOX2_INTERNAL_NEXT_DEPRECATED_VERSION'\n"
    " - update 'IOX2_INTERNAL_DEPRECATED_SINCE_V" IOX2_INTERNAL_DEPRECATED_STINGIFY_HELPER(ICEORYX_VERSION_MAJOR)
        " to call 'IOX2_INTERNAL_DEPRECATED_SINCE(VERSION, MESSAGE)'\n"
    " - update 'IOX2_INTERNAL_DEPRECATED_HEADER_SINCE_V" IOX2_INTERNAL_DEPRECATED_STINGIFY_HELPER(ICEORYX_VERSION_MAJOR)
        " to call 'IOX2_INTERNAL_DEPRECATED_HEADER_SINCE(VERSION, MESSAGE)'");
// clang-format on


// BEGIN IOX2_DEPRECATED_HEADER_SINCE macros

// when the namespace is called 'header' the warning will be "warning: 'header' is deprecated: Deprecated since v#.0
// ..." with the file and line where the 'IOX2_DEPRECATED_HEADER_SINCE' macro is used
#define IOX2_INTERNAL_DEPRECATED_HEADER_SINCE(VERSION, MESSAGE)                                                        \
    namespace iox2 {                                                                                                   \
    namespace legacy {                                                                                                 \
    namespace detail {                                                                                                 \
    namespace                                                                                                          \
        [[deprecated("Deprecated since v" #VERSION ".0 and will be removed at a later version! " MESSAGE)]] header {   \
    using iox2::legacy::detail::DeprecationMarker;                                                                     \
    }                                                                                                                  \
    using header::DeprecationMarker;                                                                                   \
    }                                                                                                                  \
    }

// clang-format off
// The 'IOX2_INTERNAL_DEPRECATED_HEADER_SINCE_V#' macros call either 'IOX2_INTERNAL_DEPRECATED_HEADER_SINCE' if the
// specific version is deprecated or expand to an empty macro. Here an example with V1 being deprecated and V2 not yet
// ----
// #define IOX2_INTERNAL_DEPRECATED_HEADER_SINCE_V1(VERSION, MESSAGE) IOX2_INTERNAL_DEPRECATED_HEADER_SINCE(VERSION, MESSAGE)
// #define IOX2_INTERNAL_DEPRECATED_HEADER_SINCE_V2(VERSION, MESSAGE)
// ----
// clang-format on

#define IOX2_INTERNAL_DEPRECATED_HEADER_SINCE_V1(VERSION, MESSAGE)                                                     \
    IOX2_INTERNAL_DEPRECATED_HEADER_SINCE(VERSION, MESSAGE)

#define IOX2_INTERNAL_DEPRECATED_HEADER_SINCE_V2(VERSION, MESSAGE)                                                     \
    IOX2_INTERNAL_DEPRECATED_HEADER_SINCE(VERSION, MESSAGE)

#define IOX2_INTERNAL_DEPRECATED_HEADER_SINCE_V3(VERSION, MESSAGE)

#define IOX2_INTERNAL_DEPRECATED_HEADER_SINCE_V4(VERSION, MESSAGE)

// This indirection is required to expand defines passed to 'IOX2_DEPRECATED_HEADER_SINCE' make code like this work
// ----
// #define V 3
// IOX2_DEPRECATED_HEADER_SINCE(V, "Please include 'iox/foo.hpp' instead.")
// ----
#define IOX2_INTERNAL_DEPRECATED_HEADER_SINCE_EXPANSION(VERSION, MESSAGE)                                              \
    IOX2_INTERNAL_DEPRECATED_HEADER_SINCE_V##VERSION(VERSION, MESSAGE)

/// @brief Macro to deprecate header depending on the iceoryx major version
/// @param[in] VERSION from when the header is deprecated
/// @param[in] MESSAGE custom message to be printed after 'Deprecated since vX.0 and will be remove at a later version!'
/// @code
///     // assuming this file is 'iox/bar/foo.hpp'
///     #include "iox2/legacy/foo.hpp"
///     IOX2_DEPRECATED_HEADER_SINCE(3, "Please use 'iox/foo.hpp' instead.")
/// @endcode
#define IOX2_DEPRECATED_HEADER_SINCE(VERSION, MESSAGE) IOX2_INTERNAL_DEPRECATED_HEADER_SINCE_EXPANSION(VERSION, MESSAGE)

// END IOX2_DEPRECATED_HEADER_SINCE macros


// BEGIN IOX2_DEPRECATED_SINCE macros

#define IOX2_INTERNAL_DEPRECATED_SINCE(VERSION, MESSAGE)                                                               \
    [[deprecated("Deprecated since v" #VERSION ".0 and will be removed at a later version! " MESSAGE)]]

// The 'IOX2_INTERNAL_DEPRECATED_SINCE_V#' macros call either 'IOX2_INTERNAL_DEPRECATED_SINCE' if the
// specific version is deprecated or expand to an empty macro. Here an example with V1 being deprecated and V2 not yet
// ----
// #define IOX2_INTERNAL_DEPRECATED_SINCE_V1(VERSION, MESSAGE) IOX2_INTERNAL_DEPRECATED_SINCE(VERSION, MESSAGE)
// #define IOX2_INTERNAL_DEPRECATED_SINCE_V2(VERSION, MESSAGE)
// ----

#define IOX2_INTERNAL_DEPRECATED_SINCE_V1(VERSION, MESSAGE) IOX2_INTERNAL_DEPRECATED_SINCE(VERSION, MESSAGE)

#define IOX2_INTERNAL_DEPRECATED_SINCE_V2(VERSION, MESSAGE) IOX2_INTERNAL_DEPRECATED_SINCE(VERSION, MESSAGE)

#define IOX2_INTERNAL_DEPRECATED_SINCE_V3(VERSION, MESSAGE)

#define IOX2_INTERNAL_DEPRECATED_SINCE_V4(VERSION, MESSAGE)

// This indirection is required to expand defines passed to 'IOX2_DEPRECATED_SINCE' make code like this work
// ----
// #define V 3
// IOX2_DEPRECATED_SINCE(V, "Please use 'iox2::legacy::foo' instead.") void bar() {}
// ----
#define IOX2_INTERNAL_DEPRECATED_SINCE_EXPANSION(VERSION, MESSAGE)                                                     \
    IOX2_INTERNAL_DEPRECATED_SINCE_V##VERSION(VERSION, MESSAGE)

/// @brief Macro to deprecate code depending on the iceoryx major version
/// @param[in] VERSION from when the code is deprecated
/// @param[in] MESSAGE custom message to be printed after 'Deprecated since vX and will be remove at a later version!'
/// @code
///     IOX2_DEPRECATED_SINCE(3, "Please use 'iox2::legacy::foo' instead.") void bar() {}
/// @endcode
#define IOX2_DEPRECATED_SINCE(VERSION, MESSAGE) IOX2_INTERNAL_DEPRECATED_SINCE_EXPANSION(VERSION, MESSAGE)

// END IOX2_DEPRECATED_SINCE macros

// NOLINTEND(cppcoreguidelines-macro-usage)

#endif // IOX2_BB_UTILITY_DEPRECATION_MARKER_HPP
