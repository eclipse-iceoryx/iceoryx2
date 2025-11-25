// Copyright (c) 2022 - 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX_HOOFS_DESIGN_FUNCTIONAL_INTERFACE_INL
#define IOX_HOOFS_DESIGN_FUNCTIONAL_INTERFACE_INL

#include "iox/detail/string_type_traits.hpp"
#include "iox/functional_interface.hpp"

namespace iox
{
namespace internal
{
///////////////
// BEGIN expect
///////////////
template <typename Derived>
template <typename StringType>
inline void Expect<Derived>::expect(const StringType& msg) const noexcept
{
    static_assert(is_char_array<StringType>::value || is_iox_string<StringType>::value,
                  "Only char arrays and iox::strings are allowed as message type.");

    const auto& derivedThis = *static_cast<const Derived*>(this);

    if (!derivedThis)
    {
        IOX_PANIC(&msg[0]);
    }
}

template <typename Derived, typename ValueType>
template <typename StringType>
inline ValueType& ExpectWithValue<Derived, ValueType>::expect(const StringType& msg) & noexcept
{
    static_assert(is_char_array<StringType>::value || is_iox_string<StringType>::value,
                  "Only char arrays and iox::strings are allowed as message type.");

    auto& derivedThis = *static_cast<Derived*>(this);

    if (!derivedThis)
    {
        IOX_PANIC(&msg[0]);
    }

    return derivedThis.value();
}

template <typename Derived, typename ValueType>
template <typename StringType>
inline const ValueType& ExpectWithValue<Derived, ValueType>::expect(const StringType& msg) const& noexcept
{
    using Self = ExpectWithValue<Derived, ValueType>;
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : const_cast avoids code duplication, is safe since the constness of the return value is restored
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<const ValueType&>(const_cast<Self*>(this)->expect(msg));
}

template <typename Derived, typename ValueType>
template <typename StringType>
inline ValueType&& ExpectWithValue<Derived, ValueType>::expect(const StringType& msg) && noexcept
{
    return std::move(this->expect(msg));
}

template <typename Derived, typename ValueType>
template <typename StringType>
inline const ValueType&& ExpectWithValue<Derived, ValueType>::expect(const StringType& msg) const&& noexcept
{
    using Self = ExpectWithValue<Derived, ValueType>;
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : const_cast avoids code duplication, is safe since the constness of the return value is restored
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<const ValueType&&>(std::move(const_cast<Self*>(this)->expect(msg)));
}
// END expect

/////////////////
// BEGIN value_or
/////////////////

// AXIVION Next Construct AutosarC++19_03-A13.3.1 : overload is invoked only on const lvalues
template <typename Derived, typename ValueType>
template <typename U>
inline ValueType ValueOr<Derived, ValueType>::value_or(U&& alternative) const& noexcept
{
    const auto& derivedThis = *static_cast<const Derived*>(this);

    if (!derivedThis)
    {
        return std::forward<U>(alternative);
    }

    return derivedThis.value();
}

// AXIVION Next Construct AutosarC++19_03-A13.3.1 : overload is invoked only on rvalues
template <typename Derived, typename ValueType>
template <typename U>
inline ValueType ValueOr<Derived, ValueType>::value_or(U&& alternative) && noexcept
{
    auto& derivedThis = *static_cast<Derived*>(this);

    if (!derivedThis)
    {
        return std::forward<U>(alternative);
    }

    return std::move(derivedThis.value());
}
// END value_or

/////////////////
// BEGIN and_then
/////////////////
template <typename Derived, typename ValueType>
template <typename Functor>
inline Derived& AndThenWithValue<Derived, ValueType>::and_then(const Functor& callable) & noexcept
{
    static_assert(is_invocable<Functor, ValueType&>::value,
                  "Only callables with a signature of void(ValueType&) are allowed!");

    auto& derivedThis = *static_cast<Derived*>(this);

    if (derivedThis)
    {
        const auto callback = static_cast<and_then_callback_t>(callable);
        callback(derivedThis.value());
    }

    return derivedThis;
}

template <typename Derived, typename ValueType>
template <typename Functor>
inline Derived&& AndThenWithValue<Derived, ValueType>::and_then(const Functor& callable) && noexcept
{
    return std::move(this->and_then(callable));
}

template <typename Derived, typename ValueType>
template <typename Functor>
inline const Derived& AndThenWithValue<Derived, ValueType>::and_then(const Functor& callable) const& noexcept
{
    static_assert(is_invocable<Functor, const ValueType&>::value,
                  "Only callables with a signature of void(const ValueType&) are allowed!");

    const auto& derivedThis = *static_cast<const Derived*>(this);

    if (derivedThis)
    {
        const auto callback = static_cast<const_and_then_callback_t>(callable);
        callback(derivedThis.value());
    }

    return derivedThis;
}

template <typename Derived, typename ValueType>
template <typename Functor>
inline const Derived&& AndThenWithValue<Derived, ValueType>::and_then(const Functor& callable) const&& noexcept
{
    return std::move(this->and_then(callable));
}

template <typename Derived>
inline Derived& AndThen<Derived>::and_then(const and_then_callback_t& callable) & noexcept
{
    auto& derivedThis = *static_cast<Derived*>(this);

    if (derivedThis)
    {
        callable();
    }

    return derivedThis;
}

template <typename Derived>
inline const Derived& AndThen<Derived>::and_then(const and_then_callback_t& callable) const& noexcept
{
    using Self = AndThen<Derived>;
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : const_cast avoids code duplication, is safe since the constness of the return value is restored
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<const Derived&>(const_cast<Self*>(this)->and_then(callable));
}

template <typename Derived>
inline Derived&& AndThen<Derived>::and_then(const and_then_callback_t& callable) && noexcept
{
    return std::move(this->and_then(callable));
}

template <typename Derived>
inline const Derived&& AndThen<Derived>::and_then(const and_then_callback_t& callable) const&& noexcept
{
    using Self = AndThen<Derived>;
    // AXIVION DISABLE STYLE AutosarC++19_03-A5.2.3 : const_cast avoids code duplication, is safe since the constness of the return value is restored
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return std::move(const_cast<const Derived&>(const_cast<Self*>(this)->and_then(callable)));
    // AXIVION ENABLE STYLE AutosarC++19_03-A5.2.3
}
// END and_then

////////////////
// BEGIN or_else
////////////////
template <typename Derived, typename ErrorType>
template <typename Functor>
inline Derived& OrElseWithValue<Derived, ErrorType>::or_else(const Functor& callable) & noexcept
{
    static_assert(is_invocable<Functor, ErrorType&>::value,
                  "Only callables with a signature of void(ErrorType&) are allowed!");

    auto& derivedThis = *static_cast<Derived*>(this);

    if (!derivedThis)
    {
        const auto callback = static_cast<or_else_callback_t>(callable);
        callback(derivedThis.error());
    }

    return derivedThis;
}

template <typename Derived, typename ErrorType>
template <typename Functor>
inline Derived&& OrElseWithValue<Derived, ErrorType>::or_else(const Functor& callable) && noexcept
{
    return std::move(this->or_else(callable));
}

template <typename Derived, typename ErrorType>
template <typename Functor>
inline const Derived& OrElseWithValue<Derived, ErrorType>::or_else(const Functor& callable) const& noexcept
{
    static_assert(is_invocable<Functor, ErrorType&>::value,
                  "Only callables with a signature of void(const ErrorType&) are allowed!");

    const auto& derivedThis = *static_cast<const Derived*>(this);

    if (!derivedThis)
    {
        auto callback = static_cast<const_or_else_callback_t>(callable);
        callback(derivedThis.error());
    }

    return derivedThis;
}

template <typename Derived, typename ErrorType>
template <typename Functor>
inline const Derived&& OrElseWithValue<Derived, ErrorType>::or_else(const Functor& callable) const&& noexcept
{
    return std::move(this->or_else(callable));
}

template <typename Derived>
inline Derived& OrElse<Derived>::or_else(const or_else_callback_t& callable) & noexcept
{
    auto& derivedThis = *static_cast<Derived*>(this);

    if (!derivedThis)
    {
        callable();
    }

    return derivedThis;
}

template <typename Derived>
inline Derived&& OrElse<Derived>::or_else(const or_else_callback_t& callable) && noexcept
{
    return std::move(this->or_else(callable));
}

template <typename Derived>
inline const Derived& OrElse<Derived>::or_else(const or_else_callback_t& callable) const& noexcept
{
    using Self = OrElse<Derived>;
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : const_cast avoids code duplication, is safe since the constness
    // of the return value is restored
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<const Derived&>(const_cast<Self*>(this)->or_else(callable));
}

template <typename Derived>
inline const Derived&& OrElse<Derived>::or_else(const or_else_callback_t& callable) const&& noexcept
{
    return std::move(this->or_else(callable));
}
// END or_else


} // namespace internal
} // namespace iox
#endif
