// Copyright (c) 2024 by ekxide IO GmbH. All rights reserved.
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

#ifndef IOX_PLATFORM_ATOMIC_HPP
#define IOX_PLATFORM_ATOMIC_HPP

#include <atomic>
#include <cstdlib>
#include <iostream>
#include <type_traits>

namespace iox
{
namespace concurrent
{
/// @brief An alias to the std::atomic_flag
using AtomicFlag = std::atomic_flag;

/// @brief A thin wrapper for a 'std::atomic' which ensures that all atomic operations are always lock-free in order to
/// ensure safe usage across processes in shared memory. For detailed documentation please have a look at
/// https://en.cppreference.com/w/cpp/atomic/atomic
template <typename T>
class Atomic
{
  private:
    template <typename U>
    using enable_if_integral_t = std::enable_if_t<std::is_same<T, U>::value && std::is_integral<U>::value, T>;

    template <typename U>
    using enable_if_pointer_t = std::enable_if_t<std::is_same<T, U>::value && std::is_pointer<U>::value, T>;

    template <typename U>
    using enable_if_integral_or_pointer_t =
        std::enable_if_t<std::is_same<T, U>::value && (std::is_integral<U>::value || std::is_pointer<U>::value), T>;

  public:
#if __cplusplus >= 201703L
    static_assert(std::atomic<T>::is_always_lock_free,
                  "The 'iox::Atomic' must work across process boundaries and must therefore be always lock-free!");

    /// @brief This is always 'true' for 'iox::Atomic'
    static constexpr bool is_always_lock_free = std::atomic<T>::is_always_lock_free;
#endif

    /// @brief Constructs a new 'iox::Atomic' with a default value
    constexpr Atomic() noexcept
        : m_value{T()}
    {
#if __cplusplus < 201703L
        if (!std::atomic_is_lock_free(&m_value))
        {
            std::cerr << "The 'iox::Atomic' must work across process boundaries and must therefore be always lock-free!"
                      << std::endl;
            std::abort();
        }
#endif
    }

    /// @brief Constructs a new 'iox::Atomic' with the given value
    explicit constexpr Atomic(T value) noexcept
        : m_value{value}
    {
    }

    /// @brief Similar to the std::atomic, the 'iox::Atomic' is not copy constructible
    Atomic(const Atomic& other) = delete;
    /// @brief Similar to the std::atomic, the 'iox::Atomic' is not copy assignable
    Atomic& operator=(const Atomic&) = delete;

    Atomic(Atomic&& rhs) noexcept = default;
    Atomic& operator=(Atomic&& rhs) noexcept = default;

    ~Atomic() = default;

    /// @brief Atomically assigns the given value to the 'iox::Atomic' and returns the given value. Equivalent to
    /// 'store(value)'
    // NOLINTNEXTLINE(cppcoreguidelines-c-copy-assignment-signature) This mimics the std::atomic copy assignment operator
    T operator=(T value) noexcept
    {
        m_value = value;
        return value;
    }

    /// @brief Atomically assigns the given value to the 'iox::Atomic' and returns the given value. Equivalent to
    /// 'store(value)'
    // NOLINTNEXTLINE(cppcoreguidelines-c-copy-assignment-signature) This mimics the std::atomic copy assignment operator
    T operator=(T value) volatile noexcept
    {
        m_value = value;
        return value;
    }

    /// @brief Atomically loads and returns the stored value. Equivalent to 'load()'
    explicit operator T() const noexcept
    {
        return m_value.operator T();
    }

    /// @brief Atomically loads and returns the stored value. Equivalent to 'load()'
    explicit operator T() const volatile noexcept
    {
        return m_value.operator T();
    }

    /// @brief Return true if all operations on an object of this type are lock-free
    bool is_lock_free() const noexcept
    {
        return m_value.is_lock_free();
    }

    /// @brief Return true if all operations on an object of this type are lock-free
    bool is_lock_free() const volatile noexcept
    {
        return m_value.is_lock_free();
    }

