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

#ifndef IOX_HOOFS_MODULETESTS_TEST_DESIGN_FUNCTIONAL_INTERFACE_COMMON_HPP
#define IOX_HOOFS_MODULETESTS_TEST_DESIGN_FUNCTIONAL_INTERFACE_COMMON_HPP

#include "iox/functional_interface.hpp"
#include "test.hpp"

namespace test_design_functional_interface
{
/// @brief Every test file uses this as a common base and extends the TYPED_TEST
///        for a specific part of the functional interface.
///
///        The idea is to have a test setup which is so generic that a user
///        which would like to enrich its class with the functional interface
///        has to write only a test factory in 'test_design_functional_interface_types.hpp'
///        for its specific type and then can add its type to the typelist and
///        the tests are generated for them.
template <typename T>
class FunctionalInterface_test : public testing::Test
{
  public:
    using TestFactoryType = T;

    void SetUp() override
    {
        // Whenever we start the next test case we inform the factory of our
        // test types that we started a new test.
        // This enables the factory to vary the underlying value/error values
        // in each test
        TestFactoryType::configureNextTestCase();
    }

    void TearDown() override
    {
    }
};

/// @brief This types is used for testing the functional interface in the case
///        of a 'value' and a 'error' method
struct GenericValueError : public iox::FunctionalInterface<GenericValueError, int, int>
{
    using value_t = int;
    using error_t = int;

    static constexpr value_t VALID_VALUE = 5;
    static constexpr value_t INVALID_VALUE = 0;

    GenericValueError(const value_t value, const error_t error) noexcept;

    explicit operator bool() const noexcept;

    value_t& value() & noexcept;
    const value_t& value() const& noexcept;
    value_t&& value() && noexcept;
    const value_t&& value() const&& noexcept;

    error_t& error() & noexcept;
    const error_t& error() const& noexcept;
    error_t&& error() && noexcept;
    const error_t&& error() const&& noexcept;

    value_t m_value = 0;
    error_t m_error = 0;
};

/// @brief This types is used for testing the functional interface in the case
///        that it is only nullable
struct GenericPlain : public iox::FunctionalInterface<GenericPlain, void, void>
{
    static constexpr int VALID_VALUE = 5;
    static constexpr int INVALID_VALUE = 0;

    explicit GenericPlain(const int value, const int);
    explicit operator bool() const noexcept;

    bool m_isValid = false;
};


} // namespace test_design_functional_interface

#endif
