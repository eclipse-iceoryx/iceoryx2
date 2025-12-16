// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2023 by ekxide IO GmbH. All rights reserved.
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

#ifndef IOX2_BB_REPORTING_LOG_BUILDING_BLOCKS_CONSOLE_LOGGER_INL
#define IOX2_BB_REPORTING_LOG_BUILDING_BLOCKS_CONSOLE_LOGGER_INL

#include "iox2/legacy/log/building_blocks/console_logger.hpp"

namespace iox2 {
namespace legacy {
namespace log {
// AXIVION Next Construct AutosarC++19_03-A3.9.1 : See at declaration in header
// AXIVION Next Construct AutosarC++19_03-A18.1.1 : See at declaration in header
template <uint32_t N>
// NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
inline constexpr uint32_t ConsoleLogger::bufferSize(const char (&)[N]) noexcept {
    return N;
}

template <typename T>
// NOLINTNEXTLINE(cppcoreguidelines-missing-std-forward) intended for this function
inline constexpr void ConsoleLogger::unused(T&&) noexcept {
}

inline LogLevel ConsoleLogger::getLogLevel() noexcept {
    return LogLevel::Info; // TODO get from iceoryx2-bb-log in Rust
}

inline void ConsoleLogger::setLogLevel(const LogLevel /* logLevel */) noexcept {
    // TODO set in iceoryx2-bb-log in Rust
}

// AXIVION Next Construct AutosarC++19_03-M9.3.3 : This is the default implementation for a logger. The design requires
// this to be non-const to enable custom initialization
inline void ConsoleLogger::initLogger(const LogLevel) noexcept {
    // nothing to do in the base implementation
}

// AXIVION Next Construct AutosarC++19_03-A3.9.1 : See at declaration in header
// AXIVION Next Construct AutosarC++19_03-M9.3.3 : This is the default implementation for a logger. The design requires
// this to be non-static to not restrict custom implementations
inline void ConsoleLogger::createLogMessageHeader(const char* file,
                                                  const int line,
                                                  const char* function,
                                                  LogLevel logLevel) noexcept {
    timespec timestamp { 0, 0 };
    // intentionally avoid using 'IOX2_POSIX_CALL' here to keep the logger dependency free
    // NOTE: the log message will eventually be forwarded to iceoryx2-bb-log; temporarily, we ignore the timestamp
    // if (iox_clock_gettime(CLOCK_REALTIME, &timestamp) != 0)
    {
        timestamp = { 0, 0 };
        // intentionally do nothing since a timestamp from 01.01.1970 already indicates  an issue with the clock
    }

    const time_t time { timestamp.tv_sec };

/// @todo iox-#1755 since this will be part of the platform at one point, we might not be able to handle this via the
/// platform abstraction; re-evaluate this when the move to the platform happens
#if defined(_WIN32)
    // seems to be thread-safe on Windows
    const auto* timeInfo = localtime(&time);
#else
    // NOLINTJUSTIFICATION will be initialized with the call to localtime_r in the statement after the declaration
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init,hicpp-member-init)
    struct tm calendarData;
    const auto* timeInfo = localtime_r(&time, &calendarData);
#endif

    // AXIVION Next Construct AutosarC++19_03-A3.9.1 : Not used as an integer but as actual character
    // AXIVION Next Construct AutosarC++19_03-A18.1.1 : This is used to get the size for the buffer where strftime
    // writes the local time
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    constexpr const char TIME_FORMAT[] { "0000-00-00 00:00:00" };
    constexpr uint32_t NULL_TERMINATION { 1 };
    constexpr uint32_t YEAR_1M_PROBLEM { 2 }; // in case iceoryx is still in use, please change to 3
    constexpr auto NULL_TERMINATED_TIMESTAMP_BUFFER_SIZE { ConsoleLogger::bufferSize(TIME_FORMAT) + YEAR_1M_PROBLEM
                                                           + NULL_TERMINATION };

    // AXIVION Next Construct AutosarC++19_03-A3.9.1 : Not used as an integer but as actual character
    // AXIVION Next Construct AutosarC++19_03-A18.1.1 : Required for strftime and safe since array bounds are taken into
    // account
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char timestampString[NULL_TERMINATED_TIMESTAMP_BUFFER_SIZE] { 0 };

