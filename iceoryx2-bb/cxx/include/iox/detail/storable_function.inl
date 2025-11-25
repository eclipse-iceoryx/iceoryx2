// Copyright (c) 2020 - 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX_HOOFS_FUNCTIONAL_STORABLE_FUNCTION_INL
#define IOX_HOOFS_FUNCTIONAL_STORABLE_FUNCTION_INL

#include "iox/assertions.hpp"
#include "iox/detail/storable_function.hpp"
#include "iox/memory.hpp"

namespace iox
{
// AXIVION DISABLE STYLE AutosarC++19_03-A12.6.1: members are initialized before read access
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init)
template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename Functor, typename>
inline storable_function<Capacity, signature<ReturnType, Args...>>::storable_function(const Functor& functor) noexcept
{
    storeFunctor(functor);
}

// AXIVION Next Construct AutosarC++19_03-A12.1.5: constructor delegation is not feasible here due
// to lack of sufficient common initialization
// AXIVION Next Construct AutosarC++19_03-M5.2.6: the converted pointer is only used
// as its original function pointer type after reconversion (type erasure)
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init) members are default initialized
template <uint64_t Capacity, typename ReturnType, typename... Args>
inline storable_function<Capacity, signature<ReturnType, Args...>>::storable_function(
    ReturnType (*function)(Args...)) noexcept
    : // AXIVION Next Construct AutosarC++19_03-A5.2.4: reinterpret_cast is required for type erasure,
      // we use type erasure in combination with compile time template arguments to restore
      // the correct type whenever the callable is used
      // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
    m_callable(reinterpret_cast<void*>(function))
    , m_invoker(&invokeFreeFunction)
{
    IOX_ENFORCE(function != nullptr, "parameter must not be a 'nullptr'");

    m_operations.copyFunction = &copyFreeFunction;
    m_operations.moveFunction = &moveFreeFunction;
    // destroy is not needed for free functions
}

// AXIVION DISABLE STYLE AutosarC++19_03-M0.3.1: Pointer p aliases a reference and method is a member function pointer that cannot be null (*)
// AXIVION DISABLE STYLE AutosarC++19_03-A5.3.2: see rule 'M0.3.1' above
// AXIVION DISABLE STYLE FaultDetection-NullPointerDereference: see rule 'M0.3.1' above

// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init) members are default initialized
template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename T, typename>
inline storable_function<Capacity, signature<ReturnType, Args...>>::storable_function(
    T& object, ReturnType (T::*method)(Args...)) noexcept
{
    T* const p{&object};
    const auto functor = [p, method](Args... args) noexcept -> ReturnType {
        return (*p.*method)(std::forward<Args>(args)...);
    };

    storeFunctor(functor);
}

// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init)
template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename T, typename>
inline storable_function<Capacity, signature<ReturnType, Args...>>::storable_function(const T& object,
                                                                                      ReturnType (T::*method)(Args...)
                                                                                          const) noexcept
{
    const T* const p{&object};
    const auto functor = [p, method](Args... args) noexcept -> ReturnType {
        return (*p.*method)(std::forward<Args>(args)...);
    };

    storeFunctor(functor);
}

// AXIVION ENABLE STYLE FaultDetection-NullPointerDereference
// AXIVION ENABLE STYLE AutosarC++19_03-A5.3.2
// AXIVION ENABLE STYLE AutosarC++19_03-M0.3.1

// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init) m_storage is default initialized
template <uint64_t Capacity, typename ReturnType, typename... Args>
inline storable_function<Capacity, signature<ReturnType, Args...>>::storable_function(
    const storable_function& other) noexcept
    : m_operations(other.m_operations)
    , m_invoker(other.m_invoker)
{
    m_operations.copy(other, *this);
}

// AXIVION Next Construct AutosarC++19_03-A12.8.4: we copy only the operation pointer table
// (required) and will perform a move with its type erased move function
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init) m_storage is default initialized
template <uint64_t Capacity, typename ReturnType, typename... Args>
inline storable_function<Capacity, signature<ReturnType, Args...>>::storable_function(
    storable_function&& other) noexcept
    : m_operations(other.m_operations)
    , m_invoker(other.m_invoker)
{
    m_operations.move(other, *this);
}
// AXIVION ENABLE STYLE AutosarC++19_03-A12.6.1

