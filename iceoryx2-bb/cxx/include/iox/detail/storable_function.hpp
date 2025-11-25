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

#ifndef IOX_HOOFS_FUNCTIONAL_STORABLE_FUNCTION_HPP
#define IOX_HOOFS_FUNCTIONAL_STORABLE_FUNCTION_HPP

#include "iox/iceoryx_hoofs_types.hpp"
#include "iox/type_traits.hpp"

#include <iostream>
#include <utility>


namespace iox
{
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
class storable_function<Capacity, signature<ReturnType, Args...>> final
{
  public:
    using signature_t = signature<ReturnType, Args...>;

    /// @brief construct from functor (including lambdas)
    template <typename Functor,
              typename = typename std::enable_if<std::is_class<Functor>::value
                                                     && is_invocable_r<ReturnType, Functor, Args...>::value,
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
    struct operations final
    {
        // function pointers defining copy, move and destroy semantics
        void (*copyFunction)(const storable_function& src, storable_function& dest){nullptr};
        void (*moveFunction)(storable_function& src, storable_function& dest){nullptr};
        void (*destroyFunction)(storable_function& f){nullptr};

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
    byte m_storage[Capacity];                           // storage for the callable
    void* m_callable{nullptr};                          // pointer to stored type-erased callable
    ReturnType (*m_invoker)(void*, Args&&...){nullptr}; // indirection to invoke the stored callable,
                                                        // nullptr if no callable is stored

    /// @note For static_storage as the StorageType we detect at compile time if the functor can be stored.
    ///       If this is not the case, compilation will fail.
    ///       For other StorageTypes where storing might fail due to insufficent memory this may not be detectable
    ///       at compile time and we call terminate at runtime if the functor could not be stored.

    template <typename Functor,
              typename = typename std::enable_if<std::is_class<Functor>::value
                                                     && is_invocable_r<ReturnType, Functor, Args...>::value,
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

} // namespace iox

#include "iox/detail/storable_function.inl"

#endif // IOX_HOOFS_FUNCTIONAL_STORABLE_FUNCTION_HPP
