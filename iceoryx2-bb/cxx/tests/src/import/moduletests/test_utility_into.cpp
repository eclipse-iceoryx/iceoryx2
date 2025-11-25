// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex AI Inc. All rights reserved.
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

#include "iox/into.hpp"
#include "test.hpp"

namespace
{
enum class A : uint8_t
{
    A1 = 13,
    A2
};

enum class B : uint8_t
{
    B1 = 42,
    B2
};
} // namespace

namespace iox
{
template <>
constexpr B from<A, B>(A e) noexcept
{
    switch (e)
    {
    case A::A1:
        return B::B1;
    case A::A2:
        return B::B2;
    }
}
} // namespace iox

namespace
{
using namespace ::testing;
using namespace iox;

TEST(into_test_from, FromWorksAsConstexpr)
{
    ::testing::Test::RecordProperty("TEST_ID", "5b7cac32-c0ef-4f29-8314-59ed8850d1f5");
    constexpr A FROM_VALUE{A::A1};
    constexpr B TO_VALUE{B::B1};
    constexpr B SUT = iox::from<A, B>(FROM_VALUE);
    EXPECT_EQ(SUT, TO_VALUE);
}

TEST(into_test_into, IntoWorksWhenFromIsSpecialized)
{
    ::testing::Test::RecordProperty("TEST_ID", "1d4331e5-f603-4e50-bdb2-75df57b0b517");
    constexpr A FROM_VALUE{A::A2};
    constexpr B TO_VALUE{B::B2};
    constexpr B SUT = iox::into<B>(FROM_VALUE);
    EXPECT_EQ(SUT, TO_VALUE);
}
} // namespace
