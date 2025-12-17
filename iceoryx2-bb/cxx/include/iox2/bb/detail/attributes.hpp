// Copyright (c) 2021-2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_DETAIL_ATTRIBUTES_HPP
#define IOX2_BB_DETAIL_ATTRIBUTES_HPP

namespace iox2 {
namespace bb {
namespace detail {
/// We use this as an alternative to "static_cast<void>(someVar)" to signal the
/// compiler an unused variable. "static_cast" produces an useless-cast warning
/// on gcc and this approach solves it cleanly.
template <typename T>
// AXIVION Next Construct AutosarC++19_03-M0.1.8 : No side effects are the intended behavior of the function
// NOLINTNEXTLINE(cppcoreguidelines-missing-std-forward) intended for this function
inline void iox2_discard_result_impl(T&& /* unused */) noexcept {
}
} // namespace detail
} // namespace bb
} // namespace iox2

// AXIVION Next Construct AutosarC++19_03-A16.0.1 : Aliasing of fully qualified templated function.
//                                                  Improves readability. No risks apparent.
// NOLINTJUSTIFICATION cannot be implemented with a function, required as inline code
// NOLINTBEGIN(cppcoreguidelines-macro-usage)
/// @brief if a function has a return value which you do not want to use then you can wrap the function with that macro.
/// Purpose is to suppress the unused compiler warning by adding an attribute to the return value
/// @param[in] expr name of the function where the return value is not used.
/// @code
///     uint32_t foo();
///     IOX2_DISCARD_RESULT(foo()); // suppress compiler warning for unused return value
/// @endcode
#define IOX2_DISCARD_RESULT(expr) ::iox2::bb::detail::iox2_discard_result_impl(expr)

/// @brief IOX2_NO_DISCARD adds the [[nodiscard]] keyword if it is available for the current compiler.

#if __cplusplus >= 201703L
#define IOX2_NO_DISCARD [[nodiscard]]
#else
#define IOX2_NO_DISCARD
#endif

/// @brief IOX2_FALLTHROUGH adds the [[fallthrough]] keyword when it is available for the current compiler.
/// @note
//    [[fallthrough]] supported since gcc 7 (https://gcc.gnu.org/projects/cxx-status.html)
///   [[fallthrough]] supported since clang 3.9 (https://clang.llvm.org/cxx_status.html)
///   activate keywords for gcc>=7 or clang>=4

#if __cplusplus >= 201703L
// clang prints a warning therefore we exclude it here
#define IOX2_FALLTHROUGH [[fallthrough]]
#elif (defined(__GNUC__) && (__GNUC__ >= 7)) || defined(__clang__)
#define IOX2_FALLTHROUGH [[gnu::fallthrough]]
#else
#define IOX2_FALLTHROUGH
#endif

/// @brief IOX2_MAYBE_UNUSED adds the [[gnu::unused]] attribute when it is available for the current
/// compiler or uses C++17's 'maybe_unused'.
#if __cplusplus >= 201703L
#define IOX2_MAYBE_UNUSED [[maybe_unused]]
#elif (defined(__GNUC__) && (__GNUC__ >= 7)) || defined(__clang__)
#define IOX2_MAYBE_UNUSED [[gnu::unused]]
#else
#define IOX2_MAYBE_UNUSED
#endif

// NOLINTEND(cppcoreguidelines-macro-usage)

#endif // IOX2_BB_DETAIL_ATTRIBUTES_HPP
