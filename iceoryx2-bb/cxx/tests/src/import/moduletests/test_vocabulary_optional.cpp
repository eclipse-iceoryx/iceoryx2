// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 by Apex.AI Inc. All rights reserved.
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

#include "iox/optional.hpp"
#include "test.hpp"

#include <memory>

namespace
{
using namespace ::testing;

/// @todo iox-#1696 create a typed test with to check if our optional has the same behaviour as the std::optional
/// @todo iox-#1696 create a parameterized test with an additional non-POD class

class Optional_test : public Test
{
  public:
    struct TestClass
    {
        int64_t value;
        int64_t secondValue;
    };

    void SetUp() override
    {
    }

    void TearDown() override
    {
    }

    iox::optional<int64_t> m_sutWithValue{0};
    iox::optional<int64_t> m_sutNoValue{iox::nullopt_t()};
};

TEST_F(Optional_test, DefaultCTorHasValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "2fd1d7b0-4d71-4177-9562-b8d3144e08c2");
    EXPECT_THAT(m_sutWithValue.has_value(), Eq(true));
}

TEST_F(Optional_test, NulloptCTor)
{
    ::testing::Test::RecordProperty("TEST_ID", "2e93637a-501a-4b34-a25e-01b3c7f99ccd");
    EXPECT_THAT(m_sutNoValue.has_value(), Eq(false));
}

TEST_F(Optional_test, emplaceWithoutPresetValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "54ce5155-eb13-4bd6-9736-a415e9002ae3");
    m_sutNoValue.emplace(123);
    EXPECT_THAT(m_sutNoValue.has_value(), Eq(true));
    EXPECT_THAT(m_sutNoValue.value(), Eq(123));
}

TEST_F(Optional_test, emplaceWithPresetValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "04ebc62f-7a89-4fd9-bad1-1b8df44343f4");
    m_sutWithValue.emplace(123);
    EXPECT_THAT(m_sutWithValue.has_value(), Eq(true));
    EXPECT_THAT(m_sutWithValue.value(), Eq(123));
}

TEST_F(Optional_test, value)
{
    ::testing::Test::RecordProperty("TEST_ID", "04e56e48-a7df-499b-9f72-1a8d5704c4c6");
    m_sutWithValue = 1234;
    EXPECT_THAT(m_sutWithValue.value(), Eq(1234));
}

TEST_F(Optional_test, const_value)
{
    ::testing::Test::RecordProperty("TEST_ID", "ac529426-2780-4c66-ad33-8b745ab3cb29");
    m_sutWithValue = 1234;
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast) const_cast to test const method
    EXPECT_THAT(const_cast<const iox::optional<int64_t>*>(&m_sutWithValue)->value(), Eq(1234));
}

TEST_F(Optional_test, resetWithValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "915d7952-6e7f-47c1-ad67-7b4b76de62ea");
    m_sutWithValue.reset();
    EXPECT_THAT(m_sutWithValue.has_value(), Eq(false));
}

TEST_F(Optional_test, resetWithoutValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "78a058f5-4f23-4f37-9b9f-d6edfe0fc3bb");
    m_sutNoValue.reset();
    EXPECT_THAT(m_sutNoValue.has_value(), Eq(false));
}

TEST_F(Optional_test, boolOperatorNoValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "b0102f46-626c-444f-bbc7-7fa9e1d3c89e");
    EXPECT_THAT(static_cast<bool>(m_sutNoValue), Eq(false));
}

TEST_F(Optional_test, boolOperatorWithValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "55f48d5b-71c2-4c09-b5a6-817b7205bc78");
    EXPECT_THAT(static_cast<bool>(m_sutWithValue), Eq(true));
}

TEST_F(Optional_test, ArrowOperator)
{
    ::testing::Test::RecordProperty("TEST_ID", "f0cad5c5-e032-454c-8934-3ba9aaf3c641");
    iox::optional<TestClass> sut{{0, 0}};
    sut->value = 1234;
    EXPECT_THAT(sut->value, Eq(1234));
}

TEST_F(Optional_test, ConstArrowOperator)
{
    ::testing::Test::RecordProperty("TEST_ID", "515aab10-cf10-4c56-b160-a7ef9d33937f");
    iox::optional<TestClass> sut{{0, 0}};
    sut->value = 12345;
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast) const_cast to test const method
    EXPECT_THAT((*const_cast<const iox::optional<TestClass>*>(&sut))->value, Eq(12345));
}