    /// @brief Atomically stores the given value with the given memory order
    void store(T value, std::memory_order order = std::memory_order_seq_cst) noexcept
    {
        m_value.store(value, order);
    }

    /// @brief Atomically stores the given value with the given memory order
    void store(T value, std::memory_order order = std::memory_order_seq_cst) volatile noexcept
    {
        m_value.store(value, order);
    }

    /// @brief Atomically loads and returns the stored value
    T load(std::memory_order order = std::memory_order_seq_cst) const noexcept
    {
        return m_value.load(order);
    }

    /// @brief Atomically loads and returns the stored value
    T load(std::memory_order order = std::memory_order_seq_cst) const volatile noexcept
    {
        return m_value.load(order);
    }

    /// @brief Atomically exchanges the given value with the stored value using the given memory order and returns the
    /// previous value
    T exchange(T value, std::memory_order order = std::memory_order_seq_cst) noexcept
    {
        return m_value.exchange(value, order);
    }

    /// @brief Atomically exchanges the given value with the stored value using the given memory order and returns the
    /// previous value
    T exchange(T value, std::memory_order order = std::memory_order_seq_cst) volatile noexcept
    {
        return m_value.exchange(value, order);
    }

    /// @brief Performs an atomic CAS operation on the stored value with the given desired value and the given memory
    /// orders for success and failure. If current value is not the expected value, the CAS operation fails and updates
    /// the expected value with the current value. Returns true on success and false on failure. The operation can fail
    /// spuriously even when the current value equals the expected value.
    bool compare_exchange_weak(T& expected, T desired, std::memory_order success, std::memory_order failure) noexcept
    {
        return m_value.compare_exchange_weak(expected, desired, success, failure);
    }

    /// @brief Performs an atomic CAS operation on the stored value with the given desired value and the given memory
    /// orders for success and failure. If current value is not the expected value, the CAS operation fails and updates
    /// the expected value with the current value. Returns true on success and false on failure. The operation can fail
    /// spuriously even when the current value equals the expected value.
    bool compare_exchange_weak(T& expected,
                               T desired,
                               std::memory_order success,
                               std::memory_order failure) volatile noexcept
    {
        return m_value.compare_exchange_weak(expected, desired, success, failure);
    }

    /// @brief Performs an atomic CAS operation on the stored value with the given desired value and the given memory
    /// order for both success and failure. If current value is not the expected value, the CAS operation fails and
    /// updates the expected value with the current value. Returns true on success and false on failure. The operation
    /// can fail spuriously even when the current value equals the expected value.
    bool compare_exchange_weak(T& expected, T desired, std::memory_order order = std::memory_order_seq_cst) noexcept
    {
        return m_value.compare_exchange_weak(expected, desired, order);
    }

    /// @brief Performs an atomic CAS operation on the stored value with the given desired value and the given memory
    /// order for both success and failure. If current value is not the expected value, the CAS operation fails and
    /// updates the expected value with the current value. Returns true on success and false on failure. The operation
    /// can fail spuriously even when the current value equals the expected value.
    bool
    compare_exchange_weak(T& expected, T desired, std::memory_order order = std::memory_order_seq_cst) volatile noexcept
    {
        return m_value.compare_exchange_weak(expected, desired, order);
    }

    /// @brief Performs an atomic CAS operation on the stored value with the given desired value and the given memory
    /// orders for success and failure. If current value is not the expected value, the CAS operation fails and updates
    /// the expected value with the current value. Returns true on success and false on failure.
    bool compare_exchange_strong(T& expected, T desired, std::memory_order success, std::memory_order failure) noexcept
    {
        return m_value.compare_exchange_strong(expected, desired, success, failure);
    }

    /// @brief Performs an atomic CAS operation on the stored value with the given desired value and the given memory
    /// orders for success and failure. If current value is not the expected value, the CAS operation fails and updates
    /// the expected value with the current value. Returns true on success and false on failure.
    bool compare_exchange_strong(T& expected,
                                 T desired,
                                 std::memory_order success,
                                 std::memory_order failure) volatile noexcept
    {
        return m_value.compare_exchange_strong(expected, desired, success, failure);
    }

