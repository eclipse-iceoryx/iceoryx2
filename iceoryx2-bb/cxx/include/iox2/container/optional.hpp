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

#ifndef IOX2_INCLUDE_GUARD_CONTAINER_OPTIONAL_HPP
#define IOX2_INCLUDE_GUARD_CONTAINER_OPTIONAL_HPP

#include "iox2/container/config.hpp"

#include <cstdlib>
#include <type_traits>
#include <utility>

#if IOX2_CONTAINER_CONFIG_USE_STD_OPTIONAL && IOX2_CONTAINER_CONFIG_USE_CUSTOM_OPTIONAL
#error Optional cannot be overriden to use both custom and std variants at the same time.
#endif

#if IOX2_CONTAINER_CONFIG_USE_STD_OPTIONAL
#include <optional>
#endif

namespace iox2 {
namespace container {

#if IOX2_CONTAINER_CONFIG_USE_STD_OPTIONAL

template <typename T>
using Optional = std::optional<T>;
using NulloptT = std::nullopt_t;

#elif defined(IOX2_CONTAINER_CONFIG_USE_CUSTOM_OPTIONAL)

template <typename T>
using Optional = IOX2_CONTAINER_CONFIG_USE_CUSTOM_OPTIONAL<T>;

#else

/// A drop-in replacement for C++17 `std::optional`.
template <class T>
class Optional;

/// A drop-in replacement for C++17 `std::nullopt_t` for use with Optional.
struct NulloptT { };


namespace detail {
/// Internal union implementation for Optional.
/// @todo Proper handling of cond. explicit, cond. noexcept and triviality of special member functions.
template <typename T>
class OptionalValueHolder {
  private:
    union {
        char m_null;
        std::remove_cv_t<T> m_value;
    };
    bool m_is_empty = true;

  public:
    constexpr OptionalValueHolder() noexcept
        : m_null() {
    }
    constexpr explicit OptionalValueHolder(T const& value)
        : m_value(value)
        , m_is_empty(false) {
    }
    constexpr explicit OptionalValueHolder(T&& value)
        : m_value(std::move(value))
        , m_is_empty(false) {
    }
    constexpr OptionalValueHolder(OptionalValueHolder const& rhs)
        : OptionalValueHolder() {
        if (!rhs.m_is_empty) {
            // NOLINTNEXTLINE(cppcoreguidelines-pro-type-union-access), access guarded by if
            set(rhs.m_value);
        }
    }
    // NOLINTNEXTLINE(modernize-type-traits), requires C++17
    constexpr OptionalValueHolder(OptionalValueHolder&& rhs) noexcept(std::is_nothrow_move_constructible<T>::value)
        : OptionalValueHolder() {
        if (!rhs.m_is_empty) {
            // NOLINTNEXTLINE(cppcoreguidelines-pro-type-union-access), access guarded by if
            set(std::move(rhs.m_value));
        }
    }

#if __cplusplus >= 202002L
    constexpr
#endif
        ~OptionalValueHolder() {
        reset();
    }

    constexpr auto operator=(OptionalValueHolder const& rhs) -> OptionalValueHolder& {
        if (this != &rhs) {
            if (rhs.m_is_empty) {
                reset();
            } else {
                // NOLINTNEXTLINE(cppcoreguidelines-pro-type-union-access), access guarded by if
                set(rhs.m_value);
            }
        }
        return *this;
    }

    // NOLINTNEXTLINE(modernize-type-traits), requires C++17
    constexpr auto operator=(OptionalValueHolder&& rhs) noexcept(std::is_nothrow_move_assignable<T>::value)
        -> OptionalValueHolder& {
        if (this != &rhs) {
            if (rhs.m_is_empty) {
                reset();
            } else {
                // NOLINTNEXTLINE(cppcoreguidelines-pro-type-union-access), access guarded by if
                set(std::move(rhs.m_value));
            }
        }
        return *this;
    }

    constexpr auto is_empty() const -> bool {
        return m_is_empty;
    }

    constexpr auto set(T const& value) -> void {
        if (m_is_empty) {
            m_is_empty = false;
            // NOLINTNEXTLINE(cppcoreguidelines-pro-type-union-access), access guarded by if
            new (&m_value) T { value };
        } else {
            // NOLINTNEXTLINE(cppcoreguidelines-pro-type-union-access), access guarded by if
            m_value = value;
        }
    }

    constexpr auto set(T&& value) -> void {
        if (m_is_empty) {
            m_is_empty = false;
            // NOLINTNEXTLINE(cppcoreguidelines-pro-type-union-access), access guarded by if
            new (&m_value) T { std::move(value) };
        } else {
            // NOLINTNEXTLINE(cppcoreguidelines-pro-type-union-access), access guarded by if
            m_value = std::move(value);
        }
    }

    constexpr auto unchecked_get() & -> T& {
        // NOLINTNEXTLINE(cppcoreguidelines-pro-type-union-access), unchecked access guarded by caller
        return m_value;
    }

