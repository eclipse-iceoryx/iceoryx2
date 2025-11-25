// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2024 by ekxide IO GmbH. All rights reserved.
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

#ifndef IOX_HOOFS_TESTING_ERROR_REPORTING_TESTING_ERROR_HANDLER_HPP
#define IOX_HOOFS_TESTING_ERROR_REPORTING_TESTING_ERROR_HANDLER_HPP

#include "iox/atomic.hpp"
#include "iox/error_reporting/custom/default/error_handler_interface.hpp"
#include "iox/error_reporting/error_logging.hpp"
#include "iox/error_reporting/source_location.hpp"
#include "iox/error_reporting/types.hpp"
#include "iox/error_reporting/violation.hpp"
#include "iox/function_ref.hpp"
#include "iox/static_lifetime_guard.hpp"

#include "iceoryx_hoofs/testing/test.hpp"

#include <vector>

// we can use this for test code
#include <mutex>

// NOLINTNEXTLINE(hicpp-deprecated-headers) required to work on some platforms
#include <setjmp.h>

namespace iox
{
namespace testing
{

/// @brief Defines the test reaction of dynamic error handling.
class TestingErrorHandler : public iox::er::ErrorHandlerInterface
{
  public:
    TestingErrorHandler() noexcept = default;
    ~TestingErrorHandler() noexcept override = default;
    TestingErrorHandler(const TestingErrorHandler&) noexcept = delete;
    TestingErrorHandler(TestingErrorHandler&&) noexcept = delete;
    TestingErrorHandler& operator=(const TestingErrorHandler&) noexcept = delete;
    TestingErrorHandler operator=(TestingErrorHandler&&) noexcept = delete;

    /// @brief Initialized the error handler. This should be called in the main function of the test binary
    /// @code
    /// #include "iceoryx_hoofs/testing/error_reporting/testing_error_handler.hpp"
    ///
    /// #include "test.hpp"
    ///
    /// int main(int argc, char* argv[])
    /// {
    ///     ::testing::InitGoogleTest(&argc, argv);
    ///
    ///     iox::testing::ErrorHandler::init();
    ///
    ///     return RUN_ALL_TESTS();
    /// }
    /// @endcode
    static void init() noexcept;

    /// @brief Defines the reaction on panic.
    void onPanic() override;

    /// @brief Defines the reaction on error.
    /// @param desc error descriptor
    void onReportError(er::ErrorDescriptor desc) override;

    /// @brief Defines the reaction on violation.
    /// @param desc error descriptor
    void onReportViolation(er::ErrorDescriptor desc) override;

    /// @brief Indicates whether there was a panic call previously.
    /// @return true if there was a panic call, false otherwise
    bool hasPanicked() const noexcept;

    /// @brief Reset panic state and clears all errors that occurred previously.
    void reset() noexcept;

    /// @brief Indicates whether any error occurred previously.
    bool hasError() const noexcept;

    /// @brief Indicates whether a specific error occurred previously.
    bool hasError(iox::er::ErrorCode code, iox::er::ModuleId module = iox::er::ModuleId()) const noexcept;

    /// @brief Indicates whether a assumption violation occurred previously.
    /// @note We do not track module id for violations.
    bool hasViolation(iox::er::ErrorCode code) const noexcept;

    /// @brief runs testFunction in a test context that can detect fatal failures;
    /// runs in the same thread
    /// @note uses setjmp/longjmp
    bool fatalFailureTestContext(const function_ref<void()> testFunction);

  private:
    void jump() noexcept;

  private:
    static constexpr int JUMPED_INDICATOR{1};

    mutable std::mutex m_mutex;
    iox::concurrent::Atomic<bool> m_panicked{false};
    std::vector<er::ErrorDescriptor> m_errors;

    // we track violations separately (leads to simple search)
    std::vector<er::ErrorDescriptor> m_violations;

    // if we would like to support concurrent jumps it gets very tricky
    // and we would need multiple jump buffers
    jmp_buf m_jumpBuffer{};

    enum class JumpState : uint8_t
    {
        Obtainable,
        Pending,
    };
    // Actually not needed to be atomic since it is not supposed to be used from multiple threads
    // (longjmp does not support this)
    // We need to ensure though that only one jump buffer is considered by panic and controlling
    // ownership of the buffer is one way to accomplish that.
    iox::concurrent::Atomic<JumpState> m_jumpState{JumpState::Obtainable};
};

/// @brief This class hooks into gTest to automatically resets the error handler on the start of a test
class ErrorHandlerSetup : public ::testing::EmptyTestEventListener
{
    void OnTestStart(const ::testing::TestInfo&) override;
};

using ErrorHandler = iox::StaticLifetimeGuard<iox::testing::TestingErrorHandler>;

} // namespace testing
} // namespace iox

#endif
