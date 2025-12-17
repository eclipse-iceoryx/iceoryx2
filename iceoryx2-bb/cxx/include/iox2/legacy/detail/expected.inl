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

#ifndef IOX2_BB_VOCABULARY_EXPECTED_INL
#define IOX2_BB_VOCABULARY_EXPECTED_INL

#include "iox2/bb/detail/assertions.hpp"
#include "iox2/legacy/expected.hpp"

namespace iox2 {
namespace legacy {
template <typename T, typename>
inline detail::ok<void> ok() {
    return detail::ok<void> {};
}

template <typename T, typename>
inline detail::ok<T> ok(const T& value) {
    return detail::ok<T> { value };
}

template <typename T, typename, typename>
inline detail::ok<T> ok(T&& value) {
    return detail::ok<T> { std::forward<T>(value) };
}

template <typename T, typename... Targs, typename>
inline detail::ok<T> ok(Targs&&... args) {
    return detail::ok<T> { std::forward<Targs>(args)... };
}

template <typename T>
inline detail::err<T> err(const T& error) {
    return detail::err<T> { error };
}

template <typename T, typename>
inline detail::err<T> err(T&& error) {
    return detail::err<T> { std::forward<T>(error) };
}

template <typename T, typename... Targs>
inline detail::err<T> err(Targs&&... args) {
    return detail::err<T> { std::forward<Targs>(args)... };
}

template <typename ValueType, typename ErrorType>
inline expected<ValueType, ErrorType>::expected(const detail::ok<ValueType>& successValue) noexcept
    : m_store(in_place, successValue.value) {
}

template <typename ValueType, typename ErrorType>
// NOLINTNEXTLINE(cppcoreguidelines-rvalue-reference-param-not-moved) the underlying value is moved
inline expected<ValueType, ErrorType>::expected(detail::ok<ValueType>&& successValue) noexcept
    : m_store(in_place, std::move(successValue.value)) {
}

template <typename ValueType, typename ErrorType>
inline expected<ValueType, ErrorType>::expected(const detail::err<ErrorType>& errorValue) noexcept
    : m_store(unexpect, errorValue.value) {
}

template <typename ValueType, typename ErrorType>
// NOLINTNEXTLINE(cppcoreguidelines-rvalue-reference-param-not-moved) the underlying value is moved
inline expected<ValueType, ErrorType>::expected(detail::err<ErrorType>&& errorValue) noexcept
    : m_store(unexpect, std::move(errorValue.value)) {
}

template <typename ValueType, typename ErrorType>
inline expected<ValueType, ErrorType>::expected(expected<ValueType, ErrorType>&& rhs) noexcept
    : m_store { std::move(rhs.m_store) } {
}

template <typename ValueType, typename ErrorType>
template <typename... Targs>
inline expected<ValueType, ErrorType>::expected(in_place_t, Targs&&... args) noexcept
    : m_store(in_place, std::forward<Targs>(args)...) {
}

template <typename ValueType, typename ErrorType>
template <typename... Targs>
inline expected<ValueType, ErrorType>::expected(unexpect_t, Targs&&... args) noexcept
    : m_store(unexpect, std::forward<Targs>(args)...) {
}

template <typename ValueType, typename ErrorType>
inline expected<ValueType, ErrorType>&
expected<ValueType, ErrorType>::operator=(const expected<ValueType, ErrorType>& rhs) noexcept {
    // AXIVION Next Construct AutosarC++19_03-M0.1.2, AutosarC++19_03-M0.1.9, FaultDetection-DeadBranches : False positive. Check needed to avoid self assignment.
    if (this != &rhs) {
        m_store = rhs.m_store;
    }
    return *this;
}

template <typename ValueType, typename ErrorType>
inline expected<ValueType, ErrorType>&
expected<ValueType, ErrorType>::operator=(expected<ValueType, ErrorType>&& rhs) noexcept {
    // AXIVION Next Construct AutosarC++19_03-M0.1.2, AutosarC++19_03-M0.1.9, FaultDetection-DeadBranches : False positive. Check needed to avoid self assignment.
    if (this != &rhs) {
        m_store = std::move(rhs.m_store);
    }
    return *this;
}

template <typename ValueType, typename ErrorType>
inline expected<ValueType, ErrorType>::operator bool() const noexcept {
    return has_value();
}

template <typename ValueType, typename ErrorType>
inline bool expected<ValueType, ErrorType>::has_value() const noexcept {
    return m_store.has_value();
}

template <typename ValueType, typename ErrorType>
inline bool expected<ValueType, ErrorType>::has_error() const noexcept {
    return m_store.has_error();
}

template <typename ValueType, typename ErrorType>
inline ErrorType& expected<ValueType, ErrorType>::error_checked() & noexcept {
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : const cast to avoid code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<ErrorType&>(const_cast<const expected<ValueType, ErrorType>*>(this)->error_checked());
}

template <typename ValueType, typename ErrorType>
inline const ErrorType& expected<ValueType, ErrorType>::error_checked() const& noexcept {
    IOX2_ENFORCE(has_error(), "Trying to access an error but a value is stored!");
    return m_store.error_unchecked();
}

template <typename ValueType, typename ErrorType>
inline ErrorType&& expected<ValueType, ErrorType>::error() && noexcept {
    return std::move(error_checked());
}

template <typename ValueType, typename ErrorType>
inline const ErrorType&& expected<ValueType, ErrorType>::error() const&& noexcept {
    return std::move(error_checked());
}

template <typename ValueType, typename ErrorType>
inline ErrorType& expected<ValueType, ErrorType>::error() & noexcept {
    return error_checked();
}

template <typename ValueType, typename ErrorType>
inline const ErrorType& expected<ValueType, ErrorType>::error() const& noexcept {
    return error_checked();
}

template <typename ValueType, typename ErrorType>
template <typename U>
inline const enable_if_non_void_t<U>& expected<ValueType, ErrorType>::value_checked() const& noexcept {
    IOX2_ENFORCE(has_value(), "Trying to access a value but an error is stored!");
    return m_store.value_unchecked();
}

template <typename ValueType, typename ErrorType>
template <typename U>
inline enable_if_non_void_t<U>& expected<ValueType, ErrorType>::value_checked() & noexcept {
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : const cast to avoid code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<ValueType&>(const_cast<const expected<ValueType, ErrorType>*>(this)->value_checked());
}

template <typename ValueType, typename ErrorType>
template <typename U>
inline enable_if_non_void_t<U>&& expected<ValueType, ErrorType>::value() && noexcept {
    return std::move(value_checked());
}

template <typename ValueType, typename ErrorType>
template <typename U>
inline const enable_if_non_void_t<U>&& expected<ValueType, ErrorType>::value() const&& noexcept {
    return std::move(value_checked());
}

template <typename ValueType, typename ErrorType>
template <typename U>
inline const enable_if_non_void_t<U>& expected<ValueType, ErrorType>::value() const& noexcept {
    return value_checked();
}

template <typename ValueType, typename ErrorType>
template <typename U>
inline enable_if_non_void_t<U>& expected<ValueType, ErrorType>::value() & noexcept {
    return value_checked();
}

template <typename ValueType, typename ErrorType>
template <typename U>
inline enable_if_non_void_t<U>* expected<ValueType, ErrorType>::operator->() noexcept {
    return &value_checked();
}

template <typename ValueType, typename ErrorType>
template <typename U>
inline const enable_if_non_void_t<U>* expected<ValueType, ErrorType>::operator->() const noexcept {
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : const_cast avoids code duplication, is safe since the
    // constness of the return value is restored
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<const ValueType*>(const_cast<expected*>(this)->operator->());
}

template <typename ValueType, typename ErrorType>
template <typename U>
inline enable_if_non_void_t<U>& expected<ValueType, ErrorType>::operator*() noexcept {
    return value_checked();
}

template <typename ValueType, typename ErrorType>
template <typename U>
inline const enable_if_non_void_t<U>& expected<ValueType, ErrorType>::operator*() const noexcept {
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : const_cast avoids code duplication and is safe here, since the
    // constness of the return value is restored
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<const ValueType&>(const_cast<expected*>(this)->operator*());
}

// AXIVION Next Construct AutosarC++19_03-A13.5.2, AutosarC++19_03-A13.5.3: see doxygen brief section in header
template <typename ValueType, typename ErrorType>
inline expected<ValueType, ErrorType>::operator expected<void, ErrorType>() const noexcept {
    if (has_error()) {
        return err(m_store.error_unchecked());
    }
    return expected<void, ErrorType>(in_place);
}

template <typename ValueType, typename ErrorType>
inline constexpr bool operator==(const expected<ValueType, ErrorType>& lhs,
                                 const expected<ValueType, ErrorType>& rhs) noexcept {
    if (lhs.has_error() != rhs.has_error()) {
        return false;
    }
    if (lhs.has_error() && rhs.has_error()) {
        return lhs.m_store.error_unchecked() == rhs.m_store.error_unchecked();
    }
    return detail::compare_expected_value<ValueType, ErrorType>::is_same_value_unchecked(lhs.m_store, rhs.m_store);
}

template <typename ValueType, typename ErrorType>
inline constexpr bool operator!=(const expected<ValueType, ErrorType>& lhs,
                                 const expected<ValueType, ErrorType>& rhs) noexcept {
    return !(lhs == rhs);
}
} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_VOCABULARY_EXPECTED_INL
