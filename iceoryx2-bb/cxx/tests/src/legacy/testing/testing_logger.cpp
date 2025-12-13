// Copyright (c) 2022 by Apex.AI Inc. All rights reserved.
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

#include "iox2/legacy/testing/testing_logger.hpp"
#include "iox2/legacy/log/building_blocks/logformat.hpp"
#include "iox2/legacy/log/logger.hpp"

#include <iostream>

#include <csignal>
#include <cstdio>
#include <cstring>

// NOLINTNEXTLINE(hicpp-deprecated-headers) required to work on some platforms
#include <setjmp.h>

namespace iox2 {
namespace legacy {
namespace testing {
void TestingLogger::init() noexcept {
    static TestingLogger logger;
    log::Logger::setActiveLogger(logger);
    log::Logger::init(log::logLevelFromEnvOr(log::LogLevel::Trace));

    const std::lock_guard<std::mutex> lock(logger.m_loggerDataLock);

    // disable logger output only after initializing the logger to get error messages from initialization
    // JUSTIFICATION getenv is required for the functionality of the testing logger and will be called only once in main
    // NOLINTNEXTLINE(concurrency-mt-unsafe)
    if (const auto* allowLogString = std::getenv("IOX2_TESTING_ALLOW_LOG")) {
        if (log::equalStrings(allowLogString, "on") || log::equalStrings(allowLogString, "ON")) {
            logger.m_loggerData.allowLog = true;
        } else {
            logger.m_loggerData.allowLog = false;
            std::cout << "" << std::endl;
            std::cout << "Invalid value for 'IOX2_TESTING_ALLOW_LOG' environment variable!'" << std::endl;
            std::cout << "Found: " << allowLogString << std::endl;
            std::cout << "Allowed is one of: on, ON" << std::endl;
        }
    } else {
        logger.m_loggerData.allowLog = false;
    }

    auto& listeners = ::testing::UnitTest::GetInstance()->listeners();
    // NOLINTNEXTLINE(cppcoreguidelines-owning-memory) required by the callee
    listeners.Append(new (std::nothrow) LogPrinter);
}

void TestingLogger::clearLogBuffer() noexcept {
    const std::lock_guard<std::mutex> lock(m_loggerDataLock);
    m_loggerData.buffer.clear();
}

void TestingLogger::printLogBuffer() noexcept {
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

uint64_t TestingLogger::getNumberOfLogMessages() noexcept {
    auto& logger = dynamic_cast<TestingLogger&>(log::Logger::get());
    const std::lock_guard<std::mutex> lock(logger.m_loggerDataLock);
    return logger.m_loggerData.buffer.size();
}

void TestingLogger::checkLogMessageIfLogLevelIsSupported(
    iox2::legacy::log::LogLevel logLevel, const std::function<void(const std::vector<std::string>&)>& check) {
    if (doesLoggerSupportLogLevel(logLevel)) {
        check(getLogMessages());
    }
}


void TestingLogger::flush() noexcept {
    const std::lock_guard<std::mutex> lock(m_loggerDataLock);
    const auto logBuffer = Base::getLogBuffer();
    m_loggerData.buffer.emplace_back(logBuffer.buffer, logBuffer.writeIndex);

    if (m_loggerData.allowLog) {
        Base::flush();
    }

    Base::assumeFlushed();
}

std::vector<std::string> TestingLogger::getLogMessages() noexcept {
    auto& logger = dynamic_cast<TestingLogger&>(log::Logger::get());
    const std::lock_guard<std::mutex> lock(logger.m_loggerDataLock);
    return logger.m_loggerData.buffer;
}

#if !defined(_WIN32)
// NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables) global variable is required as jmp target
jmp_buf exitJmpBuffer;

static void sigHandler(int sig, siginfo_t*, void*) {
    constexpr const char* COLOR_RESET { "\033[m" };

    std::cout << iox2::legacy::log::logLevelDisplayColor(iox2::legacy::log::LogLevel::Warn)
              << "Catched signal: " << iox2::legacy::log::logLevelDisplayColor(iox2::legacy::log::LogLevel::Fatal);
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

    dynamic_cast<TestingLogger&>(log::Logger::get()).printLogBuffer();

    std::cout << "\n"
              << iox2::legacy::log::logLevelDisplayColor(iox2::legacy::log::LogLevel::Warn)
              << "Aborting execution by causing a SIGSEV with 'longjmp' to prevent triggering the signal handler again!"
              << COLOR_RESET << "\n"
              << std::flush;

    constexpr int JMP_VALUE { 1 };
    // NOLINTNEXTLINE(cert-err52-cpp) exception cannot be used and longjmp/setjmp is a working fallback
    longjmp(&exitJmpBuffer[0], JMP_VALUE);
}
#endif

void LogPrinter::OnTestStart(const ::testing::TestInfo&) {
    dynamic_cast<TestingLogger&>(log::Logger::get()).clearLogBuffer();
    TestingLogger::setLogLevel(log::LogLevel::Trace);

    std::set_terminate([]() {
        std::cout << "Terminate called\n" << std::flush;
        dynamic_cast<TestingLogger&>(log::Logger::get()).printLogBuffer();
        std::abort();
    });

#if !defined(_WIN32)
    struct sigaction action = {};
    memset(&action, 0, sizeof(struct sigaction));
    sigemptyset(&action.sa_mask);

    action.sa_flags = SA_NODEFER;
    action.sa_sigaction = sigHandler;

    sigaction(SIGSEGV, &action, nullptr);
    sigaction(SIGFPE, &action, nullptr);
    sigaction(SIGABRT, &action, nullptr);
#endif
}

void LogPrinter::OnTestPartResult(const ::testing::TestPartResult& result) {
    if (result.failed()) {
        dynamic_cast<TestingLogger&>(log::Logger::get()).printLogBuffer();
    }

    /// @todo iox-#1755 de-register the signal handler from 'OnTestStart'
}

} // namespace testing
} // namespace legacy
} // namespace iox2
