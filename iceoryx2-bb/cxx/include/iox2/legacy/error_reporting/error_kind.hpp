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

#ifndef IOX2_BB_REPORTING_ERROR_REPORTING_ERROR_KIND_HPP
#define IOX2_BB_REPORTING_ERROR_REPORTING_ERROR_KIND_HPP

#include <type_traits>

namespace iox2 {
namespace legacy {
namespace er {

// Tag types for mandatory fatal error categories that always exist.
// They have the suffix "Kind" to allow using the prefix as the actual error type.
// The split between error type and error kind is intentional to allow emitting the same error
// type with a different kind if needed, similar to categorical logging.
//
// In addition, this has the advantage to be more explicit at reporting site instead of hiding
// the information in a function name or the error itself.

struct FatalKind {
    static constexpr char const* name = "Fatal Error";
};

struct AssertViolationKind {
    static constexpr char const* name = "Assert Violation";
};

struct EnforceViolationKind {
    static constexpr char const* name = "Enforce Violation";
};

template <class T>
struct IsFatal : public std::false_type {
    /// @todo iox-#1032 shouldn't there be a static_assert to prevent using this struct in a generic way without
    /// specialization?
};

// This specialization makes it impossible to specialize them differently elsewhere,
// as this would lead to a compilation error.
// This enforces that these errors are always fatal in the sense that they cause panic and abort.
template <>
struct IsFatal<FatalKind> : public std::true_type { };

template <>
struct IsFatal<AssertViolationKind> : public std::true_type { };

template <>
struct IsFatal<EnforceViolationKind> : public std::true_type { };

// The function syntax is more useful if there is already a value (instead of only a type).
// It must be consistent with the type trait, i.e. yield the same boolean value.
template <class Kind>
bool constexpr isFatal(Kind) {
    return IsFatal<Kind>::value;
}

template <>
bool constexpr isFatal<FatalKind>(FatalKind) {
    return IsFatal<FatalKind>::value;
}

template <>
bool constexpr isFatal<AssertViolationKind>(AssertViolationKind) {
    return IsFatal<AssertViolationKind>::value;
}

template <>
bool constexpr isFatal<EnforceViolationKind>(EnforceViolationKind) {
    return IsFatal<EnforceViolationKind>::value;
}

// indicates serious condition, unable to continue
constexpr FatalKind FATAL;

// indicates a bug (check only active in debug builds)
constexpr AssertViolationKind ASSERT_VIOLATION;

// indicates a bug (check always active)
constexpr EnforceViolationKind ENFORCE_VIOLATION;

} // namespace er
} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_REPORTING_ERROR_REPORTING_ERROR_LOGGING_HPP
