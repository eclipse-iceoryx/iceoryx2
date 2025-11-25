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

#include "iox/atomic.hpp"
#include "iox/polymorphic_handler.hpp"
#include "iox/static_lifetime_guard.hpp"

#include "test.hpp"

#include <chrono>
#include <iostream>
#include <thread>

namespace
{
using namespace ::testing;

struct Interface
{
    virtual ~Interface() = default;

    virtual uint32_t id() = 0;

    void reset()
    {
        value = 0;
    }

    uint32_t value{0};
};

constexpr uint32_t DEFAULT_ID = 73;
constexpr uint32_t ALTERNATE_ID = 21;

struct Default : public Interface
{
    uint32_t id() override
    {
        return DEFAULT_ID;
    }
};

struct Alternate : public Interface
{
    uint32_t id() override
    {
        return ALTERNATE_ID;
    }
};

template <typename T>
using Guard = iox::StaticLifetimeGuard<T>;

// should the handler instances be accessed, they will live at least as
// long as the guard objects
Guard<Default> defaultGuard;
Guard<Alternate> alternateGuard;

// will live at least as long as the corresponding guards
Default& defaultHandler = Guard<Default>::instance();
Alternate& alternateHandler = Guard<Alternate>::instance();

struct Hooks
{
    // to check whether the arguments are used correctly
    static void onSetAfterFinalize(Interface& currentInstance, Interface& newInstance)
    {
        currentInstance.value = currentInstance.id();
        newInstance.value = newInstance.id();
    }
};

using Handler = iox::PolymorphicHandler<Interface, Default, Hooks>;

class PolymorphicHandler_test : public Test
{
  public:
    void SetUp() override
    {
    }

    void TearDown() override
    {
        Handler::reset();
    }
};

TEST_F(PolymorphicHandler_test, handlerIsInitializedWithDefault)
{
    ::testing::Test::RecordProperty("TEST_ID", "41bb4a5e-a916-4a6d-80c4-fed3a3d8d78b");
    EXPECT_EQ(Handler::get().id(), DEFAULT_ID);
}

TEST_F(PolymorphicHandler_test, settingAlternateWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "8b2f0cfe-f13c-4fa0-aa93-5ddd4f0904d1");
    EXPECT_EQ(Handler::get().id(), DEFAULT_ID);

    bool ret = Handler::set(alternateGuard);
    auto& handler = Handler::get();

    EXPECT_EQ(handler.id(), ALTERNATE_ID);
    EXPECT_TRUE(ret);
}

TEST_F(PolymorphicHandler_test, alternatePointsToExternalMemory)
{
    ::testing::Test::RecordProperty("TEST_ID", "85ce0e51-a1fe-490c-9012-7d539512ed38");
    EXPECT_EQ(Handler::get().id(), DEFAULT_ID);
    Handler::set(alternateGuard);

    auto& handler = Handler::get();
    auto* ptr = static_cast<Interface*>(&alternateHandler);

    EXPECT_EQ(&handler, ptr);
}

TEST_F(PolymorphicHandler_test, explicitlySettingToDefaultWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "32e4d808-c848-4bf9-b878-e163ca825539");
    EXPECT_EQ(Handler::get().id(), DEFAULT_ID);
    Handler::set(alternateGuard);

    bool ret = Handler::set(defaultGuard);

    auto& handler = Handler::get();
    auto* ptr = static_cast<Interface*>(&defaultHandler);

    EXPECT_EQ(&handler, ptr);
    EXPECT_TRUE(ret);
}


TEST_F(PolymorphicHandler_test, resetToDefaultWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "ef8a99da-22a6-497e-b2ec-bf72cc3ae943");
    Handler::set(alternateGuard);
    auto& prevHandler = Handler::get();
    EXPECT_EQ(prevHandler.id(), ALTERNATE_ID);

    // note that we have to use reset to set it back to the internal default
    bool ret = Handler::reset();

    EXPECT_TRUE(ret);
    auto& handler = Handler::get();
    EXPECT_EQ(handler.id(), DEFAULT_ID);
}

