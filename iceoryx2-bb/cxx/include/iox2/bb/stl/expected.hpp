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

#ifndef IOX2_INCLUDE_GUARD_BB_STL_EXPECTED_HPP
#define IOX2_INCLUDE_GUARD_BB_STL_EXPECTED_HPP

#include "iox2/bb/detail/attributes.hpp"
#include "iox2/legacy/detail/expected_helper.hpp"
#include "iox2/legacy/expected.hpp"
#include <type_traits>

namespace iox2 {
namespace bb {
namespace stl {

struct InPlaceT { };
constexpr InPlaceT IN_PLACE {};
struct UnexpectT { };
constexpr UnexpectT UNEXPECT {};


template <typename E>
class Unexpected {
  private:
    E m_error;

  public:
    constexpr Unexpected(const Unexpected&) = default;
    constexpr Unexpected(Unexpected&&) = default;
    ~Unexpected() = default;

    auto operator=(const Unexpected&) -> Unexpected& = default;
    auto operator=(Unexpected&&) -> Unexpected& = default;

    template <typename Err = E>
    // NOLINTNEXTLINE(bugprone-forwarding-reference-overload), for consistency with C++23 code using std::unexpected
    constexpr explicit Unexpected(Err&& err)
        : m_error(std::forward<Err>(err)) {
    }

    template <class... Args>
    constexpr explicit Unexpected(InPlaceT /* unused */, Args&&... args)
        : m_error(std::forward<Args>(args)...) {
    }

    constexpr auto error() & noexcept -> E& {
        return m_error;
    }

    constexpr auto error() const& noexcept -> const E& {
        return m_error;
    }

    constexpr auto error() && noexcept -> E&& {
        return std::move(m_error);
    }

    constexpr auto error() const&& noexcept -> const E&& {
        return std::move(m_error);
    }
};

#if __cplusplus >= 201703L
template <typename E>
Unexpected(E) -> Unexpected<E>;
#endif

template <typename T, typename E>
class IOX2_NO_DISCARD Expected {
  private:
    legacy::expected<T, E> m_value;

  public:
    // BEGIN ctors
    constexpr Expected()
        : m_value(iox2::legacy::in_place) {
    }

    template <typename U,
              typename V = std::remove_cv_t<T>,
              std::enable_if_t<std::is_convertible<U, V>::value && std::is_same<U, V>::value, bool> = true>
    // NOLINTNEXTLINE(hicpp-explicit-conversions), as specified in ISO14882:2024 [expected]
    constexpr Expected(U&& value)
        : m_value(iox2::legacy::in_place_t {}, std::forward<U>(value)) {
    }

    template <typename U,
              typename V = std::remove_cv_t<T>,
              std::enable_if_t<std::is_convertible<U, V>::value && !std::is_same<U, V>::value, bool> = true>
    constexpr explicit Expected(U&& value)
        : m_value(iox2::legacy::in_place_t {}, std::forward<U>(value)) {
    }

    // NOLINTNEXTLINE(hicpp-explicit-conversions), as specified in ISO14882:2024 [expected]
    constexpr Expected(const Unexpected<E>& error)
        : m_value(iox2::legacy::unexpect_t {}, error.error()) {
    }

    // NOLINTNEXTLINE(hicpp-explicit-conversions), as specified in ISO14882:2024 [expected]
    constexpr Expected(Unexpected<E>&& error)
        : m_value(iox2::legacy::unexpect_t {}, std::forward<E>(error.error())) {
    }

    template <typename... Args>
    // NOLINTNEXTLINE(hicpp-explicit-conversions), as specified in ISO14882:2024 [expected]
    constexpr Expected(InPlaceT /* unused */, Args&&... args)
        : m_value(iox2::legacy::in_place_t {}, std::forward<Args>(args)...) {
    }

    template <typename... Args>
    // NOLINTNEXTLINE(hicpp-explicit-conversions), as specified in ISO14882:2024 [expected]
    constexpr Expected(UnexpectT /* unused */, Args&&... args)
        : m_value(iox2::legacy::unexpect_t {}, std::forward<Args>(args)...) {
    }

    constexpr Expected(const Expected&) = default;
    constexpr Expected(Expected&& rhs) noexcept = default;
    // END ctors

    ~Expected() = default;

    constexpr auto operator=(const Expected&) -> Expected& = default;

    constexpr auto operator=(Expected&& rhs) noexcept -> Expected& = default;

    constexpr auto has_value() const noexcept -> bool {
        return m_value.has_value();
    }

    constexpr explicit operator bool() const noexcept {
        return has_value();
    }

    // BEGIN value method
    template <typename U = T, std::enable_if_t<std::is_void<U>::value, bool> = true>
    constexpr auto value() const& noexcept -> void {
    }

    template <typename U = T, std::enable_if_t<std::is_void<U>::value, bool> = true>
    constexpr auto value() && noexcept -> void {
    }

    template <typename U = T, std::enable_if_t<!std::is_void<U>::value, bool> = true>
    constexpr auto value() & noexcept -> U& {
        return m_value.value();
    }

    template <typename U = T, std::enable_if_t<!std::is_void<U>::value, bool> = true>
    constexpr auto value() const& noexcept -> const U& {
        return m_value.value();
    }

    template <typename U = T, std::enable_if_t<!std::is_void<U>::value, bool> = true>
    constexpr auto value() && noexcept -> U&& {
        return std::move(m_value).value();
    }

    template <typename U = T, std::enable_if_t<!std::is_void<U>::value, bool> = true>
    constexpr auto value() const&& noexcept -> const U&& {
        return std::move(m_value).value();
    }
    // END value method

    // BEGIN operator*
    template <typename U = T, std::enable_if_t<std::is_void<U>::value, bool> = true>
    constexpr auto operator*() const noexcept -> U {
    }

    template <typename U = T, std::enable_if_t<!std::is_void<U>::value, bool> = true>
    constexpr auto operator*() & noexcept -> U& {
        return value();
    }

    template <typename U = T, std::enable_if_t<!std::is_void<U>::value, bool> = true>
    constexpr auto operator*() const& noexcept -> const U& {
        return value();
    }

    template <typename U = T, std::enable_if_t<!std::is_void<U>::value, bool> = true>
    constexpr auto operator*() && noexcept -> U&& {
        return std::move(*this).value();
    }

    template <typename U = T, std::enable_if_t<!std::is_void<U>::value, bool> = true>
    constexpr auto operator*() const&& noexcept -> const U&& {
        return std::move(*this).value();
    }
    // END operator*

    // BEGIN operator->
    template <typename U = T, std::enable_if_t<!std::is_void<U>::value, bool> = true>
    constexpr auto operator->() noexcept -> U* {
        return &value();
    }

    template <typename U = T, std::enable_if_t<!std::is_void<U>::value, bool> = true>
    constexpr auto operator->() const noexcept -> const U* {
        return &value();
    }
    // END operator->

    // BEGIN error method
    constexpr auto error() & noexcept -> E& {
        return m_value.error();
    }

    constexpr auto error() const& noexcept -> const E& {
        return m_value.error();
    }

    constexpr auto error() && noexcept -> E&& {
        return std::move(m_value).error();
    }

    constexpr auto error() const&& noexcept -> const E&& {
        return std::move(m_value).error();
    }
    // END error method
};

} // namespace stl
} // namespace bb
} // namespace iox2

#endif // IOX2_INCLUDE_GUARD_CONTAINER_EXPECTED_HPP
