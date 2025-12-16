// Copyright (c) 2019 - 2020 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2020 - 2022 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_ERROR_HANDLING_ERROR_HANDLING_HPP
#define IOX2_BB_ERROR_HANDLING_ERROR_HANDLING_HPP

// Each module (= some unit with its own errors) must provide the following.

// 1. Define the errors of the module -> see below

// 2. Include the custom reporting implementation
#include "iox2/legacy/error_reporting/custom/error_reporting.hpp"

// 3. Include the error reporting macro API
#include "iox2/legacy/error_reporting/macros.hpp"

// additional includes
#include "iox2/legacy/error_reporting/types.hpp"
#include "iox2/legacy/log/logstream.hpp"

namespace iox2 {
namespace legacy {
// clang-format off

// NOLINTJUSTIFICATION This macro is usee to define an enum and an array with corresponding enum tag names
// NOLINTNEXTLINE(cppcoreguidelines-macro-usage)
#define IOX2_BB_ERRORS(error) \
    error(DO_NOT_USE_AS_ERROR_THIS_IS_AN_INTERNAL_MARKER) // keep this always at the end of the error list

// clang-format on

// DO NOT TOUCH THE ENUM, you can doodle around with the lines above!!!

// NOLINTNEXTLINE(performance-enum-size) the type is required for error handling
enum class HoofsError : iox2::legacy::er::ErrorCode::type {
    IOX2_BB_ERRORS(IOX2_CREATE_ERROR_ENUM)
};

static const char* asStringLiteral(const HoofsError error) {
    // NOLINTJUSTIFICATION Use to map enum tag names to strings
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays)
    const char* const BB_ERROR_NAMES[] = { IOX2_BB_ERRORS(IOX2_CREATE_ERROR_STRING) };

    auto end =
        static_cast<std::underlying_type<HoofsError>::type>(HoofsError::DO_NOT_USE_AS_ERROR_THIS_IS_AN_INTERNAL_MARKER);
    auto index = static_cast<std::underlying_type<HoofsError>::type>(error);
    if (index >= end) {
        return "Unknown Error Code!";
    }
    // NOLINTJUSTIFICATION Bounds are checked and access is safe
    // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
    return BB_ERROR_NAMES[index];
}

class HoofsErrorType {
  public:
    explicit HoofsErrorType(HoofsError code)
        : m_code(static_cast<iox2::legacy::er::ErrorCode::type>(code)) {
    }

    static constexpr iox2::legacy::er::ModuleId module() {
        return MODULE_ID;
    }

    iox2::legacy::er::ErrorCode code() const {
        return m_code;
    }

    const char* name() const {
        return asStringLiteral(static_cast<HoofsError>(m_code.value));
    }

    static const char* moduleName() {
        return "iceoryx2-bb-cxx";
    }

    static constexpr iox2::legacy::er::ModuleId MODULE_ID { iox2::legacy::er::ModuleId::IOX2_BB };

  protected:
    iox2::legacy::er::ErrorCode m_code;
};

namespace er {

inline HoofsErrorType toError(HoofsError code) {
    return HoofsErrorType(code);
}

inline ModuleId toModule(HoofsError) {
    return HoofsErrorType::MODULE_ID;
}

} // namespace er

} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_ERROR_HANDLING_ERROR_HANDLING_HPP
