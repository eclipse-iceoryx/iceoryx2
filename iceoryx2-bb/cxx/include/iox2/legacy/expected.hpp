// Copyright (c) 2019 - 2020 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2020 - 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_VOCABULARY_EXPECTED_HPP
#define IOX2_BB_VOCABULARY_EXPECTED_HPP

#include "iox2/bb/detail/attributes.hpp"
#include "iox2/legacy/detail/expected_helper.hpp"

namespace iox2 {
namespace legacy {
template <typename T = void>
using success = detail::ok<T>;

template <typename T>
using error = detail::err<T>;

/// @brief convenience function to create an 'expected' with a 'void' value type
/// @tparam T helper template parameter for SFINEA
/// @code
///     expected<void, uint64_t> callMe() {
///         //...
///         return ok();
///     }
/// @endcode
template <typename T = void, typename = enable_if_void_t<T>>
detail::ok<void> ok();

/// @brief convenience function to create an 'expected' with a value type by copy
/// @tparam T value type for the 'expected'
/// @param[in] value is the value for the 'expected'
/// @code
///     expected<bool, uint64_t> callMe() {
///         //...
///         return ok(true);
///     }
/// @endcode
template <typename T, typename = enable_if_non_void_t<T>>
detail::ok<T> ok(const T& value);

/// @brief convenience function to create an 'expected' with a value type by move
/// @tparam T value type for the 'expected'
/// @param[in] value is the value for the 'expected'
/// @code
///     expected<MyClass, uint64_t> callMe() {
///         //...
///         MyClass m;
///         //...
///         return ok(std::move(m));
///     }
/// @endcode
template <typename T, typename = enable_if_non_void_t<T>, typename = enable_if_not_lvalue_referece_t<T>>
detail::ok<T> ok(T&& value);

/// @brief convenience function to create an 'expected' with a value type by argument forwarding
/// @tparam T value type for the 'expected'
/// @tparam Targs types for the constructor of the value type
/// @param[in] args... arguments which will be perfectly forwarded to the value type constructor
/// @code
///     expected<SomeClass, uint64_t> callMe() {
///         //...
///         return ok<SomeClass>(42, 73);
///     }
/// @endcode
template <typename T, typename... Targs, typename = enable_if_non_void_t<T>>
detail::ok<T> ok(Targs&&... args);

/// @brief convenience function to create an 'expected' with an error type by copy
/// @tparam T error type for the 'expected'
/// @param[in] error is the error for the 'expected'
/// @code
///     expected<bool, uint64_t> callMe() {
///         //...
///         return err(37);
///     }
/// @endcode
template <typename T>
detail::err<T> err(const T& error);

/// @brief convenience function to create an 'expected' with an error type by move
/// @tparam T error type for the 'expected'
/// @param[in] error is the error for the 'expected'
/// @code
///     expected<bool, MyError> callMe() {
///         //...
///         MyError e;
///         //...
///         return err(std::move(e));
///     }
/// @endcode
template <typename T, typename = enable_if_not_lvalue_referece_t<T>>
detail::err<T> err(T&& error);

/// @brief convenience function to create an 'expected' with an error type by argument forwarding
/// @tparam T error type for the 'expected'
/// @tparam Targs types for the constructor of the error type
/// @param[in] args... arguments which will be perfectly forwarded to the error type constructor
/// @code
///     expected<bool, SomeError> callMe() {
///         //...
///         return err<SomeError>(13, "Friday");
///     }
/// @endcode
template <typename T, typename... Targs>
detail::err<T> err(Targs&&... args);

template <typename ValueType, typename ErrorType>
class expected;

/// @brief equality check for two distinct expected types
/// @tparam ValueType type of the value stored in the expected
/// @tparam ErrorType type of the error stored in the expected
/// @param[in] lhs left side of the comparison
/// @param[in] rhs right side of the comparison
/// @return true if the expecteds are equal, otherwise false
template <typename ValueType, typename ErrorType>
constexpr bool operator==(const expected<ValueType, ErrorType>& lhs,
                          const expected<ValueType, ErrorType>& rhs) noexcept;

/// @brief inequality check for two distinct expected types
/// @tparam ValueType type of the value stored in the expected
/// @tparam ErrorType type of the error stored in the expected
/// @param[in] lhs left side of the comparison
/// @param[in] rhs right side of the comparison
/// @return true if the expecteds are not equal, otherwise false
template <typename ValueType, typename ErrorType>
constexpr bool operator!=(const expected<ValueType, ErrorType>& lhs,
                          const expected<ValueType, ErrorType>& rhs) noexcept;

/// @brief Implementation of the C++23 expected class which can contain an error or a success value
/// @param ValueType type of the value which can be stored in the expected
/// @param ErrorType type of the error which can be stored in the expected
template <typename ValueType, typename ErrorType>
class IOX2_NO_DISCARD expected final {
  public:
    /// @brief default ctor is deleted since you have to clearly state if the
    ///         expected contains a success value or an error value
    expected() = delete;

