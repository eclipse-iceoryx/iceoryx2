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
#include "iox/error_reporting/violation.hpp"

#include "module_a/errors.hpp"
#include "module_b/errors.hpp"

namespace
{

using namespace ::testing;
using namespace iox::er;

constexpr ErrorCode CODE1{73};
constexpr ErrorCode CODE2{21};
constexpr ModuleId ID1{666};
constexpr ModuleId ID2{999};
constexpr ModuleId ANY_ID{ModuleId::ANY};

struct Unknown
{
};

template <typename T>
class ErrorType_test : public Test
{
  public:
    void SetUp() override
    {
    }

    void TearDown() override
    {
    }

    using Sut = T;

    Sut sut{CODE1, ID1};
};

// We can extend this easily for other error types as they have to conform to the same interface
// Any error type must satsify these tests.
using TestTypes = Types<Violation>;
TYPED_TEST_SUITE(ErrorType_test, TestTypes, );

TYPED_TEST(ErrorType_test, constructionAndDestructionWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "7bace8bc-cb4a-41ca-a2fa-3d73be0d93fe");

    EXPECT_EQ(this->sut.code(), CODE1);
    EXPECT_EQ(this->sut.module(), ID1);
}

TYPED_TEST(ErrorType_test, singleArgumentConstructionWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "f4c7fc8d-43d7-487d-b748-ea45a1aeee73");

    typename TestFixture::Sut other(CODE1);

    EXPECT_EQ(other.code(), CODE1);
    EXPECT_EQ(other.module(), ANY_ID);
    EXPECT_NE(this->sut, other);
}

TYPED_TEST(ErrorType_test, copyCtorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "8fa95dda-dee0-468a-856e-54fa15708c26");

    typename TestFixture::Sut copy(this->sut);

    EXPECT_EQ(copy, this->sut);
}

TYPED_TEST(ErrorType_test, copyAssignmentWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "318990f3-9a11-4f38-b8a7-57a61336127d");

    typename TestFixture::Sut copy(CODE2, ID2);
    copy = this->sut;

    EXPECT_EQ(copy, this->sut);
}

TYPED_TEST(ErrorType_test, moveCtorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "f1a03ab2-abf8-4239-90fb-64067532a9ed");

    typename TestFixture::Sut copy(this->sut);
    typename TestFixture::Sut movedTo(std::move(this->sut));

    EXPECT_EQ(movedTo, copy);
}

TYPED_TEST(ErrorType_test, moveAssignmentWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "2f1aadfb-c1be-4ff2-83b8-1db55e0aacdf");

    typename TestFixture::Sut copy(this->sut);
    typename TestFixture::Sut movedTo(CODE2, ID2);
    movedTo = std::move(this->sut);

    EXPECT_EQ(movedTo, copy);
}

TYPED_TEST(ErrorType_test, equalityComparisonWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "ca26f79a-0a01-4726-b8e0-afe42dce2b4d");

    typename TestFixture::Sut same(CODE1, ID1);
    typename TestFixture::Sut different1(CODE1, ID2);
    typename TestFixture::Sut different2(CODE2, ID1);
    typename TestFixture::Sut different3(CODE2, ID2);

    EXPECT_TRUE(same == this->sut);
    EXPECT_TRUE(this->sut == same);

    EXPECT_FALSE(different1 == this->sut);
    EXPECT_FALSE(this->sut == different1);

    EXPECT_FALSE(different2 == this->sut);
    EXPECT_FALSE(this->sut == different2);

    EXPECT_FALSE(different3 == this->sut);
    EXPECT_FALSE(this->sut == different3);
}

