// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX_HOOFS_REPORTING_LOG_BUILDING_BLOCKS_CONSOLE_LOGGER_INL
#define IOX_HOOFS_REPORTING_LOG_BUILDING_BLOCKS_CONSOLE_LOGGER_INL

#include "iox/log/building_blocks/console_logger.hpp"

namespace iox
{
namespace log
{
// AXIVION Next Construct AutosarC++19_03-A3.9.1 : See at declaration in header
// AXIVION Next Construct AutosarC++19_03-A18.1.1 : See at declaration in header
template <uint32_t N>
// NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
inline constexpr uint32_t ConsoleLogger::bufferSize(const char (&)[N]) noexcept
{
    return N;
}

template <typename T>
// NOLINTNEXTLINE(cppcoreguidelines-missing-std-forward) intended for this function
inline constexpr void ConsoleLogger::unused(T&&) noexcept
{
}

// AXIVION Next Construct AutosarC++19_03-M9.3.3 : This is the default implementation for a logger. The design requires
// this to be non-static to not restrict custom implementations
// NOLINTNEXTLINE(readability-convert-member-functions-to-static)
inline void ConsoleLogger::logChar(const char value) noexcept
{
    auto& data = getThreadLocalData();
    const auto bufferWriteIndex = data.bufferWriteIndex;
    const auto bufferWriteIndexNext = bufferWriteIndex + 1;
    if (bufferWriteIndexNext <= ThreadLocalData::BUFFER_SIZE)
    {
        // NOLINTJUSTIFICATION it is ensured that the index cannot be out of bounds
        // NOLINTBEGIN(cppcoreguidelines-pro-bounds-constant-array-index)
        data.buffer[bufferWriteIndex] = value;
        data.buffer[bufferWriteIndexNext] = 0;
        // NOLINTEND(cppcoreguidelines-pro-bounds-constant-array-index)
        data.bufferWriteIndex = bufferWriteIndexNext;
    }
    else
    {
        /// @todo iox-#1755 currently we don't support log messages larger than the log buffer and everything larger
        /// than the log buffer will be truncated;
        /// it is intended to flush the buffer and create a new log message later on
    }
}

template <typename T, typename std::enable_if_t<std::is_arithmetic<T>::value, bool>>
inline void ConsoleLogger::logDec(const T value) noexcept
{
    logArithmetic(value, LOG_FORMAT_DEC<T>);
}

template <typename T,
          typename std::enable_if_t<(std::is_integral<T>::value && std::is_unsigned<T>::value)
                                        || std::is_floating_point<T>::value || std::is_pointer<T>::value,
                                    bool>>
inline void ConsoleLogger::logHex(const T value) noexcept
{
    logArithmetic(value, LOG_FORMAT_HEX<T>);
}

template <typename T, typename std::enable_if_t<std::is_integral<T>::value && std::is_unsigned<T>::value, bool>>
inline void ConsoleLogger::logOct(const T value) noexcept
{
    logArithmetic(value, LOG_FORMAT_OCT<T>);
}

template <typename T, typename std::enable_if_t<std::is_integral<T>::value && std::is_unsigned<T>::value, bool>>
inline void ConsoleLogger::logBin(const T value) noexcept
{
    constexpr uint32_t NUMBER_OF_BITS{std::numeric_limits<decltype(value)>::digits};

    auto& data = getThreadLocalData();

    auto bufferWriteIndexNext = data.bufferWriteIndex;
    auto bufferWriteIndexEnd = bufferWriteIndexNext + NUMBER_OF_BITS;
    if (bufferWriteIndexEnd > ThreadLocalData::BUFFER_SIZE)
    {
        /// @todo iox-#1755 currently we don't support log messages larger than the log buffer and everything larger
        /// than the log buffer will be truncated;
        /// it is intended to flush the buffer and create a new log message later on
        bufferWriteIndexEnd = ThreadLocalData::BUFFER_SIZE;
    }

    constexpr T ONE{1};
    T mask{ONE << (NUMBER_OF_BITS - 1)};
    for (; bufferWriteIndexNext < bufferWriteIndexEnd; ++bufferWriteIndexNext)
    {
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

// AXIVION Next Construct AutosarC++19_03-A3.9.1 : See at declaration in header
template <typename T>
inline void ConsoleLogger::logArithmetic(const T value, const char* format) noexcept
{
    auto& data = getThreadLocalData();
    // NOLINTJUSTIFICATION it is ensured that the index cannot be out of bounds
    // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
    const auto retVal = snprintf(&data.buffer[data.bufferWriteIndex],
                                 ThreadLocalData::NULL_TERMINATED_BUFFER_SIZE - data.bufferWriteIndex,
                                 format,
                                 value);
    if (retVal < 0)
    {
        /// @todo iox-#1755 this path should never be reached since we ensured the correct encoding of the character
        /// conversion specifier; nevertheless, we might want to call the error handler after the error handler
        /// refactoring was merged
        return;
    }

    const auto stringSizeToLog = static_cast<uint32_t>(retVal);
    const auto bufferWriteIndexNext = data.bufferWriteIndex + stringSizeToLog;
    if (bufferWriteIndexNext <= ThreadLocalData::BUFFER_SIZE)
    {
        data.bufferWriteIndex = bufferWriteIndexNext;
    }
    else
    {
        /// @todo iox-#1755 currently we don't support log messages larger than the log buffer and everything larger
        /// that the log buffer will be truncated;
        /// it is intended to flush the buffer and create a new log message later on
        data.bufferWriteIndex = ThreadLocalData::BUFFER_SIZE;
    }
}

} // namespace log
} // namespace iox

#endif // IOX_HOOFS_REPORTING_LOG_BUILDING_BLOCKS_CONSOLE_LOGGER_INL
