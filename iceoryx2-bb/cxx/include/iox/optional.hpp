// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT
#ifndef IOX_HOOFS_VOCABULARY_OPTIONAL_HPP
#define IOX_HOOFS_VOCABULARY_OPTIONAL_HPP

#include "iox/assertions.hpp"
#include "iox/functional_interface.hpp"
#include "iox/iceoryx_hoofs_types.hpp"

#include <new> // needed for placement new in the construct_value member function
#include <utility>

namespace iox
{
/// @brief Helper struct which is used to signal an empty optional.
///         It is equivalent to no value.
struct nullopt_t
{
};
// AXIVION Next Construct AutosarC++19_03-M17.0.2 : nullopt is defined within iox namespace which prevents easy
// misuse
constexpr nullopt_t nullopt{nullopt_t()};

/// @brief helper struct which is used to call the in-place-construction constructor
struct in_place_t
{
};

// AXIVION Next Construct AutosarC++19_03-M17.0.2 : in_place is defined within iox namespace which prevents easy
// misuse
constexpr in_place_t in_place{};

/// @brief Optional implementation from the C++17 standard with C++11. The
///         interface is analog to the C++17 standard and it can be used in
///         factory functions which can fail.
///
/// @code
///     #include "iox/optional.hpp"
///
///     optional<void*> SomeFactory() {
///         void *memory = malloc(1234);
///         if ( memory == nullptr )
///             return nullopt_t();
///         else
///             return make_optional<void*>(memory);
///     }
///
///     int main() {
///         auto var = SomeFactory();
///         // never forget the has_value call before working with an optional
///         if ( var.has_value() ) {
///             // do stuff with var
///         }
///     }
/// @endcode
template <typename T>
class optional final : public FunctionalInterface<optional<T>, T, void>
{
  public:
    using type = T;

    /// @brief Creates an optional which has no value. If you access such an
    ///         optional via .value() or the arrow operator the application
    ///         terminates.
    optional() noexcept;

    /// @brief Creates an optional which has no value. If you access such an
    ///         optional via .value() or the arrow operator the application
    ///         terminates.
    // NOLINTNEXTLINE(hicpp-explicit-conversions) the usage of 'nullopt' shall be transparent when used with an 'optional'
    optional(const nullopt_t) noexcept;

    /// @brief Creates an optional by forwarding value to the constructor of
    ///         T. This optional has a value.
    /// @param[in] value rvalue of type T which will be moved into the optional
    // AXIVION DISABLE STYLE AutosarC++19_03-A12.1.4 : the usage of 'T' shall be transparent when used with an 'optional'
    // NOLINTNEXTLINE(hicpp-explicit-conversions)
    optional(T&& value) noexcept;

    /// @brief Creates an optional by using the copy constructor of T.
    /// @param[in] value lvalue of type T which will be copy constructed into the optional
    // AXIVION DISABLE STYLE AutosarC++19_03-A12.1.4 : the usage of 'T' shall be transparent when used with an 'optional'
    // NOLINTNEXTLINE(hicpp-explicit-conversions)
    optional(const T& value) noexcept;

    /// @brief Creates an optional and an object inside the optional on construction by perfectly forwarding args to the
    /// constructor of T. Could be used e.g. when T is not copyable/movable.
    /// @tparam Targs is the template parameter pack for the perfectly forwarded arguments
    /// @param[in] in_place_t compile time variable to distinguish between constructors with certain behavior
    template <typename... Targs>
    // in_place_t is a compile time variable to call the in-place-construction
    // NOLINTNEXTLINE(hicpp-named-parameter, readability-named-parameter)
    explicit optional(in_place_t, Targs&&... args) noexcept;

    /// @brief The destructor will call the destructor of T if a value is set.
    ~optional() noexcept;

    /// @brief Constructs a value with the copy constructor if rhs has a value.
    ///         Otherwise it contains no value.
    /// @param[in] rhs source of the copy
    optional(const optional& rhs) noexcept;

    /// @brief Constructs a value with the move constructor if rhs has a value.
    ///         Otherwise it contains no value.
    /// @param[in] rhs source of the move
    optional(optional&& rhs) noexcept;

