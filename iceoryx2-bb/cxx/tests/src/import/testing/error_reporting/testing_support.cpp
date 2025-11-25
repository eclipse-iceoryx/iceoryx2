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

#include "iceoryx_hoofs/testing/error_reporting/testing_support.hpp"

namespace iox
{
namespace testing
{

bool hasPanicked()
{
    return ErrorHandler::instance().hasPanicked();
}

bool hasError()
{
    return ErrorHandler::instance().hasError();
}

bool hasAssertViolation()
{
    auto code = iox::er::Violation(iox::er::ViolationErrorCode::ASSERT_VIOLATION).code();
    return ErrorHandler::instance().hasViolation(code);
}

bool hasEnforceViolation()
{
    auto code = iox::er::Violation(iox::er::ViolationErrorCode::ENFORCE_VIOLATION).code();
    return ErrorHandler::instance().hasViolation(code);
}

bool hasViolation()
{
    return hasEnforceViolation() || hasAssertViolation();
}

bool isInNormalState()
{
    return !(hasPanicked() || hasError() || hasViolation());
}

void runInTestThread(const function_ref<void()> testFunction)
{
    auto t = std::thread([&]() {
        auto successfullRun = ErrorHandler::instance().fatalFailureTestContext(testFunction);
        if (!successfullRun)
        {
            GTEST_FAIL() << "This should not fail! Incorrect usage!";
        }
    });

    if (t.joinable())
    {
        t.join();
    }
}

} // namespace testing
} // namespace iox
