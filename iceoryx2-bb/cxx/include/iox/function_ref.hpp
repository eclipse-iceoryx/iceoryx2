// Copyright (c) 2020 by Robert Bosch GmbH. All rights reserved.
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

// AXIVION DISABLE STYLE AutosarC++19_03-A16.2.3 : <type_traits> is included through "type_traits.hpp"

#ifndef IOX_HOOFS_FUNCTIONAL_FUNCTION_REF_HPP
#define IOX_HOOFS_FUNCTIONAL_FUNCTION_REF_HPP

#include "iox/type_traits.hpp"

#include <type_traits>

namespace iox
{
template <typename SignatureType>
class function_ref;

/// @brief Type trait which checks for the same decayed type
/// @tparam[in] T1 first type
/// @tparam[in] T2 second type
template <typename T1, typename T2>
using has_same_decayed_type = typename std::
    integral_constant<bool, bool(std::is_same<typename std::decay<T1>::type, typename std::decay<T2>::type>::value)>;

/// @brief iox::function_ref is a non-owning reference to a callable.
///
///        It has these features:
///         * No heap usage
///         * No exceptions
///         * Stateful lambda support
///         * C++11/14 support
///
/// @code
///         // Usage as function parameter
///         void fuu(iox::function_ref<void()> callback)
///         {
///             callback();
///         }
///         // Call the lambda
///         fuu([]{ doSomething(); });
///
///         // Usage with l-values
///         // Pitfall: Ensure that lifetime of callable suits the point in time of calling callback()
///         auto callable = [&]{ doSomething(); };
///         iox::function_ref<void()> callback(callable);
///         // Call the callback
///         callback();
/// @endcode
template <class ReturnType, class... ArgTypes>
class function_ref<ReturnType(ArgTypes...)> final
{
  public:
    ~function_ref() noexcept = default;

    function_ref(const function_ref&) noexcept = default;

    function_ref& operator=(const function_ref&) & noexcept = default;

    /// @brief Creates a function_ref with a callable whose lifetime has to be longer than function_ref
    /// @param[in] callable that is not a function_ref
    template <typename CallableType,
              typename = std::enable_if_t<((!is_function_pointer<CallableType>::value)
                                           && (!has_same_decayed_type<CallableType, function_ref>::value))
                                          && (is_invocable<CallableType, ArgTypes...>::value)>>
    // AXIVION Next Line AutosarC++19_03-A12.1.4 : Implicit conversion is needed for lambdas
    function_ref(CallableType&& callable) noexcept; // NOLINT(hicpp-explicit-conversions)

    /// @brief Creates a function_ref from a function pointer
    /// @param[in] function function reference to function we want to reference
    ///
    /// @note This overload is needed, as the general implementation
    /// will not work properly for function pointers.
    /// This ctor is not needed anymore once we can use user-defined-deduction guides (C++17)
    // Implicit conversion needed for method pointers
    // NOLINTNEXTLINE(hicpp-explicit-conversions)
    function_ref(ReturnType (&function)(ArgTypes...)) noexcept;

    function_ref(function_ref&& rhs) noexcept;

    function_ref& operator=(function_ref&& rhs) & noexcept;

    /// @brief Calls the provided callable
    /// @param[in] Arguments are forwarded to the underlying function pointer
    /// @return Returns the data type of the underlying function pointer
    ReturnType operator()(ArgTypes... args) const noexcept;

    /// @brief Swaps the contents of two function_ref's
    /// @param[in] Reference to another function_ref
    void swap(function_ref& rhs) noexcept;

  private:
    void* m_pointerToCallable{nullptr};
    ReturnType (*m_functionPointer)(void*, ArgTypes...){nullptr};
};

template <class ReturnType, class... ArgTypes>
void swap(function_ref<ReturnType(ArgTypes...)>& lhs, function_ref<ReturnType(ArgTypes...)>& rhs) noexcept;

} // namespace iox

// AXIVION Next Line AutosarC++19_03-M16.0.1 : Include needed to split template declaration and definition
#include "iox/detail/function_ref.inl"

#endif // IOX_HOOFS_FUNCTIONAL_FUNCTION_REF_HPP
