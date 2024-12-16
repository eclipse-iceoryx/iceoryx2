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

#ifndef IOX2_ATTRIBUTE_SET_HPP
#define IOX2_ATTRIBUTE_SET_HPP

#include "iox/function.hpp"
#include "iox2/attribute.hpp"
#include "iox2/internal/iceoryx2.hpp"

#include <iostream>

namespace iox2 {
class AttributeSetView {
  public:
    auto len() const -> uint64_t;
    auto at(uint64_t index) const -> AttributeView;
    void get_key_values(const Attribute::Key& key,
                        const iox::function<CallbackProgression(const Attribute::Value&)>& callback) const;

  private:
    friend class AttributeVerifier;
    friend class AttributeSpecifier;

    explicit AttributeSetView(iox2_attribute_set_h_ref handle);

    iox2_attribute_set_h_ref m_handle;
};
} // namespace iox2

auto operator<<(std::ostream& stream, const iox2::AttributeSetView& value) -> std::ostream&;

#endif
