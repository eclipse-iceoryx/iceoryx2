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

#include "iox2/bb/detail/attributes.hpp"
#include "iox2/bb/static_function.hpp"
#include "iox2/legacy/atomic.hpp"
#include "iox2/legacy/error_reporting/custom/default/error_handler_interface.hpp"
#include "iox2/legacy/error_reporting/types.hpp"
#include "iox2/legacy/static_lifetime_guard.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

#include <vector>

// we can use this for test code
#include <mutex>

// NOLINTNEXTLINE(hicpp-deprecated-headers, modernize-deprecated-headers) required to work on some platforms
#include <setjmp.h>

namespace iox2 {
namespace bb {
namespace testing {

/// @brief Defines the test reaction of dynamic error handling.
class TestingErrorHandler : public legacy::er::ErrorHandlerInterface {
  public:
    TestingErrorHandler() noexcept = default;
    ~TestingErrorHandler() noexcept override = default;
    TestingErrorHandler(const TestingErrorHandler&) noexcept = delete;
    TestingErrorHandler(TestingErrorHandler&&) noexcept = delete;
    auto operator=(const TestingErrorHandler&) noexcept -> TestingErrorHandler& = delete;
    auto operator=(TestingErrorHandler&&) noexcept -> TestingErrorHandler = delete;

    /// @brief Initialized the error handler. This should be called in the main function of the test binary
    /// @code
    /// #include "iox2/bb/testing/testing_error_handler.hpp"
    ///
    /// #include "test.hpp"
    ///
    /// int main(int argc, char* argv[]) {
    ///     ::testing::InitGoogleTest(&argc, argv);
    ///
    ///     bb::testing::ErrorHandler::init();
    ///
    ///     return RUN_ALL_TESTS();
    /// }
    /// @endcode
    static auto init() noexcept -> void;

    /// @brief Defines the reaction on panic.
    auto on_panic() -> void override {
        m_panicked = true;
        jump();
    }

    /// @brief Defines the reaction on error.
    /// @param desc error descriptor
    auto on_report_error(legacy::er::ErrorDescriptor desc) -> void override {
        const std::lock_guard<std::mutex> guard(m_mutex);
        m_errors.push_back(desc);
    }

    /// @brief Defines the reaction on violation.
    /// @param desc error descriptor
    auto on_report_violation(legacy::er::ErrorDescriptor desc) -> void override {
        const std::lock_guard<std::mutex> guard(m_mutex);
        m_violations.push_back(desc);
    }

    /// @brief Indicates whether there was a panic call previously.
    /// @return true if there was a panic call, false otherwise
    auto has_panicked() const noexcept -> bool {
        return m_panicked.load(std::memory_order_relaxed);
    }

    /// @brief Reset panic state and clears all errors that occurred previously.
    auto reset() noexcept -> void {
        const std::lock_guard<std::mutex> guard(m_mutex);
        m_panicked = false;
        m_errors.clear();
        m_violations.clear();
        m_jumpState.store(JumpState::Obtainable);
    }

    /// @brief Indicates whether any error occurred previously.
    auto has_error() const noexcept -> bool {
        const std::lock_guard<std::mutex> guard(m_mutex);
        return !m_errors.empty();
    }

    /// @brief Indicates whether a specific error occurred previously.
    auto has_error(legacy::er::ErrorCode code, legacy::er::ModuleId module = legacy::er::ModuleId()) const noexcept
        -> bool {
        constexpr legacy::er::ModuleId ANY_MODULE { legacy::er::ModuleId::ANY };
        const std::lock_guard<std::mutex> guard(m_mutex);
        for (auto desc : m_errors) {
            if (desc.code == code) {
                if (module == ANY_MODULE) {
                    return true;
                }
                return desc.module == module;
            }
        }
        return false;
    }

    /// @brief Indicates whether a assumption violation occurred previously.
    /// @note We do not track module id for violations.
    auto has_violation(legacy::er::ErrorCode code) const noexcept -> bool {
        const std::lock_guard<std::mutex> guard(m_mutex);
        // NOLINTNEXTLINE(readability-use-anyofallof) readability of any_of vs range-based for loops is debatable
        for (auto desc : m_violations) {
            if (desc.code == code) {
                return true;
            }
        }
        return false;
    }

    /// @brief runs test_function in a test context that can detect fatal failures;
    /// runs in the same thread
    /// @note uses setjmp/longjmp
    auto fatal_failure_test_context(const bb::StaticFunction<void()>& test_function) -> bool {
        // if there are multiple threads trying to perform a test, only the winner can proceed with the jump
        if (m_jumpState.exchange(JumpState::Pending) == JumpState::Pending) {
            return false;
        };

        // setjmp must be called in a stackframe that still exists when longjmp is called
        // Therefore there cannot be a convenient abstraction that does not also
        // know the test function that is being called.
        // NOLINTNEXTLINE(cert-err52-cpp,modernize-avoid-setjmp-longjmp) exception cannot be used, required for testing to jump in case of failure
        if (setjmp(&(m_jumpBuffer)[0]) != JUMPED_INDICATOR) {
            test_function();
        }

        return true;
    }

  private:
    void jump() noexcept {
        if (m_jumpState.load(std::memory_order_relaxed) == JumpState::Pending) {
            // NOLINTNEXTLINE(cert-err52-cpp,modernize-avoid-setjmp-longjmp) exception handling is not used by design
            longjmp(&m_jumpBuffer[0], JUMPED_INDICATOR);
        }
    }

    //
    // private members
    //

    static constexpr int JUMPED_INDICATOR { 1 };

    mutable std::mutex m_mutex;
    legacy::concurrent::Atomic<bool> m_panicked { false };
    std::vector<legacy::er::ErrorDescriptor> m_errors;

    // we track violations separately (leads to simple search)
    std::vector<legacy::er::ErrorDescriptor> m_violations;

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
    legacy::concurrent::Atomic<JumpState> m_jumpState { JumpState::Obtainable };
};

/// @brief This class hooks into gTest to automatically resets the error handler on the start of a test
class ErrorHandlerSetup : public ::testing::EmptyTestEventListener {
  public:
    void OnTestStart(const ::testing::TestInfo& test_info) override;
};

using ErrorHandler = legacy::StaticLifetimeGuard<TestingErrorHandler>;


inline auto TestingErrorHandler::init() noexcept -> void {
    const ErrorHandler handler;
    legacy::er::ErrorHandler::set(handler);

    auto& listeners = ::testing::UnitTest::GetInstance()->listeners();
    // NOLINTNEXTLINE(cppcoreguidelines-owning-memory) required by the callee
    listeners.Append(new (std::nothrow) ErrorHandlerSetup);
}


inline void ErrorHandlerSetup::OnTestStart(const ::testing::TestInfo& test_info IOX2_MAYBE_UNUSED) {
    ErrorHandler::instance().reset();
}

} // namespace testing
} // namespace bb
} // namespace iox2

#endif