TEST_F(Optional_test, DereferenceOperator)
{
    ::testing::Test::RecordProperty("TEST_ID", "f8956c31-2f58-4f69-b548-10943d2edf3e");
    *m_sutWithValue = 789;
    EXPECT_THAT(*m_sutWithValue, Eq(789));
}

TEST_F(Optional_test, ConstDereferenceOperator)
{
    ::testing::Test::RecordProperty("TEST_ID", "8da28aec-48f8-4d39-896f-5a443b9eb0ab");
    *m_sutWithValue = 789;
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast) const_cast to test const method
    EXPECT_THAT(**const_cast<const iox::optional<int64_t>*>(&m_sutWithValue), Eq(789));
}

TEST_F(Optional_test, UserDefinedTypeAssignment)
{
    ::testing::Test::RecordProperty("TEST_ID", "bf37ae0d-9b7d-4878-a6d0-42ab1bd67633");
    iox::optional<TestClass> sut;
    sut = TestClass{1234, 22};
    EXPECT_THAT(sut->value, Eq(1234));
}

TEST_F(Optional_test, CompareWithEqualValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "5db271ca-8e86-45b0-be75-2e0ae535c780");
    m_sutWithValue = 123;
    iox::optional<int64_t> sut;
    sut = 123;
    EXPECT_THAT(m_sutWithValue == sut, Eq(true));
}

TEST_F(Optional_test, CompareWithEqualNullopt)
{
    ::testing::Test::RecordProperty("TEST_ID", "56a140fc-d850-4a7e-97d6-595afe48c1f7");
    iox::optional<int64_t> sut{iox::nullopt_t()};
    EXPECT_THAT(m_sutNoValue == sut, Eq(true));
}

TEST_F(Optional_test, CompareWithInequalValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "cfa1b454-982f-489b-93f6-a4fcd2f4be00");
    m_sutWithValue = 123;
    iox::optional<int64_t> sut;
    sut = 1231;
    EXPECT_THAT(m_sutWithValue == sut, Eq(false));
}

TEST_F(Optional_test, CompareWithNoValueWithValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "9a30d3c1-fb0e-4b3b-be87-ce9fd2858549");
    iox::optional<int64_t> sut;
    sut = 1231;
    EXPECT_THAT(m_sutNoValue == sut, Eq(false));
}

TEST_F(Optional_test, CompareWithNoValueWithNullopt)
{
    ::testing::Test::RecordProperty("TEST_ID", "9f1eb75d-9fca-43fd-93e6-3fd448c3583b");
    EXPECT_THAT(m_sutNoValue == iox::nullopt_t(), Eq(true));
    EXPECT_THAT(iox::nullopt_t() == m_sutNoValue, Eq(true));
}

TEST_F(Optional_test, CompareWithValueWithNullopt)
{
    ::testing::Test::RecordProperty("TEST_ID", "3dc645c7-7baf-4367-ae59-e18799bb910e");
    EXPECT_THAT(m_sutWithValue == iox::nullopt_t(), Eq(false));
    EXPECT_THAT(iox::nullopt_t() == m_sutWithValue, Eq(false));
}

TEST_F(Optional_test, NotCompareWithEqualValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "ce501d15-41ed-4666-b3ec-e22fc79c24de");
    m_sutWithValue = 123;
    iox::optional<int64_t> sut;
    sut = 123;
    EXPECT_THAT(m_sutWithValue != sut, Eq(false));
}

TEST_F(Optional_test, NotCompareWithNullopt)
{
    ::testing::Test::RecordProperty("TEST_ID", "0668eb1b-bc73-4549-9829-ece80f3700cd");
    EXPECT_THAT(m_sutWithValue != iox::nullopt_t(), Eq(true));
    EXPECT_THAT(m_sutNoValue != iox::nullopt_t(), Eq(false));
    EXPECT_THAT(iox::nullopt_t() != m_sutWithValue, Eq(true));
    EXPECT_THAT(iox::nullopt_t() != m_sutNoValue, Eq(false));
}

TEST_F(Optional_test, CopyCTorWithValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "5a65a6dd-5442-4afa-8c78-6ad5d5ec10a6");
    iox::optional<TestClass> sut{TestClass{4711, 1337}};

    iox::optional<TestClass> sut2(sut);

    ASSERT_THAT(sut2.has_value(), Eq(true));
    EXPECT_THAT(sut2->value, Eq(4711));
    EXPECT_THAT(sut2->secondValue, Eq(1337));
}