    /// @brief Copies an optional. If the optional has a value then the copy
    ///         assignment of that value is called. If the optional has no
    ///         value a new value is constructed with the copy constructor.
    /// @param[in] rhs source of the copy
    /// @return reference to the current optional
    optional& operator=(const optional& rhs) noexcept;

    /// @brief Moves an optional. If the optional has a value then the move
    ///         assignment of that value is called. If the optional has no
    ///         value a new value is constructed with the move constructor.
    /// @param[in] rhs source of the move
    /// @return reference to the current optional
    optional& operator=(optional&& rhs) noexcept;

    /// @brief Direct assignment of the underlying value. If the optional has no
    ///         value then a new T is constructed by forwarding the assignment to
    ///         T's constructor.
    ///        If the optional has a value the assignment operator of T is called.
    /// @param[in] value value to assign to the underlying optional value
    /// @return reference to the current optional
    template <typename U = T>
    // NOLINTNEXTLINE(cppcoreguidelines-c-copy-assignment-signature) return type is optional&
    typename std::enable_if<!std::is_same<U, optional<T>&>::value, optional>::type& operator=(U&& newValue) noexcept;

    /// @brief Returns a pointer to the underlying value. If the optional has no
    ///         value the application terminates. You need to verify that the
    ///         optional has a value by calling has_value() before using it.
    /// @return pointer of type const T to the underlying type
    const T* operator->() const noexcept;

    /// @brief Returns a reference to the underlying value. If the optional has no
    ///         value the application terminates. You need to verify that the
    ///         optional has a value by calling has_value() before using it.
    /// @return reference of type const T to the underlying type
    const T& operator*() const noexcept;

    /// @brief Returns a pointer to the underlying value. If the optional has no
    ///         value the application terminates. You need to verify that the
    ///         optional has a value by calling has_value() before using it.
    /// @return pointer of type T to the underlying type
    T* operator->() noexcept;

    /// @brief Returns a reference to the underlying value. If the optional has no
    ///         value the application terminates. You need to verify that the
    ///         optional has a value by calling has_value() before using it.
    /// @return reference of type T to the underlying type
    T& operator*() noexcept;

    // AXIVION Next Construct AutosarC++19_03-A13.5.3: Implemented to be as close to the STL interface as possible.
    // In combination with the keyword explicit accidental casts can be excluded and the usage is well known in the
    // C++ community.
    /// @brief Will return true if the optional contains a value, otherwise false.
    /// @return true if optional contains a value, otherwise false
    constexpr explicit operator bool() const noexcept;

    /// @brief Will return true if the optional contains a value, otherwise false.
    /// @return true if optional contains a value, otherwise false
    constexpr bool has_value() const noexcept;

    /// @brief A new element is constructed by forwarding the arguments to the
    ///         constructor of T.
    ///         If the optional has a value then the destructor of T is called.
    /// @param[in] perfectly forwards args to the constructor of T to perform a placement
    ///             new
    /// @return reference to the underlying type
    template <typename... Targs>
    T& emplace(Targs&&... args) noexcept;

    /// @brief Calls the destructor of T if the optional has a value.
    ///         If the optional has no value, nothing happens. After that call
    ///         the optional has no more value.
    void reset() noexcept;

    /// @brief Returns a reference to the underlying value. If the optional has no
    ///         value the application terminates. You need to verify that the
    ///         optional has a value by calling has_value() before using it.
    /// @return reference to the underlying type
    T& value() & noexcept;

    /// @brief Returns a const reference to the underlying value. If the optional has no
    ///         value the application terminates. You need to verify that the
    ///         optional has a value by calling has_value() before using it.
    /// @return const reference to the underlying type
    const T& value() const& noexcept;

    /// @brief Returns a rvalue reference to the underlying value. If the optional has no
    ///         value the application terminates. You need to verify that the
    ///         optional has a value by calling has_value() before using it.
    /// @return rvalue reference to the underlying type
    T&& value() && noexcept;

    /// @brief Returns a const rvalue reference to the underlying value. If the optional has no
    ///         value the application terminates. You need to verify that the
    ///         optional has a value by calling has_value() before using it.
    /// @return const rvalue reference to the underlying type
    const T&& value() const&& noexcept;

