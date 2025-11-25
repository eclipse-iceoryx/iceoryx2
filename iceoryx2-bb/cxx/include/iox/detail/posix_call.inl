// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX_HOOFS_POSIX_DESIGN_POSIX_CALL_INL
#define IOX_HOOFS_POSIX_DESIGN_POSIX_CALL_INL

#include "iceoryx_platform/errno.hpp"
#include "iox/logging.hpp"
#include "iox/posix_call.hpp"

namespace iox
{
namespace detail
{
template <typename ReturnType, typename... FunctionArguments>
inline PosixCallBuilder<ReturnType, FunctionArguments...>
// NOLINTJUSTIFICATION this function is never used directly, only be the macro IOX_POSIX_CALL
// NOLINTNEXTLINE(readability-function-size)
createPosixCallBuilder(ReturnType (*IOX_POSIX_CALL)(FunctionArguments...),
                       const char* posixFunctionName,
                       const char* file,
                       const int32_t line,
                       const char* callingFunction) noexcept
{
    return PosixCallBuilder<ReturnType, FunctionArguments...>(
        IOX_POSIX_CALL, posixFunctionName, file, line, callingFunction);
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
    , line(line)
{
}

/// the overload is required since on most linux systems there are two different implementations
/// of "strerror_r", the posix compliant one which returns an int and stores the message in the buffer
/// and a gnu version which returns a pointer to the message and sometimes stores the message
/// in the buffer
inline string<POSIX_CALL_ERROR_STRING_SIZE> errorLiteralToString(const int returnCode IOX_MAYBE_UNUSED,
                                                                 char* const buffer)
{
    return string<POSIX_CALL_ERROR_STRING_SIZE>(TruncateToCapacity, buffer);
}

inline string<POSIX_CALL_ERROR_STRING_SIZE> errorLiteralToString(const char* msg, char* const buffer IOX_MAYBE_UNUSED)
{
    return string<POSIX_CALL_ERROR_STRING_SIZE>(TruncateToCapacity, msg);
}
} // namespace detail

template <typename T>
inline string<POSIX_CALL_ERROR_STRING_SIZE> PosixCallResult<T>::getHumanReadableErrnum() const noexcept
{
    // NOLINTJUSTIFICATION needed by POSIX function which is wrapped here
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char buffer[POSIX_CALL_ERROR_STRING_SIZE];
    return string<POSIX_CALL_ERROR_STRING_SIZE>(TruncateToCapacity,
                                                iox_gnu_strerror_r(errnum, &buffer[0], POSIX_CALL_ERROR_STRING_SIZE));
}

template <typename ReturnType, typename... FunctionArguments>
inline PosixCallBuilder<ReturnType, FunctionArguments...>::PosixCallBuilder(FunctionType_t IOX_POSIX_CALL,
                                                                            const char* posixFunctionName,
                                                                            const char* file,
                                                                            const int32_t line,
                                                                            const char* callingFunction) noexcept
    : m_IOX_POSIX_CALL{IOX_POSIX_CALL}
    , m_details{posixFunctionName, file, line, callingFunction}
{
}

template <typename ReturnType, typename... FunctionArguments>
inline PosixCallVerificator<ReturnType>
PosixCallBuilder<ReturnType, FunctionArguments...>::operator()(FunctionArguments... arguments) && noexcept
{
    for (uint64_t i = 0U; i < POSIX_CALL_EINTR_REPETITIONS; ++i)
    {
        errno = 0;
        m_details.result.value = m_IOX_POSIX_CALL(arguments...);
        m_details.result.errnum = errno;

        if (m_details.result.errnum != EINTR)
        {
            break;
        }
    }

    return PosixCallVerificator<ReturnType>(m_details);
}

template <typename ReturnType>
inline PosixCallVerificator<ReturnType>::PosixCallVerificator(detail::PosixCallDetails<ReturnType>& details) noexcept
    : m_details{details}
{
}

template <typename ReturnType>
template <typename... SuccessReturnValues>
inline PosixCallEvaluator<ReturnType>
PosixCallVerificator<ReturnType>::successReturnValue(const SuccessReturnValues... successReturnValues) && noexcept
{
    m_details.hasSuccess = algorithm::doesContainValue(m_details.result.value, successReturnValues...);

    return PosixCallEvaluator<ReturnType>(m_details);
}

template <typename ReturnType>
template <typename... FailureReturnValues>
inline PosixCallEvaluator<ReturnType>
PosixCallVerificator<ReturnType>::failureReturnValue(const FailureReturnValues... failureReturnValues) && noexcept
{
    using ValueType = decltype(m_details.result.value);
    m_details.hasSuccess =
        !algorithm::doesContainValue(m_details.result.value, static_cast<ValueType>(failureReturnValues)...);

    return PosixCallEvaluator<ReturnType>(m_details);
}

template <typename ReturnType>
inline PosixCallEvaluator<ReturnType> PosixCallVerificator<ReturnType>::returnValueMatchesErrno() && noexcept
{
    m_details.hasSuccess = m_details.result.value == 0;
    m_details.result.errnum = static_cast<int32_t>(m_details.result.value);

    return PosixCallEvaluator<ReturnType>(m_details);
}

template <typename ReturnType>
inline PosixCallEvaluator<ReturnType>::PosixCallEvaluator(detail::PosixCallDetails<ReturnType>& details) noexcept
    : m_details{details}
{
}

template <typename ReturnType>
template <typename... IgnoredErrnos>
inline PosixCallEvaluator<ReturnType>
PosixCallEvaluator<ReturnType>::ignoreErrnos(const IgnoredErrnos... ignoredErrnos) const&& noexcept
{
    if (!m_details.hasSuccess)
    {
        m_details.hasIgnoredErrno |= algorithm::doesContainValue(m_details.result.errnum, ignoredErrnos...);
    }

    return *this;
}

template <typename ReturnType>
template <typename... SilentErrnos>
inline PosixCallEvaluator<ReturnType>
PosixCallEvaluator<ReturnType>::suppressErrorMessagesForErrnos(const SilentErrnos... silentErrnos) const&& noexcept
{
    if (!m_details.hasSuccess)
    {
        m_details.hasSilentErrno |= algorithm::doesContainValue(m_details.result.errnum, silentErrnos...);
    }

    return *this;
}

template <typename ReturnType>
inline expected<PosixCallResult<ReturnType>, PosixCallResult<ReturnType>>
PosixCallEvaluator<ReturnType>::evaluate() const&& noexcept
{
    if (m_details.hasSuccess || m_details.hasIgnoredErrno)
    {
        return ok<PosixCallResult<ReturnType>>(m_details.result);
    }

    if (!m_details.hasSilentErrno)
    {
        IOX_LOG(Error,
                m_details.file << ":" << m_details.line << " { " << m_details.callingFunction << " -> "
                               << m_details.posixFunctionName << " }  :::  [ " << m_details.result.errnum << " ]  "
                               << m_details.result.getHumanReadableErrnum());
    }

    return err<PosixCallResult<ReturnType>>(m_details.result);
}

} // namespace iox

#endif // IOX_HOOFS_POSIX_WRAPPER_POSIX_CALL_INL
