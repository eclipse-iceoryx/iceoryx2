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

constexpr bool TYPE_HAS_VALUE_METHOD = true;
constexpr bool TYPE_HAS_NO_VALUE_METHOD = false;

// the macro is used as code generator to make the tests more readable. because of the
// template nature of those tests this cannot be implemented in the same readable fashion
// as with macros
// NOLINTNEXTLINE(cppcoreguidelines-macro-usage)
#define IOX_TEST_FUNCTIONAL_INTERFACE(variationPoint)                                                                  \
    using SutType = typename TestFixture::TestFactoryType::Type;                                                       \
    constexpr bool HAS_VALUE_METHOD = iox::internal::HasValueMethod<SutType>::value;                                   \
    ValueOrReturnsValueWhenValid<HAS_VALUE_METHOD>::template performTest<typename TestFixture::TestFactoryType>(       \
        [](auto& sut, auto alternativeValue) { return (variationPoint).value_or(alternativeValue); });


template <bool HasValue>
struct ValueOrReturnsValueWhenValid;

template <>
struct ValueOrReturnsValueWhenValid<TYPE_HAS_NO_VALUE_METHOD>
{
    template <typename TestFactory, typename ValueOrCall>
    static void performTest(const ValueOrCall& callValueOr IOX_MAYBE_UNUSED)
    {
    }
};

template <>
struct ValueOrReturnsValueWhenValid<TYPE_HAS_VALUE_METHOD>
{
    template <typename TestFactory, typename ValueOrCall>
    static void performTest(const ValueOrCall& callValueOr)
    {
        auto sut = TestFactory::createValidObject();
        EXPECT_THAT(callValueOr(sut, TestFactory::anotherTestValue), Eq(TestFactory::usedTestValue));
    }
};

TYPED_TEST(FunctionalInterface_test, ValueOrReturnsValueWhenValid_LValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "88a8f419-6df9-4d8c-9e60-039100d67efa");
    IOX_TEST_FUNCTIONAL_INTERFACE(sut);
}

TYPED_TEST(FunctionalInterface_test, ValueOrReturnsValueWhenValid_RValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "2783061c-e746-4413-88e9-6b10065dd06a");
    IOX_TEST_FUNCTIONAL_INTERFACE(std::move(sut));
}

template <bool HasValue>
struct ValueOrReturnsArgumentWhenInalid;

template <>
struct ValueOrReturnsArgumentWhenInalid<TYPE_HAS_NO_VALUE_METHOD>
{
    template <typename TestFactory, typename ValueOrCall>
    static void performTest(const ValueOrCall& callValueOr IOX_MAYBE_UNUSED)
    {
    }
};

template <>
struct ValueOrReturnsArgumentWhenInalid<TYPE_HAS_VALUE_METHOD>
{
    template <typename TestFactory, typename ValueOrCall>
    static void performTest(const ValueOrCall& callValueOr)
    {
        auto sut = TestFactory::createInvalidObject();
        EXPECT_THAT(callValueOr(sut, TestFactory::anotherTestValue), Eq(TestFactory::anotherTestValue));
    }
};

TYPED_TEST(FunctionalInterface_test, ValueOrReturnsArgumentWhenInalid_LValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "b1398860-a440-4857-9a25-7e5bb9dc2fc9");
    IOX_TEST_FUNCTIONAL_INTERFACE(sut);
}

TYPED_TEST(FunctionalInterface_test, ValueOrReturnsArgumentWhenInalid_RValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "f85dcc3d-684d-4b32-9f1d-e7ac5ee45c0f");
    IOX_TEST_FUNCTIONAL_INTERFACE(std::move(sut));
}

#undef IOX_TEST_FUNCTIONAL_INTERFACE
} // namespace
