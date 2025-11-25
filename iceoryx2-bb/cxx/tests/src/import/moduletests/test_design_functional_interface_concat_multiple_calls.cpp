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

// the macro is used as code generator to make the tests more readable. because of the
// template nature of those tests this cannot be implemented in the same readable fashion
// as with macros
// NOLINTNEXTLINE(cppcoreguidelines-macro-usage)
#define IOX_TEST_FUNCTIONAL_INTERFACE(TestName, variationPoint)                                                        \
    using SutType = typename TestFixture::TestFactoryType::Type;                                                       \
    constexpr bool HAS_VALUE_METHOD = iox::internal::HasValueMethod<SutType>::value;                                   \
    constexpr bool HAS_GET_ERROR_METHOD = iox::internal::HasGetErrorMethod<SutType>::value;                            \
    /* NOLINTNEXTLINE(bugprone-macro-parentheses) prevents clang-tidy parsing failures */                              \
    TestName<HAS_VALUE_METHOD, HAS_GET_ERROR_METHOD>::template performTest<typename TestFixture::TestFactoryType>(     \
        [](auto& sut, auto andThenCallback, auto orElseCallback) {                                                     \
            (variationPoint).and_then(andThenCallback).or_else(orElseCallback);                                        \
        })

constexpr bool TYPE_HAS_VALUE_METHOD = true;
constexpr bool TYPE_HAS_NO_VALUE_METHOD = false;
constexpr bool TYPE_HAS_GET_ERROR_METHOD = true;
constexpr bool TYPE_HAS_NO_GET_ERROR_METHOD = false;

template <bool HasValue, bool HasError>
struct AndThenOrElseConcatenatedWorksWhenInvalid;

template <>
struct AndThenOrElseConcatenatedWorksWhenInvalid<TYPE_HAS_NO_VALUE_METHOD, TYPE_HAS_NO_GET_ERROR_METHOD>
{
    template <typename TestFactory, typename SutCall>
    static void performTest(const SutCall& sutCall)
    {
        auto sut = TestFactory::createInvalidObject();
        bool wasAndThenCalled = false;
        bool wasOrElseCalled = false;
        sutCall(sut, [&] { wasAndThenCalled = true; }, [&] { wasOrElseCalled = true; });

        EXPECT_FALSE(wasAndThenCalled);
        EXPECT_TRUE(wasOrElseCalled);
    }
};

template <>
struct AndThenOrElseConcatenatedWorksWhenInvalid<TYPE_HAS_VALUE_METHOD, TYPE_HAS_NO_GET_ERROR_METHOD>
{
    template <typename TestFactory, typename SutCall>
    static void performTest(const SutCall& sutCall)
    {
        auto sut = TestFactory::createInvalidObject();
        bool wasAndThenCalled = false;
        bool wasOrElseCalled = false;
        sutCall(sut, [&](auto&) { wasAndThenCalled = true; }, [&] { wasOrElseCalled = true; });

        EXPECT_FALSE(wasAndThenCalled);
        EXPECT_TRUE(wasOrElseCalled);
    }
};

template <>
struct AndThenOrElseConcatenatedWorksWhenInvalid<TYPE_HAS_NO_VALUE_METHOD, TYPE_HAS_GET_ERROR_METHOD>
{
    template <typename TestFactory, typename SutCall>
    static void performTest(const SutCall& sutCall)
    {
        auto sut = TestFactory::createInvalidObject();
        bool wasAndThenCalled = false;
        bool wasOrElseCalled = false;
        sutCall(
            sut,
            [&] { wasAndThenCalled = true; },
            [&](auto& error) {
                wasOrElseCalled = true;
                EXPECT_THAT(error, Eq(TestFactory::usedErrorValue));
            });

        EXPECT_FALSE(wasAndThenCalled);
        EXPECT_TRUE(wasOrElseCalled);
    }
};

template <>
struct AndThenOrElseConcatenatedWorksWhenInvalid<TYPE_HAS_VALUE_METHOD, TYPE_HAS_GET_ERROR_METHOD>
{
    template <typename TestFactory, typename SutCall>
    static void performTest(const SutCall& sutCall)
    {
        auto sut = TestFactory::createInvalidObject();
        bool wasAndThenCalled = false;
        bool wasOrElseCalled = false;

        sutCall(
            sut,
            [&](auto&) { wasAndThenCalled = true; },
            [&](auto& error) {
                wasOrElseCalled = true;
                EXPECT_THAT(error, Eq(TestFactory::usedErrorValue));
            });
        EXPECT_FALSE(wasAndThenCalled);
        EXPECT_TRUE(wasOrElseCalled);
    }
};

TYPED_TEST(FunctionalInterface_test, AndThenOrElseConcatenatedWorksWhenInvalid_LValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "e3deeadd-425a-48bb-a77b-89fcdfea0178");
    IOX_TEST_FUNCTIONAL_INTERFACE(AndThenOrElseConcatenatedWorksWhenInvalid, sut);
}

TYPED_TEST(FunctionalInterface_test, AndThenOrElseConcatenatedWorksWhenInvalid_ConstLValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "7810e0de-ac7f-4247-9adc-4177294bb60f");
    // const_cast avoids code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    IOX_TEST_FUNCTIONAL_INTERFACE(AndThenOrElseConcatenatedWorksWhenInvalid, const_cast<const SutType&>(sut));
}

