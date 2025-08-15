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

#ifndef INCLUDE_GUARD_IOX2_CONTAINER_OPTIONAL_HPP
#define INCLUDE_GUARD_IOX2_CONTAINER_OPTIONAL_HPP

#include "iox2/container/config.hpp"

#include <cstdlib>
#include <type_traits>
#include <utility>

#if IOX2_CONTAINER_CONFIG_USE_STD_OPTIONAL && IOX2_CONTAINER_CONFIG_USE_CUSTOM_OPTIONAL
#   error Optional cannot be overriden to use both custom and std variants at the same time.
#endif

#if IOX2_CONTAINER_CONFIG_USE_STD_OPTIONAL
#   include <optional>
#endif

namespace iox2 {
namespace container {

#if IOX2_CONTAINER_CONFIG_USE_STD_OPTIONAL

template<typename T>
using Optional = std::optional <T>;
using NulloptT = std::nullopt_t;

#elif defined(IOX2_CONTAINER_CONFIG_USE_CUSTOM_OPTIONAL)

template<typename T>
using Optional = IOX2_CONTAINER_CONFIG_USE_CUSTOM_OPTIONAL <T>;

#else

/// A drop-in replacement for C++17 `std::optional`.
template<class T>
class Optional;

/// A drop-in replacement for C++17 `std::nullopt_t` for use with Optional.
struct NulloptT {};


namespace detail {
    /// Internal union implementation for Optional.
    /// @todo Proper handling of cond. explicit, cond. noexcept and triviality of special member functions.
    template<typename T>
    struct OptionalValueHolder {
        union {
            char null;
            typename std::remove_cv<T>::type value;
        };
        bool isEmpty;

        constexpr OptionalValueHolder() noexcept: null(), isEmpty(true) {}
        constexpr OptionalValueHolder(T const& v): value(v), isEmpty(false) {}
        constexpr OptionalValueHolder(T&& v): value(std::move(v)), isEmpty(false) {}
        constexpr OptionalValueHolder(OptionalValueHolder const& rhs)
        :OptionalValueHolder()
        {
            if (!rhs.isEmpty) { set(rhs.value); }
        }
        constexpr OptionalValueHolder(OptionalValueHolder&& rhs)
        :OptionalValueHolder()
        {
            if (!rhs.isEmpty) { set(std::move(rhs.value)); }
        }

#       if __cplusplus >= 202002L
        constexpr
#       endif
        ~OptionalValueHolder() {
            if (!isEmpty) {
                value.~T();
            }
        }

        constexpr OptionalValueHolder& operator=(OptionalValueHolder const& rhs) {
            if (this != &rhs) {
                if (rhs.isEmpty) {
                    reset();
                } else {
                    set(rhs.value);
                }
            }
            return *this;
        }

        constexpr OptionalValueHolder& operator=(OptionalValueHolder&& rhs) {
            if (this != &rhs) {
                if (rhs.isEmpty) {
                    reset();
                } else {
                    set(std::move(rhs.value));
                }
            }
            return *this;
        }

        constexpr void set(T const& v) {
            if (isEmpty) {
                isEmpty = false;
                new(&value) T{ v };
            } else {
                value = v;
            }
        }

        constexpr void set(T&& v) {
            if (isEmpty) {
                isEmpty = false;
                new(&value) T{ std::move(v) };
            } else {
                value = std::move(v);
            }
        }

        constexpr void reset() {
            if (!isEmpty) {
                value.~T();
                isEmpty = true;
            }
        }
    };
}

/// @todo Iterator and monadic APIs. 
template<typename T>
class Optional {
private:
    detail::OptionalValueHolder<T> m_value;
public:
    using value_type = T;

    // constructors
    constexpr Optional() noexcept = default;

    constexpr Optional(const Optional& rhs) = default;
    constexpr Optional(Optional&& rhs) = default;

    constexpr Optional(NulloptT) noexcept
    :Optional()
    {}

    template<typename U = typename std::remove_cv<T>::type,
             typename std::enable_if<
                    std::is_constructible<T, U>::value &&
                    !std::is_same<typename std::decay<U>::type, Optional<T>>::value &&
                    !std::is_same<typename std::decay<U>::type, NulloptT>::value,
                bool>::type = true>
    constexpr Optional(U&& value)
    :m_value(std::forward<U>(value))
    {}

    // destructor
#if __cplusplus >= 202002L
    constexpr
#endif
    ~Optional() = default;

    // assignment
    constexpr Optional& operator=(NulloptT) noexcept {
        reset();
        return *this;
    }

    constexpr Optional& operator=(const Optional& rhs) = default;
    constexpr Optional& operator=(Optional&& rhs) = default;

    // observers
    constexpr const T* operator->() const noexcept {
        if (m_value.isEmpty) {
            return nullptr;
        } else {
            return &m_value.value;
        }
    }

    constexpr T* operator->() noexcept {
        if (m_value.isEmpty) {
            return nullptr;
        } else {
            return &m_value.value;
        }
    }

    constexpr const T& operator*() const & noexcept {
        if (m_value.isEmpty) {
            std::abort();
        }
        return m_value.value;
    }

    constexpr T& operator*() & noexcept {
        if (m_value.isEmpty) {
            std::abort();
        }
        return m_value.value;
    }

    constexpr T&& operator*() && noexcept {
        if (m_value.isEmpty) {
            std::abort();
        }
        return std::move(m_value.value);
    }

    constexpr const T&& operator*() const&& noexcept {
        if (m_value.isEmpty) {
            std::abort();
        }
        return std::move(m_value.value);
    }

    constexpr explicit operator bool() const noexcept {
        return !m_value.isEmpty;
    }

    constexpr bool has_value() const noexcept {
        return !m_value.isEmpty;
    }

    constexpr const T& value() const & {
        return **this;
    }

    constexpr T& value() & {
        return **this;
    }

    constexpr T&& value() && {
        return std::move(**this);
    }

    constexpr const T&& value() const && {
        return std::move(**this);
    }

    template<class U = typename std::remove_cv<T>::type> constexpr T value_or(U&& v) const & {
        if (m_value.isEmpty) {
            return std::forward<U>(v);
        } else {
            return m_value.value;
        }
    }

    template<class U = typename std::remove_cv<T>::type> constexpr T value_or(U&& v) && {
        if (m_value.isEmpty) {
            return std::forward<U>(v);
        } else {
            return std::move(m_value.value);
        }
    }

    // modifiers
    constexpr void reset() noexcept {
        m_value.reset();
    }
};

#if __cplusplus >= 201703L
template<class T>
Optional(T) -> Optional<T>;
#endif

#endif

#if __cplusplus >= 201703L
inline constexpr NulloptT nullopt;
#endif

} // namespace container
} // namespace iox2

#endif
