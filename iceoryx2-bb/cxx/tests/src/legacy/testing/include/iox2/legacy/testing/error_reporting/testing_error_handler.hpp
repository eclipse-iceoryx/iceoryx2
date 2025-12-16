// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2024 by ekxide IO GmbH. All rights reserved.
// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#ifndef IOX2_BB_TESTING_ERROR_REPORTING_TESTING_ERROR_HANDLER_HPP
#define IOX2_BB_TESTING_ERROR_REPORTING_TESTING_ERROR_HANDLER_HPP

#include "iox2/bb/static_function.hpp"
#include "iox2/legacy/atomic.hpp"
#include "iox2/legacy/error_reporting/custom/default/error_handler_interface.hpp"
#include "iox2/legacy/error_reporting/error_logging.hpp"
#include "iox2/legacy/error_reporting/source_location.hpp"
#include "iox2/legacy/error_reporting/types.hpp"
#include "iox2/legacy/error_reporting/violation.hpp"
#include "iox2/legacy/static_lifetime_guard.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

#include <vector>

// we can use this for test code
#include <mutex>

// NOLINTNEXTLINE(hicpp-deprecated-headers) required to work on some platforms
#include <setjmp.h>

namespace iox2 {
namespace legacy {
namespace testing {

/// @brief Defines the test reaction of dynamic error handling.
class TestingErrorHandler : public iox2::legacy::er::ErrorHandlerInterface {
  public:
    TestingErrorHandler() noexcept = default;
    ~TestingErrorHandler() noexcept override = default;
    TestingErrorHandler(const TestingErrorHandler&) noexcept = delete;
    TestingErrorHandler(TestingErrorHandler&&) noexcept = delete;
    TestingErrorHandler& operator=(const TestingErrorHandler&) noexcept = delete;
    TestingErrorHandler operator=(TestingErrorHandler&&) noexcept = delete;

    /// @brief Initialized the error handler. This should be called in the main function of the test binary
    /// @code
    /// #include "iox2/legacy/testing/error_reporting/testing_error_handler.hpp"
    ///
    /// #include "test.hpp"
    ///
    /// int main(int argc, char* argv[])
    /// {
    ///     ::testing::InitGoogleTest(&argc, argv);
    ///
    ///     iox2::legacy::testing::ErrorHandler::init();
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
    bool hasError(iox2::legacy::er::ErrorCode code,
                  iox2::legacy::er::ModuleId module = iox2::legacy::er::ModuleId()) const noexcept;

    /// @brief Indicates whether a assumption violation occurred previously.
    /// @note We do not track module id for violations.
    bool hasViolation(iox2::legacy::er::ErrorCode code) const noexcept;

    /// @brief runs testFunction in a test context that can detect fatal failures;
    /// runs in the same thread
    /// @note uses setjmp/longjmp
    bool fatalFailureTestContext(const bb::StaticFunction<void()> testFunction);

  private:
    void jump() noexcept;

  private:
    static constexpr int JUMPED_INDICATOR { 1 };

    mutable std::mutex m_mutex;
    iox2::legacy::concurrent::Atomic<bool> m_panicked { false };
    std::vector<er::ErrorDescriptor> m_errors;

    // we track violations separately (leads to simple search)
    std::vector<er::ErrorDescriptor> m_violations;

    // if we would like to support concurrent jumps it gets very tricky
    // and we would need multiple jump buffers
    jmp_buf m_jumpBuffer {};

    enum class JumpState : uint8_t {
        Obtainable,
        Pending,
    };
    // Actually not needed to be atomic since it is not supposed to be used from multiple threads
    // (longjmp does not support this)
    // We need to ensure though that only one jump buffer is considered by panic and controlling
    // ownership of the buffer is one way to accomplish that.
    iox2::legacy::concurrent::Atomic<JumpState> m_jumpState { JumpState::Obtainable };
};

/// @brief This class hooks into gTest to automatically resets the error handler on the start of a test
class ErrorHandlerSetup : public ::testing::EmptyTestEventListener {
    void OnTestStart(const ::testing::TestInfo&) override;
};

using ErrorHandler = iox2::legacy::StaticLifetimeGuard<iox2::legacy::testing::TestingErrorHandler>;

} // namespace testing
} // namespace legacy
} // namespace iox2

#endif
