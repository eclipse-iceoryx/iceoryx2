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

#include "iox2/legacy/testing/error_reporting/testing_error_handler.hpp"
#include "iox2/bb/static_function.hpp"
#include "iox2/legacy/error_reporting/custom/default/error_handler.hpp"
#include "iox2/legacy/error_reporting/custom/default/error_handler_interface.hpp"
#include "iox2/legacy/error_reporting/types.hpp"

// NOLINTNEXTLINE(hicpp-deprecated-headers) required to work on some platforms
#include <setjmp.h>

#include <iostream>

namespace iox2 {
namespace legacy {
namespace testing {

using namespace iox2::legacy::er;

void TestingErrorHandler::init() noexcept {
    iox2::legacy::testing::ErrorHandler handler;
    iox2::legacy::er::ErrorHandler::set(handler);

    auto& listeners = ::testing::UnitTest::GetInstance()->listeners();
    // NOLINTNEXTLINE(cppcoreguidelines-owning-memory) required by the callee
    listeners.Append(new (std::nothrow) ErrorHandlerSetup);
}

void TestingErrorHandler::onPanic() {
    m_panicked = true;
    jump();
}

void TestingErrorHandler::onReportError(er::ErrorDescriptor desc) {
    std::lock_guard<std::mutex> g(m_mutex);
    m_errors.push_back(desc);
}

void TestingErrorHandler::onReportViolation(er::ErrorDescriptor desc) {
    std::lock_guard<std::mutex> g(m_mutex);
    m_violations.push_back(desc);
}

bool TestingErrorHandler::hasPanicked() const noexcept {
    return m_panicked.load(std::memory_order_relaxed);
}

void TestingErrorHandler::reset() noexcept {
    std::lock_guard<std::mutex> g(m_mutex);
    m_panicked = false;
    m_errors.clear();
    m_violations.clear();
    m_jumpState.store(JumpState::Obtainable);
}

bool TestingErrorHandler::hasError() const noexcept {
    std::lock_guard<std::mutex> g(m_mutex);
    return !m_errors.empty();
}

bool TestingErrorHandler::hasError(ErrorCode code, iox2::legacy::er::ModuleId module) const noexcept {
    constexpr iox2::legacy::er::ModuleId ANY_MODULE { iox2::legacy::er::ModuleId::ANY };
    std::lock_guard<std::mutex> g(m_mutex);
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

bool TestingErrorHandler::hasViolation(ErrorCode code) const noexcept {
    std::lock_guard<std::mutex> g(m_mutex);
    for (auto desc : m_violations) {
        if (desc.code == code) {
            return true;
        }
    }
    return false;
}

bool TestingErrorHandler::fatalFailureTestContext(const bb::StaticFunction<void()> testFunction) {
    // if there are multiple threads trying to perform a test, only the winner can proceed with the jump
    if (m_jumpState.exchange(JumpState::Pending) == JumpState::Pending) {
        return false;
    };

    // setjmp must be called in a stackframe that still exists when longjmp is called
    // Therefore there cannot be a convenient abstraction that does not also
    // know the test function that is being called.
    // NOLINTNEXTLINE(cert-err52-cpp) exception cannot be used, required for testing to jump in case of failure
    if (setjmp(&(m_jumpBuffer)[0]) != JUMPED_INDICATOR) {
        testFunction();
    }

    return true;
}

void TestingErrorHandler::jump() noexcept {
    if (m_jumpState.load(std::memory_order_relaxed) == JumpState::Pending) {
        // NOLINTNEXTLINE(cert-err52-cpp) exception handling is not used by design
        longjmp(&m_jumpBuffer[0], JUMPED_INDICATOR);
    }
}

void ErrorHandlerSetup::OnTestStart(const ::testing::TestInfo&) {
    ErrorHandler::instance().reset();
}

} // namespace testing
} // namespace legacy
} // namespace iox2