    /// @brief Performs an atomic CAS operation on the stored value with the given desired value and the given memory
    /// order for both success and failure. If current value is not the expected value, the CAS operation fails and
    /// updates the expected value with the current value. Returns true on success and false on failure.
    bool compare_exchange_strong(T& expected, T desired, std::memory_order order = std::memory_order_seq_cst) noexcept
    {
        return m_value.compare_exchange_strong(expected, desired, order);
    }

    /// @brief Performs an atomic CAS operation on the stored value with the given desired value and the given memory
    /// order for both success and failure. If current value is not the expected value, the CAS operation fails and
    /// updates the expected value with the current value. Returns true on success and false on failure.
    bool compare_exchange_strong(T& expected,
                                 T desired,
                                 std::memory_order order = std::memory_order_seq_cst) volatile noexcept
    {
        return m_value.compare_exchange_strong(expected, desired, order);
    }

    /// @brief Atomically adds the given value to the stored value with the given memory order and returns the previous
    /// value
    template <typename U = T>
    enable_if_integral_t<U> fetch_add(T value, std::memory_order order = std::memory_order_seq_cst) noexcept
    {
        return m_value.fetch_add(value, order);
    }

    /// @brief Atomically adds the given value to the stored value with the given memory order and returns the previous
    /// value
    template <typename U = T>
    enable_if_integral_t<U> fetch_add(T value, std::memory_order order = std::memory_order_seq_cst) volatile noexcept
    {
        return m_value.fetch_add(value, order);
    }

    /// @brie Atomically adds the given difference to the stored pointer value with the given memory order and returns
    /// the previous pointer value
    template <typename U = T>
    enable_if_pointer_t<U> fetch_add(std::ptrdiff_t value, std::memory_order order = std::memory_order_seq_cst) noexcept
    {
        return m_value.fetch_add(value, order);
    }

    /// @brie Atomically adds the given difference to the stored pointer value with the given memory order and returns
    /// the previous pointer value
    template <typename U = T>
    enable_if_pointer_t<U> fetch_add(std::ptrdiff_t value,
                                     std::memory_order order = std::memory_order_seq_cst) volatile noexcept
    {
        return m_value.fetch_add(value, order);
    }

    /// @brief Atomically substracts the given value to the stored value with the given memory order and returns the
    /// previous value
    template <typename U = T>
    enable_if_integral_t<U> fetch_sub(T value, std::memory_order order = std::memory_order_seq_cst) noexcept
    {
        return m_value.fetch_sub(value, order);
    }

    /// @brief Atomically substracts the given value to the stored value with the given memory order and returns the
    /// previous value
    template <typename U = T>
    enable_if_integral_t<U> fetch_sub(T value, std::memory_order order = std::memory_order_seq_cst) volatile noexcept
    {
        return m_value.fetch_sub(value, order);
    }

    /// @brie Atomically substracts the given difference to the stored pointer value with the given memory order and
    /// returns the previous pointer value
    template <typename U = T>
    enable_if_pointer_t<U> fetch_sub(std::ptrdiff_t value, std::memory_order order = std::memory_order_seq_cst) noexcept
    {
        return m_value.fetch_sub(value, order);
    }

    /// @brie Atomically substracts the given difference to the stored pointer value with the given memory order and
    /// returns the previous pointer value
    template <typename U = T>
    enable_if_pointer_t<U> fetch_sub(std::ptrdiff_t value,
                                     std::memory_order order = std::memory_order_seq_cst) volatile noexcept
    {
        return m_value.fetch_sub(value, order);
    }

    /// @brief Atomically adds the given value to the stored value and returns the resulting value. Equivalent to
    /// 'fetch_add(value) + value'
    template <typename U = T>
    enable_if_integral_t<U> operator+=(T value) noexcept
    {
        return m_value.operator+=(value);
    }

