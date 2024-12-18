// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

#ifndef IOX2_ATTRIBUTE_HPP
#define IOX2_ATTRIBUTE_HPP

#include "iox/string.hpp"
#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {
/// Represents a single service attribute (key-value) pair that can be defined when the service
/// is being created.
class Attribute {
  public:
    using Key = iox::string<IOX2_ATTRIBUTE_KEY_LENGTH>;
    using Value = iox::string<IOX2_ATTRIBUTE_VALUE_LENGTH>;
};

/// Represents a single view service attribute (key-value) pair that can be defined when the service
/// is being created.
///
/// @attention The parent from which the view was extracted MUST live longer than the
///            [`AttributeView`].
class AttributeView {
  public:
    /// Acquires the service attribute key
    auto key() const -> Attribute::Key;

    /// Acquires the service attribute value
    auto value() const -> Attribute::Value;

  private:
    friend class AttributeSetView;
    explicit AttributeView(iox2_attribute_h_ref handle);

    iox2_attribute_h_ref m_handle = nullptr;
};
} // namespace iox2
  //
auto operator<<(std::ostream& stream, const iox2::AttributeView& value) -> std::ostream&;

#endif
