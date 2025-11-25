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

#ifndef IOX_HOOFS_TESTING_FATAL_FAILURE_HPP
#define IOX_HOOFS_TESTING_FATAL_FAILURE_HPP

#include "iox/attributes.hpp"
#include "iox/error_reporting/error_kind.hpp"
#include "iox/error_reporting/types.hpp"
#include "iox/function_ref.hpp"
#include "iox/logging.hpp"

#include "iceoryx_hoofs/testing/error_reporting/testing_support.hpp"
#include "test.hpp"

#include <thread>

// NOLINTNEXTLINE(hicpp-deprecated-headers) required to work on some platforms
#include <setjmp.h>

namespace iox
{
namespace testing
{
/// @brief This function is used in cases a fatal failure is expected. The function only works in combination with the
/// iceoryx error handler.
/// @code
/// TEST(MyTest, valueOnNulloptIsFatal) {
///     iox::optional<bool> sut;
///     IOX_EXPECT_FATAL_FAILURE([&] { sut.value(); }, iox::er::ENFORCE_VIOLATION));
/// }
/// @endcode
/// @tparam[in] ErrorType The error type which is expected, e.g. 'iox::HoofsError'
/// @param[in] testFunction This function will be executed as SUT and is expected to call the error handler
/// @param[in] expectedError The error value which triggered the fatal failure
/// @return true if a fatal failure occurs, false otherwise
template <typename ErrorType,
          std::enable_if_t<!std::is_same<ErrorType, iox::er::FatalKind>::value
                               && !std::is_same<ErrorType, iox::er::EnforceViolationKind>::value
                               && !std::is_same<ErrorType, iox::er::AssertViolationKind>::value,
                           bool> = true>
bool IOX_EXPECT_FATAL_FAILURE(const function_ref<void()> testFunction, const ErrorType expectedError);

template <typename ErrorType, std::enable_if_t<std::is_same<ErrorType, iox::er::FatalKind>::value, bool> = true>
bool IOX_EXPECT_FATAL_FAILURE(const function_ref<void()> testFunction, const ErrorType expectedError);

template <typename ErrorType,
          std::enable_if_t<std::is_same<ErrorType, iox::er::EnforceViolationKind>::value, bool> = true>
bool IOX_EXPECT_FATAL_FAILURE(const function_ref<void()> testFunction, const ErrorType expectedError);

template <typename ErrorType,
          std::enable_if_t<std::is_same<ErrorType, iox::er::AssertViolationKind>::value, bool> = true>
bool IOX_EXPECT_FATAL_FAILURE(const function_ref<void()> testFunction, const ErrorType expectedError);

/// @brief This function is used in cases no fatal failure is expected but could potentially occur. The function only
/// works in combination with the iceoryx error handler.
/// @code
/// TEST(MyTest, valueIsNotFatal) {
///     iox::optional<bool> sut{false};
///     IOX_EXPECT_NO_FATAL_FAILURE([&] { sut.value(); });
/// }
/// @endcode
/// @param[in] testFunction This function will be executed as SUT and is not expected to call the error handler
/// @return true if no fatal failure occurs, false otherwise
bool IOX_EXPECT_NO_FATAL_FAILURE(const function_ref<void()> testFunction);

} // namespace testing
} // namespace iox

#include "iceoryx_hoofs/testing/fatal_failure.inl"

#endif // IOX_HOOFS_TESTING_FATAL_FAILURE_HPP
