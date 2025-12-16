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

#ifndef IOX2_BB_POSIX_DESIGN_POSIX_CALL_INL
#define IOX2_BB_POSIX_DESIGN_POSIX_CALL_INL

#include "iox2/legacy/logging.hpp"
#include "iox2/legacy/posix_call.hpp"

namespace iox2 {
namespace legacy {
namespace detail {
template <typename ReturnType, typename... FunctionArguments>
inline PosixCallBuilder<ReturnType, FunctionArguments...>
// NOLINTJUSTIFICATION this function is never used directly, only be the macro IOX2_POSIX_CALL
// NOLINTNEXTLINE(readability-function-size)
createPosixCallBuilder(ReturnType (*IOX2_POSIX_CALL)(FunctionArguments...),
                       const char* posixFunctionName,
                       const char* file,
                       const int32_t line,
                       const char* callingFunction) noexcept {
    return PosixCallBuilder<ReturnType, FunctionArguments...>(
        IOX2_POSIX_CALL, posixFunctionName, file, line, callingFunction);
}

template <typename ReturnType>
// NOLINTJUSTIFICATION used only internally, the function and file name are provided by
//                     compiler macros and are of type char*
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters)
inline PosixCallDetails<ReturnType>::PosixCallDetails(const char* posixFunctionName,
                                                      const char* file,
                                                      int line,
                                                      const char* callingFunction) noexcept
    : posixFunctionName(posixFunctionName)
    , file(file)
    , callingFunction(callingFunction)
    , line(line) {
}

/// the overload is required since on most linux systems there are two different implementations
/// of "strerror_r", the posix compliant one which returns an int and stores the message in the buffer
/// and a gnu version which returns a pointer to the message and sometimes stores the message
/// in the buffer

/// @brief Finalizes the recursion of doesContainValue
/// @return always false
template <typename T>
inline constexpr bool doesContainValue(const T) noexcept {
    return false;
}

/// @brief Returns true if value of T is found in the ValueList, otherwise false
/// @tparam T type of the value to check
/// @tparam ValueList is a list of values to check for a specific value
/// @param[in] value to look for in the ValueList
/// @param[in] firstValueListEntry is the first variadic argument of ValueList
/// @param[in] remainingValueListEntries are the remaining variadic arguments of ValueList
/// @return true if value is contained in the ValueList, otherwise false
/// @note be aware that value is tested for exact equality with the entries of ValueList and regular floating-point
/// comparison rules apply
template <typename T1, typename T2, typename... ValueList>
inline constexpr bool
doesContainValue(const T1 value, const T2 firstValueListEntry, const ValueList... remainingValueListEntries) noexcept {
    // AXIVION Next Line AutosarC++19_03-M6.2.2 : intentional check for exact equality
    return (value == firstValueListEntry) ? true : doesContainValue(value, remainingValueListEntries...);
}
} // namespace detail

template <typename ReturnType, typename... FunctionArguments>
inline PosixCallBuilder<ReturnType, FunctionArguments...>::PosixCallBuilder(FunctionType_t IOX2_POSIX_CALL,
                                                                            const char* posixFunctionName,
                                                                            const char* file,
                                                                            const int32_t line,
                                                                            const char* callingFunction) noexcept
    : m_IOX2_POSIX_CALL { IOX2_POSIX_CALL }
    , m_details { posixFunctionName, file, line, callingFunction } {
}

template <typename ReturnType, typename... FunctionArguments>
inline PosixCallVerificator<ReturnType>
PosixCallBuilder<ReturnType, FunctionArguments...>::operator()(FunctionArguments... arguments) && noexcept {
    for (uint64_t i = 0U; i < POSIX_CALL_EINTR_REPETITIONS; ++i) {
        errno = 0;
        m_details.result.value = m_IOX2_POSIX_CALL(arguments...);
        m_details.result.errnum = errno;

        if (m_details.result.errnum != EINTR) {
            break;
        }
    }

    return PosixCallVerificator<ReturnType>(m_details);
}

template <typename ReturnType>
inline PosixCallVerificator<ReturnType>::PosixCallVerificator(detail::PosixCallDetails<ReturnType>& details) noexcept
    : m_details { details } {
}

template <typename ReturnType>
template <typename... SuccessReturnValues>
inline PosixCallEvaluator<ReturnType>
PosixCallVerificator<ReturnType>::successReturnValue(const SuccessReturnValues... successReturnValues) && noexcept {
    m_details.hasSuccess = detail::doesContainValue(m_details.result.value, successReturnValues...);

    return PosixCallEvaluator<ReturnType>(m_details);
}

template <typename ReturnType>
template <typename... FailureReturnValues>
inline PosixCallEvaluator<ReturnType>
PosixCallVerificator<ReturnType>::failureReturnValue(const FailureReturnValues... failureReturnValues) && noexcept {
    using ValueType = decltype(m_details.result.value);
    m_details.hasSuccess =
        !detail::doesContainValue(m_details.result.value, static_cast<ValueType>(failureReturnValues)...);

    return PosixCallEvaluator<ReturnType>(m_details);
}

template <typename ReturnType>
inline PosixCallEvaluator<ReturnType> PosixCallVerificator<ReturnType>::returnValueMatchesErrno() && noexcept {
    m_details.hasSuccess = m_details.result.value == 0;
    m_details.result.errnum = static_cast<int32_t>(m_details.result.value);

    return PosixCallEvaluator<ReturnType>(m_details);
}

template <typename ReturnType>
inline PosixCallEvaluator<ReturnType>::PosixCallEvaluator(detail::PosixCallDetails<ReturnType>& details) noexcept
    : m_details { details } {
}

template <typename ReturnType>
template <typename... IgnoredErrnos>
inline PosixCallEvaluator<ReturnType>
PosixCallEvaluator<ReturnType>::ignoreErrnos(const IgnoredErrnos... ignoredErrnos) const&& noexcept {
    if (!m_details.hasSuccess) {
        m_details.hasIgnoredErrno |= detail::doesContainValue(m_details.result.errnum, ignoredErrnos...);
    }

    return *this;
}

template <typename ReturnType>
template <typename... SilentErrnos>
inline PosixCallEvaluator<ReturnType>
PosixCallEvaluator<ReturnType>::suppressErrorMessagesForErrnos(const SilentErrnos... silentErrnos) const&& noexcept {
    if (!m_details.hasSuccess) {
        m_details.hasSilentErrno |= detail::doesContainValue(m_details.result.errnum, silentErrnos...);
    }

    return *this;
}

template <typename ReturnType>
inline expected<PosixCallResult<ReturnType>, PosixCallResult<ReturnType>>
PosixCallEvaluator<ReturnType>::evaluate() const&& noexcept {
    if (m_details.hasSuccess || m_details.hasIgnoredErrno) {
        return ok<PosixCallResult<ReturnType>>(m_details.result);
    }

    if (!m_details.hasSilentErrno) {
        IOX2_LOG(Error,
                 m_details.file << ":" << m_details.line << " { " << m_details.callingFunction << " -> "
                                << m_details.posixFunctionName << " }  :::  [ errno: " << m_details.result.errnum
                                << " ]");
    }

    return err<PosixCallResult<ReturnType>>(m_details.result);
}

} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_POSIX_WRAPPER_POSIX_CALL_INL
