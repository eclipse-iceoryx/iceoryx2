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

#ifndef IOX_HOOFS_FUNCTIONAL_FUNCTION_REF_INL
#define IOX_HOOFS_FUNCTIONAL_FUNCTION_REF_INL

#include "iox/assertions.hpp"
#include "iox/function_ref.hpp"

#include <memory>

namespace iox
{
template <class ReturnType, class... ArgTypes>
template <typename CallableType, typename>
// AXIVION Next Construct AutosarC++19_03-A12.1.5 : Other c'tors can't be used as delegating c'tor
// AXIVION Next Construct AutosarC++19_03-A8.4.6 : Only ArgTypes needs to be forwarded
// NOLINTNEXTLINE(cppcoreguidelines-missing-std-forward)
inline function_ref<ReturnType(ArgTypes...)>::function_ref(CallableType&& callable) noexcept
    // AXIVION Next Construct AutosarC++19_03-A5.2.4, AutosarC++19_03-A5.2.3, CertC++-EXP55 : Type-safety ensured by
    // casting back on call
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast, cppcoreguidelines-pro-type-const-cast)
    : m_pointerToCallable(const_cast<void*>(reinterpret_cast<const void*>(std::addressof(callable))))
    // AXIVION Next Construct AutosarC++19_03-A15.4.4, FaultDetection-NoexceptViolations : Lambda not 'noexcept' as callable might throw
    // AXIVION Next Line AutosarC++19_03-M7.1.2 : Can't be const because the pointer to the callable needs to be mutable
    , m_functionPointer([](void* const target, ArgTypes... args) -> ReturnType {
        // AXIVION Next Construct AutosarC++19_03-A5.2.4, CertC++-EXP36 : The class design ensures a cast to the actual
        // type of target
        // AXIVION Next Construct AutosarC++19_03-A5.3.2, AutosarC++19_03-M5.2.8 : Check for 'nullptr' is
        // performed on call
        // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
        return (*reinterpret_cast<typename std::add_pointer<CallableType>::type>(target))(
            std::forward<ArgTypes>(args)...);
    })
{
}

template <class ReturnType, class... ArgTypes>
inline function_ref<ReturnType(ArgTypes...)>::function_ref(ReturnType (&function)(ArgTypes...)) noexcept
    // the cast is required to work on POSIX systems
    // AXIVION Next Construct AutosarC++19_03-A5.2.4, AutosarC++19_03-M5.2.6 : Type-safety ensured by casting
    // back function pointer on call using the type provided as template parameter and encapsulated in this class
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
    : m_pointerToCallable(reinterpret_cast<void*>(function))
    ,
    // the lambda does not capture and is thus convertible to a function pointer
    // (required by the C++ standard)
    m_functionPointer([](void* const target, ArgTypes... args) -> ReturnType {
        using PointerType = ReturnType (*)(ArgTypes...);
        // AXIVION Next Construct AutosarC++19_03-A5.2.4 : The class design ensures a cast to the actual type of target
        // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast,hicpp-use-auto)
        PointerType f{reinterpret_cast<PointerType>(target)};
        // AXIVION Next Line AutosarC++19_03-A5.3.2 : Check for 'nullptr' is performed on call
        return f(args...);
    })
{
}

template <class ReturnType, class... ArgTypes>
inline function_ref<ReturnType(ArgTypes...)>::function_ref(function_ref&& rhs) noexcept
{
    *this = std::move(rhs);
}

template <class ReturnType, class... ArgTypes>
inline function_ref<ReturnType(ArgTypes...)>&
function_ref<ReturnType(ArgTypes...)>::operator=(function_ref<ReturnType(ArgTypes...)>&& rhs) & noexcept
{
    if (this != &rhs)
    {
        m_pointerToCallable = rhs.m_pointerToCallable;
        m_functionPointer = rhs.m_functionPointer;
        // Make sure no undefined behavior can happen by marking the rhs as invalid
        rhs.m_pointerToCallable = nullptr;
        rhs.m_functionPointer = nullptr;
    }
    return *this;
}

template <class ReturnType, class... ArgTypes>
// AXIVION Next Construct AutosarC++19_03-A15.4.2, AutosarC++19_03-A15.5.3, FaultDetection-NoexceptViolations : Intentional behavior. The library itself does not throw and on the implementation side a try-catch block can be used
inline ReturnType function_ref<ReturnType(ArgTypes...)>::operator()(ArgTypes... args) const noexcept
{
    auto wasCallableAssignedBeforehand = (m_pointerToCallable != nullptr) && (m_functionPointer != nullptr);
    if (!wasCallableAssignedBeforehand)
    {
        IOX_PANIC("Empty function_ref invoked");
    }
    return m_functionPointer(m_pointerToCallable, std::forward<ArgTypes>(args)...);
}

template <class ReturnType, class... ArgTypes>
inline void function_ref<ReturnType(ArgTypes...)>::swap(function_ref<ReturnType(ArgTypes...)>& rhs) noexcept
{
    std::swap(m_pointerToCallable, rhs.m_pointerToCallable);
    std::swap(m_functionPointer, rhs.m_functionPointer);
}

template <class ReturnType, class... ArgTypes>
// AXIVION Next Line AutosarC++19_03-A2.10.5 : False positive, overload for swap(function_ref, function_ref) as in STL
inline void swap(function_ref<ReturnType(ArgTypes...)>& lhs, function_ref<ReturnType(ArgTypes...)>& rhs) noexcept
{
    lhs.swap(rhs);
}

} // namespace iox

#endif // IOX_HOOFS_FUNCTIONAL_FUNCTION_REF_INL
