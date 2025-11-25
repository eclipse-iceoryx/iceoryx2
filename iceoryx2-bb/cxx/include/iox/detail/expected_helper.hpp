// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX_HOOFS_VOCABULARY_EXPECTED_HELPER_HPP
#define IOX_HOOFS_VOCABULARY_EXPECTED_HELPER_HPP

#include "iox/optional.hpp"
#include "iox/variant.hpp"

namespace iox
{
/// @brief helper struct which is used to call the in-place-construction constructor for error types
struct unexpect_t
{
};
constexpr unexpect_t unexpect{};

/// @brief helper trait for SFINEA to disable specific functions for 'void' value type
/// @tparam T type to be checked for 'void'
template <typename T>
using enable_if_non_void_t = typename std::enable_if<!std::is_void<T>::value, T>::type;

/// @brief helper trait for SFINEA to disable specific functions for non 'void' value type
/// @tparam T type to be checked for 'void'
template <typename T>
using enable_if_void_t = typename std::enable_if<std::is_void<T>::value, T>::type;

/// @brief helper trait for SFINEA to disable specific functions for lvalue references
/// @tparam T type to be checked for lvalue reference
template <typename T>
using enable_if_not_lvalue_referece_t = typename std::enable_if<!std::is_lvalue_reference<T>::value, T>::type;

namespace detail
{
/// @brief helper struct to create an expected which is signalling success more easily
template <typename T = void>
struct ok
{
    // AXIVION Next Construct AutosarC++19_03-A12.1.5 : This is a false positive since there is no fitting constructor
    // available for delegation
    explicit ok(const T& t) noexcept
        : value(t)
    {
    }

    // AXIVION Next Construct AutosarC++19_03-A18.9.2 : For universal references std::forward must be used
    template <typename U = T, typename = enable_if_not_lvalue_referece_t<U>>
    // NOLINTNEXTLINE(cppcoreguidelines-rvalue-reference-param-not-moved) perfect forwarding is used
    explicit ok(T&& t) noexcept
        : value(std::forward<T>(t))
    {
    }

    // AXIVION Next Construct AutosarC++19_03-A15.4.2, FaultDetection-NoexceptViolations : Intentional behavior. 'ok' is not intended to be used with a type which throws
    template <typename... Targs>
    explicit ok(Targs&&... args) noexcept
        : value(std::forward<Targs>(args)...)
    {
    }

    T value;
};

/// @brief helper struct to handle 'void' value type specialization
template <>
struct ok<void>
{
    // dummy value
    bool value{true};
};

/// @brief helper struct to create an expected which is signalling an error more easily
template <typename T>
struct err
{
    // AXIVION Next Construct AutosarC++19_03-A12.1.5 : This is a false positive since there is no fitting constructor
    // available for delegation
    explicit err(const T& t) noexcept
        : value(t)
    {
    }

    // AXIVION Next Construct AutosarC++19_03-A18.9.2 : For universal references std::forward must be used
    template <typename U = T, typename = enable_if_not_lvalue_referece_t<U>>
    // NOLINTNEXTLINE(cppcoreguidelines-rvalue-reference-param-not-moved) perfect forwarding is used
    explicit err(T&& t) noexcept
        : value(std::forward<T>(t))
    {
    }

    template <typename... Targs>
    explicit err(Targs&&... args) noexcept
        : value(std::forward<Targs>(args)...)
    {
    }

    T value;
};

/// @brief helper class to be able to handle 'void' value type specialization
template <typename ValueType, typename ErrorType>
class expected_storage
{
  public:
    expected_storage() noexcept = delete;

    template <typename... Targs>
    explicit expected_storage(in_place_t, Targs&&... args)
        : data(in_place_index<VALUE_INDEX>(), std::forward<Targs>(args)...)
    {
    }

    template <typename... Targs>
    // NOLINTNEXTLINE(cppcoreguidelines-rvalue-reference-param-not-moved) perfect forwarding is used
    explicit expected_storage(unexpect_t, Targs&&... args)
        : data(in_place_index<ERROR_INDEX>(), std::forward<Targs>(args)...)
    {
    }

    bool has_value() const
    {
        return data.index() == VALUE_INDEX;
    }

    bool has_error() const
    {
        return data.index() == ERROR_INDEX;
    }

    ValueType& value_unchecked()
    {
        return *data.template get_at_index<VALUE_INDEX>();
    }

    const ValueType& value_unchecked() const
    {
        return *data.template get_at_index<VALUE_INDEX>();
    }

    ErrorType& error_unchecked()
    {
        return *data.template get_at_index<ERROR_INDEX>();
    }

    const ErrorType& error_unchecked() const
    {
        return *data.template get_at_index<ERROR_INDEX>();
    }

  private:
    static constexpr uint64_t VALUE_INDEX{0};
    static constexpr uint64_t ERROR_INDEX{1};

    iox::variant<ValueType, ErrorType> data;
};

/// @brief helper struct to handle 'void' value type specialization
template <typename ErrorType>
class expected_storage<void, ErrorType>
{
  public:
    expected_storage() noexcept = delete;

    template <typename... Targs>
    // NOLINTNEXTLINE(cppcoreguidelines-missing-std-forward) Targs is not used but required for template meta-programming
    explicit expected_storage(in_place_t, Targs&&...)
        : data(in_place_index<VALUE_INDEX>(), DUMMY_VALUE)
    {
    }

    template <typename... Targs>
    explicit expected_storage(unexpect_t, Targs&&... args)
        : data(in_place_index<ERROR_INDEX>(), std::forward<Targs>(args)...)
    {
    }

    bool has_value() const
    {
        return data.index() == VALUE_INDEX;
    }

    bool has_error() const
    {
        return data.index() == ERROR_INDEX;
    }

    void value_unchecked() const
    {
        // nothing to do
    }

    ErrorType& error_unchecked()
    {
        return *data.template get_at_index<ERROR_INDEX>();
    }

    const ErrorType& error_unchecked() const
    {
        return *data.template get_at_index<ERROR_INDEX>();
    }

  private:
    static constexpr uint64_t VALUE_INDEX{0};
    static constexpr uint64_t ERROR_INDEX{1};

    using DummyValueType = bool;
    static constexpr DummyValueType DUMMY_VALUE{true};

    iox::variant<DummyValueType, ErrorType> data;
};

template <typename ErrorType>
constexpr typename expected_storage<void, ErrorType>::DummyValueType expected_storage<void, ErrorType>::DUMMY_VALUE;

/// @brief helper struct for 'operator==' to be able to handle 'void' value type specialization
template <typename T, typename E>
struct compare_expected_value
{
    static constexpr bool is_same_value_unchecked(const expected_storage<T, E>& lhs, const expected_storage<T, E>& rhs)
    {
        return lhs.value_unchecked() == rhs.value_unchecked();
    }
};

/// @brief helper struct to handle 'void' value type specialization
template <typename E>
struct compare_expected_value<void, E>
{
    static constexpr bool is_same_value_unchecked(const expected_storage<void, E>&, const expected_storage<void, E>&)
    {
        return true;
    }
};

} // namespace detail
} // namespace iox

#endif // IOX_HOOFS_VOCABULARY_EXPECTED_HELPER_HPP
