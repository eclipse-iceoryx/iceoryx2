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

#ifndef IOX_HOOFS_DESIGN_STATIC_LIFETIME_GUARD_INL
#define IOX_HOOFS_DESIGN_STATIC_LIFETIME_GUARD_INL

#include "iox/atomic.hpp"
#include "iox/static_lifetime_guard.hpp"

#include <thread>

namespace iox
{

// NOLINTJUSTIFICATION these static variables are private and mutability is required
// NOLINTBEGIN (cppcoreguidelines-avoid-non-const-global-variables)
template <typename T>
typename StaticLifetimeGuard<T>::storage_t StaticLifetimeGuard<T>::s_storage;
template <typename T>
concurrent::Atomic<uint64_t> StaticLifetimeGuard<T>::s_count{0};
template <typename T>
concurrent::Atomic<uint32_t> StaticLifetimeGuard<T>::s_instanceState{UNINITIALIZED};
template <typename T>
T* StaticLifetimeGuard<T>::s_instance{nullptr};
// NOLINTEND (cppcoreguidelines-avoid-non-const-global-variables)

template <typename T>
StaticLifetimeGuard<T>::StaticLifetimeGuard() noexcept
{
    s_count.fetch_add(1, std::memory_order_relaxed);
}

template <typename T>
StaticLifetimeGuard<T>::StaticLifetimeGuard(const StaticLifetimeGuard&) noexcept
{
    s_count.fetch_add(1, std::memory_order_relaxed);
}

template <typename T>
StaticLifetimeGuard<T>::StaticLifetimeGuard(StaticLifetimeGuard&&) noexcept
{
    // we have to increment the counter here as well as it is only
    // decremented in the dtor (which was not yet called for the moved object)
    s_count.fetch_add(1, std::memory_order_relaxed);
}

template <typename T>
StaticLifetimeGuard<T>::~StaticLifetimeGuard() noexcept
{
    if (s_count.fetch_sub(1, std::memory_order_relaxed) == 1)
    {
        destroy();
    }
}

template <typename T>
template <typename... Args>
T& StaticLifetimeGuard<T>::instance(Args&&... args) noexcept
{
    static StaticLifetimeGuard<T> primaryGuard;

    // we determine wether this call has to initialize the instance
    // via CAS (without mutex!)
    // NB: this shows how CAS acts as consensus primitive to determine the initializing call
    uint32_t exp{UNINITIALIZED};
    if (s_instanceState.compare_exchange_strong(exp, INITALIZING, std::memory_order_acq_rel, std::memory_order_acquire))
    {
        // NOLINTJUSTIFICATION s_instance is managed by reference counting
        // NOLINTNEXTLINE(cppcoreguidelines-owning-memory)
        s_instance = new (&s_storage) T(std::forward<Args>(args)...);

        // synchronize s_instance
        s_instanceState.store(INITALIZED, std::memory_order_release);
        return *s_instance;
    }
    // design constraint: no mutex - makes it more intricate
    // (otherwise we could simply use double checked locking)

    // either this call initialized the instance and we already returned
    // or a concurrent call is doing so and we have to wait until it completes ...

    while (s_instanceState.load(std::memory_order_acquire) != INITALIZED)
    {
        // wait, guaranteed to complete with fair scheduling
        std::this_thread::yield();
    }
    // guaranteed to be non-null after initialization
    return *s_instance;
}

template <typename T>
uint64_t StaticLifetimeGuard<T>::setCount(uint64_t count)
{
    return s_count.exchange(count, std::memory_order_relaxed);
}

template <typename T>
uint64_t StaticLifetimeGuard<T>::count()
{
    return s_count.load(std::memory_order_relaxed);
}

template <typename T>
void StaticLifetimeGuard<T>::destroy()
{
    // instance either exists, i.e. instance() was called and returned
    // or is being called for the first time and has already set the instance
    // 1) was called is no problem, because this means the primary guard must have
    //    gone out of scope at program end as otherwise destroy would not be called
    // 2) is called for first time and has not yet returned
    // - the state is INITIALIZED or INITIALIZING
    // - s_count is > 0 due to the primary guard
    // - s_count went up after it went to zero due to the guard that triggered this destroy
    //
    // NB: instance can be called for the first time in a destructor
    // of a static object that is destroyed after main;
    // unusual but OK if guards are used correctly to control the destruction order of all statics
    if (s_instance)
    {
        // there is an instance, so there MUST be a primary guard (that may have
        // triggered this destroy)
        //
        // check the counter again, if it is zero the primary guard and all others that existed
        // are already destroyed or being destroyed (one of them triggered this destroy)
        uint64_t exp{0};
        // destroy is a rare operation and the memory order is intentional to ensure
        // memory synchronization of s_instance and limit reordering.
        if (s_count.compare_exchange_strong(exp, 0, std::memory_order_acq_rel))
        {
            // s_count is 0, so we know the primary guard was destroyed before this OR
            // has triggered this destroy
            // this will only happen at program end when the primary guard goes out of scope
            s_instance->~T();
            s_instance = nullptr;
            // NB: reinitialization is normally only possible at program end (static destruction)
            s_instanceState = UNINITIALIZED;
        }
    }
}

} // namespace iox

#endif // IOX_HOOFS_DESIGN_STATIC_LIFETIME_GUARD_INL