    bool timeStampConversionSuccessful { false };
    if (timeInfo != nullptr) {
        const auto strftimeRetVal =
            strftime(&timestampString[0], NULL_TERMINATED_TIMESTAMP_BUFFER_SIZE, "%Y-%m-%d %H:%M:%S", timeInfo);
        timeStampConversionSuccessful = strftimeRetVal != 0;
    }

    if (!timeStampConversionSuccessful) {
        // this will clearly indicate that something went wrong with the time conversion; no need to abort the log
        // output
        strncpy(&timestampString[0], &TIME_FORMAT[0], ConsoleLogger::bufferSize(TIME_FORMAT));
    }

    constexpr uint32_t MILLISECS_PER_SEC { 1000 };
    constexpr uint32_t NANOSECS_PER_MILLISEC { 1000000 };
    // convert nanoseconds to milliseconds and compute the remaining milliseconds in a second
    const auto milliseconds = static_cast<int32_t>((timestamp.tv_nsec / NANOSECS_PER_MILLISEC) % MILLISECS_PER_SEC);

    /// @todo iox-#1755 do we also want to always log the iceoryx version and commit sha? Maybe do that only in
    /// 'initLogger' with LogDebug

    /// @todo iox-#1755 add an option to also print file, line and function
    unused(file);
    unused(line);
    unused(function);