    /// @brief Atomically adds the given value to the stored value and returns the resulting value. Equivalent to
    /// 'fetch_add(value) + value'
    template <typename U = T>
    enable_if_integral_t<U> operator+=(T value) volatile noexcept
    {
        return m_value.operator+=(value);
    }

    /// @brief Atomically substracts the given value to the stored value and returns the resulting value. Equivalent to
    /// 'fetch_sub(value) - value'
    template <typename U = T>
    enable_if_integral_t<U> operator-=(T value) noexcept
    {
        return m_value.operator-=(value);
    }

    /// @brief Atomically substracts the given value to the stored value and returns the resulting value. Equivalent to
    /// 'fetch_sub(value) - value'
    template <typename U = T>
    enable_if_integral_t<U> operator-=(T value) volatile noexcept
    {
        return m_value.operator-=(value);
    }

    /// @brief Atomically adds the given difference to the stored pointer value and returns the resulting new pointer
    /// value. Equivalent to 'fetch_add(value) + value'
    template <typename U = T>
    enable_if_pointer_t<U> operator+=(std::ptrdiff_t value) noexcept
    {
        return m_value.operator+=(value);
    }

    /// @brief Atomically adds the given difference to the stored pointer value and returns the resulting new pointer
    /// value. Equivalent to 'fetch_add(value) + value'
    template <typename U = T>
    enable_if_pointer_t<U> operator+=(std::ptrdiff_t value) volatile noexcept
    {
        return m_value.operator+=(value);
    }

    /// @brief Atomically substracts the given difference to the stored pointer value and returns the resulting new
    /// pointer value. Equivalent to 'fetch_sub(value) - value'
    template <typename U = T>
    enable_if_pointer_t<U> operator-=(std::ptrdiff_t value) noexcept
    {
        return m_value.operator-=(value);
    }

    /// @brief Atomically substracts the given difference to the stored pointer value and returns the resulting new
    /// pointer value. Equivalent to 'fetch_sub(value) - value'
    template <typename U = T>
    enable_if_pointer_t<U> operator-=(std::ptrdiff_t value) volatile noexcept
    {
        return m_value.operator-=(value);
    }

    /// @brief Atomic pre-increment operator, equivalent to 'return fetch_add(1) + 1'
    template <typename U = T>
    enable_if_integral_or_pointer_t<U> operator++() noexcept
    {
        return m_value.operator++();
    }

    /// @brief Atomic pre-increment operator, equivalent to 'return fetch_add(1) + 1'
    template <typename U = T>
    enable_if_integral_or_pointer_t<U> operator++() volatile noexcept
    {
        return m_value.operator++();
    }

    /// @brief Atomic post-increment operator, equivalent to 'return fetch_add(1)'
    template <typename U = T>
    enable_if_integral_or_pointer_t<U> operator++(int) noexcept
    {
        return m_value.operator++(0);
    }

    /// @brief Atomic post-increment operator, equivalent to 'return fetch_add(1)'
    template <typename U = T>
    enable_if_integral_or_pointer_t<U> operator++(int) volatile noexcept
    {
        return m_value.operator++(0);
    }

    /// @brief Atomic pre-decrement operator, equivalent to 'return fetch_sub(1) - 1'
    template <typename U = T>
    enable_if_integral_or_pointer_t<U> operator--() noexcept
    {
        return m_value.operator--();
    }

    /// @brief Atomic pre-decrement operator, equivalent to 'return fetch_sub(1) - 1'
    template <typename U = T>
    enable_if_integral_or_pointer_t<U> operator--() volatile noexcept
    {
        return m_value.operator--();
    }

    /// @brief Atomic post-decrement operator, equivalent to 'return fetch_sub(1)'
    template <typename U = T>
    enable_if_integral_or_pointer_t<U> operator--(int) noexcept
    {
        return m_value.operator--(0);
    }

    /// @brief Atomic post-decrement operator, equivalent to 'return fetch_sub(1)'
    template <typename U = T>
    enable_if_integral_or_pointer_t<U> operator--(int) volatile noexcept
    {
        return m_value.operator--(0);
    }

