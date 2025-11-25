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

TYPED_TEST(FunctionalInterface_test, OrElseHasCorrectSignature)
{
    ::testing::Test::RecordProperty("TEST_ID", "ede81b23-cd69-45a4-86aa-b81baa8e281b");
    using Factory = typename TestFixture::TestFactoryType;
    constexpr bool DOES_OR_ELSE_HAVE_A_VALUE = iox::internal::HasGetErrorMethod<typename Factory::Type>::value;

    EXPECT_THAT(DOES_OR_ELSE_HAVE_A_VALUE, Eq(Factory::EXPECT_OR_ELSE_WITH_VALUE));
}

// the macro is used as code generator to make the tests more readable. because of the
// template nature of those tests this cannot be implemented in the same readable fashion
// as with macros
// NOLINTNEXTLINE(cppcoreguidelines-macro-usage)
#define IOX_TEST_FUNCTIONAL_INTERFACE(TestName, variationPoint)                                                        \
    using SutType = typename TestFixture::TestFactoryType::Type;                                                       \
    constexpr bool HAS_GET_ERROR_METHOD = iox::internal::HasGetErrorMethod<SutType>::value;                            \
    /* NOLINTNEXTLINE(bugprone-macro-parentheses) prevents clang-tidy parsing failures */                              \
    TestName<HAS_GET_ERROR_METHOD>::template performTest<typename TestFixture::TestFactoryType>(                       \
        [](auto& sut, auto callback) { (variationPoint).or_else(callback); })

constexpr bool TYPE_HAS_GET_ERROR_METHOD = true;
constexpr bool TYPE_HAS_NO_GET_ERROR_METHOD = false;

template <bool HasError>
struct OrElseIsCalledCorrectlyWhenInvalid;

template <>
struct OrElseIsCalledCorrectlyWhenInvalid<TYPE_HAS_NO_GET_ERROR_METHOD>
{
    template <typename TestFactory, typename OrElseCall>
    static void performTest(const OrElseCall& orElseCall)
    {
        auto sut = TestFactory::createInvalidObject();
        bool wasCallbackCalled = false;
        orElseCall(sut, [&] { wasCallbackCalled = true; });
        EXPECT_TRUE(wasCallbackCalled);
    }
};

template <>
struct OrElseIsCalledCorrectlyWhenInvalid<TYPE_HAS_GET_ERROR_METHOD>
{
    template <typename TestFactory, typename OrElseCall>
    static void performTest(const OrElseCall& orElseCall)
    {
        auto sut = TestFactory::createInvalidObject();
        bool wasCallbackCalled = false;
        orElseCall(sut, [&](auto& arg) {
            wasCallbackCalled = true;
            EXPECT_EQ(arg, TestFactory::usedErrorValue);
        });
        EXPECT_TRUE(wasCallbackCalled);
    }
};

TYPED_TEST(FunctionalInterface_test, OrElseIsCalledCorrectlyWhenInvalid_LValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "deddc99a-eec0-466f-a5ba-4018dd372c47");
    IOX_TEST_FUNCTIONAL_INTERFACE(OrElseIsCalledCorrectlyWhenInvalid, sut);
}

TYPED_TEST(FunctionalInterface_test, OrElseIsCalledCorrectlyWhenInvalid_ConstLValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "851ca90c-4433-4a6d-9a7b-08cdca78b3c4");
    // const_cast avoids code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    IOX_TEST_FUNCTIONAL_INTERFACE(OrElseIsCalledCorrectlyWhenInvalid, const_cast<const SutType&>(sut));
}

TYPED_TEST(FunctionalInterface_test, OrElseIsCalledCorrectlyWhenInvalid_RValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "eb4d7b93-201e-44d9-8482-af23a6ae854b");
    IOX_TEST_FUNCTIONAL_INTERFACE(OrElseIsCalledCorrectlyWhenInvalid, std::move(sut));
}

TYPED_TEST(FunctionalInterface_test, OrElseIsCalledCorrectlyWhenInvalid_ConstRValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "1c85d1bb-7934-43ad-b08e-87cafa5dce26");
    // const_cast avoids code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    IOX_TEST_FUNCTIONAL_INTERFACE(OrElseIsCalledCorrectlyWhenInvalid, std::move(const_cast<const SutType&>(sut)));
}

template <bool HasError>
struct OrElseIsNotCalledWhenValid;

template <>
struct OrElseIsNotCalledWhenValid<TYPE_HAS_NO_GET_ERROR_METHOD>
{
    template <typename TestFactory, typename OrElseCall>
    static void performTest(const OrElseCall& orElseCall)
    {
        auto sut = TestFactory::createValidObject();
        bool wasCallbackCalled = false;
        orElseCall(sut, [&] { wasCallbackCalled = true; });
        EXPECT_FALSE(wasCallbackCalled);
    }
};

template <>
struct OrElseIsNotCalledWhenValid<TYPE_HAS_GET_ERROR_METHOD>
{
    template <typename TestFactory, typename OrElseCall>
    static void performTest(const OrElseCall& orElseCall)
    {
        auto sut = TestFactory::createValidObject();
        bool wasCallbackCalled = false;
        orElseCall(sut, [&](auto&) { wasCallbackCalled = true; });
        EXPECT_FALSE(wasCallbackCalled);
    }
};

TYPED_TEST(FunctionalInterface_test, OrElseIsNotCalledWhenValid_LValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "d9dcf588-f15b-4065-8427-cbf7b6873038");
    IOX_TEST_FUNCTIONAL_INTERFACE(OrElseIsNotCalledWhenValid, sut);
}

TYPED_TEST(FunctionalInterface_test, OrElseIsNotCalledWhenValid_ConstLValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "4a061c42-eb93-4fc4-ad30-a117f8703659");
    // const_cast avoids code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    IOX_TEST_FUNCTIONAL_INTERFACE(OrElseIsNotCalledWhenValid, const_cast<const SutType&>(sut));
}

TYPED_TEST(FunctionalInterface_test, OrElseIsNotCalledWhenValid_RValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "2e371008-c74c-408b-ae4f-70329b487874");
    IOX_TEST_FUNCTIONAL_INTERFACE(OrElseIsNotCalledWhenValid, std::move(sut));
}

TYPED_TEST(FunctionalInterface_test, OrElseIsNotCalledWhenValid_ConstRValueCase)
{
    ::testing::Test::RecordProperty("TEST_ID", "6e58eee9-9c99-4ade-b144-d83821a25170");
    // const_cast avoids code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    IOX_TEST_FUNCTIONAL_INTERFACE(OrElseIsNotCalledWhenValid, std::move(const_cast<const SutType&>(sut)));
}

#undef IOX_TEST_FUNCTIONAL_INTERFACE
} // namespace