    // AXIVION Next Construct AutosarC++19_03-A3.9.1 : Not used as an integer but as string literal
    // AXIVION Next Construct AutosarC++19_03-M2.13.2 : Required for the color codes; only valid octal digits are used
    constexpr const char* COLOR_GRAY { "\033[0;90m" };
    // AXIVION Next Construct AutosarC++19_03-A3.9.1 : Not used as an integer but as string literal
    // AXIVION Next Construct AutosarC++19_03-M2.13.2 : Required for the color codes; only valid octal digits are used
    constexpr const char* COLOR_RESET { "\033[m" };
    // NOLINTJUSTIFICATION snprintf required to populate char array so that it can be flushed in one piece
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-vararg,hicpp-vararg)
    const auto retVal = snprintf(&getThreadLocalData().buffer[0],
                                 ThreadLocalData::NULL_TERMINATED_BUFFER_SIZE,
                                 "%s%s.%03d %s%s%s: ",
                                 COLOR_GRAY,
                                 &timestampString[0],
                                 milliseconds,
                                 logLevelDisplayColor(logLevel),
                                 logLevelDisplayText(logLevel),
                                 COLOR_RESET);
    if (retVal < 0) {
        /// @todo iox-#1755 this path should never be reached since we ensured the correct encoding of the character
        /// conversion specifier; nevertheless, we might want to call the error handler after the error handler
        /// refactoring was merged
    } else {
        const auto stringSizeToLog = static_cast<uint32_t>(retVal);
        if (stringSizeToLog <= ThreadLocalData::BUFFER_SIZE) {
            getThreadLocalData().bufferWriteIndex = stringSizeToLog;
        } else {
            /// @todo iox-#1755 currently the buffer is large enough that this does not happen but once the file or
            /// function will also be printed, they might be too long to fit into the buffer and will be truncated; once
            /// that feature is implemented, we need to take care of it
            getThreadLocalData().bufferWriteIndex = ThreadLocalData::BUFFER_SIZE;
        }
    }
    getThreadLocalData().logLevel = logLevel;
}

inline void ConsoleLogger::flush() noexcept {
    auto& data = getThreadLocalData();
    // NOLINTJUSTIFICATION it is ensured that the index cannot be out of bounds
    // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
    data.buffer[data.bufferWriteIndex] = '\n'; // overwrite null-termination with line ending
    // constexpr uint32_t LINE_ENDING_SIZE { 1 };

    // NOTE: the log message will eventually be forwarded to iceoryx2-bb-log; for now, we just use cerr for critical log
    // messages if (iox_write(STDERR_FILENO, &data.buffer[0], data.bufferWriteIndex + LINE_ENDING_SIZE) < 0)
    // {
    //     /// @todo iox-#1755 printing to the console failed; call the error handler after the error handler
    //     refactoring
    //     /// was merged
    //}
    if (getThreadLocalData().logLevel == LogLevel::Error || getThreadLocalData().logLevel == LogLevel::Fatal) {
        std::cerr << getThreadLocalData().buffer << std::endl;
    }
    assumeFlushed();
}

// AXIVION Next Construct AutosarC++19_03-M9.3.3 : This is the default implementation for a logger. The design requires
// this to be non-static to not restrict custom implementations
// NOLINTNEXTLINE(readability-convert-member-functions-to-static)
inline LogBuffer ConsoleLogger::getLogBuffer() const noexcept {
    auto& data = getThreadLocalData();
    return LogBuffer { &data.buffer[0], data.bufferWriteIndex };
}

// AXIVION Next Construct AutosarC++19_03-M9.3.3 : This is the default implementation for a logger. The design requires
// this to be non-static to not restrict custom implementations
// NOLINTNEXTLINE(readability-convert-member-functions-to-static)
inline void ConsoleLogger::assumeFlushed() noexcept {
    auto& data = getThreadLocalData();
    data.buffer[0] = 0;
    data.bufferWriteIndex = 0;
}

// AXIVION Next Construct AutosarC++19_03-M9.3.3 : This is the default implementation for a logger. The design requires
// this to be non-static to not restrict custom implementations
// NOLINTNEXTLINE(readability-convert-member-functions-to-static)
inline void ConsoleLogger::logString(const char* message) noexcept {
    auto& data = getThreadLocalData();
    // NOLINTJUSTIFICATION snprintf required to populate char array so that it can be flushed in one piece
    // NOLINTBEGIN(cppcoreguidelines-pro-type-vararg,hicpp-vararg)
    // NOLINTJUSTIFICATION it is ensured that the index cannot be out of bounds
    // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
    const auto retVal = snprintf(&data.buffer[data.bufferWriteIndex],
                                 ThreadLocalData::NULL_TERMINATED_BUFFER_SIZE - data.bufferWriteIndex,
                                 "%s",
                                 message);
    // NOLINTEND(cppcoreguidelines-pro-type-vararg,hicpp-vararg)

    if (retVal < 0) {
        /// @todo iox-#1755 this path should never be reached since we ensured the correct encoding of the character
        /// conversion specifier; nevertheless, we might want to call the error handler after the error handler
        /// refactoring was merged
    } else {
        const auto stringSizeToLog = static_cast<uint32_t>(retVal);
        const auto bufferWriteIndexNext = data.bufferWriteIndex + stringSizeToLog;
        if (bufferWriteIndexNext <= ThreadLocalData::BUFFER_SIZE) {
            data.bufferWriteIndex = bufferWriteIndexNext;
        } else {
            /// @todo iox-#1755 currently we don't support log messages larger than the log buffer and everything larger
            /// that the log buffer will be truncated;
            /// it is intended to flush the buffer and create a new log message later on
            data.bufferWriteIndex = ThreadLocalData::BUFFER_SIZE;
        }
    }
}

// AXIVION Next Construct AutosarC++19_03-M9.3.3 : This is the default implementation for a logger. The design requires
// this to be non-static to not restrict custom implementations
// NOLINTNEXTLINE(readability-convert-member-functions-to-static)
inline void ConsoleLogger::logChar(const char value) noexcept {
    auto& data = getThreadLocalData();
    const auto bufferWriteIndex = data.bufferWriteIndex;
    const auto bufferWriteIndexNext = bufferWriteIndex + 1;
    if (bufferWriteIndexNext <= ThreadLocalData::BUFFER_SIZE) {
        // NOLINTJUSTIFICATION it is ensured that the index cannot be out of bounds
        // NOLINTBEGIN(cppcoreguidelines-pro-bounds-constant-array-index)
        data.buffer[bufferWriteIndex] = value;
        data.buffer[bufferWriteIndexNext] = 0;
        // NOLINTEND(cppcoreguidelines-pro-bounds-constant-array-index)
        data.bufferWriteIndex = bufferWriteIndexNext;
    } else {
        /// @todo iox-#1755 currently we don't support log messages larger than the log buffer and everything larger
        /// than the log buffer will be truncated;
        /// it is intended to flush the buffer and create a new log message later on
    }
}

inline void ConsoleLogger::logBool(const bool value) noexcept {
    logString(value ? "true" : "false");
}

template <typename T, typename std::enable_if_t<std::is_arithmetic<T>::value, bool>>
inline void ConsoleLogger::logDec(const T value) noexcept {
    logArithmetic(value, LOG_FORMAT_DEC<T>);
}

template <typename T,
          typename std::enable_if_t<(std::is_integral<T>::value && std::is_unsigned<T>::value)
                                        || std::is_floating_point<T>::value || std::is_pointer<T>::value,
                                    bool>>
inline void ConsoleLogger::logHex(const T value) noexcept {
    logArithmetic(value, LOG_FORMAT_HEX<T>);
}

template <typename T, typename std::enable_if_t<std::is_integral<T>::value && std::is_unsigned<T>::value, bool>>
inline void ConsoleLogger::logOct(const T value) noexcept {
    logArithmetic(value, LOG_FORMAT_OCT<T>);
}

template <typename T, typename std::enable_if_t<std::is_integral<T>::value && std::is_unsigned<T>::value, bool>>
inline void ConsoleLogger::logBin(const T value) noexcept {
    constexpr uint32_t NUMBER_OF_BITS { std::numeric_limits<decltype(value)>::digits };

    auto& data = getThreadLocalData();

    auto bufferWriteIndexNext = data.bufferWriteIndex;
    auto bufferWriteIndexEnd = bufferWriteIndexNext + NUMBER_OF_BITS;
    if (bufferWriteIndexEnd > ThreadLocalData::BUFFER_SIZE) {
        /// @todo iox-#1755 currently we don't support log messages larger than the log buffer and everything larger
        /// than the log buffer will be truncated;
        /// it is intended to flush the buffer and create a new log message later on
        bufferWriteIndexEnd = ThreadLocalData::BUFFER_SIZE;
    }

    constexpr T ONE { 1 };
    T mask { ONE << (NUMBER_OF_BITS - 1) };
    for (; bufferWriteIndexNext < bufferWriteIndexEnd; ++bufferWriteIndexNext) {
        // NOLINTJUSTIFICATION it is ensured that the index cannot be out of bounds
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        data.buffer[bufferWriteIndexNext] = (value & mask) ? '1' : '0';
        mask = static_cast<T>(mask >> 1);
    }
    // NOLINTJUSTIFICATION it is ensured that the index cannot be out of bounds
    // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
    data.buffer[bufferWriteIndexNext] = 0;
    data.bufferWriteIndex = bufferWriteIndexNext;
}

inline void ConsoleLogger::logRaw(const void* const data, const uint64_t size) noexcept {
    logString("0x[");
    if (data == nullptr) {
        logString("nullptr, ");
        logDec(size);
    } else {
        for (uint64_t i = 0; i < size; ++i) {
            if (i > 0) {
                logChar(' ');
            }
            // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-pointer-arithmetic)
            logArithmetic(static_cast<const uint8_t*>(data)[i], "%02hhx");
        }
    }
    logChar(']');
}

// AXIVION Next Construct AutosarC++19_03-A3.9.1 : See at declaration in header
template <typename T>
inline void ConsoleLogger::logArithmetic(const T value, const char* format) noexcept {
    auto& data = getThreadLocalData();
    // NOLINTJUSTIFICATION it is ensured that the index cannot be out of bounds
    // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
    const auto retVal = snprintf(&data.buffer[data.bufferWriteIndex],
                                 ThreadLocalData::NULL_TERMINATED_BUFFER_SIZE - data.bufferWriteIndex,
                                 format,
                                 value);
    if (retVal < 0) {
        /// @todo iox-#1755 this path should never be reached since we ensured the correct encoding of the character
        /// conversion specifier; nevertheless, we might want to call the error handler after the error handler
        /// refactoring was merged
        return;
    }

    const auto stringSizeToLog = static_cast<uint32_t>(retVal);
    const auto bufferWriteIndexNext = data.bufferWriteIndex + stringSizeToLog;
    if (bufferWriteIndexNext <= ThreadLocalData::BUFFER_SIZE) {
        data.bufferWriteIndex = bufferWriteIndexNext;
    } else {
        /// @todo iox-#1755 currently we don't support log messages larger than the log buffer and everything larger
        /// that the log buffer will be truncated;
        /// it is intended to flush the buffer and create a new log message later on
        data.bufferWriteIndex = ThreadLocalData::BUFFER_SIZE;
    }
}

inline ConsoleLogger::ThreadLocalData& ConsoleLogger::getThreadLocalData() noexcept {
    thread_local static ThreadLocalData data;
    return data;
}

} // namespace log
} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_REPORTING_LOG_BUILDING_BLOCKS_CONSOLE_LOGGER_INL