TYPED_TEST(ErrorType_test, unequalityComparisonWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "d6e6fed2-ac98-4d8d-b1ec-fa2ffd4de14b");

    typename TestFixture::Sut same(CODE1, ID1);
    typename TestFixture::Sut different1(CODE1, ID2);
    typename TestFixture::Sut different2(CODE2, ID1);
    typename TestFixture::Sut different3(CODE2, ID2);

    EXPECT_FALSE(same != this->sut);
    EXPECT_FALSE(this->sut != same);

    EXPECT_TRUE(different1 != this->sut);
    EXPECT_TRUE(this->sut != different1);

    EXPECT_TRUE(different2 != this->sut);
    EXPECT_TRUE(this->sut != different2);

    EXPECT_TRUE(different3 != this->sut);
    EXPECT_TRUE(this->sut != different3);
}

TYPED_TEST(ErrorType_test, toCodeWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "9f84bf6f-d5fb-4fe0-960f-db1a90cce025");

    auto code = toCode(this->sut);
    EXPECT_EQ(code, this->sut.code());
}

// cannot be tested with the typed test ErrorType_test
TEST(ErrorCode_test, toCodeWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "408c6c10-a4ce-4c7e-b9f4-e8f8c7033fa0");

    ErrorCode sut(73);
    ErrorCode code = toCode(sut);
    EXPECT_EQ(code, sut);
}

TYPED_TEST(ErrorType_test, toModuleWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "231406c4-43ef-4e0f-91cf-09779bbb2327");

    auto module = toModule(this->sut);
    EXPECT_EQ(module, this->sut.module());
}

// while it does not do so by default, it is allowed to transform the error in other ways
TYPED_TEST(ErrorType_test, toErrorPreservesCodeAndModule)
{
    ::testing::Test::RecordProperty("TEST_ID", "cd08b3ce-d683-492b-8ed6-f6ac398600d0");
    auto err = toError(this->sut);

    EXPECT_EQ(err.code(), this->sut.code());
    EXPECT_EQ(err.module(), this->sut.module());
}

TEST(Violation_test, createEnforceViolationWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "9a2f9d1c-41dc-4a7e-a0dc-aa40047ae9a0");

    auto sut = Violation::createEnforceViolation();
    auto exp = Violation{iox::er::ViolationErrorCode::ENFORCE_VIOLATION};

    EXPECT_EQ(sut, exp);
}

TEST(Violation_test, createAssertViolationWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "2a5f24a7-4d82-4c27-bc45-57d9f1c759fa");

    auto sut = Violation::createAssertViolation();
    auto exp = Violation{iox::er::ViolationErrorCode::ASSERT_VIOLATION};

    EXPECT_EQ(sut, exp);
}

TEST(ErrorNameTranslation_test, knownErrorTranslatesToCorrectErrorString)
{
    ::testing::Test::RecordProperty("TEST_ID", "26f3e6bb-2d85-47f6-995b-6ca48425e710");

    using ErrorA = module_a::errors::Error;
    using CodeA = module_a::errors::Code;

    ErrorA errorA(CodeA::OutOfMemory);
    const char* resultA = toErrorName(errorA);
    EXPECT_EQ(resultA, errorA.name());

    using ErrorB = module_b::errors::Error;
    using CodeB = module_b::errors::Code;

    ErrorB errorB(CodeB::OutOfMemory);
    const char* resultB = toErrorName(errorB);
    EXPECT_EQ(resultB, errorB.name());
}

TEST(ErrorNameTranslation_test, knownModuleTranslatesToCorrectModuleString)
{
    ::testing::Test::RecordProperty("TEST_ID", "88036900-c9e0-4a07-86fd-411f2273bbf6");

    using Error = module_a::errors::Error;
    using Code = module_a::errors::Code;

    Error error(Code::OutOfMemory);
    const char* resultA = toModuleName(error);
    EXPECT_EQ(resultA, error.moduleName());

    using ErrorB = module_b::errors::Error;
    using CodeB = module_b::errors::Code;

    ErrorB errorB(CodeB::OutOfMemory);
    const char* resultB = toModuleName(errorB);
    EXPECT_EQ(resultB, errorB.moduleName());
}

} // namespace