TEST_F(Optional_test, CopyCTorWithNoValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "597ca8af-264b-4261-9223-15854e7f351a");
    iox::optional<TestClass> sut = iox::nullopt_t();
    // NOLINTNEXTLINE(performance-unnecessary-copy-initialization) copy constructor shall be tested
    iox::optional<TestClass> sut2(sut);

    ASSERT_THAT(sut2.has_value(), Eq(false));
}

TEST_F(Optional_test, CopyAssignmentWithValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "af7f0a3b-feef-49dc-9f4f-af1eb4af4ef1");
    iox::optional<TestClass> sut2{TestClass{7474, 33331}};
    {
        iox::optional<TestClass> sut{TestClass{4711, 1337}};
        sut2 = sut;
    }

    ASSERT_THAT(sut2.has_value(), Eq(true));
    EXPECT_THAT(sut2->value, Eq(4711));
    EXPECT_THAT(sut2->secondValue, Eq(1337));
}

TEST_F(Optional_test, CopyAssignmentNoValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "8e6e5a06-91ec-4adb-aa99-58d7d9424410");
    iox::optional<TestClass> sut2{TestClass{7474, 33331}};
    {
        iox::optional<TestClass> sut = iox::nullopt_t();
        sut2 = sut;
    }

    ASSERT_THAT(sut2.has_value(), Eq(false));
}

TEST_F(Optional_test, CopyAssignmentFromNoValueToNoValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "ef04eb32-4a55-4367-823c-7e075fecfc41");
    iox::optional<TestClass> sut1;
    iox::optional<TestClass> sut2;
    sut2 = sut1;

    ASSERT_THAT(sut2.has_value(), Eq(false));
}

TEST_F(Optional_test, DirectCopyAssignmentWithNoValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "8dddd1c5-e59b-4f3c-9e6c-6fa9ac1daa86");
    iox::optional<TestClass> sut;
    const TestClass value{4711, 1337};

    sut = value;
    ASSERT_THAT(sut.has_value(), Eq(true));
    EXPECT_THAT(sut->value, Eq(4711));
    EXPECT_THAT(sut->secondValue, Eq(1337));
}

TEST_F(Optional_test, DirectCopyAssignmentWithValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "66fa19ab-0a08-48d3-824c-7b259e6f15b0");
    iox::optional<TestClass> sut{TestClass{7474, 33331}};
    TestClass value{4711, 1337};

    sut = value;
    ASSERT_THAT(sut.has_value(), Eq(true));
    EXPECT_THAT(sut->value, Eq(4711));
    EXPECT_THAT(sut->secondValue, Eq(1337));
}

TEST_F(Optional_test, MoveCTorWithValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "a7694c42-fb4d-4c53-930b-f0be78127027");
    iox::optional<TestClass> sut{TestClass{4711, 1337}};

    iox::optional<TestClass> sut2(std::move(sut));

    ASSERT_THAT(sut2.has_value(), Eq(true));
    EXPECT_THAT(sut2->value, Eq(4711));
    EXPECT_THAT(sut2->secondValue, Eq(1337));
    // NOLINTJUSTIFICATION we explicitly want to test the defined state of a moved object
    // NOLINTNEXTLINE(bugprone-use-after-move,hicpp-invalid-access-moved,clang-analyzer-cplusplus.Move)
    EXPECT_THAT(sut.has_value(), Eq(false));
}


TEST_F(Optional_test, MoveCTorWithNoValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "039e0cd4-5825-41a3-be56-87922f429cc6");
    iox::optional<TestClass> sut = iox::nullopt_t();
    iox::optional<TestClass> sut2(std::move(sut));

    ASSERT_THAT(sut2.has_value(), Eq(false));
    // NOLINTJUSTIFICATION we explicitly want to test the defined state of a moved object
    // NOLINTNEXTLINE(bugprone-use-after-move,hicpp-invalid-access-moved,clang-analyzer-cplusplus.Move)
    EXPECT_THAT(sut.has_value(), Eq(false));
}

TEST_F(Optional_test, MoveAssignmentWithValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "d671c710-c6e6-4f70-a05c-29134648f2df");
    iox::optional<TestClass> sut2{TestClass{7718, 80091}};
    {
        iox::optional<TestClass> sut{TestClass{4711, 1337}};
        sut2 = std::move(sut);
    }

    ASSERT_THAT(sut2.has_value(), Eq(true));
    EXPECT_THAT(sut2->value, Eq(4711));
    EXPECT_THAT(sut2->secondValue, Eq(1337));
}

