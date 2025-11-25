// Copyright (c) 2020 by Robert Bosch GmbH. All rights reserved.
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

#include "iceoryx_hoofs/testing/error_reporting/testing_support.hpp"
#include "iox/attributes.hpp"
#include "iox/detail/hoofs_error_reporting.hpp"
#include "iox/function_ref.hpp"
#include "test.hpp"

namespace
{
using namespace ::testing;
using namespace iox;
using namespace iox::testing;

constexpr int FREE_FUNC_TEST_VALUE = 42 + 42;
constexpr int FUNCTOR_TEST_VALUE = 11;
constexpr int MEMBER_FUNC_TEST_VALUE = 4273;
constexpr int SAME_SIGNATURE_INT_TEST_VALUE = 12345;
constexpr int SAME_SIGNATURE_VOID_TEST_VALUE = 12346;
constexpr int SAME_SIGNATURE_INT_INT_TEST_VALUE = 12347;

int freeFunction()
{
    return FREE_FUNC_TEST_VALUE;
}

void freeVoidFunction(int& arg)
{
    arg = FREE_FUNC_TEST_VALUE;
}

class Functor
{
  public:
    int operator()()
    {
        return FUNCTOR_TEST_VALUE;
    }
};

struct ComplexType
{
    char a;
    int b;
    float c;

    bool operator==(const ComplexType& rhs) const
    {
        return (a == rhs.a && b == rhs.b && c == rhs.c);
    }
};

ComplexType returnComplexType(ComplexType foo)
{
    return foo;
}

int SameSignature(function_ref<int(int)> callback)
{
    return callback(SAME_SIGNATURE_INT_TEST_VALUE);
}

int SameSignature(function_ref<int(void)> callback)
{
    return callback();
}

int SameSignature(function_ref<int(int, int)> callback)
{
    return callback(SAME_SIGNATURE_INT_INT_TEST_VALUE, SAME_SIGNATURE_INT_INT_TEST_VALUE);
}

class function_refTest : public Test
{
  public:
    void SetUp() override
    {
    }

    void TearDown() override
    {
    }

    static int foobar()
    {
        return MEMBER_FUNC_TEST_VALUE;
    }

    uint8_t m_iter{0};
};

using function_refDeathTest = function_refTest;

TEST_F(function_refTest, CallValidByAssignResultEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "99c66fbd-2df5-48d9-bc89-8394e99c76ba");
    auto lambda = []() -> int { return 7253; };
    auto wrongLambda = []() -> int { return 5372; };
    function_ref<int()> sut{wrongLambda};
    sut = lambda;
    EXPECT_THAT(sut(), Eq(7253));
}

TEST_F(function_refTest, CallValidByCopyConstructResultEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "cb30b36d-1c3d-4848-a497-d6d3e72edbd5");
    auto lambda = []() -> int { return 3527; };
    function_ref<int()> sut1{lambda};
    function_ref<int()> sut2(sut1);
    EXPECT_THAT(sut2(), Eq(3527));
}

TEST_F(function_refTest, CreateValidByCopyAssignResultEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "fb8b568c-06fe-4af2-8d2a-c2527f799ad9");
    auto lambda = []() -> int { return 43; };
    auto lambda2 = []() -> int { return 34; };
    function_ref<int()> sut2{lambda2};
    {
        function_ref<int()> sut1{lambda};
        EXPECT_THAT(sut1(), Eq(43));
        sut2 = sut1;
    }
    EXPECT_THAT(sut2(), Eq(43));
}

TEST_F(function_refTest, CreateValidByMoveResultEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "b7b5ac66-a703-429b-9e38-44ebcd9a7519");
    auto lambda = []() -> int { return 123; };
    function_ref<int()> sut1{lambda};
    function_ref<int()> sut2{std::move(sut1)};
    EXPECT_THAT(sut2(), Eq(123));
}