    /// @brief the copy constructor calls the copy constructor of the contained success value
    ///         or the error value - depending on what is stored in the expected
    expected(const expected&) noexcept = default;

    /// @brief the move constructor calls the move constructor of the contained success value
    ///         or the error value - depending on what is stored in the expected
    /// @note The move c'tor does not explicitly invalidate the moved-from object but relies on the move c'tor of
    /// ValueType or ErrorType to correctly invalidate the stored object
    expected(expected&& rhs) noexcept;

    /// @brief Creates an expected which is signaling success and perfectly forwards
    ///        the args to the constructor of ValueType
    /// @tparam Targs is the template parameter pack for the perfectly forwarded arguments
    /// @param[in] in_place_t compile time variable to distinguish between constructors with certain behavior
    /// @param[in] args... arguments which will be forwarded to the 'ValueType' constructor
    template <typename... Targs>
    explicit expected(in_place_t, Targs&&... args) noexcept;

    /// @brief Creates an expected which is signaling an error and perfectly forwards
    ///        the args to the constructor of ErrorType
    /// @tparam Targs is the template parameter pack for the perfectly forwarded arguments
    /// @param[in] unexpect_t compile time variable to distinguish between constructors with certain behavior
    /// @param[in] args... arguments which will be forwarded to the 'ErrorType' constructor
    template <typename... Targs>
    explicit expected(unexpect_t, Targs&&... args) noexcept;

    /// @brief calls the destructor of the success value or error value - depending on what
    ///         is stored in the expected
    ~expected() noexcept = default;

    /// @brief  calls the copy assignment operator of the contained success value
    ///         or the error value - depending on what is stored in the expected
    expected& operator=(const expected&) noexcept;

    /// @brief  calls the move assignment operator of the contained success value
    ///         or the error value - depending on what is stored in the expected
    /// @note The move assignment operator does not explicitly invalidate the moved-from object but relies on the move
    /// assignment operator of ValueType or ErrorType to correctly invalidate the stored object
    expected& operator=(expected&& rhs) noexcept;

    /// @brief  constructs an expected which is signaling success and uses the value
    ///         provided by successValue to copy construct its success value
    /// @param[in] successValue value which will be stored in the expected
    //
    // we would like to use 'return ok(myValue)' with an implicit
    // conversion to return an expected easily
    // NOLINTNEXTLINE(hicpp-explicit-conversions)
    expected(const detail::ok<ValueType>& successValue) noexcept;

    /// @brief  constructs an expected which is signaling success and uses the value
    ///         provided by successValue to move construct its success value
    /// @param[in] successValue value which will be moved into the expected
    //
    // we would like to use 'return ok(myValue)' with an implicit
    // conversion to return an expected easily
    // NOLINTNEXTLINE(hicpp-explicit-conversions)
    expected(detail::ok<ValueType>&& successValue) noexcept;

    /// @brief  constructs an expected which is signaling an error and stores the
    ///         error value provided by errorValue
    /// @param[in] errorValue error value which will be stored in the expected
    ///
    // we would like to use 'return err(myErrorValue)' with an implicit
    // conversion to return an expected easily
    // NOLINTNEXTLINE(hicpp-explicit-conversions)
    expected(const detail::err<ErrorType>& errorValue) noexcept;

    /// @brief  constructs an expected which is signaling an error and stores the
    ///         error value provided by errorValue
    /// @param[in] errorValue error value which will be moved into the expected
    ///
    // we would like to use 'return err(myErrorValue)' with an implicit
    // conversion to return an expected easily
    // NOLINTNEXTLINE(hicpp-explicit-conversions)
    expected(detail::err<ErrorType>&& errorValue) noexcept;

    // AXIVION Next Construct AutosarC++19_03-A13.5.3: Implementation is inspired from std::expected
    /// @brief  returns true if the expected contains a value type and false if it is an error type
    /// @return bool which contains true if the expected contains a value type and false if it is an error type
    explicit operator bool() const noexcept;

    /// @brief  returns true if the expected contains a value type and false if it is an error type
    /// @return bool which contains true if the expected contains an error
    bool has_value() const noexcept;

    /// @brief  returns true if the expected contains an error otherwise false
    /// @return bool which contains true if the expected contains an error
    bool has_error() const noexcept;

    /// @brief  returns a lvalue reference to the contained error value, if the expected
    ///         does not contain an error the error handler is called
    /// @return lvalue reference to the internally contained error
    ErrorType& error() & noexcept;

    /// @brief  returns a const lvalue reference to the contained error value, if the expected
    ///         does not contain an error the error handler is called
    /// @return const lvalue reference to the internally contained error
    const ErrorType& error() const& noexcept;

    /// @brief  returns a rvalue reference to the contained error value, if the expected
    ///         does not contain an error the error handler is called
    /// @return rvalue reference to the internally contained error
    ErrorType&& error() && noexcept;

    /// @brief  returns a const rvalue reference to the contained error value, if the expected
    ///         does not contain an error the error handler is called
    /// @return const rvalue reference to the internally contained error
    const ErrorType&& error() const&& noexcept;

