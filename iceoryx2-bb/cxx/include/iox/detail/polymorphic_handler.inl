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

#ifndef IOX_HOOFS_DESIGN_POLYMORPHIC_HANDLER_INL
#define IOX_HOOFS_DESIGN_POLYMORPHIC_HANDLER_INL

#include "iox/polymorphic_handler.hpp"
#include "iox/static_lifetime_guard.hpp"

#include <cstdlib>
#include <type_traits>

namespace iox
{
namespace detail
{

template <typename Interface>
[[noreturn]] void DefaultHooks<Interface>::onSetAfterFinalize(Interface&, Interface&) noexcept
{
    // we should not use an error handling construct (e.g. some IOX_ASSERT) here for dependency reasons
    // we could in principle do nothing by default as well, but the misuse failure should have visible consequences
    std::abort();
}

} // namespace detail

// The get method is considered to be on the hot path and works as follows:
//
// On first call (in a thread):
// 1. localHandler is initialized
//    - getCurrent is called
//    - instantiates singleton instance()
//    - instantiates default handler
//    - sets m_current of instance to default instance (release)

// 2. If any thread changes the handler with set or reset, it will:
//    - be detected by comparison with local handler
//    - set the current handler to the new handler
//

// On any call after the handler was changed in another thread
// 1. We detect the change by comparison with the localHandler (can assume that addresses are unique by
// design as we use singletons in static memory)
// 2. If it was changed we perform a synchronizing load and obtain the current handler
// Note that it may change again but this is unavoidable if it can change concurrently
// without locks and not a problem.
template <typename Interface, typename Default, typename Hooks>
Interface& PolymorphicHandler<Interface, Default, Hooks>::get() noexcept
{
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables) false positive, thread_local
    thread_local Interface* localHandler = getCurrentSync(); // initialized once per thread on first call

    // required since it could be set in another thread concurrently
    auto* currentHandler = getCurrentRelaxed();

    // is the known localHandler outdated?
    // NB: this is a variation of double checked locking but with atomics:
    // we avoid the more costly operation on the hot path unless there was a change (rare case)
    if (localHandler != currentHandler)
    {
        // stronger sync of memory
        localHandler = getCurrentSync();
        // note that it may concurrently change again but we do not loop to check as there is no
        // point in such a lock-free code (it may also change while returning)
    }

    return *localHandler;
}

template <typename Interface, typename Default, typename Hooks>
template <typename Handler>
bool PolymorphicHandler<Interface, Default, Hooks>::set(StaticLifetimeGuard<Handler> handlerGuard) noexcept
{
    static_assert(std::is_base_of<Interface, Handler>::value,
                  "Handler must inherit from Interface or be of type Interface");
    static StaticLifetimeGuard<Handler> guard(handlerGuard);
    // we now have protected the handler instance and it will exist long enough
    return setHandler(StaticLifetimeGuard<Handler>::instance());
}

template <typename Interface, typename Default, typename Hooks>
bool PolymorphicHandler<Interface, Default, Hooks>::setHandler(Interface& handler) noexcept
{
    auto& s = self();
    // m_current is now guaranteed to be set

    // ensure we cannot miss that it was set to true concurrently
    // OK, since it will never change back from true to false
    bool exp{true};
    if (s.m_isFinal.compare_exchange_strong(exp, true, std::memory_order_relaxed))
    {
        // it must be ensured that the handlers still exist and are thread-safe,
        // this is ensured for the default handler by m_defaultGuard
        // (the primary guard that is constructed with the instance alone is not sufficient)
        Hooks::onSetAfterFinalize(*getCurrentSync(), handler);
        return false;
    }

    // Note that if finalization takes effect here, setHandler will still change the handler
    // This is still correct concurrent behavior in the sense that it maps
    // to a sequential execution where the handler is set before finalization.

    s.m_current.exchange(&handler, std::memory_order_acq_rel);
    return true;
}

template <typename Interface, typename Default, typename Hooks>
bool PolymorphicHandler<Interface, Default, Hooks>::reset() noexcept
{
    return setHandler(getDefault());
}

template <typename Interface, typename Default, typename Hooks>
void PolymorphicHandler<Interface, Default, Hooks>::finalize() noexcept
{
    self().m_isFinal.store(true, std::memory_order_relaxed);
}

template <typename Interface, typename Default, typename Hooks>
PolymorphicHandler<Interface, Default, Hooks>::PolymorphicHandler() noexcept
{
    m_current.store(&getDefault(), std::memory_order_release);
}

template <typename Interface, typename Default, typename Hooks>
PolymorphicHandler<Interface, Default, Hooks>& PolymorphicHandler<Interface, Default, Hooks>::self() noexcept
{
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables) singleton pattern
    static auto& ins = StaticLifetimeGuard<Self>::instance();
    return ins;
}

template <typename Interface, typename Default, typename Hooks>
Default& PolymorphicHandler<Interface, Default, Hooks>::getDefault() noexcept
{
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables) singleton pattern
    static auto& ins = StaticLifetimeGuard<Default>::instance();
    return ins;
}

template <typename Interface, typename Default, typename Hooks>
Interface* PolymorphicHandler<Interface, Default, Hooks>::getCurrentRelaxed() noexcept
{
    // only load the pointer atomically
    return self().m_current.load(std::memory_order_relaxed);
}

template <typename Interface, typename Default, typename Hooks>
Interface* PolymorphicHandler<Interface, Default, Hooks>::getCurrentSync() noexcept
{
    // must be strong enough to sync memory of the object pointed to
    return self().m_current.load(std::memory_order_acquire);
}

template <typename Interface, typename Default, typename Hooks>
StaticLifetimeGuard<typename PolymorphicHandler<Interface, Default, Hooks>::Self>
PolymorphicHandler<Interface, Default, Hooks>::guard() noexcept
{
    return StaticLifetimeGuard<Self>();
}

} // namespace iox

#endif // IOX_HOOFS_DESIGN_POLYMORPHIC_HANDLER_INL
