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

#ifndef IOX2_BB_STATIC_FUNCTION_HPP
#define IOX2_BB_STATIC_FUNCTION_HPP

#include "iox2/bb/detail/assertions.hpp"
#include "iox2/legacy/type_traits.hpp"
#include "iox2/legacy/uninitialized_array.hpp"

#include <utility>

namespace iox2 {
namespace bb {
namespace detail {
template <typename ReturnType, typename... Args>
using Signature = ReturnType(Args...);

template <uint64_t Capacity, typename T>
class StaticFunction;

/// @brief A static alternative of std::function which is fixed size.

/// @note This is not achievable with std::function and a custom allocator, as then the memory will still not
///       be part of the object and copying (and moving may cause subtle issues). Hence a complete implementation
///       is required.
///       Furthermore the allocator support of std::function in the STL is deprecated.

/// @tparam Capacity    The maximum capacity of the static function
/// @tparam ReturnType  The return type of the stored callable.
/// @tparam Args        The arguments of the stored callable.
template <uint64_t Capacity, typename ReturnType, typename... Args>
class StaticFunction<Capacity, Signature<ReturnType, Args...>> final {
  private:
    struct Operations;
    Operations m_operations; // operations depending on type-erased callable (copy, move, destroy)

    legacy::UninitializedArray<char, Capacity> m_storage;  // storage for the callable
    void* m_callable { nullptr };                          // pointer to stored type-erased callable
    ReturnType (*m_invoker)(void*, Args&&...) { nullptr }; // indirection to invoke the stored callable,
                                                           // nullptr if no callable is stored

  public:
    using SignatureT = Signature<ReturnType, Args...>;

    /// @brief construct from functor (including lambdas)
    template <typename Functor,
              typename = std::enable_if_t<std::is_class<Functor>::value
                                              && legacy::is_invocable_r<ReturnType, Functor, Args...>::value,
                                          void>>
    // AXIVION Next Construct AutosarC++19_03-A12.1.4: implicit conversion of functors is intentional,
    // the static function should implicitly behave like any generic constructor, adding
    // explicit would require a static_cast. Furthermore, the 'StaticFunction' stores a copy
    // which avoids implicit misbehaviors or ownership problems caused by implicit conversion.
    // NOLINTNEXTLINE(hicpp-explicit-conversions)
    StaticFunction(const Functor& functor) noexcept;

    /// @brief construct from function pointer (including static functions)
    // NOLINTJUSTIFICATION the static function should implicitly behave like any generic constructor, adding
    //                     explicit would require a static_cast. Furthermore, the 'StaticFunction' stores a copy
    //                     which avoids implicit misbehaviors or ownership problems caused by implicit conversion.
    // NOLINTNEXTLINE(hicpp-explicit-conversions)
    StaticFunction(ReturnType (*function)(Args...)) noexcept;

    /// @brief construct from object reference and member function
    /// only a pointer to the object is stored for the call
    template <typename T, typename = std::enable_if_t<std::is_class<T>::value, void>>
    StaticFunction(T& object, ReturnType (T::*method)(Args...)) noexcept;

    /// @brief construct from object reference and const member function
    /// only a pointer to the object is stored for the call
    template <typename T, typename = std::enable_if_t<std::is_class<T>::value, void>>
    StaticFunction(const T& object, ReturnType (T::*method)(Args...) const) noexcept;

    StaticFunction(const StaticFunction& other) noexcept;

    StaticFunction(StaticFunction&& other) noexcept;

    auto operator=(const StaticFunction& rhs) noexcept -> StaticFunction&;

    auto operator=(StaticFunction&& rhs) noexcept -> StaticFunction&;

    ~StaticFunction() noexcept;

    /// @brief invoke the stored function
    /// @param args arguments to invoke the stored function with
    /// @return return value of the stored function
    ///
    /// @note 1) If arguments are passed by value, the copy constructor may be invoked twice:
    ///          once when passing the arguments to operator() and once when they are passed to the stored callable
    ///          itself. This appears to be unavoidable and also happens in std::function.
    ///          The user can always provide a wrapped callable which takes a reference,
    ///          which is generally preferable for large objects anyway.
    ///
    ///       2) Arguments of class type cannot have the move constructor explicitly deleted since the arguments
    ///          must be forwarded internally which is done by move or, if no move is specified, by copy.
    ///          If the move operation is explicitly deleted the compiler will not fall back to copy but emit an error.
    ///          Not specifying move or using a default implementation is fine.
    ///          This is also the case for std::function (for the gcc implementation at least).
    ///
    auto operator()(Args... args) const noexcept -> ReturnType;

