// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
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

#include "iox/memory.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

#include <cstdint>

namespace
{
using namespace testing;

namespace
{
struct Bar
{
    // required for testing, a struct with a defined size
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays,cppcoreguidelines-avoid-c-arrays)
    alignas(8) uint8_t m_dummy[73];
};
struct Foo
{
    // required for testing, a struct with a defined size
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays,cppcoreguidelines-avoid-c-arrays)
    uint8_t m_dummy[73];
};
struct FooBar
{
    // required for testing, a struct with a defined size
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays,cppcoreguidelines-avoid-c-arrays)
    alignas(32) uint8_t m_dummy[73];
};
struct FuBar
{
    // required for testing, a struct with a defined size
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays,cppcoreguidelines-avoid-c-arrays)
    alignas(32) uint8_t m_dummy[73];
};
} // namespace

TEST(memory_test, MaxSizeWorksAsExpected)
{
    ::testing::Test::RecordProperty("TEST_ID", "5b3e938d-aec5-478d-b1c1-49ff2cc4e3ef");
    EXPECT_THAT(iox::maxSize<Foo>(), Eq(sizeof(Foo)));

    EXPECT_THAT(sizeof(Bar), Ne(sizeof(Foo)));
    EXPECT_THAT((iox::maxSize<Bar, Foo>()), Eq(sizeof(Bar)));

    EXPECT_THAT(sizeof(Bar), Ne(sizeof(FooBar)));
    EXPECT_THAT(sizeof(Foo), Ne(sizeof(FooBar)));
    EXPECT_THAT((iox::maxSize<Bar, Foo, FooBar>()), Eq(sizeof(FooBar)));

    EXPECT_THAT(sizeof(FooBar), Eq(sizeof(FuBar)));
    EXPECT_THAT((iox::maxSize<FooBar, FuBar>()), Eq(sizeof(FooBar)));
}

TEST(memory_test, MaxAlignmentWorksAsExpected)
{
    ::testing::Test::RecordProperty("TEST_ID", "7d5d3de1-f22c-47c1-b7fd-cacc35eef13c");
    EXPECT_THAT(iox::maxAlignment<Foo>(), Eq(alignof(Foo)));

    EXPECT_THAT(alignof(Bar), Ne(alignof(Foo)));
    EXPECT_THAT((iox::maxAlignment<Bar, Foo>()), Eq(alignof(Bar)));

    EXPECT_THAT(alignof(Bar), Ne(alignof(FooBar)));
    EXPECT_THAT(alignof(Foo), Ne(alignof(FooBar)));
    EXPECT_THAT((iox::maxAlignment<Bar, Foo, FooBar>()), Eq(alignof(FooBar)));

    EXPECT_THAT(alignof(FooBar), Eq(alignof(FuBar)));
    EXPECT_THAT((iox::maxAlignment<FooBar, FuBar>()), Eq(alignof(FooBar)));
}
} // namespace