TEST_F(Optional_test, MoveAssignmentWithNoValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "50435160-968f-4286-a1af-3a8ab42c50fb");
    iox::optional<TestClass> sut2{{0, 0}};
    sut2->value = 7718;
    sut2->secondValue = 80091;
    {
        iox::optional<TestClass> sut = iox::nullopt_t();
        sut2 = std::move(sut);
    }

    ASSERT_THAT(sut2.has_value(), Eq(false));
}

TEST_F(Optional_test, MoveAssignmentFromNoValueToNoValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "bcc0cbed-e969-43c8-963c-8370c9b48f55");
    iox::optional<TestClass> sut1;
    iox::optional<TestClass> sut2;
    sut2 = std::move(sut1);

    ASSERT_THAT(sut2.has_value(), Eq(false));
}

namespace
{
struct DTorTest
{
    static uint64_t dtorCounter;
    ~DTorTest()
    {
        dtorCounter++;
    }
};
uint64_t DTorTest::dtorCounter = 0;
} // namespace

TEST_F(Optional_test, Destructor)
{
    ::testing::Test::RecordProperty("TEST_ID", "41a225e4-6964-45da-9f94-ca9c79685814");
    {
        DTorTest::dtorCounter = 0;
        iox::optional<DTorTest> sut{DTorTest()};
        EXPECT_THAT(DTorTest::dtorCounter, Eq(1)); // dtor of temporary object only
        DTorTest::dtorCounter = 0;
    }
    EXPECT_THAT(DTorTest::dtorCounter, Eq(1));
}

TEST_F(Optional_test, DestructorOnCopyCTor)
{
    ::testing::Test::RecordProperty("TEST_ID", "42a8d5f6-2d7f-4ad8-8433-f4e4b82b4eec");
    {
        iox::optional<DTorTest> sut{DTorTest()};
        {
            DTorTest::dtorCounter = 0;
            // destructor on copy constructor shall be tested
            // NOLINTNEXTLINE(performance-unnecessary-copy-initialization)
            iox::optional<DTorTest> sut2{sut};
            EXPECT_THAT(DTorTest::dtorCounter, Eq(0));
        }
        EXPECT_THAT(DTorTest::dtorCounter, Eq(1)); // dtor of sut2
        EXPECT_THAT(sut.has_value(), Eq(true));
        DTorTest::dtorCounter = 0;
    }
    EXPECT_THAT(DTorTest::dtorCounter, Eq(1)); // dtor of sut
}

TEST_F(Optional_test, DestructorOnCopyAssignment)
{
    ::testing::Test::RecordProperty("TEST_ID", "49aae157-ea1f-4998-b4bd-07e5ade6ce02");
    {
        iox::optional<DTorTest> sut{DTorTest()};
        {
            iox::optional<DTorTest> sut2{DTorTest()};
            DTorTest::dtorCounter = 0;
            sut = sut2;
            EXPECT_THAT(DTorTest::dtorCounter, Eq(0));
        }
        EXPECT_THAT(DTorTest::dtorCounter, Eq(1)); // dtor of sut2
        EXPECT_THAT(sut.has_value(), Eq(true));
        DTorTest::dtorCounter = 0;
    }
    EXPECT_THAT(DTorTest::dtorCounter, Eq(1)); // dtor of sut
}

TEST_F(Optional_test, DestructorOnMoveCTor)
{
    ::testing::Test::RecordProperty("TEST_ID", "95126b4b-7700-4537-b15f-c9b4697b2d28");
    {
        iox::optional<DTorTest> sut{DTorTest()};
        {
            DTorTest::dtorCounter = 0;
            iox::optional<DTorTest> sut2{std::move(sut)};
            EXPECT_THAT(DTorTest::dtorCounter, Eq(1)); // dtor of sut
            // NOLINTJUSTIFICATION we explicitly want to test the defined state of a moved object
            // NOLINTNEXTLINE(bugprone-use-after-move,hicpp-invalid-access-moved,clang-analyzer-cplusplus.Move)
            EXPECT_THAT(sut.has_value(), Eq(false));
            DTorTest::dtorCounter = 0;
        }
        EXPECT_THAT(DTorTest::dtorCounter, Eq(1)); // dtor of sut2
        DTorTest::dtorCounter = 0;
    }
    EXPECT_THAT(DTorTest::dtorCounter, Eq(0)); // sut dtor already called
}