    /// @brief  returns a lvalue reference to the contained success value, if the expected
    ///         does not contain a success value the error handler is called
    /// @tparam U helper template parameter for SFINEA
    /// @return lvalue reference to the internally contained value
    /// @note this only works for non void ValueTypes
    template <typename U = ValueType>
    enable_if_non_void_t<U>& value() & noexcept;

    /// @brief  returns a const lvalue reference to the contained success value, if the expected
    ///         does not contain a success value the error handler is called
    /// @tparam U helper template parameter for SFINEA
    /// @return const lvalue reference to the internally contained value
    /// @note this only works for non void ValueTypes
    template <typename U = ValueType>
    const enable_if_non_void_t<U>& value() const& noexcept;

    /// @brief  returns a rvalue reference to the contained success value, if the expected
    ///         does not contain a success value the error handler is called
    /// @tparam U helper template parameter for SFINEA
    /// @return rvalue reference to the internally contained value
    template <typename U = ValueType>
    enable_if_non_void_t<U>&& value() && noexcept;

    /// @brief  returns a const rvalue reference to the contained success value, if the expected
    ///         does not contain a success value the error handler is called
    /// @tparam U helper template parameter for SFINEA
    /// @return const rvalue reference to the internally contained value
    template <typename U = ValueType>
    const enable_if_non_void_t<U>&& value() const&& noexcept;

    /// @brief dereferencing operator which returns a reference to the contained
    ///         success value. if the expected contains an error the error handler is called
    /// @tparam U helper template parameter for SFINEA
    /// @return reference to the contained value
    /// @note this only works for non void ValueTypes
    /// @code
    ///     expected<int, float> frodo(ok(45));
    ///     *frodo += 12;
    ///     IOX2_LOG(Info, *frodo); // prints 57
    /// @endcode
    template <typename U = ValueType>
    enable_if_non_void_t<U>& operator*() noexcept;

    /// @brief dereferencing operator which returns a reference to the contained
    ///         success value. if the expected contains an error the error handler is called
    /// @tparam U helper template parameter for SFINEA
    /// @return const reference to the contained value
    /// @note this only works for non void ValueTypes
    /// @code
    ///     expected<int, float> frodo(ok(45));
    ///     *frodo += 12;
    ///     IOX2_LOG(Info, *frodo); // prints 57
    /// @endcode
    template <typename U = ValueType>
    const enable_if_non_void_t<U>& operator*() const noexcept;

    /// @brief arrow operator which returns the pointer to the contained success value
    ///         if the expected contains an error the error handler is called
    /// @tparam U helper template parameter for SFINEA
    /// @return pointer of type ValueType to the contained value
    /// @note this only works for non void ValueTypes
    /// @code
    ///     expected<std::vector<int>, int> holyPiotr(ok<std::vector<int>>({1,2,3}));
    ///     holyPiotr->push_back(4);
    /// @endcode
    template <typename U = ValueType>
    enable_if_non_void_t<U>* operator->() noexcept;

    /// @brief arrow operator which returns the pointer to the contained success value
    ///         if the expected contains an error the the error handler is called
    /// @tparam U helper template parameter for SFINEA
    /// @return pointer of type const ValueType to the contained value
    /// @note this only works for non void ValueTypes
    /// @code
    ///     expected<std::vector<int>, int> holyPiotr(ok<std::vector<int>>({1,2,3}));
    ///     holyPiotr->push_back(4);
    /// @endcode
    template <typename U = ValueType>
    const enable_if_non_void_t<U>* operator->() const noexcept;

    /// @brief conversion operator to a 'void' value type expected which can be useful
    ///         if you would like to return only the success of a function
    /// @return converts an expected which can contain a value and an error to an
    ///         expected which contains only an error
    /// @code
    ///     expected<int, int> someErrorProneFunction(){}
    ///
    ///     expected<void, int> isItSuccessful() {
    ///         return someErrorProneFunction();
    ///     }
    /// @endcode
    //
    // AXIVION Next Construct AutosarC++19_03-A13.5.2 , AutosarC++19_03-A13.5.3: see doxygen brief section
    // template <typename E>
    // NOLINTNEXTLINE(hicpp-explicit-conversions)
    operator expected<void, ErrorType>() const noexcept;

    template <typename T, typename E>
    friend constexpr bool ::iox2::legacy::operator==(const expected<T, E>&, const expected<T, E>&) noexcept;

  private:
    template <typename U = ValueType>
    enable_if_non_void_t<U>& value_checked() & noexcept;
    template <typename U = ValueType>
    const enable_if_non_void_t<U>& value_checked() const& noexcept;

    ErrorType& error_checked() & noexcept;
    const ErrorType& error_checked() const& noexcept;

  private:
    detail::expected_storage<ValueType, ErrorType> m_store;
};

} // namespace legacy
} // namespace iox2

#include "iox2/legacy/detail/expected.inl"

#endif // IOX2_BB_VOCABULARY_EXPECTED_HPP
