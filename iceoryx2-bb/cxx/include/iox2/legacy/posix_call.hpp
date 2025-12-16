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

#ifndef IOX2_BB_POSIX_DESIGN_POSIX_CALL_HPP
#define IOX2_BB_POSIX_DESIGN_POSIX_CALL_HPP

#include "iox2/legacy/detail/platform_correction.hpp"

#include "iox2/bb/detail/attributes.hpp"
#include "iox2/legacy/expected.hpp"

#include <cstdint>

namespace iox2 {
namespace legacy {
static constexpr uint32_t POSIX_CALL_ERROR_STRING_SIZE = 128U;
static constexpr uint64_t POSIX_CALL_EINTR_REPETITIONS = 5U;
static constexpr int32_t POSIX_CALL_INVALID_ERRNO = -1;

template <typename ReturnType, typename... FunctionArguments>
class PosixCallBuilder;

/// @brief result of a posix call
template <typename T>
struct PosixCallResult {
    PosixCallResult() noexcept = default;

    /// @brief the return value of the posix function call
    T value {};

    /// @brief the errno value which was set by the posix function call
    int32_t errnum = POSIX_CALL_INVALID_ERRNO;
};

namespace detail {
template <typename ReturnType, typename... FunctionArguments>
PosixCallBuilder<ReturnType, FunctionArguments...>
createPosixCallBuilder(ReturnType (*IOX2_POSIX_CALL)(FunctionArguments...),
                       const char* posixFunctionName,
                       const char* file,
                       const int32_t line,
                       const char* callingFunction) noexcept;

template <typename ReturnType>
struct PosixCallDetails {
    PosixCallDetails(const char* posixFunctionName, const char* file, int line, const char* callingFunction) noexcept;
    const char* posixFunctionName = nullptr;
    const char* file = nullptr;
    const char* callingFunction = nullptr;
    int32_t line = 0;
    bool hasSuccess = true;
    bool hasIgnoredErrno = false;
    bool hasSilentErrno = false;

    PosixCallResult<ReturnType> result;
};
} // namespace detail

/// @brief class which is created by the verificator to evaluate the result of a posix call
template <typename ReturnType>
class IOX2_NO_DISCARD PosixCallEvaluator {
  public:
    /// @brief ignore specified errnos from the evaluation
    /// @tparam IgnoredErrnos a list of int32_t variables
    /// @param[in] ignoredErrnos the int32_t values of the errnos which should be ignored
    /// @return a PosixCallEvaluator for further setup of the evaluation
    template <typename... IgnoredErrnos>
    PosixCallEvaluator<ReturnType> ignoreErrnos(const IgnoredErrnos... ignoredErrnos) const&& noexcept;

    /// @brief silence specified errnos from printing error messages in the evaluation
    /// @tparam SilentErrnos a list of int32_t variables
    /// @param[in] silentErrnos the int32_t values of the errnos which should be silent and not cause an error log
    /// @return a PosixCallEvaluator for further setup of the evaluation
    template <typename... SilentErrnos>
    PosixCallEvaluator<ReturnType> suppressErrorMessagesForErrnos(const SilentErrnos... silentErrnos) const&& noexcept;

    /// @brief evaluate the result of a posix call
    /// @return returns an expected which contains in both cases a PosixCallResult<ReturnType> with the return value
    /// (.value) and the errno value (.errnum) of the function call
    expected<PosixCallResult<ReturnType>, PosixCallResult<ReturnType>> evaluate() const&& noexcept;

  private:
    template <typename>
    friend class PosixCallVerificator;

    explicit PosixCallEvaluator(detail::PosixCallDetails<ReturnType>& details) noexcept;

  private:
    // NOLINTJUSTIFICATION refences are intentionally used since the class does not need to be assignable
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-const-or-ref-data-members)
    detail::PosixCallDetails<ReturnType>& m_details;
};

/// @brief class which verifies the return value of a posix function call
template <typename ReturnType>
class IOX2_NO_DISCARD PosixCallVerificator {
  public:
    /// @brief the posix function call defines success through a single value
    /// @param[in] successReturnValues a list of values which define success
    /// @return the PosixCallEvaluator which evaluates the errno values
    template <typename... SuccessReturnValues>
    PosixCallEvaluator<ReturnType> successReturnValue(const SuccessReturnValues... successReturnValues) && noexcept;