    /// @brief swap this with another static function
    /// @param f the function to swap this with
    void swap(StaticFunction& other) noexcept;

    /// @brief size in bytes required to store a CallableType in a 'StaticFunction'
    /// @return number of bytes
    /// @note this is not smallest possible due to alignment, it may work with a smaller size but
    ///       is not guaranteed (but it is guaranteed to work with the number of bytes returned)
    template <typename CallableType>
    static constexpr auto required_storage_size() noexcept -> uint64_t;

    /// @brief checks whether CallableType is storable
    /// @return true if CallableType can be stored, false if it is not guaranteed that it can be stored
    /// @note it might be storable for some alignments of CallableType even if it returns false,
    ///       in this case it is advised to increase the Capacity.
    template <typename CallableType>
    static constexpr auto is_storable() noexcept -> bool;

  private:
    // Required to perform the correct operations with the underlying erased type
    // This means 'StaticFunction' cannot be used where pointers become invalid, e.g. across process boundaries
    // Therefore we cannot store a 'StaticFunction' in shared memory (the same holds for std::function).
    // This is inherent to the type erasure technique we (have to) use.
    struct Operations final {
        // NOLINTBEGIN(misc-non-private-member-variables-in-classes): this is an internal helper class
        // function pointers defining copy, move and destroy semantics
        void (*copy_function)(const StaticFunction& src, StaticFunction& dest) { nullptr };
        void (*move_function)(StaticFunction& src, StaticFunction& dest) { nullptr };
        void (*destroy_function)(StaticFunction& func) { nullptr };
        // NOLINTEND(misc-non-private-member-variables-in-classes)

        Operations() noexcept = default;
        Operations(const Operations& other) noexcept = default;
        auto operator=(const Operations& other) noexcept -> Operations& = default;
        Operations(Operations&& other) noexcept = default;
        auto operator=(Operations&& other) noexcept -> Operations& = default;
        ~Operations() = default;

        void copy(const StaticFunction& src, StaticFunction& dest) const noexcept;

        void move(StaticFunction& src, StaticFunction& dest) const noexcept;

        void destroy(StaticFunction& func) const noexcept;
    };

    template <typename Functor,
              typename = std::enable_if_t<std::is_class<Functor>::value
                                              && legacy::is_invocable_r<ReturnType, Functor, Args...>::value,
                                          void>>
    void store_functor(const Functor& functor) noexcept;

    // we need these templates to preserve the actual CallableType for the underlying call
    template <typename CallableType>
    static void copy(const StaticFunction& src, StaticFunction& dest) noexcept;

    template <typename CallableType>
    static void move(StaticFunction& src, StaticFunction& dest) noexcept;

    template <typename CallableType>
    static void destroy(StaticFunction& func) noexcept;

    template <typename CallableType>
    static auto invoke(void* callable, Args&&... args) noexcept -> ReturnType;

    static void copy_free_function(const StaticFunction& src, StaticFunction& dest) noexcept;

    static void move_free_function(StaticFunction& src, StaticFunction& dest) noexcept;

    // AXIVION Next Construct AutosarC++19_03-M7.1.2: callable cannot be const void* since
    // m_invoker is initialized with this function and has to work with functors as well
    // (functors may change due to invocation)
    static auto invoke_free_function(void* callable, Args&&... args) noexcept -> ReturnType;

