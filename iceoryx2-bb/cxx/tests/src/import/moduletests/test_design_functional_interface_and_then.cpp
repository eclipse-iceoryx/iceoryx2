// Copyright (c) 2022 by Apex.AI Inc. All rights reserved.
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

#include "test_design_functional_interface_types.hpp"

namespace
{
using namespace test_design_functional_interface;
using namespace ::testing;

TYPED_TEST(FunctionalInterface_test, AndThenHasCorrectSignature)
{
    ::testing::Test::RecordProperty("TEST_ID", "7636fda5-090f-4dd6-b3a0-3d71bdbca787");
    using Factory = typename TestFixture::TestFactoryType;
    constexpr bool DOES_AND_THEN_HAVE_A_VALUE = iox::internal::HasValueMethod<typename Factory::Type>::value;

    EXPECT_THAT(DOES_AND_THEN_HAVE_A_VALUE, Eq(Factory::EXPECT_AND_THEN_WITH_VALUE));
}

// the macro is used as code generator to make the tests more readable. because of the
// template nature of those tests this cannot be implemented in the same readable fashion
// as with macros
// NOLINTNEXTLINE(cppcoreguidelines-macro-usage)
#define IOX_TEST_FUNCTIONAL_INTERFACE(TestName, variationPoint)                                                        \
    using SutType = typename TestFixture::TestFactoryType::Type;                                                       \
    constexpr bool HAS_VALUE_METHOD = iox::internal::HasValueMethod<SutType>::value;                                   \
    /* NOLINTNEXTLINE(bugprone-macro-parentheses) prevents clang-tidy parsing failures */                              \
    TestName<HAS_VALUE_METHOD>::template performTest<typename TestFixture::TestFactoryType>(                           \
        [](auto& sut, auto callback) { (variationPoint).and_then(callback); })

constexpr bool TYPE_HAS_VALUE_METHOD = true;
constexpr bool TYPE_HAS_NO_VALUE_METHOD = false;

template <bool HasValue>
struct AndThenIsCalledCorrectlyWhenValid;

template <>
struct AndThenIsCalledCorrectlyWhenValid<TYPE_HAS_NO_VALUE_METHOD>
{
    template <typename TestFactory, typename AndThenCall>
    static void performTest(const AndThenCall& callAndThen)
    {
        auto sut = TestFactory::createValidObject();
        bool wasCallbackCalled = false;
        auto andThenCallbackArgument = [&] { wasCallbackCalled = true; };
        callAndThen(sut, andThenCallbackArgument);
        EXPECT_TRUE(wasCallbackCalled);
    }
};

template <>
struct AndThenIsCalledCorrectlyWhenValid<TYPE_HAS_VALUE_METHOD>
{
    template <typename TestFactory, typename AndThenCall>
    static void performTest(const AndThenCall& callAndThen)
    {
        auto sut = TestFactory::createValidObject();
        bool wasCallbackCalled = false;
        auto andThenCallbackArgument = [&](auto& arg) {
            wasCallbackCalled = true;
            EXPECT_EQ(arg, TestFactory::usedTestValue);
        };
        callAndThen(sut, andThenCallbackArgument);
        EXPECT_TRUE(wasCallbackCalled);
    }
};

TYPED_TEST(FunctionalInterface_test, AndThenIsCalledCorrectlyWhenValid_LValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "7f1b42e0-c834-47f4-96cc-c6466647615c");
    IOX_TEST_FUNCTIONAL_INTERFACE(AndThenIsCalledCorrectlyWhenValid, sut);
}

TYPED_TEST(FunctionalInterface_test, AndThenIsCalledCorrectlyWhenValid_ConstLValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "80724fcd-78a4-4f52-82fe-1613069823f0");
    // const_cast avoids code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    IOX_TEST_FUNCTIONAL_INTERFACE(AndThenIsCalledCorrectlyWhenValid, const_cast<const SutType&>(sut));
}

TYPED_TEST(FunctionalInterface_test, AndThenIsCalledCorrectlyWhenValid_RValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "ddf400a0-3231-49d3-b29f-338873038549");
    IOX_TEST_FUNCTIONAL_INTERFACE(AndThenIsCalledCorrectlyWhenValid, std::move(sut));
}

TYPED_TEST(FunctionalInterface_test, AndThenIsCalledCorrectlyWhenValid_ConstRValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "225f1e86-6b37-47db-9e1f-f44040040e8a");
    // const_cast avoids code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    IOX_TEST_FUNCTIONAL_INTERFACE(AndThenIsCalledCorrectlyWhenValid, std::move(const_cast<const SutType&>(sut)));
}

template <bool HasValue>
struct AndThenIsNotCalledWhenInvalid;

template <>
struct AndThenIsNotCalledWhenInvalid<TYPE_HAS_NO_VALUE_METHOD>
{
    template <typename TestFactory, typename AndThenCall>
    static void performTest(const AndThenCall& callAndThen)
    {
        auto sut = TestFactory::createInvalidObject();
        bool wasCallbackCalled = false;
        auto andThenCallbackArgument = [&] { wasCallbackCalled = true; };
        callAndThen(sut, andThenCallbackArgument);
        EXPECT_FALSE(wasCallbackCalled);
    }
};

template <>
struct AndThenIsNotCalledWhenInvalid<TYPE_HAS_VALUE_METHOD>
{
    template <typename TestFactory, typename AndThenCall>
    static void performTest(const AndThenCall& callAndThen)
    {
        auto sut = TestFactory::createInvalidObject();
        bool wasCallbackCalled = false;
        auto andThenCallbackArgument = [&](auto&) { wasCallbackCalled = true; };
        callAndThen(sut, andThenCallbackArgument);
        EXPECT_FALSE(wasCallbackCalled);
    }
};

TYPED_TEST(FunctionalInterface_test, AndThenIsNotCalledWhenInvalid_LValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "53eef31f-ab30-41c8-887e-edac3c51966c");
    IOX_TEST_FUNCTIONAL_INTERFACE(AndThenIsNotCalledWhenInvalid, sut);
}

TYPED_TEST(FunctionalInterface_test, AndThenIsNotCalledWhenInvalid_ConstLValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "1fcd75d8-ce17-49c3-8a0a-d676d649b985");
    // const_cast avoids code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    IOX_TEST_FUNCTIONAL_INTERFACE(AndThenIsNotCalledWhenInvalid, const_cast<const SutType&>(sut));
}

TYPED_TEST(FunctionalInterface_test, AndThenIsNotCalledWhenInvalid_RValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "ba6ed865-0bc1-451c-ace2-b09ce273e6ca");
    IOX_TEST_FUNCTIONAL_INTERFACE(AndThenIsNotCalledWhenInvalid, std::move(sut));
}

TYPED_TEST(FunctionalInterface_test, AndThenIsNotCalledWhenInvalid_ConstRValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "d4162bb7-c2b3-4c82-bb78-bc63acf4b3b9");
    // const_cast avoids code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    IOX_TEST_FUNCTIONAL_INTERFACE(AndThenIsNotCalledWhenInvalid, std::move(const_cast<const SutType&>(sut)));
}

#undef IOX_TEST_FUNCTIONAL_INTERFACE
} // namespace
