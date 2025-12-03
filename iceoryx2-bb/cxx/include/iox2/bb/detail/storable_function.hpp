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

#ifndef IOX2_BB_STORABLE_FUNCTION_HPP
#define IOX2_BB_STORABLE_FUNCTION_HPP

#include "iox2/legacy/assertions.hpp"
#include "iox2/legacy/memory.hpp"
#include "iox2/legacy/type_traits.hpp"

#include <iostream>
#include <utility>


namespace iox2 {
namespace bb {
namespace detail {
template <typename ReturnType, typename... Args>
using signature = ReturnType(Args...);

template <uint64_t Capacity, typename T>
class storable_function;

/// @brief A storable alternative of std::function which is fixed size.

/// @note This is not achievable with std::function and a custom allocator, as then the memory will still not
///       be part of the object and copying (and moving may cause subtle issues). Hence a complete implementation
///       is required.
///       Furthermore the allocator support of std::function in the STL is deprecated.

/// @tparam Capacity    The maximum capacity of the storable function
/// @tparam ReturnType  The return type of the stored callable.
/// @tparam Args        The arguments of the stored callable.
template <uint64_t Capacity, typename ReturnType, typename... Args>
class storable_function<Capacity, signature<ReturnType, Args...>> final {
  public:
    using signature_t = signature<ReturnType, Args...>;

    /// @brief construct from functor (including lambdas)
    template <typename Functor,
              typename = typename std::enable_if<std::is_class<Functor>::value
                                                     && legacy::is_invocable_r<ReturnType, Functor, Args...>::value,
                                                 void>::type>
    // AXIVION Next Construct AutosarC++19_03-A12.1.4: implicit conversion of functors is intentional,
    // the storable function should implicitly behave like any generic constructor, adding
    // explicit would require a static_cast. Furthermore, the storable_functor stores a copy
    // which avoids implicit misbehaviors or ownership problems caused by implicit conversion.
    // NOLINTNEXTLINE(hicpp-explicit-conversions)
    storable_function(const Functor& functor) noexcept;

    /// @brief construct from function pointer (including static functions)
    // NOLINTJUSTIFICATION the storable function should implicitly behave like any generic constructor, adding
    //                     explicit would require a static_cast. Furthermore, the storable_functor stores a copy
    //                     which avoids implicit misbehaviors or ownership problems caused by implicit conversion.
    // NOLINTNEXTLINE(hicpp-explicit-conversions)
    storable_function(ReturnType (*function)(Args...)) noexcept;

    /// @brief construct from object reference and member function
    /// only a pointer to the object is stored for the call
    template <typename T, typename = typename std::enable_if<std::is_class<T>::value, void>::type>
    storable_function(T& object, ReturnType (T::*method)(Args...)) noexcept;

    /// @brief construct from object reference and const member function
    /// only a pointer to the object is stored for the call
    template <typename T, typename = typename std::enable_if<std::is_class<T>::value, void>::type>
    storable_function(const T& object, ReturnType (T::*method)(Args...) const) noexcept;

    storable_function(const storable_function& other) noexcept;

    storable_function(storable_function&& other) noexcept;

    storable_function& operator=(const storable_function& rhs) noexcept;

    storable_function& operator=(storable_function&& rhs) noexcept;

    ~storable_function() noexcept;

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
    ReturnType operator()(Args... args) const noexcept;

    /// @brief swap this with another storable function
    /// @param f the function to swap this with
    void swap(storable_function& f) noexcept;

    /// @brief size in bytes required to store a CallableType in a storable_function
    /// @return number of bytes
    /// @note this is not smallest possible due to alignment, it may work with a smaller size but
    ///       is not guaranteed (but it is guaranteed to work with the number of bytes returned)
    template <typename CallableType>
    static constexpr uint64_t required_storage_size() noexcept;

    /// @brief checks whether CallableType is storable
    /// @return true if CallableType can be stored, false if it is not guaranteed that it can be stored
    /// @note it might be storable for some alignments of CallableType even if it returns false,
    ///       in this case it is advised to increase the Capacity.
    template <typename CallableType>
    static constexpr bool is_storable() noexcept;

  private:
    // Required to perform the correct operations with the underlying erased type
    // This means storable_function cannot be used where pointers become invalid, e.g. across process boundaries
    // Therefore we cannot store a storable_function in shared memory (the same holds for std::function).
    // This is inherent to the type erasure technique we (have to) use.
    struct operations final {
        // function pointers defining copy, move and destroy semantics
        void (*copyFunction)(const storable_function& src, storable_function& dest) { nullptr };
        void (*moveFunction)(storable_function& src, storable_function& dest) { nullptr };
        void (*destroyFunction)(storable_function& f) { nullptr };