    template <typename T>
    static constexpr auto safe_align(void* start_address) noexcept -> void*;
};

/// @brief swap two static functions
/// @param f the first function to swap with g
/// @param g the second function to swap with f
template <uint64_t Capacity, typename T>
void swap(StaticFunction<Capacity, T>& left, StaticFunction<Capacity, T>& right) noexcept;

// AXIVION DISABLE STYLE AutosarC++19_03-A12.6.1: members are initialized before read access
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init)
template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename Functor, typename>
inline StaticFunction<Capacity, Signature<ReturnType, Args...>>::StaticFunction(const Functor& functor) noexcept {
    store_functor(functor);
}

// AXIVION Next Construct AutosarC++19_03-A12.1.5: constructor delegation is not feasible here due
// to lack of sufficient common initialization
// AXIVION Next Construct AutosarC++19_03-M5.2.6: the converted pointer is only used
// as its original function pointer type after reconversion (type erasure)
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init) members are default initialized
template <uint64_t Capacity, typename ReturnType, typename... Args>
inline StaticFunction<Capacity, Signature<ReturnType, Args...>>::StaticFunction(
    ReturnType (*function)(Args...)) noexcept
    : // AXIVION Next Construct AutosarC++19_03-A5.2.4: reinterpret_cast is required for type erasure,
    // we use type erasure in combination with compile time template arguments to restore
    // the correct type whenever the callable is used
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
    m_callable(reinterpret_cast<void*>(function))
    , m_invoker(&invoke_free_function) {
    IOX2_ENFORCE(function != nullptr, "parameter must not be a 'nullptr'");

    m_operations.copy_function = &copy_free_function;
    m_operations.move_function = &move_free_function;
    // destroy is not needed for free functions
}

// AXIVION DISABLE STYLE AutosarC++19_03-M0.3.1: Pointer p aliases a reference and method is a member function pointer that cannot be null (*)
// AXIVION DISABLE STYLE AutosarC++19_03-A5.3.2: see rule 'M0.3.1' above
// AXIVION DISABLE STYLE FaultDetection-NullPointerDereference: see rule 'M0.3.1' above

// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init) members are default initialized
template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename T, typename>
inline StaticFunction<Capacity, Signature<ReturnType, Args...>>::StaticFunction(
    T& object, ReturnType (T::*method)(Args...)) noexcept {
    T* const ptr { &object };
    const auto functor = [ptr, method](Args... args) noexcept -> ReturnType {
        return (*ptr.*method)(std::forward<Args>(args)...);
    };

    store_functor(functor);
}

// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init)
template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename T, typename>
inline StaticFunction<Capacity, Signature<ReturnType, Args...>>::StaticFunction(const T& object,
                                                                                ReturnType (T::*method)(Args...)
                                                                                    const) noexcept {
    const T* const ptr { &object };
    const auto functor = [ptr, method](Args... args) noexcept -> ReturnType {
        return (*ptr.*method)(std::forward<Args>(args)...);
    };

    store_functor(functor);
}

// AXIVION ENABLE STYLE FaultDetection-NullPointerDereference
// AXIVION ENABLE STYLE AutosarC++19_03-A5.3.2
// AXIVION ENABLE STYLE AutosarC++19_03-M0.3.1

// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init) m_storage is default initialized
template <uint64_t Capacity, typename ReturnType, typename... Args>
inline StaticFunction<Capacity, Signature<ReturnType, Args...>>::StaticFunction(const StaticFunction& other) noexcept
    : m_operations(other.m_operations)
    , m_invoker(other.m_invoker) {
    m_operations.copy(other, *this);
}

// AXIVION Next Construct AutosarC++19_03-A12.8.4: we copy only the operation pointer table
// (required) and will perform a move with its type erased move function
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init) m_storage is default initialized
template <uint64_t Capacity, typename ReturnType, typename... Args>
inline StaticFunction<Capacity, Signature<ReturnType, Args...>>::StaticFunction(StaticFunction&& other) noexcept
    : m_operations(other.m_operations)
    , m_invoker(other.m_invoker) {
    m_operations.move(other, *this);
}
// AXIVION ENABLE STYLE AutosarC++19_03-A12.6.1