TEST_F(function_refTest, CreateValidByMoveAssignResultEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "e641e34f-0e8a-4092-8224-b5f52b964a16");
    auto lambda1 = []() -> int { return 118; };
    auto lambda2 = []() -> int { return 999; };
    function_ref<int()> sut1{lambda1};
    {
        function_ref<int()> sut2{lambda2};
        sut1 = std::move(sut2);
    }
    EXPECT_THAT(sut1(), Eq(999));
}

TEST_F(function_refDeathTest, CallMovedFromLeadsToTermination)
{
    ::testing::Test::RecordProperty("TEST_ID", "3402f27e-ced5-483f-ab39-0069cfd172ac");

    auto lambda = []() -> int { return 7654; };
    function_ref<int()> sut1{lambda};
    function_ref<int()> sut2{std::move(sut1)};

    // NOLINTJUSTIFICATION Use after move is tested here
    // NOLINTBEGIN(bugprone-use-after-move, hicpp-invalid-access-moved)
    runInTestThread([&] { sut1(); });
    // NOLINTEND(bugprone-use-after-move, hicpp-invalid-access-moved)

    IOX_TESTING_EXPECT_PANIC();
}

TEST_F(function_refTest, CreateValidAndSwapResultEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "0ef9f5f0-c914-4b9d-9841-0c2e153a8451");
    auto lambda1 = []() -> int { return 42; };
    auto lambda2 = []() -> int { return 73; };
    function_ref<int()> sut1(lambda1);
    function_ref<int()> sut2(lambda2);
    EXPECT_THAT(sut1(), Eq(42));
    EXPECT_THAT(sut2(), Eq(73));
    sut1.swap(sut2);
    EXPECT_THAT(sut1(), Eq(73));
    EXPECT_THAT(sut2(), Eq(42));
}

TEST_F(function_refTest, CreateValidWithCapturingLambdaVoidVoidIncremented)
{
    ::testing::Test::RecordProperty("TEST_ID", "d4d55fdf-2cce-4a8a-bcd7-053f2075304f");
    auto lambda = [&] { m_iter++; };
    function_ref<void(void)> sut(lambda);
    sut();
    EXPECT_THAT(m_iter, Eq(1));
}

TEST_F(function_refTest, CreateValidWithLambdaIntVoidResultEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "81630738-c0b0-4730-9a2e-b47f5f3b6d22");
    auto lambda = [](void) -> int { return 1337; };
    function_ref<int(void)> sut(lambda);
    EXPECT_THAT(sut(), Eq(1337));
}

TEST_F(function_refTest, CreateValidWithLambdaIntIntIncremented)
{
    ::testing::Test::RecordProperty("TEST_ID", "4ef4025b-b67b-49d8-b607-97dfb9bfa26f");
    auto lambda = [](int var) -> int { return ++var; };
    function_ref<int(int)> sut(lambda);
    EXPECT_THAT(sut(0), Eq(1));
}

TEST_F(function_refTest, CreateValidWithFreeFunctionResultEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "aaf49b6b-054a-4f8f-b176-6d92bb2918da");
    function_ref<int()> sut(freeFunction);
    EXPECT_THAT(sut(), Eq(FREE_FUNC_TEST_VALUE));
}

TEST_F(function_refTest, CreateValidWithComplexTypeResultEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "7c6a4bf0-989f-4d15-a905-03fddf6d80bc");
    ComplexType fuubar{1, 2, 1.3F};
    function_ref<ComplexType(ComplexType)> sut(returnComplexType);
    EXPECT_THAT(sut(fuubar), Eq(fuubar));
}

TEST_F(function_refTest, CreateValidWithFunctorResultEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "6fd3609b-3254-429a-96ae-20f6dbe99b2a");
    Functor foo;
    function_ref<int()> sut(foo);
    EXPECT_THAT(sut(), Eq(FUNCTOR_TEST_VALUE));
}

