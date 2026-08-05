// Copyright (c) 2022 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_TESTING_TESTING_LOGGER_HPP
#define IOX2_BB_TESTING_TESTING_LOGGER_HPP

#include "iox2/bb/detail/attributes.hpp"
#include "iox2/legacy/log/building_blocks/logformat.hpp"
#include "iox2/legacy/log/logger.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

#include <functional>
#include <iostream>
#include <mutex>

#include <csignal>
#include <cstdio>
#include <cstring>

// NOLINTNEXTLINE(hicpp-deprecated-headers,modernize-deprecated-headers) required to work on some platforms
#include <setjmp.h>

namespace iox2 {
namespace bb {
namespace testing {
/// @brief This class hooks into gTest to automatically clear the log messages on the start of a test an print the
/// cached log messages from failed tests
class LogPrinter : public ::testing::EmptyTestEventListener {
  public:
    auto OnTestStart(const ::testing::TestInfo& info) -> void override;
    auto OnTestPartResult(const ::testing::TestPartResult& result) -> void override;

#ifndef _WIN32
  private:
    struct sigaction m_sigsegv_old_action = {};
    struct sigaction m_sigfpe_old_action = {};
    struct sigaction m_sigabrt_old_action = {};
#endif
};

/// @brief This logger is used for tests. It caches all the log messages and prints them to the console when a test
/// fails. For debug purposes this behaviour can be overwritten with the 'IOX2_TESTING_ALLOW_LOG' environment variable,
/// e.g. 'IOX2_TESTING_ALLOW_LOG=ON ./hoofs_moduletests --gtest_filter=SharedMemoryObject_Test\*'. Furthermore, it can
/// also be used to check for the occurrence on specific log messages, e.g. when a function is expected to log an error.
/// @code
/// call_to_function_Which_logs_an_error();
/// iox2::bb::testing::TestingLogger::check_log_message_if_log_level_is_supported(iox2::legacy::log::LogLevel::Error,
/// [](const auto& log_messages){
///     ASSERT_THAT(log_messages.size(), Eq(1U));
///     EXPECT_THAT(log_messages[0], HasSubstr(expected_output));
/// });
/// @endcode
class TestingLogger : public legacy::log::TestingLoggerBase {
    using Base = legacy::log::TestingLoggerBase;

  public:
    ~TestingLogger() override = default;

    TestingLogger(const TestingLogger&) = delete;
    TestingLogger(TestingLogger&&) = delete;

    auto operator=(const TestingLogger&) -> TestingLogger& = delete;
    auto operator=(TestingLogger&&) -> TestingLogger& = delete;

    /// @brief Initialized the logger. This should be called in the main function of the test binary
    /// @code
    /// #include "iox2/bb/testing/testing_logger.hpp"
    ///
    /// #include "test.hpp"
    ///
    /// int main(int argc, char* argv[]) {
    ///     ::testing::InitGoogleTest(&argc, argv);
    ///
    ///     iox2::bb::testing::TestingLogger::init();
    ///
    ///     return RUN_ALL_TESTS();
    /// }
    /// @endcode
    static auto init() noexcept -> void {
        using namespace iox2::legacy;

        static TestingLogger LOGGER;
        log::Logger::setActiveLogger(LOGGER);
        log::Logger::init(log::logLevelFromEnvOr(log::LogLevel::Trace));

        const std::lock_guard<std::mutex> lock(LOGGER.m_loggerDataLock);

        // disable logger output only after initializing the logger to get error messages from initialization
        // JUSTIFICATION getenv is required for the functionality of the testing logger and will be called only once in
        // main NOLINTNEXTLINE(concurrency-mt-unsafe)
        if (const auto* allow_log_string = std::getenv("IOX2_TESTING_ALLOW_LOG")) {
            if (log::equalStrings(allow_log_string, "on") || log::equalStrings(allow_log_string, "ON")) {
                LOGGER.m_loggerData.allowLog = true;
            } else {
                LOGGER.m_loggerData.allowLog = false;
                std::cout << "" << std::endl;
                std::cout << "Invalid value for 'IOX2_TESTING_ALLOW_LOG' environment variable!'" << std::endl;
                std::cout << "Found: " << allow_log_string << std::endl;
                std::cout << "Allowed is one of: on, ON" << std::endl;
            }
        } else {
            LOGGER.m_loggerData.allowLog = false;
        }

        auto& listeners = ::testing::UnitTest::GetInstance()->listeners();
        // NOLINTNEXTLINE(cppcoreguidelines-owning-memory) required by the callee
        listeners.Append(new (std::nothrow) LogPrinter);
    }

    /// @brief Removes all log messages from the internal cache. This is automatically done at the start of each test.
    auto clear_log_buffer() noexcept -> void {
        const std::lock_guard<std::mutex> lock(m_loggerDataLock);
        m_loggerData.buffer.clear();
    }

    /// @brief Prints all log messages from the internal cache. This is automatically done at the end of a failed test.
    auto print_log_buffer() noexcept -> void {
        const std::lock_guard<std::mutex> lock(m_loggerDataLock);
        if (m_loggerData.buffer.empty()) {
            return;
        }
        puts("#### Log start ####");
        for (const auto& log : m_loggerData.buffer) {
            puts(log.c_str());
        }
        puts("#### Log end ####");
        m_loggerData.buffer.clear();
    }