template <uint64_t Capacity, typename ReturnType, typename... Args>
inline auto StaticFunction<Capacity, Signature<ReturnType, Args...>>::operator=(const StaticFunction& rhs) noexcept
    -> StaticFunction<Capacity, Signature<ReturnType, Args...>>& {
    if (&rhs != this) {
        // this operations is needed for destroy, then changed to source (rhs) operations
        m_operations.destroy(*this);
        m_operations = rhs.m_operations;
        m_invoker = rhs.m_invoker;
        m_operations.copy(rhs, *this);
    }

    return *this;
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
inline auto StaticFunction<Capacity, Signature<ReturnType, Args...>>::operator=(StaticFunction&& rhs) noexcept
    -> StaticFunction<Capacity, Signature<ReturnType, Args...>>& {
    if (&rhs != this) {
        // this operations is needed for destroy, then changed to source (rhs) operations
        m_operations.destroy(*this);
        m_operations = rhs.m_operations;
        m_invoker = rhs.m_invoker;
        m_operations.move(rhs, *this);
    }

    return *this;
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
inline StaticFunction<Capacity, Signature<ReturnType, Args...>>::~StaticFunction() noexcept {
    m_operations.destroy(*this);
}

// AXIVION Next Construct AutosarC++19_03-A7.5.2: false positive, operator() does not call itself
// but the invoked function can be recursive in general (entirely controllable by caller)
// AXIVION Next Construct AutosarC++19_03-A2.10.1: false positive, args does not hide anything
template <uint64_t Capacity, typename ReturnType, typename... Args>
inline auto StaticFunction<Capacity, Signature<ReturnType, Args...>>::operator()(Args... args) const noexcept
    -> ReturnType {
#if (defined(__GNUC__) && __GNUC__ >= 11 && __GNUC__ <= 12 && !defined(__clang__))
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wmaybe-uninitialized"
#endif
    IOX2_ENFORCE(m_callable != nullptr, "should not happen unless incorrectly used after move");
    // AXIVION Next Construct AutosarC++19_03-M0.3.1, FaultDetection-NullPointerDereference: m_invoker is initialized in ctor or assignment,
    // can only be nullptr if this was moved from (calling operator() is illegal in this case)
    return m_invoker(m_callable, std::forward<Args>(args)...);
#if (defined(__GNUC__) && __GNUC__ >= 11 && __GNUC__ <= 12 && !defined(__clang__))
#pragma GCC diagnostic pop
#endif
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
inline void StaticFunction<Capacity, Signature<ReturnType, Args...>>::swap(StaticFunction& other) noexcept {
    StaticFunction tmp { std::move(other) };
    other = std::move(*this);
    *this = std::move(tmp);
}

template <uint64_t Capacity, typename T>
inline void swap(StaticFunction<Capacity, T>& left, StaticFunction<Capacity, T>& right) noexcept {
    left.swap(right);
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename T>
constexpr auto StaticFunction<Capacity, Signature<ReturnType, Args...>>::safe_align(void* start_address) noexcept
    -> void* {
    static_assert(is_storable<T>(), "type does not fit into storage");
    // AXIVION DISABLE STYLE AutosarC++19_03-A5.2.4 : Cast required for low level pointer alignment
    // AXIVION DISABLE STYLE AutosarC++19_03-M5.2.9 : Conversion required for low level pointer alignment
    // NOLINTBEGIN(cppcoreguidelines-pro-type-reinterpret-cast, performance-no-int-to-ptr)
    const uint64_t alignment { alignof(T) };
    const auto align = [](const uint64_t value, const uint64_t alignment) -> auto {
        return (value + (alignment - 1U)) & (~alignment + 1U);
    };
    const uint64_t aligned_position { align(reinterpret_cast<uint64_t>(start_address), alignment) };
    return reinterpret_cast<void*>(aligned_position);
    // NOLINTEND(cppcoreguidelines-pro-type-reinterpret-cast, performance-no-int-to-ptr)
    // AXIVION ENABLE STYLE AutosarC++19_03-M5.2.9
    // AXIVION ENABLE STYLE AutosarC++19_03-A5.2.4
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename Functor, typename>
inline void StaticFunction<Capacity, Signature<ReturnType, Args...>>::store_functor(const Functor& functor) noexcept {
    using StoredType = std::remove_reference_t<Functor>;
    m_callable = safe_align<StoredType>(m_storage.begin());

    // erase the functor type and store as reference to the call in storage
    // AXIVION Next Construct AutosarC++19_03-A18.5.10: False positive! 'safeAlign' takes care of proper alignment and size
    new (m_callable) StoredType(functor);

    m_invoker = &invoke<StoredType>;
    m_operations.copy_function = &copy<StoredType>;
    m_operations.move_function = &move<StoredType>;
    m_operations.destroy_function = &destroy<StoredType>;
}

// AXIVION Next Construct AutosarC++19_03-A8.4.8: output parameter required by design and for efficiency
template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename CallableType>
inline void StaticFunction<Capacity, Signature<ReturnType, Args...>>::copy(const StaticFunction& src,
                                                                           StaticFunction& dest) noexcept {
    dest.m_callable = safe_align<CallableType>(dest.m_storage.begin());

    // AXIVION Next Construct AutosarC++19_03-M5.2.8: type erasure - conversion to compatible type
    const auto obj = static_cast<CallableType*>(src.m_callable);
    IOX2_ENFORCE(obj != nullptr, "should not happen unless src is incorrectly used after move");

    // AXIVION Next Construct AutosarC++19_03-A18.5.10: False positive! 'safeAlign' takes care of proper alignment and size
    // NOLINTNEXTLINE(clang-analyzer-core.NonNullParamChecker) checked two lines above
    new (dest.m_callable) CallableType(*obj);
    dest.m_invoker = src.m_invoker;
}

// AXIVION Next Construct AutosarC++19_03-A8.4.4, AutosarC++19_03-A8.4.8: output parameter required by design and for
// efficiency
template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename CallableType>
inline void StaticFunction<Capacity, Signature<ReturnType, Args...>>::move(StaticFunction& src,
                                                                           StaticFunction& dest) noexcept {
    dest.m_callable = safe_align<CallableType>(dest.m_storage.begin());

    // AXIVION Next Construct AutosarC++19_03-M5.2.8: type erasure - conversion to compatible type
    const auto obj = static_cast<CallableType*>(src.m_callable);
    IOX2_ENFORCE(obj != nullptr, "should not happen unless src is incorrectly used after move");

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
inline void StaticFunction<Capacity, Signature<ReturnType, Args...>>::destroy(StaticFunction& func) noexcept {
    if (func.m_callable != nullptr) {
        // AXIVION Next Construct AutosarC++19_03-M5.2.8: type erasure - conversion to compatible type
        const auto ptr = static_cast<CallableType*>(func.m_callable);
        // AXIVION Next Construct AutosarC++19_03-A5.3.2: ptr is guaranteed not to be nullptr
        ptr->~CallableType();
    }
}

// AXIVION Next Construct AutosarC++19_03-A8.4.8: Out parameter is required for the intended functionality of the internal helper function
template <uint64_t Capacity, typename ReturnType, typename... Args>
inline void
StaticFunction<Capacity, Signature<ReturnType, Args...>>::copy_free_function(const StaticFunction& src,
                                                                             StaticFunction& dest) noexcept {
    dest.m_invoker = src.m_invoker;
    dest.m_callable = src.m_callable;
}

// AXIVION Next Construct AutosarC++19_03-A8.4.4, AutosarC++19_03-A8.4.8: output parameter required by design and for
// efficiency
template <uint64_t Capacity, typename ReturnType, typename... Args>
inline void
StaticFunction<Capacity, Signature<ReturnType, Args...>>::move_free_function(StaticFunction& src,
                                                                             StaticFunction& dest) noexcept {
    dest.m_invoker = src.m_invoker;
    dest.m_callable = src.m_callable;
    src.m_invoker = nullptr;
    src.m_callable = nullptr;
}

// AXIVION Next Construct AutosarC++19_03-M7.1.2: callable cannot be const void* since
// m_invoker is initialized with this function and has to work with functors as well
template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename CallableType>
inline auto StaticFunction<Capacity, Signature<ReturnType, Args...>>::invoke(void* callable, Args&&... args) noexcept
    -> ReturnType {
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
inline auto StaticFunction<Capacity, Signature<ReturnType, Args...>>::invoke_free_function(void* callable,
                                                                                           Args&&... args) noexcept
    -> ReturnType {
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
constexpr auto StaticFunction<Capacity, Signature<ReturnType, Args...>>::required_storage_size() noexcept -> uint64_t {
    const uint64_t size { sizeof(T) };
    const uint64_t alignment { alignof(T) };
    return (size + alignment) - 1;
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename T>
constexpr auto StaticFunction<Capacity, Signature<ReturnType, Args...>>::is_storable() noexcept -> bool {
    return (required_storage_size<T>() <= Capacity) && legacy::is_invocable_r<ReturnType, T, Args...>::value;
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
inline void
StaticFunction<Capacity, Signature<ReturnType, Args...>>::Operations::copy(const StaticFunction& src,
                                                                           StaticFunction& dest) const noexcept {
    if (copy_function != nullptr) {
        copy_function(src, dest);
    }
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
inline void
StaticFunction<Capacity, Signature<ReturnType, Args...>>::Operations::move(StaticFunction& src,
                                                                           StaticFunction& dest) const noexcept {
    if (move_function != nullptr) {
        move_function(src, dest);
    }
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
inline void
StaticFunction<Capacity, Signature<ReturnType, Args...>>::Operations::destroy(StaticFunction& func) const noexcept {
    if (destroy_function != nullptr) {
        destroy_function(func);
    }
}

} // namespace detail
} // namespace bb
} // namespace iox2

#endif // IOX2_BB_STATIC_FUNCTION_HPP
