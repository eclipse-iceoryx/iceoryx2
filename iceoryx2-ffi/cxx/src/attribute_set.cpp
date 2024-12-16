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

#include "iox2/attribute_set.hpp"
#include "iox2/internal/callback_context.hpp"

namespace iox2 {
AttributeSetView::AttributeSetView(iox2_attribute_set_h_ref handle)
    : m_handle { handle } {
}

auto AttributeSetView::len() const -> uint64_t {
    return iox2_attribute_set_len(m_handle);
}

auto AttributeSetView::at(const uint64_t index) const -> AttributeView {
    return AttributeView(iox2_attribute_set_at(m_handle, index));
}

auto get_key_values_callback(const char* value, iox2_callback_context context) -> iox2_callback_progression_e {
    auto* callback = internal::ctx_cast<iox::function<CallbackProgression(const Attribute::Value&)>>(context);
    auto typed_value = Attribute::Value(iox::TruncateToCapacity, value);
    return iox::into<iox2_callback_progression_e>(callback->value()(typed_value));
}

void AttributeSetView::get_key_values(
    const Attribute::Key& key, const iox::function<CallbackProgression(const Attribute::Value&)>& callback) const {
    auto ctx = internal::ctx(callback);
    iox2_attribute_set_get_key_values(m_handle, key.c_str(), get_key_values_callback, static_cast<void*>(&ctx));
}
} // namespace iox2

auto operator<<(std::ostream& stream, const iox2::AttributeSetView& value) -> std::ostream& {
    stream << "AttributeSet { ";
    for (uint64_t idx = 0; idx < value.len(); ++idx) {
        auto attribute = value.at(idx);
        stream << attribute;
    }
    return stream;
}
