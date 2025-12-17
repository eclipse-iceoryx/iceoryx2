// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_VOCABULARY_VARIANT_HPP
#define IOX2_BB_VOCABULARY_VARIANT_HPP

#include "iox2/legacy/detail/platform_correction.hpp"

#include "iox2/legacy/detail/variant_internal.hpp"

#include <cstdint>
#include <iostream>
#include <limits>
#include <type_traits>

namespace iox2 {
namespace legacy {
/// @brief helper struct to perform an emplacement at a predefined index
///        in the constructor of a variant
/// @tparam[in] N index where to perform the placement new
/// @code
///     variant<int, float, int> someVariant(in_place_index<2>(), 42);
/// @endcode
template <uint64_t N>
struct in_place_index {
    static constexpr uint64_t value { N };
};

/// @brief helper struct to perform an emplacement of a predefined type in
///        in the constructor of a variant
/// @tparam[in] T type which should be created
/// @code
///     variant<int, float, double> someVariant(in_place_type<float>(), 123.456f);
/// @endcode
template <typename T>
struct in_place_type {
    using type = T;
};

/// @brief value which an invalid variant index occupies
/// @code
///     variant<int, float> someVariant;
///
///     // every unset variant has an invalid variant in the beginning
///     if ( someVariant.index() == INVALID_VARIANT_INDEX ) ...
///
///     variant<int, float> someVariant2(in_place_type<int>, 12);
///
///     // variant with setted value therefore the index is not invalid
///     if ( someVariant.index() != INVALID_VARIANT_INDEX ) ...
/// @endcode
static constexpr uint64_t INVALID_VARIANT_INDEX { std::numeric_limits<uint64_t>::max() };

namespace detail {
/// template recursion stopper for maximum size calculation
template <std::size_t S = 0>
constexpr std::size_t maxSize() noexcept {
    return S;
}

/// calculate maximum size of supplied types
template <typename T, typename... Args>
constexpr std::size_t maxSize() noexcept {
    return (sizeof(T) > maxSize<Args...>()) ? sizeof(T) : maxSize<Args...>();
}
} // namespace detail

/// @brief Variant implementation from the C++17 standard with C++11. The
///         interface is inspired by the C++17 standard but it has changes in
///         get and emplace since we are not allowed to throw exceptions.
/// @param Types... variadic list of types which the variant should be able to store
///
/// @code
///     #include "iox2/legacy/variant.hpp"
///     #include <iostream>
///
///     variant<int, float, double> someVariant;
///
///     // ... do stuff
///
///     if ( someVariant.index() == INVALID_VARIANT_INDEX )
///     {
///         someVariant.emplace<float>(123.456f);
///     }
///     else if ( someVariant.index() == 1)
///     {
///         auto blubb = someVariant.template get_at_index<1>();
///         IOX2_LOG(Info, *blubb);
///
///         auto sameAsBlubb = someVariant.get<float>();
///         IOX2_LOG(Info, *sameAsBlubb);
///     }
///
///     // .. do stuff
///
///     int defaultValue = 123;
///     int * fuu = someVariant.get_if<int>(&defaultValue);
///     IOX2_LOG(Info, *fuu);
///
/// @endcode
template <typename... Types>
class variant final {
  private:
    /// @brief contains the size of the largest element
    static constexpr uint64_t TYPE_SIZE { detail::maxSize<Types...>() };

  public:
    /// @brief the default constructor constructs a variant which does not contain
    ///     an element and returns INVALID_VARIANT_INDEX when .index() is called
    constexpr variant() noexcept = default;

    /// @brief creates a variant and perform an in place construction of the type
    ///         stored at index N. If the index N is out of bounds you get a compiler
    ///         error.
    /// @tparam[in] N index where to perform the placement new
    /// @tparam[in] CTorArguments variadic types of the c'tor arguments
    /// @param[in] index index of the type which should be constructed
    /// @param[in] args variadic list of arguments which will be forwarded to the constructor to
    ///                 the type at index
    template <uint64_t N, typename... CTorArguments>
    constexpr explicit variant(const in_place_index<N>& index, CTorArguments&&... args) noexcept;

    /// @brief creates a variant and perform an in place construction of the type T.
    ///         If T is not part of the variant you get a compiler error.
    /// @tparam[in] T type which should be created inside the variant
    /// @tparam[in] CTorArguments variadic types of the c'tor arguments
    /// @param[in] type type which should be created inside the variant
    /// @param[in] args variadic list of arguments which will be forwarded to the constructor to
    ///                 the type
    template <typename T, typename... CTorArguments>
    constexpr explicit variant(const in_place_type<T>& type, CTorArguments&&... args) noexcept;

