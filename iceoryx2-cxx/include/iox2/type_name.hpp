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

#ifndef IOX2_TYPE_NAME_HPP
#define IOX2_TYPE_NAME_HPP

#include "iox2/bb/static_string.hpp"
#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {

using TypeName = iox2::bb::StaticString<IOX2_TYPE_NAME_LENGTH>;

/// Customization point that assigns an externally controlled type identity to a
/// type which cannot carry an `IOX2_TYPE_NAME` member, e.g. a type emitted by an
/// IDL/code generator whose output must not be edited.
///
/// A type must not define both a `TypeNameSpecialization` and an
/// `IOX2_TYPE_NAME` member; doing so is a compile error.
template <typename T>
struct TypeNameSpecialization;

} // namespace iox2

/// Specializes `iox2::TypeNameSpecialization` for `Type` so that `Type` uses
/// `NameExpr` as its iceoryx2 type identity.
// NOLINTNEXTLINE(cppcoreguidelines-macro-usage) : a function template cannot specialize from arbitrary scopes
#define IOX2_DEFINE_TYPE_NAME(Type, NameExpr)                                                                          \
    template <>                                                                                                        \
    struct iox2::TypeNameSpecialization<Type> {                                                                        \
        static auto value() -> const char* {                                                                           \
            return (NameExpr);                                                                                         \
        }                                                                                                              \
    }

#endif