TEST_F(PolymorphicHandler_test, setToCurrentHandlerWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "54e22290-a7b4-4552-a18f-953571381d38");

    // change to alternateHandler
    Handler::set(alternateGuard);

    // set to alternateHandler again
    // while this is a useless operation, we cannot forbid it via interface
    bool ret = Handler::set(alternateGuard);
    auto& handler = Handler::get();

    EXPECT_TRUE(ret);
    EXPECT_EQ(&handler, &alternateHandler);
}

TEST_F(PolymorphicHandler_test, defaultHandlerIsVisibleInAllThreads)
{
    ::testing::Test::RecordProperty("TEST_ID", "caa1e507-7cc1-4233-8c9c-5c4e56be9fb3");

    Handler::set(defaultGuard);

    iox::concurrent::Atomic<uint32_t> count{0};

    auto checkHandler = [&]() {
        auto& h = Handler::get();

        if (h.id() == DEFAULT_ID)
        {
            ++count;
        }
    };

    constexpr uint32_t NUM_THREADS{2}; // including main thread

    std::thread t(checkHandler);
    t.join();

    checkHandler();

    EXPECT_EQ(count.load(), NUM_THREADS);
}

TEST_F(PolymorphicHandler_test, handlerChangePropagatesBetweenThreads)
{
    ::testing::Test::RecordProperty("TEST_ID", "f0a8e941-e064-4889-a6db-425b35a3b7b0");

    Handler::set(defaultGuard);
    EXPECT_EQ(Handler::get().id(), DEFAULT_ID);

    std::thread t([]() { Handler::set(alternateGuard); });

    t.join();

    // the handler should now be visible in the main thread
    EXPECT_EQ(Handler::get().id(), ALTERNATE_ID);
}

TEST_F(PolymorphicHandler_test, settingAfterFinalizeCallsHook)
{
    ::testing::Test::RecordProperty("TEST_ID", "171ac802-01b9-4e08-80a6-6f2defecaf6d");

    // we always finalize it to be alternateHandler
    Handler::set(alternateGuard);

    // reset the handler value to zero and check later whether they are set to non-zero as expected
    defaultHandler.reset();
    alternateHandler.reset();

    // NB: we know that the current handler is alternateHandler before finalize
    Handler::finalize();

    // verify the handler values are 0 before calling set
    // (the hook should change this)
    EXPECT_EQ(defaultHandler.value, 0);
    EXPECT_EQ(alternateHandler.value, 0);

    bool ret = Handler::set(defaultGuard);
    EXPECT_FALSE(ret);

    // does the hook set the values to the corresponding arguments?
    EXPECT_EQ(defaultHandler.value, DEFAULT_ID);
    EXPECT_EQ(alternateHandler.value, ALTERNATE_ID);

    // handler should be unchanged
    auto& handler = Handler::get();
    EXPECT_EQ(&handler, &alternateHandler);
}

TEST_F(PolymorphicHandler_test, resetAfterFinalizeCallsHook)
{
    ::testing::Test::RecordProperty("TEST_ID", "996220e3-7985-4d57-bd3f-844987cf99dc");

    // we always finalize it to be alternateHandler (in the other test or here)
    Handler::set(alternateGuard);

    defaultHandler.reset();
    alternateHandler.reset();

    // NB: we know that the current handler is alternateHandler before finalize
    // it does not matter whether it was called before
    Handler::finalize();

    // verify the handler values are 0 before calling reset
    // (the hook should change this)
    EXPECT_EQ(defaultHandler.value, 0);
    EXPECT_EQ(alternateHandler.value, 0);

    bool ret = Handler::reset();

    EXPECT_FALSE(ret);
    // does the hook set the values to the corresponding arguments?
    EXPECT_EQ(defaultHandler.value, DEFAULT_ID);
    EXPECT_EQ(alternateHandler.value, ALTERNATE_ID);

    // handler should be unchanged
    auto& handler = Handler::get();
    EXPECT_EQ(&handler, &alternateHandler);
}

TEST_F(PolymorphicHandler_test, obtainingGuardWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "694f7399-598a-4918-b1e8-4b8546484245");
    EXPECT_EQ(Guard<Handler>::count(), 1);
    Guard<Handler> guard(Handler::guard());
    EXPECT_EQ(Guard<Handler>::count(), 2);
}

} // namespace
