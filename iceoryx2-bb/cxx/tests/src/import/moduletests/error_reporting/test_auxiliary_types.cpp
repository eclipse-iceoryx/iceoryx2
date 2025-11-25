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

#include "test.hpp"
#include <gtest/gtest.h>

#include "iox/error_reporting/types.hpp"

namespace
{
using namespace ::testing;
using namespace iox::er;

template <typename T>
class RegularType_test : public Test
{
  public:
    void SetUp() override
    {
    }

    void TearDown() override
    {
    }

    using Sut = T;
    using V = typename Sut::type;

    static constexpr V VALUE{73};
    // could be constexpr but is more cumbersome to access
    V value{VALUE};
    V differentValue{VALUE + 1};

    Sut sut{VALUE};
};

using TestTypes = Types<ErrorCode, ModuleId>;
TYPED_TEST_SUITE(RegularType_test, TestTypes, );

TYPED_TEST(RegularType_test, constructionAndDestructionWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "d2482a4d-5cbd-4de9-8890-db4659752409");

    EXPECT_EQ(this->sut.value, this->value);
}

TYPED_TEST(RegularType_test, copyCtorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "51c80f47-0b22-4641-94bd-fa1cec8028d0");

    typename TestFixture::Sut copy(this->sut);

    EXPECT_EQ(copy.value, this->value);
    EXPECT_EQ(copy, this->sut);
}

TYPED_TEST(RegularType_test, copyAssignmentWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "ffc6dec9-6ac6-45e3-b3d6-39f1a04c0475");

    typename TestFixture::Sut copy(this->differentValue);
    copy = this->sut;

    EXPECT_EQ(copy.value, this->value);
    EXPECT_EQ(copy, this->sut);
}

TYPED_TEST(RegularType_test, moveCtorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "04a18398-9122-48ce-8bad-28e5331f9e68");

    typename TestFixture::Sut copy(this->sut);
    typename TestFixture::Sut movedTo(std::move(this->sut));

    EXPECT_EQ(movedTo.value, this->value);
    EXPECT_EQ(movedTo, copy);
}

TYPED_TEST(RegularType_test, moveAssignmentWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "46d8ce73-7b6b-4c0f-aca2-d2e404e8e0e6");

    typename TestFixture::Sut copy(this->sut);
    typename TestFixture::Sut movedTo(this->differentValue);
    movedTo = std::move(this->sut);

    EXPECT_EQ(movedTo.value, this->value);
    EXPECT_EQ(movedTo, copy);
}

TYPED_TEST(RegularType_test, equalComparisonWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "f6474ff1-b202-43c3-ae1b-de26a20bffcd");

    typename TestFixture::Sut same(this->value);
    typename TestFixture::Sut different(this->differentValue);

    EXPECT_TRUE(same == this->sut);
    EXPECT_TRUE(this->sut == same);

    EXPECT_FALSE(different == this->sut);
    EXPECT_FALSE(this->sut == different);
}

TYPED_TEST(RegularType_test, unequalComparisonWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "941b75b6-6627-40aa-aac2-689686430107");

    typename TestFixture::Sut same(this->value);
    typename TestFixture::Sut different(this->differentValue);

    EXPECT_FALSE(same != this->sut);
    EXPECT_FALSE(this->sut != same);

    EXPECT_TRUE(different != this->sut);
    EXPECT_TRUE(this->sut != different);
}

} // namespace