TEST_F(Optional_test, DestructorOnMoveAssignment)
{
    ::testing::Test::RecordProperty("TEST_ID", "c9071ba6-71eb-4926-bf7b-8348c6543e59");
    {
        iox::optional<DTorTest> sut{DTorTest()};
        {
            iox::optional<DTorTest> sut2{DTorTest()};
            DTorTest::dtorCounter = 0;
            sut = std::move(sut2);
            EXPECT_THAT(DTorTest::dtorCounter, Eq(1)); // dtor of sut2
            DTorTest::dtorCounter = 0;
        }
        EXPECT_THAT(DTorTest::dtorCounter, Eq(0));
    }
    EXPECT_THAT(DTorTest::dtorCounter, Eq(1)); // dtor of sut
}

TEST_F(Optional_test, DestructorOnEmplace)
{
    ::testing::Test::RecordProperty("TEST_ID", "974aa499-8dc2-4b7d-8ed8-fea66f3f3358");
    {
        iox::optional<DTorTest> sut{DTorTest()};
        {
            iox::optional<DTorTest> sut2{DTorTest()};
            DTorTest::dtorCounter = 0;
            sut2.emplace(sut.value());
            EXPECT_THAT(DTorTest::dtorCounter, Eq(1)); // dtor of previous sut2 value
            DTorTest::dtorCounter = 0;
        }
        EXPECT_THAT(DTorTest::dtorCounter, Eq(1)); // dtor of sut2
        DTorTest::dtorCounter = 0;
    }
    EXPECT_THAT(DTorTest::dtorCounter, Eq(1)); // dtor of sut
}

TEST_F(Optional_test, MakeOptional)
{
    ::testing::Test::RecordProperty("TEST_ID", "eda0230c-ea45-4303-8995-9fca39d1732e");
    struct Make
    {
        Make() = default;

        // NOLINTNEXTLINE(bugprone-easily-swappable-parameters) only for testing purposes
        Make(int a, int b)
            : a(a)
            , b(b)
        {
        }
        int a{0};
        int b{0};
    };

    auto sut1 = iox::make_optional<Make>(123, 456);
    EXPECT_THAT(sut1->a, Eq(123));
    EXPECT_THAT(sut1->b, Eq(456));
    auto sut2 = iox::make_optional<Make>();
    EXPECT_THAT(sut2->a, Eq(0));
    EXPECT_THAT(sut2->b, Eq(0));
}

TEST_F(Optional_test, ReturningNulloptWithoutConstruction)
{
    ::testing::Test::RecordProperty("TEST_ID", "689f4d36-5e66-4bae-8122-40bfc3f7c8f1");
    auto val = []() -> iox::optional<int> { return iox::nullopt; }();
    EXPECT_THAT(val.has_value(), Eq(false));
}

TEST_F(Optional_test, CopyConstructionWithElementWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "5839d59d-b564-4d82-acee-b324903cd4f9");
    const TestClass testClass{5, 6};
    iox::optional<TestClass> sut(testClass);

    ASSERT_TRUE(sut.has_value());
    EXPECT_THAT(sut->value, Eq(5));
    EXPECT_THAT(sut->secondValue, Eq(6));
}

const std::string DEFAULT_STRING{"Live long and prosper"};
constexpr int8_t DEFAULT_INT{0};
constexpr int8_t DEFAULT_MULTIPLICATOR{2};

struct TestStructForInPlaceConstruction
{
    TestStructForInPlaceConstruction() = default;
    ~TestStructForInPlaceConstruction() = default;

    TestStructForInPlaceConstruction(const TestStructForInPlaceConstruction&) = delete;
    TestStructForInPlaceConstruction(TestStructForInPlaceConstruction&&) = delete;
    TestStructForInPlaceConstruction& operator=(const TestStructForInPlaceConstruction&) = delete;
    TestStructForInPlaceConstruction& operator=(TestStructForInPlaceConstruction&&) = delete;

    explicit TestStructForInPlaceConstruction(const int8_t& val)
        : val(val)
    {
    }

    // NOLINTNEXTLINE(cppcoreguidelines-rvalue-reference-param-not-moved) intended for the test
    explicit TestStructForInPlaceConstruction(int8_t&& val)
        : val(static_cast<int8_t>(DEFAULT_MULTIPLICATOR * val))
    {
    }