    /// @brief Number of caches log messages
    /// @return the number of the log messages from the internal cache
    /// @note This can be used in tests which check for a specific log output
    static auto get_number_of_log_messages() noexcept -> uint64_t {
        auto& logger = dynamic_cast<TestingLogger&>(legacy::log::Logger::get());
        const std::lock_guard<std::mutex> lock(logger.m_loggerDataLock);
        return logger.m_loggerData.buffer.size();
    }

    /// @brief Runs the provided checker function for the collected log messages
    /// @note This can be used in tests to verify the collected log messages
    static auto check_log_message_if_log_level_is_supported(
        legacy::log::LogLevel log_level, const std::function<void(const std::vector<std::string>&)>& check) -> void {
        if (does_logger_support_log_level(log_level)) {
            check(get_log_messages());
        }
    }

    /// @brief Checks if the the LogLevel is above the minimal supported LogLevel compiled into the binary
    /// @param[in] log_level is the log level to check if it is supported
    /// @return true if the log level support is compiled into the binary, false otherwise
    /// @note This can be used in tests which check for a specific log output
    static constexpr auto does_logger_support_log_level(const legacy::log::LogLevel log_level) noexcept -> bool {
        return legacy::log::MINIMAL_LOG_LEVEL >= log_level;
    }

  protected:
    void flush() noexcept override {
        const std::lock_guard<std::mutex> lock(m_loggerDataLock);
        const auto log_buffer = Base::getLogBuffer();
        m_loggerData.buffer.emplace_back(log_buffer.buffer, log_buffer.writeIndex);

        if (m_loggerData.allowLog) {
            Base::flush();
        }

        Base::assumeFlushed();
    }

  private:
    TestingLogger() noexcept = default;

    static auto get_log_messages() noexcept -> std::vector<std::string> {
        auto& logger = dynamic_cast<TestingLogger&>(legacy::log::Logger::get());
        const std::lock_guard<std::mutex> lock(logger.m_loggerDataLock);
        return logger.m_loggerData.buffer;
    }

    struct LoggerData {
        std::vector<std::string> buffer;
        bool allowLog { true };
    };

    std::mutex m_loggerDataLock;
    LoggerData m_loggerData;
};

#ifndef _WIN32
namespace detail {
// NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables) global variable is required as jmp target
static jmp_buf exit_jmp_buffer;

static void sig_handler_flush_logger(int sig, siginfo_t* info IOX2_MAYBE_UNUSED, void* ucontext IOX2_MAYBE_UNUSED) {
    using namespace iox2::legacy;

    constexpr const char* COLOR_RESET { "\033[m" };

    std::cout << log::logLevelDisplayColor(log::LogLevel::Warn)
              << "Catched signal: " << log::logLevelDisplayColor(log::LogLevel::Fatal);
    switch (sig) {
    case SIGSEGV:
        std::cout << "SIGSEGV" << std::flush;
        break;
    case SIGFPE:
        std::cout << "SIGFPE" << std::flush;
        break;
    case SIGABRT:
        std::cout << "SIGABRT" << std::flush;
        break;
    default:
        std::cout << sig;
        break;
    }

    std::cout << COLOR_RESET << "\n\n" << std::flush;

    dynamic_cast<TestingLogger&>(log::Logger::get()).print_log_buffer();

    std::cout << "\n"
              << log::logLevelDisplayColor(log::LogLevel::Warn)
              << "Aborting execution by causing a SIGSEV with 'longjmp' to prevent triggering the signal handler again!"
              << COLOR_RESET << "\n"
              << std::flush;

    constexpr int JMP_VALUE { 1 };
    // NOLINTNEXTLINE(cert-err52-cpp,modernize-avoid-setjmp-longjmp) exception cannot be used and longjmp/setjmp is a working fallback
    longjmp(&exit_jmp_buffer[0], JMP_VALUE);
}
} // namespace detail
#endif

inline auto LogPrinter::OnTestStart(const ::testing::TestInfo& info IOX2_MAYBE_UNUSED) -> void {
    dynamic_cast<TestingLogger&>(legacy::log::Logger::get()).clear_log_buffer();
    TestingLogger::setLogLevel(legacy::log::LogLevel::Trace);

    std::set_terminate([]() -> auto {
        std::cout << "Terminate called\n" << std::flush;
        dynamic_cast<TestingLogger&>(legacy::log::Logger::get()).print_log_buffer();
        std::abort();
    });

#ifndef _WIN32
    struct sigaction action = {};
    memset(&action, 0, sizeof(struct sigaction));
    sigemptyset(&action.sa_mask);

    action.sa_flags = SA_NODEFER;
    action.sa_sigaction = detail::sig_handler_flush_logger;

    sigaction(SIGSEGV, &action, &m_sigsegv_old_action);
    sigaction(SIGFPE, &action, &m_sigfpe_old_action);
    sigaction(SIGABRT, &action, &m_sigabrt_old_action);
#endif
}

inline auto LogPrinter::OnTestPartResult(const ::testing::TestPartResult& result) -> void {
    if (result.failed()) {
        dynamic_cast<TestingLogger&>(legacy::log::Logger::get()).print_log_buffer();
    }

#ifndef _WIN32
    sigaction(SIGSEGV, &m_sigsegv_old_action, nullptr);
    m_sigsegv_old_action = {};
    sigaction(SIGFPE, &m_sigfpe_old_action, nullptr);
    m_sigfpe_old_action = {};
    sigaction(SIGABRT, &m_sigabrt_old_action, nullptr);
    m_sigabrt_old_action = {};
#endif
}

} // namespace testing
} // namespace bb
} // namespace iox2

#endif // IOX2_BB_TESTING_TESTING_LOGGER_HPP
