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

#ifndef IOX2_ATTRIBUTE_ERROR_HPP
#define IOX2_ATTRIBUTE_ERROR_HPP

#include <cstdint>

namespace iox2 {
/// Failures that can occur when the [`AttributeVerifier`] fails the verification.
enum class AttributeVerificationError : uint8_t {
    /// A key defined via [`AttributeVerifier::require_key()`] is missing.
    NonExistingKey,
    /// A key defined via [`AttributeVerifier::require()`] has the wrong value.
    IncompatibleAttribute,
};

/// Failures that can occur when defining [`Attribute`]s with [`AttributeSpecifier::define()`].
enum class AttributeDefinitionError : uint8_t {
    /// The new [`Attribute`] would exceed the maximum supported number of [`Attribute`]s
    ExceedsMaxSupportedAttributes,
};
} // namespace iox2
#endif
