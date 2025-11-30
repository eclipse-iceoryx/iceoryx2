// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_REPORTING_ERROR_REPORTING_LOCATION_HPP
#define IOX2_BB_REPORTING_ERROR_REPORTING_LOCATION_HPP

namespace iox2 {
namespace legacy {
namespace er {
struct SourceLocation {
    constexpr SourceLocation(const char* file, int line, const char* function)
        : file(file)
        , line(line)
        , function(function) {
    }

    const char* file { nullptr };
    int line { 0 };
    const char* function { nullptr };
};

} // namespace er
} // namespace legacy
} // namespace iox2

// NOLINTNEXTLINE(cppcoreguidelines-macro-usage) macro is required for use of location intrinsics (__FILE__ etc.)
#define IOX2_CURRENT_SOURCE_LOCATION                                                                                   \
    iox2::legacy::er::SourceLocation {                                                                                 \
        __FILE__, __LINE__, static_cast<const char*>(__FUNCTION__)                                                     \
    } // NOLINT(cppcoreguidelines-pro-bounds-array-to-pointer-decay,hicpp-no-array-decay)
      // needed for source code location, safely wrapped in macro

#endif // IOX2_BB_REPORTING_ERROR_REPORTING_LOCATION_HPP