    explicit TestStructForInPlaceConstruction(std::unique_ptr<std::string> ptr)
        : ptr(std::move(ptr))
    {
    }

    TestStructForInPlaceConstruction(const int8_t& val, std::unique_ptr<std::string> ptr)
        : val(val)
        , ptr(std::move(ptr))
    {
    }

    int8_t val{DEFAULT_INT};
    std::unique_ptr<std::string> ptr{new std::string(DEFAULT_STRING)};
};

TEST_F(Optional_test, InPlaceConstructionCtorCallsDefCtorWhenCalledWithoutArgs)
{
    ::testing::Test::RecordProperty("TEST_ID", "64c0b0ff-7362-4e21-b7b7-189af9a3a060");
    iox::optional<TestStructForInPlaceConstruction> sut(iox::in_place);
    ASSERT_TRUE(sut.has_value());
    EXPECT_THAT(sut->val, Eq(DEFAULT_INT));
    ASSERT_TRUE(sut->ptr);
    EXPECT_THAT(sut->ptr->c_str(), StrEq(DEFAULT_STRING));
}

TEST_F(Optional_test, InPlaceConstructionCtorCallsCorrectCtorWhenCalledWithLVal)
{
    ::testing::Test::RecordProperty("TEST_ID", "132aeab3-e370-4afb-a418-f115b97ce7a5");
    constexpr int8_t VAL = 46;
    iox::optional<TestStructForInPlaceConstruction> sut(iox::in_place, VAL);
    ASSERT_TRUE(sut.has_value());
    EXPECT_THAT(sut->val, Eq(VAL));
    ASSERT_TRUE(sut->ptr);
    EXPECT_THAT(sut->ptr->c_str(), StrEq(DEFAULT_STRING));
}

TEST_F(Optional_test, InPlaceConstructionCtorCallsCorrectCtorWhenCalledWithPodRVal)
{
    ::testing::Test::RecordProperty("TEST_ID", "de7e36ea-44f9-4b82-9b0d-0bce8af2a10a");
    constexpr int8_t VALUE = 23;
    int8_t val = VALUE;
    // move constructor of TestStructForInPlaceConstruction is called
    // NOLINTNEXTLINE(hicpp-move-const-arg, performance-move-const-arg)
    iox::optional<TestStructForInPlaceConstruction> sut(iox::in_place, std::move(val));
    ASSERT_TRUE(sut.has_value());
    EXPECT_THAT(sut->val, Eq(DEFAULT_MULTIPLICATOR * VALUE));
    ASSERT_TRUE(sut->ptr);
    EXPECT_THAT(sut->ptr->c_str(), StrEq(DEFAULT_STRING));
}

TEST_F(Optional_test, InPlaceConstructionCtorCallsCorrectCtorWhenCalledWithComplexTypeRVal)
{
    ::testing::Test::RecordProperty("TEST_ID", "2a43bdf4-dfdf-4b3b-908b-d162b13435a9");
    const std::string NEW_STRING{"Without followers, evil cannot spread"};
    std::unique_ptr<std::string> ptr(new std::string(NEW_STRING));
    iox::optional<TestStructForInPlaceConstruction> sut(iox::in_place, std::move(ptr));
    ASSERT_TRUE(sut.has_value());
    EXPECT_THAT(sut->val, Eq(DEFAULT_INT));
    ASSERT_TRUE(sut->ptr);
    EXPECT_THAT(sut->ptr->c_str(), NEW_STRING);
}

TEST_F(Optional_test, InPlaceConstructionCtorCallsCorrectCtorWhenCalledWithMixedArgs)
{
    ::testing::Test::RecordProperty("TEST_ID", "49f1376c-6723-4231-83da-4682e89f1b6e");
    constexpr int8_t VAL = 11;
    const std::string NEW_STRING{"Insufficient facts always invite danger"};

    std::unique_ptr<std::string> ptr(new std::string(NEW_STRING));
    iox::optional<TestStructForInPlaceConstruction> sut(iox::in_place, VAL, std::move(ptr));
    ASSERT_TRUE(sut.has_value());
    EXPECT_THAT(sut->val, Eq(VAL));
    ASSERT_TRUE(sut->ptr);
    EXPECT_THAT(sut->ptr->c_str(), NEW_STRING);
}
} // namespace