    /// @brief Atomically performs a bitwise 'AND' operation to the stored value with the given memory order and returns
    /// the previous value
    template <typename U = T>
    enable_if_integral_t<U> fetch_and(T value, std::memory_order order = std::memory_order_seq_cst) noexcept
    {
        return m_value.fetch_and(value, order);
    }

    /// @brief Atomically performs a bitwise 'AND' operation to the stored value with the given memory order and returns
    /// the previous value
    template <typename U = T>
    enable_if_integral_t<U> fetch_and(T value, std::memory_order order = std::memory_order_seq_cst) volatile noexcept
    {
        return m_value.fetch_and(value, order);
    }

    /// @brief Atomically performs a bitwise 'OR' operation to the stored value with the given memory order and returns
    /// the previous value
    template <typename U = T>
    enable_if_integral_t<U> fetch_or(T value, std::memory_order order = std::memory_order_seq_cst) noexcept
    {
        return m_value.fetch_or(value, order);
    }

    /// @brief Atomically performs a bitwise 'OR' operation to the stored value with the given memory order and returns
    /// the previous value
    template <typename U = T>
    enable_if_integral_t<U> fetch_or(T value, std::memory_order order = std::memory_order_seq_cst) volatile noexcept
    {
        return m_value.fetch_or(value, order);
    }

    /// @brief Atomically performs a bitwise 'XOR' operation to the stored value with the given memory order and returns
    /// the previous value
    template <typename U = T>
    enable_if_integral_t<U> fetch_xor(T value, std::memory_order order = std::memory_order_seq_cst) noexcept
    {
        return m_value.fetch_xor(value, order);
    }

    /// @brief Atomically performs a bitwise 'XOR' operation to the stored value with the given memory order and returns
    /// the previous value
    template <typename U = T>
    enable_if_integral_t<U> fetch_xor(T value, std::memory_order order = std::memory_order_seq_cst) volatile noexcept
    {
        return m_value.fetch_xor(value, order);
    }

    /// @brief Atomically performs a bitwise 'AND' operation to the stored value and returns the resulting value,
    /// Equivalent to 'return fetch_and(value) & value;'
    template <typename U = T>
    enable_if_integral_t<U> operator&=(T value) noexcept
    {
        return m_value.operator&=(value);
    }

    /// @brief Atomically performs a bitwise 'AND' operation to the stored value and returns the resulting value,
    /// Equivalent to 'return fetch_and(value) & value;'
    template <typename U = T>
    enable_if_integral_t<U> operator&=(T value) volatile noexcept
    {
        return m_value.operator&=(value);
    }

    /// @brief Atomically performs a bitwise 'OR' operation to the stored value and returns the resulting value,
    /// Equivalent to 'return fetch_or(value) & value;'
    template <typename U = T>
    enable_if_integral_t<U> operator|=(T value) noexcept
    {
        return m_value.operator|=(value);
    }

    /// @brief Atomically performs a bitwise 'OR' operation to the stored value and returns the resulting value,
    /// Equivalent to 'return fetch_or(value) & value;'
    template <typename U = T>
    enable_if_integral_t<U> operator|=(T value) volatile noexcept
    {
        return m_value.operator|=(value);
    }

    /// @brief Atomically performs a bitwise 'OR' operation to the stored value and returns the resulting value,
    /// Equivalent to 'return fetch_xor(value) & value;'
    template <typename U = T>
    enable_if_integral_t<U> operator^=(T value) noexcept
    {
        return m_value.operator^=(value);
    }

    /// @brief Atomically performs a bitwise 'OR' operation to the stored value and returns the resulting value,
    /// Equivalent to 'return fetch_xor(value) & value;'
    template <typename U = T>
    enable_if_integral_t<U> operator^=(T value) volatile noexcept
    {
        return m_value.operator^=(value);
    }

  private:
    std::atomic<T> m_value;
};
} // namespace concurrent
} // namespace iox

#endif // IOX_PLATFORM_ATOMIC_HPP
