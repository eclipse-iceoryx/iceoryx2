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

namespace test_design_functional_interface
{
//////////////////////////////////
/// BEGIN GenericValueErrorFactory
//////////////////////////////////
GenericValueErrorFactory::value_t GenericValueErrorFactory::usedTestValue = 1;
GenericValueErrorFactory::value_t GenericValueErrorFactory::anotherTestValue = 2;
GenericValueErrorFactory::error_t GenericValueErrorFactory::usedErrorValue = 3;
GenericValueErrorFactory::error_t GenericValueErrorFactory::anotherErrorValue = 4;

void GenericValueErrorFactory::configureNextTestCase() noexcept
{
    // we increment all the values with an arbitrary value (23) int every test case
    // so that we have some variation with every test and reduce the false positives
    // probability due to memory corruptions
    constexpr value_t NEXT_CASE_INCREMENT_VALUE = 23;
    usedTestValue += NEXT_CASE_INCREMENT_VALUE;
    anotherTestValue += NEXT_CASE_INCREMENT_VALUE;
    usedErrorValue += NEXT_CASE_INCREMENT_VALUE;
    anotherErrorValue += NEXT_CASE_INCREMENT_VALUE;
}

GenericValueErrorFactory::Type GenericValueErrorFactory::createValidObject() noexcept
{
    return GenericValueError(usedTestValue, usedErrorValue);
}

GenericValueErrorFactory::Type GenericValueErrorFactory::createInvalidObject() noexcept
{
    return GenericValueError(GenericValueError::INVALID_VALUE, usedErrorValue);
}
//////////////////////////////////
/// END GenericValueErrorFactory
//////////////////////////////////

/////////////////////////////
/// BEGIN GenericPlainFactory
/////////////////////////////
void GenericPlainFactory::configureNextTestCase() noexcept
{
}

GenericPlain GenericPlainFactory::createValidObject() noexcept
{
    return GenericPlain(GenericPlain::VALID_VALUE, GenericPlain::VALID_VALUE);
}

GenericPlain GenericPlainFactory::createInvalidObject() noexcept
{
    return GenericPlain(GenericPlain::INVALID_VALUE, GenericPlain::INVALID_VALUE);
}
///////////////////////////
/// END GenericPlainFactory
///////////////////////////

/////////////////////////
/// BEGIN OptionalFactory
/////////////////////////
OptionalFactory::value_t OptionalFactory::usedTestValue = 56;
OptionalFactory::value_t OptionalFactory::anotherTestValue = 1174;

void OptionalFactory::configureNextTestCase() noexcept
{
    // we increment all the values with an arbitrary value every test case
    // so that we have some variation with every test and reduce the false positives
    // probability due to memory corruptions
    constexpr value_t NEXT_CASE_VALUE_INCREMENT = 67;
    constexpr value_t NEXT_CASE_VALUE_2_INCREMENT = 69;
    usedTestValue += NEXT_CASE_VALUE_INCREMENT;
    anotherTestValue += NEXT_CASE_VALUE_2_INCREMENT;
}

OptionalFactory::Type OptionalFactory::createValidObject() noexcept
{
    return OptionalFactory::Type(iox::in_place, usedTestValue);
}

OptionalFactory::Type OptionalFactory::createInvalidObject() noexcept
{
    return iox::nullopt;
}
///////////////////////
/// END OptionalFactory
///////////////////////

//////////////////////////////////
/// BEGIN ExpectedValueErrorFactory
//////////////////////////////////
ExpectedValueErrorFactory::value_t ExpectedValueErrorFactory::usedTestValue = 11;
ExpectedValueErrorFactory::value_t ExpectedValueErrorFactory::anotherTestValue = 12;
ExpectedValueErrorFactory::error_t ExpectedValueErrorFactory::usedErrorValue = 13;
ExpectedValueErrorFactory::error_t ExpectedValueErrorFactory::anotherErrorValue = 14;

void ExpectedValueErrorFactory::configureNextTestCase() noexcept
{
    // we increment all the values with an static arbitrary value in every test case
    // so that we have some variation with every test and reduce the false positives
    // probability due to memory corruptions
    constexpr value_t NEXT_CASE_VALUE_INCREMENT = 189;
    constexpr error_t NEXT_CASE_ERROR_INCREMENT = 191;
    usedTestValue += NEXT_CASE_VALUE_INCREMENT;
    anotherTestValue += NEXT_CASE_VALUE_INCREMENT;
    usedErrorValue += NEXT_CASE_ERROR_INCREMENT;
    anotherErrorValue += NEXT_CASE_ERROR_INCREMENT;
}

ExpectedValueErrorFactory::Type ExpectedValueErrorFactory::createValidObject() noexcept
{
    return iox::ok(usedTestValue);
}

ExpectedValueErrorFactory::Type ExpectedValueErrorFactory::createInvalidObject() noexcept
{
    return iox::err(usedErrorValue);
}
//////////////////////////////////
/// END ExpectedValueErrorFactory
//////////////////////////////////

//////////////////////////////
/// BEGIN ExpectedErrorFactory
//////////////////////////////
ExpectedErrorFactory::error_t ExpectedErrorFactory::usedErrorValue = 13;
ExpectedErrorFactory::error_t ExpectedErrorFactory::anotherErrorValue = 14;

void ExpectedErrorFactory::configureNextTestCase() noexcept
{
    // we increment all the values with an static arbitrary value in every test case
    // so that we have some variation with every test and reduce the false positives
    // probability due to memory corruptions
    constexpr error_t NEXT_CASE_INCREMENT_VALUE = 191;
    usedErrorValue += NEXT_CASE_INCREMENT_VALUE;
    anotherErrorValue += NEXT_CASE_INCREMENT_VALUE;
}

ExpectedErrorFactory::Type ExpectedErrorFactory::createValidObject() noexcept
{
    return iox::ok();
}

ExpectedErrorFactory::Type ExpectedErrorFactory::createInvalidObject() noexcept
{
    return iox::err(usedErrorValue);
}
////////////////////////////
/// END ExpectedErrorFactory
////////////////////////////


} // namespace test_design_functional_interface
