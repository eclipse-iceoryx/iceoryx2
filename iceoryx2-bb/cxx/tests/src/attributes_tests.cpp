// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

#include "iox2/bb/detail/attributes.hpp"

#include <gtest/gtest.h>

namespace {

#if defined(_MSVC_LANG) && (_MSVC_LANG > __cplusplus)
constexpr long EXPECTED_CXX_STANDARD_VERSION = _MSVC_LANG;
#else
constexpr long EXPECTED_CXX_STANDARD_VERSION = __cplusplus;
#endif

struct ConstexprDtorProbe {
    ConstexprDtorProbe() = default;
    ConstexprDtorProbe(ConstexprDtorProbe const&) = default;
    ConstexprDtorProbe(ConstexprDtorProbe&&) = default;
    auto operator=(ConstexprDtorProbe const&) -> ConstexprDtorProbe& = default;
    auto operator=(ConstexprDtorProbe&&) -> ConstexprDtorProbe& = default;
    IOX2_CONSTEXPR_DTOR ~ConstexprDtorProbe() = default;
};

#if IOX2_HAS_CONSTEXPR_DTOR
constexpr auto has_constexpr_destructor_support() -> bool {
    ConstexprDtorProbe probe;
    (void) probe;
    return true;
}

static_assert(has_constexpr_destructor_support(),
              "IOX2_CONSTEXPR_DTOR must expand to constexpr when the active compiler mode supports it.");
#endif

TEST(Attributes, cxx_standard_version_uses_the_effective_compiler_setting) {
    EXPECT_EQ(IOX2_CXX_STANDARD_VERSION, EXPECTED_CXX_STANDARD_VERSION);
}

TEST(Attributes, constexpr_dtor_support_flag_matches_the_detected_standard_version) {
    EXPECT_EQ(IOX2_HAS_CONSTEXPR_DTOR, IOX2_CXX_STANDARD_VERSION >= 202002L ? 1 : 0);
}

} // namespace