  private:
    // the hasValue member should be first member in the memory layout to reveal casting
    // bugs early.
    //
    // See the following problem:
    //   void initHandle(void * ptr) {
    //     Handle * handle = static_cast<Handle>(ptr);
    //   }
    //   void doStuff(optional<Handle> & handle) {
    //     // uses optional<Handle> instead of handle, if the bool is the second parameter such
    //     // mistakes can be introduced quickly in low level c abstraction classes
    //     initHandle(&handle);
    //   }
    bool m_hasValue{false};
    // AXIVION DISABLE STYLE AutosarC++19_03-A9.6.1 : False positive. Used type has defined size.
    struct alignas(T) element_t
    {
        // AXIVION Next Construct AutosarC++19_03-M0.1.3 : the field is intentionally unused and serves as a mean to provide memory
        // AXIVION Next Construct AutosarC++19_03-A1.1.1 : object size depends on template parameter and has to be taken care of at the specific template instantiation
        // AXIVION Next Construct AutosarC++19_03-A18.1.1 : required as low level building block, encapsulated in abstraction and not directly used
        // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
        byte data[sizeof(T)];
    };
    // AXIVION Next Construct AutosarC++19_03-A1.1.1 : object size depends on template parameter and has to be taken care of at the specific template instantiation
    element_t m_data;

  private:
    template <typename... Targs>
    void construct_value(Targs&&... args) noexcept;
    void destruct_value() noexcept;
};

// AXIVION Next Construct AutosarC++19_03-M17.0.3 : make_optional is defined within iox which prevents easy misuse
/// @brief Creates an optional which contains a value by forwarding the arguments
///         to the constructor of T.
/// @param[in] args arguments which will be perfectly forwarded to the constructor
///             of T
/// @return optional which contains T constructed with args
template <typename OptionalBaseType, typename... Targs>
optional<OptionalBaseType> make_optional(Targs&&... args) noexcept;

/// @brief Compare two optionals for equality.
/// @param[in] lhs optional
/// @param[in] rhs optional to which lhs should be compared to
/// @return true if the contained values are equal or both have no value, otherwise false
template <typename T>
bool operator==(const optional<T>& lhs, const optional<T>& rhs) noexcept;

/// @brief Compare two optionals for inequality.
/// @param[in] lhs optional
/// @param[in] rhs optional to which lhs should be compared to
/// @return true if the contained values are not equal, otherwise false
template <typename T>
bool operator!=(const optional<T>& lhs, const optional<T>& rhs) noexcept;

// AXIVION DISABLE STYLE AutosarC++19_03-A13.5.5: Comparison with nullopt_t is required
/// @brief Comparison for equality with nullopt_t for easier unset optional comparison
/// @param[in] lhs empty optional, nullopt_t
/// @param[in] rhs optional to which lhs should be compared to
/// @return true if the rhs is not set, otherwise false
template <typename T>
bool operator==(const nullopt_t, const optional<T>& rhs) noexcept;

/// @brief Comparison for equality with nullopt_t for easier unset optional comparison
/// @param[in] lhs optional which should be compared to nullopt_t
/// @param[in] rhs empty optional
/// @return true if the lhs is not set, otherwise false
template <typename T>
bool operator==(const optional<T>& lhs, const nullopt_t) noexcept;

/// @brief Comparison for inequality with nullopt_t for easier unset optional comparison
/// @param[in] lhs empty optional, nullopt_t
/// @param[in] rhs optional to which lhs should be compared to
/// @return true if the optional is set, otherwise false
template <typename T>
bool operator!=(const nullopt_t, const optional<T>& rhs) noexcept;

/// @brief Comparison for inequality with nullopt_t for easier unset optional comparison
/// @param[in] lhs optional which should be compared to nullopt_t
/// @param[in] rhs empty optional
/// @return true if the optional is set, otherwise false
template <typename T>
bool operator!=(const optional<T>& lhs, const nullopt_t) noexcept;

template <typename... T>
struct is_optional : std::false_type
{
};

template <typename T>
struct is_optional<optional<T>> : std::true_type
{
};
// AXIVION ENABLE STYLE AutosarC++19_03-A13.5.5
} // namespace iox

#include "iox/detail/optional.inl"

#endif // IOX_HOOFS_VOCABULARY_OPTIONAL_HPP