    constexpr auto unchecked_get() const& -> T const& {
        // NOLINTNEXTLINE(cppcoreguidelines-pro-type-union-access), unchecked access guarded by caller
        return m_value;
    }

    constexpr auto unchecked_get() && -> T&& {
        // NOLINTNEXTLINE(cppcoreguidelines-pro-type-union-access), unchecked access guarded by caller
        return std::move(m_value);
    }

    constexpr auto unchecked_get() const&& -> T const&& {
        // NOLINTNEXTLINE(cppcoreguidelines-pro-type-union-access), unchecked access guarded by caller
        return std::move(m_value);
    }

    constexpr auto reset() -> void {
        if (!m_is_empty) {
            // NOLINTNEXTLINE(cppcoreguidelines-pro-type-union-access), access guarded by if
            m_value.~T();
            m_is_empty = true;
        }
    }
};
} // namespace detail

/// @todo Iterator and monadic APIs.
template <typename T>
class Optional {
  private:
    detail::OptionalValueHolder<T> m_value;

  public:
    // NOLINTNEXTLINE(readability-identifier-naming), as specified in ISO14882:2017 [optional]
    using value_type = T;

    // constructors
    constexpr Optional() noexcept = default;

    constexpr Optional(const Optional& rhs) = default;
    constexpr Optional(Optional&& rhs) = default;

    // NOLINTNEXTLINE(hicpp-explicit-conversions), as specified in ISO14882:2017 [optional]
    constexpr Optional(NulloptT /* unused */) noexcept
        : Optional() {
    }

    template <typename U = std::remove_cv_t<T>,
              // NOLINTNEXTLINE(modernize-type-traits), requires C++17
              std::enable_if_t<std::is_constructible<T, U>::value
                                   // NOLINTNEXTLINE(modernize-type-traits), requires C++17
                                   && !std::is_same<typename std::decay<U>::type, Optional<T>>::value
                                   // NOLINTNEXTLINE(modernize-type-traits), requires C++17
                                   && !std::is_same<typename std::decay<U>::type, NulloptT>::value,
                               bool> = true>
    // NOLINTNEXTLINE(hicpp-explicit-conversions), as specified in ISO14882:2017 [optional]
    constexpr Optional(U&& value)
        : m_value(std::forward<U>(value)) {
    }

    // destructor
#if __cplusplus >= 202002L
    constexpr
#endif
        ~Optional() = default;

    // assignment
    constexpr auto operator=(NulloptT /* unused */) noexcept -> Optional& {
        reset();
        return *this;
    }

    constexpr auto operator=(const Optional& rhs) -> Optional& = default;
    constexpr auto operator=(Optional&& rhs) -> Optional& = default;

    // observers
    constexpr auto operator->() const noexcept -> const T* {
        return m_value.is_empty() ? nullptr : &(m_value.unchecked_get());
    }

    constexpr auto operator->() noexcept -> T* {
        return m_value.is_empty() ? nullptr : &(m_value.unchecked_get());
    }

    constexpr auto operator*() const& noexcept -> const T& {
        if (m_value.is_empty()) {
            std::abort();
        }
        return m_value.unchecked_get();
    }

    constexpr auto operator*() & noexcept -> T& {
        if (m_value.is_empty()) {
            std::abort();
        }
        return m_value.unchecked_get();
    }

    constexpr auto operator*() && noexcept -> T&& {
        if (m_value.is_empty()) {
            std::abort();
        }
        return std::move(m_value).unchecked_get();
    }

    constexpr auto operator*() const&& noexcept -> const T&& {
        if (m_value.is_empty()) {
            std::abort();
        }
        return std::move(m_value).unchecked_get();
    }

    constexpr explicit operator bool() const noexcept {
        return !m_value.is_empty();
    }

    constexpr auto has_value() const noexcept -> bool {
        return !m_value.is_empty();
    }

    constexpr auto value() const& -> const T& {
        return **this;
    }

    constexpr auto value() & -> T& {
        return **this;
    }

    constexpr auto value() && -> T&& {
        return std::move(**this);
    }

    constexpr auto value() const&& -> const T&& {
        return std::move(**this);
    }

    template <class U = std::remove_cv_t<T>>
    constexpr auto value_or(U&& fallback) const& -> T {
        return m_value.is_empty() ? std::forward<U>(fallback) : m_value.unchecked_get();
    }

    template <class U = std::remove_cv_t<T>>
    constexpr auto value_or(U&& fallback) && -> T {
        return m_value.is_empty() ? std::forward<U>(fallback) : std::move(m_value).unchecked_get();
    }

    // modifiers
    constexpr auto reset() noexcept -> void {
        m_value.reset();
    }
};

#if __cplusplus >= 201703L
template <class T>
Optional(T) -> Optional<T>;
#endif

#endif

#if __cplusplus >= 201703L
// NOLINTNEXTLINE(readability-identifier-naming), for consistency with C++17 code using std::optional
inline constexpr NulloptT nullopt;
#endif

} // namespace container
} // namespace iox2

#endif