    /// @brief creates a variant from a user supplied value
    /// @tparam[in] T type of the value to be stored in the variant
    /// @param[in] arg arg to be forwared to the c'tor of T
    template <typename T,
              typename = std::enable_if_t<!std::is_same<std::decay_t<T>, variant>::value>,
              typename std::enable_if_t<!internal::is_in_place_index<std::decay_t<T>>::value, bool> = false,
              typename std::enable_if_t<!internal::is_in_place_type<std::decay_t<T>>::value, bool> = false>
    constexpr explicit variant(T&& arg) noexcept;

    /// @brief if the variant contains an element the elements copy constructor is called
    ///     otherwise an empty variant is copied
    /// @param[in] rhs source of the copy
    constexpr variant(const variant& rhs) noexcept;

    /// @brief if the variant contains an element the elements copy assignment operator is called
    ///     otherwise an empty variant is copied
    /// @param[in] rhs source of the copy assignment
    /// @return reference to the variant itself
    constexpr variant& operator=(const variant& rhs) noexcept;

    /// @brief if the variant contains an element the elements move constructor is called
    ///     otherwise an empty variant is moved
    /// @param[in] rhs source of the move
    /// @note The move c'tor does not explicitly invalidate the moved-from object but relies on the move c'tor of
    /// Types to correctly invalidate the stored object
    constexpr variant(variant&& rhs) noexcept;

    /// @brief if the variant contains an element the elements move assignment operator is called
    ///     otherwise an empty variant is moved
    /// @param[in] rhs source of the move assignment
    /// @return reference to the variant itself
    /// @note The move assignment operator does not explicitly invalidate the moved-from object but relies on the move
    /// assignment operator of Types to correctly invalidate the stored object
    constexpr variant& operator=(variant&& rhs) noexcept;

    /// @brief if the variant contains an element the elements destructor is called otherwise
    ///         nothing happens
    ~variant() noexcept;

    /// @brief if the variant contains an element the elements assignment operator is called otherwise
    ///         we have undefined behavior. It is important that you make sure that the variant really
    ///         contains that type T.
    /// @tparam[in] T Type of the rhs
    /// @param[in] rhs source object for the underlying move assignment
    /// @return reference to the variant itself
    template <typename T>
    // NOLINTJUSTIFICATION Correct return type is used through enable_if
    // NOLINTNEXTLINE(cppcoreguidelines-c-copy-assignment-signature)
    typename std::enable_if<!std::is_same<T, variant<Types...>&>::value, variant<Types...>>::type&
    operator=(T&& rhs) noexcept;

    /// @brief calls the constructor of the type at index TypeIndex and perfectly forwards the arguments
    ///         to this constructor. (not stl compliant)
    /// @tparam TypeIndex index of the type which will be created
    /// @tparam CTorArguments variadic types of the c'tor arguments
    /// @param[in] args arguments which will be forwarded to the constructor to the type at TypeIndex
    template <uint64_t TypeIndex, typename... CTorArguments>
    void emplace_at_index(CTorArguments&&... args) noexcept;

    /// @brief calls the constructor of the type T and perfectly forwards the arguments
    ///         to the constructor of T.
    /// @tparam[in] T type which is created inside the variant
    /// @tparam[in] CTorArguments variadic types of the c'tor arguments
    template <typename T, typename... CTorArguments>
    void emplace(CTorArguments&&... args) noexcept;

    /// @brief returns a pointer to the type stored at index TypeIndex. (not stl compliant)
    /// @tparam[in] TypeIndex index of the stored type
    /// @return if the variant does contain the type at index TypeIndex it returns a valid
    ///             pointer, if it does contain no type at all or a different type it returns
    ///             nullptr.
    /// @code
    ///     variant<int, float> someVariant(in_place_type<int>(), 12);
    ///     int * someNumber = someVariant.template get_at_index<0>();
    /// @endcode
    template <uint64_t TypeIndex>
    typename internal::get_type_at_index<0, TypeIndex, Types...>::type* get_at_index() noexcept;

    /// @brief returns a pointer to the type stored at index TypeIndex. (not stl compliant)
    /// @tparam[in] TypeIndex index of the stored type
    /// @return a pointer to the type at index
    /// @attention this function is unsafe and does not check if the type at the index is the current active one; only
    /// use this function when it is ensured that the type at index is indeed the current active one; it is undefined
    /// behavior to call this on an index which is not the active type
    template <uint64_t TypeIndex>
    typename internal::get_type_at_index<0, TypeIndex, Types...>::type* unsafe_get_at_index_unchecked() noexcept;

    /// @brief returns a pointer to the type stored at index TypeIndex. (not stl compliant)
    /// @tparam[in] TypeIndex index of the stored type
    /// @return if the variant does contain the type at index TypeIndex it returns a valid
    ///             pointer, if it does contain no type at all or a different type it returns
    ///             nullptr.
    /// @code
    ///     variant<int, float> someVariant(in_place_type<int>(), 12);
    ///     int * someNumber = someVariant.template get_at_index<0>();
    /// @endcode
    template <uint64_t TypeIndex>
    const typename internal::get_type_at_index<0, TypeIndex, Types...>::type* get_at_index() const noexcept;

