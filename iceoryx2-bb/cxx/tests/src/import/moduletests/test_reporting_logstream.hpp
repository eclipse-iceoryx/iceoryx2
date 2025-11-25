// Copyright (c) 2019, 2021 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2023 by ekxide IO GmbH. All rights reserved.
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

#include "iceoryx_hoofs/testing/mocks/logger_mock.hpp"
#include "iceoryx_hoofs/testing/test.hpp"
#include "iox/log/logstream.hpp"
#include "iox/logging.hpp"

using namespace ::testing;

using iox::testing::Logger_Mock;

class LogStreamSut : public iox::log::LogStream
{
  public:
    explicit LogStreamSut(iox::log::Logger& logger)
        : iox::log::LogStream(logger, "file", 42, "function", iox::log::LogLevel::Trace)
    {
    }
};

class IoxLogStreamBase_test : public Test
{
  public:
    Logger_Mock loggerMock;
};
