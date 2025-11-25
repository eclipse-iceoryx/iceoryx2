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

#ifndef IOX_HOOFS_DESIGN_STATIC_LIFETIME_GUARD_HPP
#define IOX_HOOFS_DESIGN_STATIC_LIFETIME_GUARD_HPP

#include "iox/atomic.hpp"

#include <type_traits>
#include <utility>

namespace iox
{
/// @brief Manages a static instance of type T in a way so that each
/// existing StaticLifetimeGuard prevents the destruction of
/// the instance.
/// 1. instance() creates a static guard itself and hence has static lifetime
/// 2. Any static StaticLifetimeGuard G created before that prolongs the lifetime
/// of the instance at least until G is destroyed
/// 3. The instance is lazily constructed, i.e. only when first used.
/// Hence there can be guards without any instance existing.
/// These guards still protect the instance from destruction if it is ever constructed.
/// 4. If and once the instance is constructed, it will be destructed only after main exits (static
/// destruction).
/// 5. Guards can be used in static variables to control destruction order of static (singleton)
/// instances if a specific order of destruction is required.
/// @tparam T the type of the instance to be guarded
///
/// @note all public functions are thread-safe
///
/// @code
/// // instance will be destroyed after guard
/// // (or later if there are guards preceding guard in construction order)
///
/// static StaticLifetimeGuard<T> guard;
/// static T& instance = StaticLifetimeGuard<T>::instance();
/// @endcode
template <typename T>
class StaticLifetimeGuard
{
  public:
    StaticLifetimeGuard() noexcept;

    StaticLifetimeGuard(const StaticLifetimeGuard&) noexcept;

    StaticLifetimeGuard(StaticLifetimeGuard&&) noexcept;

    ~StaticLifetimeGuard() noexcept;

    // assignment does nothing since the objects have no state,
    // copy and move exist to support passing/returning a value object
    StaticLifetimeGuard& operator=(const StaticLifetimeGuard&) noexcept = default;
    StaticLifetimeGuard& operator=(StaticLifetimeGuard&&) noexcept = default;

    /// @brief Construct the instance to be guarded with constructor arguments.
    /// @param args constructor arguments
    /// @return reference to the constructed instance or the existing instance
    /// if it already exists
    /// @note creates an implicit static StaticLifetimeGuard to ensure
    /// the instance is destroyed in the static destruction phase
    template <typename... Args>
    static T& instance(Args&&... args) noexcept;

    /// @brief Get the current count value.
    /// @return current count value
    static uint64_t count();

  private:
    struct alignas(T) storage_t
    {
        // AXIVION Next Construct AutosarC++19_03-M0.1.3 : the field is intentionally unused and serves as a mean to provide memory
        // AXIVION Next Construct AutosarC++19_03-A1.1.1 : object size depends on template parameter and has to be taken care of at the specific template instantiation
        // AXIVION Next Construct AutosarC++19_03-A18.1.1 : required as low level building block, encapsulated in abstraction and not directly used
        // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
        unsigned char data[sizeof(T)];
    };

    static constexpr uint32_t UNINITIALIZED{0};
    static constexpr uint32_t INITALIZING{1};
    static constexpr uint32_t INITALIZED{2};

    // NOLINTJUSTIFICATION these static variables are private and mutability is required
    // NOLINTBEGIN (cppcoreguidelines-avoid-non-const-global-variables)
    static storage_t s_storage;
    static concurrent::Atomic<uint64_t> s_count;
    static concurrent::Atomic<uint32_t> s_instanceState;
    static T* s_instance;
    // NOLINTEND (cppcoreguidelines-avoid-non-const-global-variables)

    static void destroy();

  protected:
    /// @brief Set the instance life time count.
    /// @param count value to be set
    /// @return previous count value
    /// @note This can be used to additionally extend or shorten the instance lifetime,
    /// This has to be done carefully, to ensure destruction or prevent early destruction.
    /// It is useful for testing purposes.
    /// @note Only to be used for testing.
    /// @attention setCount is only supposed to be used while no guards are created
    /// concurrently as it will influence the guard mechanism.
    /// The instance is destroyed once the counter reaches zero upon destruction of a guard,
    /// i.e. changing the counter may lead to the instance never being destroyed
    /// or being destroyed before the last guard.
    static uint64_t setCount(uint64_t count);
};

} // namespace iox

#include "iox/detail/static_lifetime_guard.inl"

#endif // IOX_HOOFS_DESIGN_STATIC_LIFETIME_GUARD_HPP