TEST_F(function_refTest, CreateValidWithStdBindResultEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "f5f82896-44db-4d2d-96d0-c1b0fbbe5508");
    auto callable = std::bind(&function_refTest::foobar);
    function_ref<int()> sut(callable);
    EXPECT_THAT(sut(), Eq(MEMBER_FUNC_TEST_VALUE));
}

TEST_F(function_refTest, CreateValidWithStdFunctionResultEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "4ed2254b-a8fd-4b16-8f48-195868c044c0");
    std::function<int()> baz;
    baz = []() -> int { return 24; };
    function_ref<int()> sut(baz);
    EXPECT_THAT(sut(), Eq(24));
}

TEST_F(function_refTest, StoreInStdFunctionResultEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "99d9ac22-dddb-44fc-a80b-fe559f6acf63");
    auto lambda = []() -> int { return 37; };
    function_ref<int()> moep(lambda);
    // Moves iox::function_ref into std::function, needs copy c'tor
    std::function<int()> sut(moep);
    EXPECT_THAT(sut(), Eq(37));
}

TEST_F(function_refTest, CallOverloadedFunctionResultsInCallOfInt)
{
    ::testing::Test::RecordProperty("TEST_ID", "3910ee08-305a-4764-82b3-8b8aa7e7038e");
    auto value = SameSignature([](int value) -> int { return value; });
    EXPECT_THAT(value, Eq(SAME_SIGNATURE_INT_TEST_VALUE));
}

TEST_F(function_refTest, CallOverloadedFunctionResultsInCallOfVoid)
{
    ::testing::Test::RecordProperty("TEST_ID", "ca8e8384-0b20-4e4a-b372-698c4e6672b7");
    auto value = SameSignature([](void) -> int { return SAME_SIGNATURE_VOID_TEST_VALUE; });
    EXPECT_THAT(value, Eq(SAME_SIGNATURE_VOID_TEST_VALUE));
}

TEST_F(function_refTest, CallOverloadedFunctionResultsInCallOfIntInt)
{
    ::testing::Test::RecordProperty("TEST_ID", "b37158b6-8100-4f80-bd62-d2957a7d9c46");
    auto value = SameSignature([](int value1, int value2 IOX_MAYBE_UNUSED) -> int { return value1; });
    EXPECT_THAT(value, Eq(SAME_SIGNATURE_INT_INT_TEST_VALUE));
}

TEST_F(function_refTest, CreationWithFunctionPointerWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "082bd86b-85d8-478b-b723-3d1f0db2d01d");
    constexpr auto fp = &freeFunction;
    function_ref<int(void)> sut(fp);

    auto result = sut();
    EXPECT_EQ(result, FREE_FUNC_TEST_VALUE);
}

TEST_F(function_refTest, CreationWithFunctionPointerWithRefArgWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "2d75aa14-9743-49ee-b80c-b47b1326b96b");
    constexpr auto fp = &freeVoidFunction;
    function_ref<void(int&)> sut(fp);

    int arg{0};
    sut(arg);
    EXPECT_EQ(arg, FREE_FUNC_TEST_VALUE);
}

TEST_F(function_refTest, CreationWithFunctionPointerWithComplexTypeArgWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "769a00c8-9187-4cff-b352-790311c2c42f");
    constexpr auto fp = &returnComplexType;
    function_ref<ComplexType(ComplexType)> sut(fp);

    ComplexType arg{1, 2, 3.3F};
    auto result = sut(arg);
    EXPECT_EQ(result, arg);
}

template <typename Functor>
void canBeConstructedFromConstReferenceCallable(const Functor& f)
{
    function_ref<void()> sut(f);
}

TEST_F(function_refTest, CanBeConstructedFromConstReference)
{
    ::testing::Test::RecordProperty("TEST_ID", "df97fc89-3b37-4be2-af8e-80d405fa9797");
    // this is a compile time test. the function_ref should be constructable from
    // const references to callables. if this is not the case the test fails with
    // a compilation error
    canBeConstructedFromConstReferenceCallable([] {});
}

} // namespace