    /// @brief returns a pointer to the type stored at index TypeIndex. (not stl compliant)
    /// @tparam[in] TypeIndex index of the stored type
    /// @return a const pointer to the type at index
    /// @attention this function is unsafe and does not check if the type at the index is the current active one; only
    /// use this function when it is ensured that the type at index is indeed the current active one; it is undefined
    /// behavior to call this on an index which is not the active type
    template <uint64_t TypeIndex>
    const typename internal::get_type_at_index<0, TypeIndex, Types...>::type*
    unsafe_get_at_index_unchecked() const noexcept;

    /// @brief returns a pointer to the type T stored in the variant. (not stl compliant)
    /// @tparam[in] T type of the returned pointer
    /// @return if the variant does contain the type T it returns a valid pointer otherwise
    ///         if the variant does contain no type at all or a different type it returns
    ///         nullptr
    template <typename T>
    const T* get() const noexcept;

    /// @brief returns a pointer to the type T stored in the variant. (not stl compliant)
    /// @tparam[in] T type of the returned pointer
    /// @return if the variant does contain the type T it returns a valid pointer otherwise
    ///         if the variant does contain no type at all or a different type it returns
    ///         nullptr
    template <typename T>
    T* get() noexcept;

    /// @brief returns a pointer to the type T if its stored in the variant otherwise
    ///         it returns the provided defaultValue
    /// @return pointer to the stored value if it is of type T, otherwise defaultValue
    template <typename T>
    T* get_if(T* defaultValue) noexcept;

    /// @brief returns a pointer to the type T if its stored in the variant otherwise
    ///         it returns the provided defaultValue
    /// @tparam[in] T type of the returned pointer
    /// @param[in] defaultValue value which is returned in case of empty or different type in variant
    /// @return pointer to the stored value if it is of type T, otherwise defaultValue
    template <typename T>
    const T* get_if(const T* defaultValue) const noexcept;

    /// @brief returns the index of the stored type in the variant. if the variant does
    ///         not contain any type it returns INVALID_VARIANT_INDEX
    /// @return index of the stored type
    constexpr uint64_t index() const noexcept;

  private:
    // AXIVION Next Construct AutosarC++19_03-A9.6.1 : false positive. internal::byte_t is a type alias for uint8_t
    struct alignas(Types...) storage_t {
        // AXIVION Next Construct AutosarC++19_03-M0.1.3 : data provides the actual storage and is accessed via m_storage since &m_storage.data = &m_storage
        // AXIVION Next Construct AutosarC++19_03-A18.1.1 : safe access is guaranteed since the c-array is wrapped inside the variant class
        // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays)
        char data[TYPE_SIZE];
    };
    storage_t m_storage {};
    uint64_t m_type_index { INVALID_VARIANT_INDEX };

  private:
    template <typename T>
    bool has_bad_variant_element_access() const noexcept;
    // AXIVION Next Construct AutosarC++19_03-A3.9.1 : internal convenience function to easily use IOX2_LOG
    static void error_message(const char* source, const char* msg) noexcept;

    void call_element_destructor() noexcept;

    template <typename... Ts>
    friend constexpr bool operator==(const variant<Ts...>& lhs, const variant<Ts...>& rhs) noexcept;
    template <typename... Ts>
    friend constexpr bool operator!=(const variant<Ts...>& lhs, const variant<Ts...>& rhs) noexcept;
};

/// @brief returns true if the variant holds a given type T, otherwise false
template <typename T, typename... Types>
constexpr bool holds_alternative(const variant<Types...>& variant) noexcept;

/// @brief equality check for two distinct variant types
/// @tparam Types variadic number of types which can be stored in the variant
/// @param[in] lhs left side of the comparison
/// @param[in] rhs right side of the comparison
/// @return true if the variants are equal, otherwise false
template <typename... Types>
constexpr bool operator==(const variant<Types...>& lhs, const variant<Types...>& rhs) noexcept;

/// @brief inequality check for two distinct variant types
/// @tparam Types variadic number of types which can be stored in the variant
/// @param[in] lhs left side of the comparison
/// @param[in] rhs right side of the comparison
/// @return true if the variants are not equal, otherwise false
template <typename... Types>
constexpr bool operator!=(const variant<Types...>& lhs, const variant<Types...>& rhs) noexcept;

} // namespace legacy
} // namespace iox2

#include "iox2/legacy/detail/variant.inl"

#endif // IOX2_BB_VOCABULARY_VARIANT_HPP