    /// @brief the posix function call defines failure through a single value
    /// @param[in] failureReturnValues a list of values which define failure
    /// @return the PosixCallEvaluator which evaluates the errno values
    template <typename... FailureReturnValues>
    PosixCallEvaluator<ReturnType> failureReturnValue(const FailureReturnValues... failureReturnValues) && noexcept;

    /// @brief the posix function call defines failure through return of the errno value instead of setting the errno
    /// @return the PosixCallEvaluator which evaluates the errno values
    PosixCallEvaluator<ReturnType> returnValueMatchesErrno() && noexcept;

  private:
    template <typename, typename...>
    friend class PosixCallBuilder;

    explicit PosixCallVerificator(detail::PosixCallDetails<ReturnType>& details) noexcept;

  private:
    // NOLINTJUSTIFICATION refences are intentionally used since the class does not need to be assignable
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-const-or-ref-data-members)
    detail::PosixCallDetails<ReturnType>& m_details;
};

template <typename ReturnType, typename... FunctionArguments>
class IOX2_NO_DISCARD PosixCallBuilder {
  public:
    /// @brief input function type
    using FunctionType_t = ReturnType (*)(FunctionArguments...);

    /// @brief Call the underlying function with the provided arguments. If the underlying function fails and sets the
    /// errno to EINTR the call is repeated at most POSIX_CALL_EINTR_REPETITIONS times
    /// @param[in] arguments arguments which will be provided to the posix function
    /// @return the PosixCallVerificator to verify the return value
    PosixCallVerificator<ReturnType> operator()(FunctionArguments... arguments) && noexcept;

  private:
    template <typename ReturnTypeFriend, typename... FunctionArgumentsFriend>
    friend PosixCallBuilder<ReturnTypeFriend, FunctionArgumentsFriend...>
    detail::createPosixCallBuilder(ReturnTypeFriend (*IOX2_POSIX_CALL)(FunctionArgumentsFriend...),
                                   const char* posixFunctionName,
                                   const char* file,
                                   const int32_t line,
                                   const char* callingFunction) noexcept;

    PosixCallBuilder(FunctionType_t IOX2_POSIX_CALL,
                     const char* posixFunctionName,
                     const char* file,
                     const int32_t line,
                     const char* callingFunction) noexcept;

  private:
    FunctionType_t m_IOX2_POSIX_CALL = nullptr;
    detail::PosixCallDetails<ReturnType> m_details;
};
} // namespace legacy
} // namespace iox2

#include "iox2/legacy/detail/posix_call.inl"

/// @brief Calling a posix function with automated error handling. If the posix function returns
///        void you do not need to use 'IOX2_POSIX_CALL' since it cannot fail, (see: man errno).
///        We use a builder pattern to create a design which sets the usage contract so that it
///        cannot be used in the wrong way.
/// @code
///        IOX2_POSIX_CALL(sem_timedwait)(handle, timeout)
///             .successReturnValue(0)
///             .ignoreErrnos(ETIMEDOUT) // can be a comma separated list of errnos
///             .evaluate()
///             .and_then([](auto & result){
///                 IOX2_LOG(Info, result.value); // return value of sem_timedwait
///                 IOX2_LOG(Info, result.errno); // errno which was set by sem_timedwait
///             })
///             .or_else([](auto & result){
///                 IOX2_LOG(Info, result.value); // return value of sem_timedwait
///                 IOX2_LOG(Info, result.errno); // errno which was set by sem_timedwait
///             })
///
///        // when your posix call signals failure with one specific return value use
///        // .failureReturnValue(_) instead of .successReturnValue(_)
///        // when your posix call signals failure by returning the errno value instead of setting the errno use
///        // .returnValueMatchesErrno() instead of .successReturnValue(_)
/// @endcode
// NOLINTJUSTIFICATION a template or constexpr function does not have access to source code origin file, line, function
//                     name
// NOLINTNEXTLINE(cppcoreguidelines-macro-usage)
#define IOX2_POSIX_CALL(f)                                                                                             \
    iox2::legacy::detail::createPosixCallBuilder(                                                                      \
        &(f),                                                                                                          \
        (#f),                                                                                                          \
        __FILE__,                                                                                                      \
        __LINE__,                                                                                                      \
        __PRETTY_FUNCTION__) // NOLINT(cppcoreguidelines-pro-bounds-array-to-pointer-decay,hicpp-no-array-decay)
// needed for source code location, safely wrapped in macro

#endif // IOX2_BB_POSIX_DESIGN_POSIX_CALL_HPP