        operations() noexcept = default;
        operations(const operations& other) noexcept = default;
        operations& operator=(const operations& other) noexcept = default;
        operations(operations&& other) noexcept = default;
        operations& operator=(operations&& other) noexcept = default;
        ~operations() = default;

        void copy(const storable_function& src, storable_function& dest) const noexcept;

        void move(storable_function& src, storable_function& dest) const noexcept;

        void destroy(storable_function& f) const noexcept;
    };

  private:
    operations m_operations; // operations depending on type-erased callable (copy, move, destroy)

    // AXIVION Next Construct AutosarC++19_03-A18.1.1 : safe access is guaranteed since the c-array is wrapped inside the storable_function
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays, hicpp-avoid-c-arrays)
    legacy::byte m_storage[Capacity];                      // storage for the callable
    void* m_callable { nullptr };                          // pointer to stored type-erased callable
    ReturnType (*m_invoker)(void*, Args&&...) { nullptr }; // indirection to invoke the stored callable,
                                                           // nullptr if no callable is stored

    template <typename Functor,
              typename = typename std::enable_if<std::is_class<Functor>::value
                                                     && legacy::is_invocable_r<ReturnType, Functor, Args...>::value,
                                                 void>::type>
    void storeFunctor(const Functor& functor) noexcept;

    // we need these templates to preserve the actual CallableType for the underlying call
    template <typename CallableType>
    static void copy(const storable_function& src, storable_function& dest) noexcept;

    template <typename CallableType>
    static void move(storable_function& src, storable_function& dest) noexcept;

    template <typename CallableType>
    static void destroy(storable_function& f) noexcept;

    template <typename CallableType>
    static ReturnType invoke(void* callable, Args&&... args) noexcept;

    static void copyFreeFunction(const storable_function& src, storable_function& dest) noexcept;

    static void moveFreeFunction(storable_function& src, storable_function& dest) noexcept;

    // AXIVION Next Construct AutosarC++19_03-M7.1.2: callable cannot be const void* since
    // m_invoker is initialized with this function and has to work with functors as well
    // (functors may change due to invocation)
    static ReturnType invokeFreeFunction(void* callable, Args&&... args) noexcept;

