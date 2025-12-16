// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_REPORTING_LOG_BUILDING_BLOCKS_LOGGER_INL
#define IOX2_BB_REPORTING_LOG_BUILDING_BLOCKS_LOGGER_INL

#include "iox2/legacy/log/building_blocks/logger.hpp"

#include <cstdint>
#include <cstring>
#include <mutex>

namespace iox2 {
namespace legacy {
namespace log {
inline constexpr const char* asStringLiteral(const LogLevel value) noexcept {
    switch (value) {
    case LogLevel::Off:
        return "LogLevel::Off";
    case LogLevel::Fatal:
        return "LogLevel::Fatal";
    case LogLevel::Error:
        return "LogLevel::Error";
    case LogLevel::Warn:
        return "LogLevel::Warn";
    case LogLevel::Info:
        return "LogLevel::Info";
    case LogLevel::Debug:
        return "LogLevel::Debug";
    case LogLevel::Trace:
        return "LogLevel::Trace";
    }

    return "[Undefined LogLevel]";
}

template <uint32_t N>
// NOLINTJUSTIFICATION See at declaration in header
// NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
inline bool equalStrings(const char* lhs, const char (&rhs)[N]) noexcept {
    return strncmp(lhs, rhs, N) == 0;
}

inline LogLevel logLevelFromEnvOr(const LogLevel logLevel) noexcept {
    auto specifiedLogLevel = logLevel;

    // AXIVION Next Construct AutosarC++19_03-M18.0.3 : Use of getenv is allowed in MISRA amendment#6312
    // JUSTIFICATION getenv is required for the functionality of this function; see also declaration in header
    // NOLINTNEXTLINE(concurrency-mt-unsafe)
    if (const auto* logLevelString = std::getenv("IOX2_LOG_LEVEL")) {
        if (equalStrings(logLevelString, "off")) {
            specifiedLogLevel = LogLevel::Off;
        } else if (equalStrings(logLevelString, "fatal")) {
            specifiedLogLevel = LogLevel::Fatal;
        } else if (equalStrings(logLevelString, "error")) {
            specifiedLogLevel = LogLevel::Error;
        } else if (equalStrings(logLevelString, "warn")) {
            specifiedLogLevel = LogLevel::Warn;
        } else if (equalStrings(logLevelString, "info")) {
            specifiedLogLevel = LogLevel::Info;
        } else if (equalStrings(logLevelString, "debug")) {
            specifiedLogLevel = LogLevel::Debug;
        } else if (equalStrings(logLevelString, "trace")) {
            specifiedLogLevel = LogLevel::Trace;
        } else {
            puts("Invalid value for 'IOX2_LOG_LEVEL' environment variable!'");
            puts("Found:");
            puts(logLevelString);
            puts("Allowed is one of: off, fatal, error, warn, info, debug, trace");
        }
    }
    return specifiedLogLevel;
}

namespace internal {
template <typename BaseLogger>
inline Logger<BaseLogger>& Logger<BaseLogger>::get() noexcept {
    /// @todo iox-#1755 use the PolymorphicHandler for the handling with the logger exchange once available

    // NOLINTJUSTIFICATION needed for the functionality
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables)
    thread_local static Logger* logger = &Logger::activeLogger();
    if (!logger->m_isActive.load(std::memory_order_relaxed)) {
        // no need to loop until m_isActive is true since this is an inherent race
        //   - the logger needs to be active for the whole lifetime of the application anyway
        //   - if the logger was changed again, the next call will update the logger
        //   - furthermore, it is not recommended to change the logger more than once
        logger = &Logger::activeLogger();
    }
    return *logger;
}

template <typename BaseLogger>
inline void Logger<BaseLogger>::init(const LogLevel logLevel) noexcept {
    Logger::get().initLoggerInternal(logLevel);
}

template <typename BaseLogger>
inline void Logger<BaseLogger>::setActiveLogger(Logger<BaseLogger>& newLogger) noexcept {
    Logger::activeLogger(&newLogger);
}

template <typename BaseLogger>
inline Logger<BaseLogger>& Logger<BaseLogger>::activeLogger(Logger<BaseLogger>* newLogger) noexcept {
    static std::mutex mtx;
    std::lock_guard<std::mutex> lock(mtx);
    static Logger defaultLogger;

    // NOLINTJUSTIFICATION needed for the functionality
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables)
    static Logger* logger { &defaultLogger };

    if (newLogger) {
        if (logger->m_isFinalized.load(std::memory_order_relaxed)) {
            logger->createLogMessageHeader(__FILE__, __LINE__, __FUNCTION__, LogLevel::Error);
            logger->logString("Trying to replace logger after already initialized!");
            logger->flush();
            newLogger->createLogMessageHeader(__FILE__, __LINE__, __FUNCTION__, LogLevel::Error);
            logger->logString("Trying to replace logger after already initialized!");
            newLogger->flush();
            /// @todo iox-#1755 call error handler after the error handler refactoring was merged
        } else {
            logger->m_isActive.store(false);
            logger = newLogger;
            logger->m_isActive.store(true);
        }
    }

    return *logger;
}

template <typename BaseLogger>
inline void Logger<BaseLogger>::initLoggerInternal(const LogLevel logLevel) noexcept {
    if (!m_isFinalized.load(std::memory_order_relaxed)) {
        BaseLogger::setLogLevel(logLevel);
        BaseLogger::initLogger(logLevel);
        m_isFinalized.store(true, std::memory_order_relaxed);
    } else {
        BaseLogger::createLogMessageHeader(__FILE__, __LINE__, __FUNCTION__, LogLevel::Error);
        BaseLogger::logString("Multiple initLogger calls");
        BaseLogger::flush();
        /// @todo iox-#1755 call error handler after the error handler refactoring was merged
    }
}

} // namespace internal
} // namespace log
} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_REPORTING_LOG_BUILDING_BLOCKS_LOGGER_INL