TYPED_TEST(FunctionalInterface_test, AndThenOrElseConcatenatedWorksWhenInvalid_RValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "2e034af1-52af-48ac-b9b9-4c2d3e7cd60c");
    IOX_TEST_FUNCTIONAL_INTERFACE(AndThenOrElseConcatenatedWorksWhenInvalid, std::move(sut));
}

TYPED_TEST(FunctionalInterface_test, AndThenOrElseConcatenatedWorksWhenInvalid_ConstRValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "4ca6c6d0-fa72-45ff-a2ae-9b7a9574c450");
    IOX_TEST_FUNCTIONAL_INTERFACE(AndThenOrElseConcatenatedWorksWhenInvalid,
                                  // const_cast avoids code duplication
                                  // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
                                  std::move(const_cast<const SutType&>(sut)));
}

template <bool HasValue, bool HasError>
struct AndThenOrElseConcatenatedWorkWhenValid;

template <>
struct AndThenOrElseConcatenatedWorkWhenValid<TYPE_HAS_NO_VALUE_METHOD, TYPE_HAS_NO_GET_ERROR_METHOD>
{
    template <typename TestFactory, typename SutCall>
    static void performTest(const SutCall& sutCall)
    {
        auto sut = TestFactory::createValidObject();
        bool wasAndThenCalled = false;
        bool wasOrElseCalled = false;
        sutCall(sut, [&] { wasAndThenCalled = true; }, [&] { wasOrElseCalled = true; });

        EXPECT_TRUE(wasAndThenCalled);
        EXPECT_FALSE(wasOrElseCalled);
    }
};

template <>
struct AndThenOrElseConcatenatedWorkWhenValid<TYPE_HAS_VALUE_METHOD, TYPE_HAS_NO_GET_ERROR_METHOD>
{
    template <typename TestFactory, typename SutCall>
    static void performTest(const SutCall& sutCall)
    {
        auto sut = TestFactory::createValidObject();
        bool wasAndThenCalled = false;
        bool wasOrElseCalled = false;
        sutCall(
            sut,
            [&](auto& value) {
                wasAndThenCalled = true;
                EXPECT_THAT(value, Eq(TestFactory::usedTestValue));
            },
            [&] { wasOrElseCalled = true; });

        EXPECT_TRUE(wasAndThenCalled);
        EXPECT_FALSE(wasOrElseCalled);
    }
};

template <>
struct AndThenOrElseConcatenatedWorkWhenValid<TYPE_HAS_NO_VALUE_METHOD, TYPE_HAS_GET_ERROR_METHOD>
{
    template <typename TestFactory, typename SutCall>
    static void performTest(const SutCall& sutCall)
    {
        auto sut = TestFactory::createValidObject();
        bool wasAndThenCalled = false;
        bool wasOrElseCalled = false;
        sutCall(sut, [&] { wasAndThenCalled = true; }, [&](auto&) { wasOrElseCalled = true; });

        EXPECT_TRUE(wasAndThenCalled);
        EXPECT_FALSE(wasOrElseCalled);
    }
};

template <>
struct AndThenOrElseConcatenatedWorkWhenValid<TYPE_HAS_VALUE_METHOD, TYPE_HAS_GET_ERROR_METHOD>
{
    template <typename TestFactory, typename SutCall>
    static void performTest(const SutCall& sutCall)
    {
        auto sut = TestFactory::createValidObject();
        bool wasAndThenCalled = false;
        bool wasOrElseCalled = false;

        sutCall(
            sut,
            [&](auto& value) {
                wasAndThenCalled = true;
                EXPECT_THAT(value, Eq(TestFactory::usedTestValue));
            },
            [&](auto&) { wasOrElseCalled = true; });
        EXPECT_TRUE(wasAndThenCalled);
        EXPECT_FALSE(wasOrElseCalled);
    }
};

TYPED_TEST(FunctionalInterface_test, AndThenOrElseConcatenatedWorkWhenValid_LValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "99af4d72-8e30-4f63-97a2-92fdd861c615");
    IOX_TEST_FUNCTIONAL_INTERFACE(AndThenOrElseConcatenatedWorkWhenValid, sut);
}

TYPED_TEST(FunctionalInterface_test, AndThenOrElseConcatenatedWorkWhenValid_ConstLValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "d70fd26a-f8bb-4976-b119-409651301e1b");
    // const_cast avoids code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    IOX_TEST_FUNCTIONAL_INTERFACE(AndThenOrElseConcatenatedWorkWhenValid, const_cast<SutType&>(sut));
}

TYPED_TEST(FunctionalInterface_test, AndThenOrElseConcatenatedWorkWhenValid_RValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "469b0b63-c06d-44b9-8dae-2bb6629d49ba");
    IOX_TEST_FUNCTIONAL_INTERFACE(AndThenOrElseConcatenatedWorkWhenValid, std::move(sut));
}

TYPED_TEST(FunctionalInterface_test, AndThenOrElseConcatenatedWorkWhenValid_ConstRValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "96b3f8d5-07e0-407e-8e7e-5c7ae258a623");
    // const_cast avoids code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    IOX_TEST_FUNCTIONAL_INTERFACE(AndThenOrElseConcatenatedWorkWhenValid, std::move(const_cast<SutType&>(sut)));
}
} // namespace