    template <typename T>
    static constexpr void* safeAlign(void* startAddress) noexcept;
};

/// @brief swap two storable functions
/// @param f the first function to swap with g
/// @param g the second function to swap with f
template <uint64_t Capacity, typename T>
void swap(storable_function<Capacity, T>& f, storable_function<Capacity, T>& g) noexcept;

// AXIVION DISABLE STYLE AutosarC++19_03-A12.6.1: members are initialized before read access
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init)
template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename Functor, typename>
inline storable_function<Capacity, signature<ReturnType, Args...>>::storable_function(const Functor& functor) noexcept {
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
    , m_invoker(&invokeFreeFunction) {
    IOX2_ENFORCE(function != nullptr, "parameter must not be a 'nullptr'");

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
    T& object, ReturnType (T::*method)(Args...)) noexcept {
    T* const p { &object };
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
                                                                                          const) noexcept {
    const T* const p { &object };
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
    , m_invoker(other.m_invoker) {
    m_operations.copy(other, *this);
}

// AXIVION Next Construct AutosarC++19_03-A12.8.4: we copy only the operation pointer table
// (required) and will perform a move with its type erased move function
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init) m_storage is default initialized
template <uint64_t Capacity, typename ReturnType, typename... Args>
inline storable_function<Capacity, signature<ReturnType, Args...>>::storable_function(
    storable_function&& other) noexcept
    : m_operations(other.m_operations)
    , m_invoker(other.m_invoker) {
    m_operations.move(other, *this);
}
// AXIVION ENABLE STYLE AutosarC++19_03-A12.6.1

template <uint64_t Capacity, typename ReturnType, typename... Args>
inline storable_function<Capacity, signature<ReturnType, Args...>>&
storable_function<Capacity, signature<ReturnType, Args...>>::operator=(const storable_function& rhs) noexcept {
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
inline storable_function<Capacity, signature<ReturnType, Args...>>&
storable_function<Capacity, signature<ReturnType, Args...>>::operator=(storable_function&& rhs) noexcept {
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
inline storable_function<Capacity, signature<ReturnType, Args...>>::~storable_function() noexcept {
    m_operations.destroy(*this);
}

// AXIVION Next Construct AutosarC++19_03-A7.5.2: false positive, operator() does not call itself
// but the invoked function can be recursive in general (entirely controllable by caller)
// AXIVION Next Construct AutosarC++19_03-A2.10.1: false positive, args does not hide anything
template <uint64_t Capacity, typename ReturnType, typename... Args>
inline ReturnType storable_function<Capacity, signature<ReturnType, Args...>>::operator()(Args... args) const noexcept {
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
inline void storable_function<Capacity, signature<ReturnType, Args...>>::swap(storable_function& f) noexcept {
    storable_function tmp { std::move(f) };
    f = std::move(*this);
    *this = std::move(tmp);
}

template <uint64_t Capacity, typename T>
inline void swap(storable_function<Capacity, T>& f, storable_function<Capacity, T>& g) noexcept {
    f.swap(g);
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename T>
inline constexpr void*
storable_function<Capacity, signature<ReturnType, Args...>>::safeAlign(void* startAddress) noexcept {
    static_assert(is_storable<T>(), "type does not fit into storage");
    // AXIVION DISABLE STYLE AutosarC++19_03-A5.2.4 : Cast required for low level pointer alignment
    // AXIVION DISABLE STYLE AutosarC++19_03-M5.2.9 : Conversion required for low level pointer alignment
    // NOLINTBEGIN(cppcoreguidelines-pro-type-reinterpret-cast, performance-no-int-to-ptr)
    const uint64_t alignment { alignof(T) };
    const uint64_t alignedPosition { legacy::align(reinterpret_cast<uint64_t>(startAddress), alignment) };
    return reinterpret_cast<void*>(alignedPosition);
    // NOLINTEND(cppcoreguidelines-pro-type-reinterpret-cast, performance-no-int-to-ptr)
    // AXIVION ENABLE STYLE AutosarC++19_03-M5.2.9
    // AXIVION ENABLE STYLE AutosarC++19_03-A5.2.4
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename Functor, typename>
inline void storable_function<Capacity, signature<ReturnType, Args...>>::storeFunctor(const Functor& functor) noexcept {
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
                                                                              storable_function& dest) noexcept {
    dest.m_callable = safeAlign<CallableType>(&dest.m_storage[0]);

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
inline void storable_function<Capacity, signature<ReturnType, Args...>>::move(storable_function& src,
                                                                              storable_function& dest) noexcept {
    dest.m_callable = safeAlign<CallableType>(&dest.m_storage[0]);

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
inline void storable_function<Capacity, signature<ReturnType, Args...>>::destroy(storable_function& f) noexcept {
    if (f.m_callable != nullptr) {
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
                                                                              storable_function& dest) noexcept {
    dest.m_invoker = src.m_invoker;
    dest.m_callable = src.m_callable;
}

// AXIVION Next Construct AutosarC++19_03-A8.4.4, AutosarC++19_03-A8.4.8: output parameter required by design and for
// efficiency
template <uint64_t Capacity, typename ReturnType, typename... Args>
inline void
storable_function<Capacity, signature<ReturnType, Args...>>::moveFreeFunction(storable_function& src,
                                                                              storable_function& dest) noexcept {
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
                                                                                      Args&&... args) noexcept {
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
storable_function<Capacity, signature<ReturnType, Args...>>::invokeFreeFunction(void* callable,
                                                                                Args&&... args) noexcept {
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
inline constexpr uint64_t
storable_function<Capacity, signature<ReturnType, Args...>>::required_storage_size() noexcept {
    const uint64_t size { sizeof(T) };
    const uint64_t alignment { alignof(T) };
    return (size + alignment) - 1;
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
template <typename T>
inline constexpr bool storable_function<Capacity, signature<ReturnType, Args...>>::is_storable() noexcept {
    return (required_storage_size<T>() <= Capacity) && legacy::is_invocable_r<ReturnType, T, Args...>::value;
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
inline void
storable_function<Capacity, signature<ReturnType, Args...>>::operations::copy(const storable_function& src,
                                                                              storable_function& dest) const noexcept {
    if (copyFunction != nullptr) {
        copyFunction(src, dest);
    }
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
inline void
storable_function<Capacity, signature<ReturnType, Args...>>::operations::move(storable_function& src,
                                                                              storable_function& dest) const noexcept {
    if (moveFunction != nullptr) {
        moveFunction(src, dest);
    }
}

template <uint64_t Capacity, typename ReturnType, typename... Args>
inline void
storable_function<Capacity, signature<ReturnType, Args...>>::operations::destroy(storable_function& f) const noexcept {
    if (destroyFunction != nullptr) {
        destroyFunction(f);
    }
}

} // namespace detail
} // namespace bb
} // namespace iox2

#endif // IOX2_BB_STORABLE_FUNCTION_HPP
