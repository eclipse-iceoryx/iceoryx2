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

#include "test.hpp"
#include <gtest/gtest.h>

#include "iox/error_reporting/custom/default/error_handler.hpp"
#include "iox/error_reporting/source_location.hpp"

namespace
{

using namespace ::testing;
using namespace iox::er;

constexpr ErrorCode CODE{73};

class DefaultErrorHandler_test : public Test
{
  public:
    void SetUp() override
    {
    }

    void TearDown() override
    {
    }

    DefaultErrorHandler sut;
};

// Can only check that it can be called, there are no observable effects.
TEST_F(DefaultErrorHandler_test, panicDoesNothing)
{
    ::testing::Test::RecordProperty("TEST_ID", "0d7f7048-94d3-42b7-a25a-1a7b506fd835");
    sut.onPanic();
}

// Can only check that it can be called, there are no observable effects.
TEST_F(DefaultErrorHandler_test, reportDoesNothing)
{
    ::testing::Test::RecordProperty("TEST_ID", "9e288318-c756-4666-b779-b944b89ffaf5");
    sut.onReportError(ErrorDescriptor{IOX_CURRENT_SOURCE_LOCATION, CODE});
    sut.onReportViolation(ErrorDescriptor{IOX_CURRENT_SOURCE_LOCATION, CODE});
}

} // namespace
