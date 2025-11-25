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

#ifndef IOX_HOOFS_MODULETESTS_TEST_DESIGN_FUNCTIONAL_INTERFACE_TYPES_HPP
#define IOX_HOOFS_MODULETESTS_TEST_DESIGN_FUNCTIONAL_INTERFACE_TYPES_HPP

#include "iox/expected.hpp"
#include "iox/optional.hpp"
#include "test_design_functional_interface_common.hpp"

namespace test_design_functional_interface
{
struct GenericValueErrorFactory
{
    using Type = GenericValueError;

    using value_t = typename Type::value_t;
    using error_t = typename Type::error_t;

    static constexpr bool EXPECT_AND_THEN_WITH_VALUE = true;
    static constexpr bool EXPECT_OR_ELSE_WITH_VALUE = true;

    static value_t usedTestValue;
    static value_t anotherTestValue;

    static error_t usedErrorValue;
    static error_t anotherErrorValue;

    static void configureNextTestCase() noexcept;
    static Type createValidObject() noexcept;
    static Type createInvalidObject() noexcept;
};

struct GenericPlainFactory
{
    using Type = GenericPlain;

    static constexpr bool EXPECT_AND_THEN_WITH_VALUE = false;
    static constexpr bool EXPECT_OR_ELSE_WITH_VALUE = false;

    static void configureNextTestCase() noexcept;
    static Type createValidObject() noexcept;
    static Type createInvalidObject() noexcept;
};

struct OptionalFactory
{
    using value_t = uint64_t;
    using Type = iox::optional<value_t>;

    static constexpr bool EXPECT_AND_THEN_WITH_VALUE = true;
    static constexpr bool EXPECT_OR_ELSE_WITH_VALUE = false;

    static value_t usedTestValue;
    static value_t anotherTestValue;

    static void configureNextTestCase() noexcept;
    static Type createValidObject() noexcept;
    static Type createInvalidObject() noexcept;
};

struct ExpectedValueErrorFactory
{
    using value_t = uint64_t;
    using error_t = uint64_t;

    using Type = iox::expected<value_t, error_t>;

    static constexpr bool EXPECT_AND_THEN_WITH_VALUE = true;
    static constexpr bool EXPECT_OR_ELSE_WITH_VALUE = true;

    static value_t usedTestValue;
    static value_t anotherTestValue;

    static error_t usedErrorValue;
    static error_t anotherErrorValue;

    static void configureNextTestCase() noexcept;
    static Type createValidObject() noexcept;
    static Type createInvalidObject() noexcept;
};

struct ExpectedErrorFactory
{
    using error_t = uint64_t;

    using Type = iox::expected<void, error_t>;

    static constexpr bool EXPECT_AND_THEN_WITH_VALUE = false;
    static constexpr bool EXPECT_OR_ELSE_WITH_VALUE = true;

    static error_t usedErrorValue;
    static error_t anotherErrorValue;

    static void configureNextTestCase() noexcept;
    static Type createValidObject() noexcept;
    static Type createInvalidObject() noexcept;
};

/// @brief Add here a type which inherits from FunctionalInterface and should
///        be tested. Please consider GenericValueErrorFactory and GenericPlainFactory
///        as a template.
///
///     Nullable class:
///        If the class is just nullable but does not contain a value or an error
///        then you have to create a struct like GenericPlainFactory with the methods
///        * using Type =;
///            type alias for the type which will be tested
///
///        * static void configureNextTestCase();
///            called before every test case, can be useful to bring some randomisation into the
///            createValidObject/createInvalidObject process.
///
///        * static Type createValidObject();
///            creates a valid object (the operator bool has to return true, required for and_then case)
///
///        * static Type createInvalidObject();
///            creates an invalid object (the operator bool has to return false, required for or_else case)
///
///     Class with value:
///        A class with a value method requires additionally:
///        * using value_t = ;
///            Type alias of the value type
///
///        * static value_t usedTestValue;
///            Value which was used while creating a valid object
///
///        * static value_t anotherTestValue;
///            Another value which can be compared to usedTestValue and is not equal to it
///
///     Class with error:
///        A class with a error method requires additionally:
///        * using error_t = ;
///            Type alias of the error type
///
///        * static error_t usedErrorValue;
///            Error value which was used while creating an invalid object
///
///        * static error_t anotherErrorValue
///            Another error value which can be compared to usedErrorValue and is not equal to it

using FunctionalInterfaceImplementations = testing::Types<GenericValueErrorFactory,
                                                          GenericPlainFactory,
                                                          OptionalFactory,
                                                          ExpectedValueErrorFactory,
                                                          ExpectedErrorFactory>;

TYPED_TEST_SUITE(FunctionalInterface_test, FunctionalInterfaceImplementations, );

} // namespace test_design_functional_interface

#endif