template <uint64_t Capacity, typename ReturnType, typename... Args>
inline storable_function<Capacity, signature<ReturnType, Args...>>&
storable_function<Capacity, signature<ReturnType, Args...>>::operator=(const storable_function& rhs) noexcept
{
    if (&rhs != this)
    {
        // this operations is needed for destroy, then changed to source (rhs) operations
        m_operations.destroy(*this);
        m_operations = rhs.m_operations;
        m_invoker = rhs.m_invoker;
        m_operations.copy(rhs, *this);
    }

    return *this;
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
inline storable_function<Capacity, signature<ReturnType, Args...>>&
storable_function<Capacity, signature<ReturnType, Args...>>::operator=(storable_function&& rhs) noexcept
{
    if (&rhs != this)
    {
        // this operations is needed for destroy, then changed to source (rhs) operations
        m_operations.destroy(*this);
        m_operations = rhs.m_operations;
        m_invoker = rhs.m_invoker;
        m_operations.move(rhs, *this);
    }

    return *this;
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
inline storable_function<Capacity, signature<ReturnType, Args...>>::~storable_function() noexcept
{
    m_operations.destroy(*this);
}

// AXIVION Next Construct AutosarC++19_03-A7.5.2: false positive, operator() does not call itself
// but the invoked function can be recursive in general (entirely controllable by caller)
// AXIVION Next Construct AutosarC++19_03-A2.10.1: false positive, args does not hide anything
template <uint64_t Capacity, typename ReturnType, typename... Args>
inline ReturnType storable_function<Capacity, signature<ReturnType, Args...>>::operator()(Args... args) const noexcept
{
#if (defined(__GNUC__) && __GNUC__ >= 11 && __GNUC__ <= 12 && !defined(__clang__))
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wmaybe-uninitialized"
#endif
    IOX_ENFORCE(m_callable != nullptr, "should not happen unless incorrectly used after move");
    // AXIVION Next Construct AutosarC++19_03-M0.3.1, FaultDetection-NullPointerDereference: m_invoker is initialized in ctor or assignment,
    // can only be nullptr if this was moved from (calling operator() is illegal in this case)
    return m_invoker(m_callable, std::forward<Args>(args)...);
#if (defined(__GNUC__) && __GNUC__ >= 11 && __GNUC__ <= 12 && !defined(__clang__))
#pragma GCC diagnostic pop
#endif
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
inline void storable_function<Capacity, signature<ReturnType, Args...>>::swap(storable_function& f) noexcept
{
    storable_function tmp{std::move(f)};
    f = std::move(*this);
    *this = std::move(tmp);
}

template <uint64_t Capacity, typename T>
inline void swap(storable_function<Capacity, T>& f, storable_function<Capacity, T>& g) noexcept
{
    f.swap(g);
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename T>
inline constexpr void*
storable_function<Capacity, signature<ReturnType, Args...>>::safeAlign(void* startAddress) noexcept
{
    static_assert(is_storable<T>(), "type does not fit into storage");
    // AXIVION DISABLE STYLE AutosarC++19_03-A5.2.4 : Cast required for low level pointer alignment
    // AXIVION DISABLE STYLE AutosarC++19_03-M5.2.9 : Conversion required for low level pointer alignment
    // NOLINTBEGIN(cppcoreguidelines-pro-type-reinterpret-cast, performance-no-int-to-ptr)
    const uint64_t alignment{alignof(T)};
    const uint64_t alignedPosition{align(reinterpret_cast<uint64_t>(startAddress), alignment)};
    return reinterpret_cast<void*>(alignedPosition);
    // NOLINTEND(cppcoreguidelines-pro-type-reinterpret-cast, performance-no-int-to-ptr)
    // AXIVION ENABLE STYLE AutosarC++19_03-M5.2.9
    // AXIVION ENABLE STYLE AutosarC++19_03-A5.2.4
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename Functor, typename>
inline void storable_function<Capacity, signature<ReturnType, Args...>>::storeFunctor(const Functor& functor) noexcept
{
    using StoredType = typename std::remove_reference<Functor>::type;
    m_callable = safeAlign<StoredType>(&m_storage[0]);

    // erase the functor type and store as reference to the call in storage
    // AXIVION Next Construct AutosarC++19_03-A18.5.10: False positive! 'safeAlign' takes care of proper alignment and size
    new (m_callable) StoredType(functor);

    m_invoker = &invoke<StoredType>;
    m_operations.copyFunction = &copy<StoredType>;
    m_operations.moveFunction = &move<StoredType>;
    m_operations.destroyFunction = &destroy<StoredType>;
}

// AXIVION Next Construct AutosarC++19_03-A8.4.8: output parameter required by design and for efficiency
template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename CallableType>
inline void storable_function<Capacity, signature<ReturnType, Args...>>::copy(const storable_function& src,
                                                                              storable_function& dest) noexcept
{
    dest.m_callable = safeAlign<CallableType>(&dest.m_storage[0]);

    // AXIVION Next Construct AutosarC++19_03-M5.2.8: type erasure - conversion to compatible type
    const auto obj = static_cast<CallableType*>(src.m_callable);
    IOX_ENFORCE(obj != nullptr, "should not happen unless src is incorrectly used after move");

    // AXIVION Next Construct AutosarC++19_03-A18.5.10: False positive! 'safeAlign' takes care of proper alignment and size
    // NOLINTNEXTLINE(clang-analyzer-core.NonNullParamChecker) checked two lines above
    new (dest.m_callable) CallableType(*obj);
    dest.m_invoker = src.m_invoker;
}

// AXIVION Next Construct AutosarC++19_03-A8.4.4, AutosarC++19_03-A8.4.8: output parameter required by design and for
// efficiency
template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename CallableType>
inline void storable_function<Capacity, signature<ReturnType, Args...>>::move(storable_function& src,
                                                                              storable_function& dest) noexcept
{
    dest.m_callable = safeAlign<CallableType>(&dest.m_storage[0]);

    // AXIVION Next Construct AutosarC++19_03-M5.2.8: type erasure - conversion to compatible type
    const auto obj = static_cast<CallableType*>(src.m_callable);
    IOX_ENFORCE(obj != nullptr, "should not happen unless src is incorrectly used after move");

    // AXIVION Next Construct AutosarC++19_03-A18.5.10: False positive! 'safeAlign' takes care of proper alignment and size
    // NOLINTNEXTLINE(clang-analyzer-core.NonNullParamChecker) checked two lines above
    new (dest.m_callable) CallableType(std::move(*obj));
    dest.m_invoker = src.m_invoker;
    src.m_operations.destroy(src);
    src.m_callable = nullptr;
    src.m_invoker = nullptr;
}

// AXIVION Next Construct AutosarC++19_03-M0.1.8: False positive! The function calls the destructor of a member of the parameter
template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename CallableType>
inline void storable_function<Capacity, signature<ReturnType, Args...>>::destroy(storable_function& f) noexcept
{
    if (f.m_callable != nullptr)
    {
        // AXIVION Next Construct AutosarC++19_03-M5.2.8: type erasure - conversion to compatible type
        const auto ptr = static_cast<CallableType*>(f.m_callable);
        // AXIVION Next Construct AutosarC++19_03-A5.3.2: ptr is guaranteed not to be nullptr
        ptr->~CallableType();
    }
}

// AXIVION Next Construct AutosarC++19_03-A8.4.8: Out parameter is required for the intended functionality of the internal helper function
template <uint64_t Capacity, typename ReturnType, typename... Args>
inline void
storable_function<Capacity, signature<ReturnType, Args...>>::copyFreeFunction(const storable_function& src,
                                                                              storable_function& dest) noexcept
{
    dest.m_invoker = src.m_invoker;
    dest.m_callable = src.m_callable;
}

// AXIVION Next Construct AutosarC++19_03-A8.4.4, AutosarC++19_03-A8.4.8: output parameter required by design and for
// efficiency
template <uint64_t Capacity, typename ReturnType, typename... Args>
inline void
storable_function<Capacity, signature<ReturnType, Args...>>::moveFreeFunction(storable_function& src,
                                                                              storable_function& dest) noexcept
{
    dest.m_invoker = src.m_invoker;
    dest.m_callable = src.m_callable;
    src.m_invoker = nullptr;
    src.m_callable = nullptr;
}

// AXIVION Next Construct AutosarC++19_03-M7.1.2: callable cannot be const void* since
// m_invoker is initialized with this function and has to work with functors as well
template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename CallableType>
inline ReturnType storable_function<Capacity, signature<ReturnType, Args...>>::invoke(void* callable,
                                                                                      Args&&... args) noexcept
{
    // AXIVION DISABLE STYLE AutosarC++19_03-A18.9.2: we use idiomatic perfect forwarding
    // AXIVION Next Construct AutosarC++19_03-M5.2.8: type erasure - conversion to compatible type
    // AXIVION Next Construct AutosarC++19_03-A5.3.2: callable is guaranteed not to be nullptr
    // when invoke is called (it is private and only used for type erasure)
    // NOLINTNEXTLINE(clang-analyzer-core.CallAndMessage) see justification above
    return (*static_cast<CallableType*>(callable))(std::forward<Args>(args)...);
    // AXIVION ENABLE STYLE AutosarC++19_03-A18.9.2
}

// AXIVION Next Construct AutosarC++19_03-A2.10.1: false positive, args does not hide anything
// AXIVION Next Construct AutosarC++19_03-M7.1.2: callable cannot be const void* since
// m_invoker is initialized with this function and has to work with functors as well
// (functors may change due to invocation)
template <uint64_t Capacity, typename ReturnType, typename... Args>
inline ReturnType
storable_function<Capacity, signature<ReturnType, Args...>>::invokeFreeFunction(void* callable, Args&&... args) noexcept
{
    // AXIVION Next Construct AutosarC++19_03-A18.9.2: we use idiomatic perfect forwarding
    // AXIVION Next Construct AutosarC++19_03-A5.3.2: callable is guaranteed not to be nullptr
    // when invokeFreeFunction is called (it is private and only used for type erasure)
    // AXIVION Next Construct AutosarC++19_03-M5.2.8: type erasure - conversion to compatible type
    // AXIVION Next Construct AutosarC++19_03-A5.2.4: reinterpret_cast is required for type erasure
    // type erasure in combination with compile time template arguments to restore the correct type
    // when the callable is called
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
    return (reinterpret_cast<ReturnType (*)(Args...)>(callable))(std::forward<Args>(args)...);
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename T>
inline constexpr uint64_t storable_function<Capacity, signature<ReturnType, Args...>>::required_storage_size() noexcept
{
    const uint64_t size{sizeof(T)};
    const uint64_t alignment{alignof(T)};
    return (size + alignment) - 1;
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename T>
inline constexpr bool storable_function<Capacity, signature<ReturnType, Args...>>::is_storable() noexcept
{
    return (required_storage_size<T>() <= Capacity) && is_invocable_r<ReturnType, T, Args...>::value;
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
inline void
storable_function<Capacity, signature<ReturnType, Args...>>::operations::copy(const storable_function& src,
                                                                              storable_function& dest) const noexcept
{
    if (copyFunction != nullptr)
    {
        copyFunction(src, dest);
    }
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
inline void
storable_function<Capacity, signature<ReturnType, Args...>>::operations::move(storable_function& src,
                                                                              storable_function& dest) const noexcept
{
    if (moveFunction != nullptr)
    {
        moveFunction(src, dest);
    }
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
inline void
storable_function<Capacity, signature<ReturnType, Args...>>::operations::destroy(storable_function& f) const noexcept
{
    if (destroyFunction != nullptr)
    {
        destroyFunction(f);
    }
}

} // namespace iox

#endif // IOX_HOOFS_FUNCTIONAL_STORABLE_FUNCTION_INL
