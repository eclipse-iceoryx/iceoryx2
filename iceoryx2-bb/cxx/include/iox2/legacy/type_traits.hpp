// Copyright (c) 2020 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_PRIMITIVES_TYPE_TRAITS_HPP
#define IOX2_BB_PRIMITIVES_TYPE_TRAITS_HPP

#include <cstdint>
#include <type_traits>

namespace iox2 {
namespace legacy {
/// @brief Conditionally add const to type T if C has the const qualifier
/// @tparam T is the type to conditionally add the const qualifier
/// @tparam Condition is the type which determines if the const qualifier needs to be added to T
///
template <typename T, typename C>
struct add_const_conditionally {
    using type = T;
};
template <typename T, typename C>
struct add_const_conditionally<T, const C> {
    using type = const T;
};
///
/// @brief Helper type for add_const_conditionally which adds const to type T if C has the const qualifier
///
template <typename T, typename C>
using add_const_conditionally_t = typename add_const_conditionally<T, C>::type;

///
/// @brief Helper value to bind a static_assert to a type
/// @code
/// static_assert(always_false_v<Foo>, "Not implemented for the given type!");
/// @endcode
///
template <typename>
constexpr bool always_false_v { false };

#if __cplusplus >= 201703L
template <typename C, typename... Cargs>
using invoke_result = std::invoke_result<C, Cargs...>;
#else
template <typename C, typename... Cargs>
using invoke_result = std::result_of<C(Cargs...)>;
#endif

///
/// @brief Verifies whether the passed Callable type is in fact invocable with the given arguments
///
template <typename Callable, typename... ArgTypes>
struct is_invocable {
    // This variant is chosen when Callable(ArgTypes) successfully resolves to a valid type, i.e. is invocable.
    template <typename C, typename... As>
    static constexpr std::true_type test(typename iox2::legacy::invoke_result<C, As...>::type*) noexcept {
        return {};
    }

    // AXIVION Next Construct AutosarC++19_03-A8.4.1 : we require a SFINEA failure case where all
    // parameter types (non invokable ones) are allowed, this can be achieved with variadic arguments
    // This is chosen if Callable(ArgTypes) does not resolve to a valid type.
    template <typename C, typename... As>
    // NOLINTNEXTLINE(cert-dcl50-cpp)
    static constexpr std::false_type test(...) noexcept {
        return {};
    }

    // Test with nullptr as this can stand in for a pointer to any type.
    static constexpr bool value { decltype(test<Callable, ArgTypes...>(nullptr))::value };
};

///
/// @brief Verifies whether the passed Callable type is in fact invocable with the given arguments
///        and the result of the invocation is convertible to ReturnType.
///
/// @note This is an implementation of std::is_invokable_r (C++17).
///
template <typename ReturnType, typename Callable, typename... ArgTypes>
struct is_invocable_r {
    template <typename C, typename... As>
    static constexpr std::true_type
    test(std::enable_if_t<
         std::is_convertible<typename iox2::legacy::invoke_result<C, As...>::type, ReturnType>::value>*) noexcept {
        return {};
    }
    // AXIVION Next Construct AutosarC++19_03-A8.4.1 : we require a SFINEA failure case where all
    // parameter types (non invokable ones) are allowed, this can be achieved with variadic arguments
    template <typename C, typename... As>
    // NOLINTNEXTLINE(cert-dcl50-cpp)
    static constexpr std::false_type test(...) noexcept {
        return {};
    }

    // Test with nullptr as this can stand in for a pointer to any type.
    static constexpr bool value { decltype(test<Callable, ArgTypes...>(nullptr))::value };
};

///
/// @brief Check whether T is a function pointer with arbitrary signature
///
template <typename T>
struct is_function_pointer : std::false_type { };
template <typename ReturnType, typename... ArgTypes>
struct is_function_pointer<ReturnType (*)(ArgTypes...)> : std::true_type { };

/// @brief struct to check whether an argument is a char array
template <typename T>
struct is_char_array : std::false_type { };

template <uint64_t N>
// AXIVION DISABLE STYLE AutosarC++19_03-A18.1.1 : struct used to deduce char array types, it does not use them
// NOLINTNEXTLINE(hicpp-avoid-c-arrays,cppcoreguidelines-avoid-c-arrays)
struct is_char_array<char[N]> : std::true_type { };
// AXIVION ENABLE STYLE AutosarC++19_03-A18.1.1

/// @brief Maps a sequence of any types to the type void
template <typename...>
using void_t = void;

/// @brief Implementation C++17 bool_constant helper
template <bool B>
using bool_constant = std::integral_constant<bool, B>;

/// @brief Implementation of C++17 negation
template <class B>
struct negation : bool_constant<!bool(B::value)> { };

template <bool...>
struct bool_pack { };

/// @brief Implementation of C++17 std::conjunction
template <class...>
struct conjunction : std::true_type { };

template <class Arg>
struct conjunction<Arg> : Arg { };

template <class Arg, class... Args>
struct conjunction<Arg, Args...> : std::conditional_t<!bool(Arg::value), Arg, conjunction<Args...>> { };

/// @brief Implementation of C++20's std::remove_cvref.
//
// References:
// - https://en.cppreference.com/w/cpp/types/remove_cvref
// - https://wg21.link/meta.trans.other#lib:remove_cvref
template <typename T>
struct remove_cvref {
    using type_t = std::remove_cv_t<std::remove_reference_t<T>>;
};

/// @brief Implementation of C++20's std::remove_cvref_t.
//
// References:
// - https://en.cppreference.com/w/cpp/types/remove_cvref
// - https://wg21.link/meta.type.synop#lib:remove_cvref_t
template <typename T>
using remove_cvref_t = typename remove_cvref<T>::type_t;

template <typename T>
using is_c_array_t = std::is_array<std::remove_reference_t<T>>;

template <typename T>
using is_not_c_array_t = iox2::legacy::negation<is_c_array_t<T>>;

template <typename From, typename To>
// NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays, hicpp-avoid-c-arrays)
using is_convertible_t = std::is_convertible<From (*)[], To (*)[]>;

template <typename Iter>
using iter_reference_t = decltype(*std::declval<Iter&>());

template <typename Iter, typename T>
using iter_has_convertible_ref_type_t =
    iox2::legacy::is_convertible_t<std::remove_reference_t<iter_reference_t<Iter>>, T>;

/// @brief Helper template from C++17
/// @tparam From Source type
/// @tparam To Destination type
template <class From, class To>
constexpr bool is_convertible_v = std::is_convertible<From, To>::value;

} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_PRIMITIVES_TYPE_TRAITS_HPP
