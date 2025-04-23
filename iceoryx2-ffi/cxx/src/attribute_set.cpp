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
#include "iox/uninitialized_array.hpp"
#include "iox2/internal/callback_context.hpp"

namespace iox2 {
namespace {
auto get_key_values_callback(const char* value, iox2_callback_context context) -> iox2_callback_progression_e {
    auto* callback = internal::ctx_cast<iox::function<CallbackProgression(const Attribute::Value&)>>(context);
    auto typed_value = Attribute::Value(iox::TruncateToCapacity, value);
    return iox::into<iox2_callback_progression_e>(callback->value()(typed_value));
}
} // namespace

/////////////////////////////
/// BEGIN: AttributeSetView
/////////////////////////////

AttributeSetView::AttributeSetView(iox2_attribute_set_ptr handle)
    : m_handle { handle } {
}

auto AttributeSetView::len() const -> uint64_t {
    return iox2_attribute_set_len(m_handle);
}

auto AttributeSetView::at(const uint64_t index) const -> AttributeView {
    return AttributeView(iox2_attribute_set_at(m_handle, index));
}

auto AttributeSetView::get_key_value_len(const Attribute::Key& key) const -> uint64_t {
    return iox2_attribute_set_get_key_value_len(m_handle, key.c_str());
}

auto AttributeSetView::get_key_value_at(const Attribute::Key& key, const uint64_t idx)
    -> iox::optional<Attribute::Value> {
    iox::UninitializedArray<char, Attribute::Value::capacity()> buffer;
    iox2_attribute_set_get_key_value_at(m_handle, key.c_str(), idx, &buffer[0], Attribute::Value::capacity());

    if (buffer[0] == 0) {
        return iox::nullopt;
    }

    return Attribute::Value(iox::TruncateToCapacity, &buffer[0]);
}

void AttributeSetView::get_key_values(
    const Attribute::Key& key, const iox::function<CallbackProgression(const Attribute::Value&)>& callback) const {
    auto ctx = internal::ctx(callback);
    iox2_attribute_set_get_key_values(m_handle, key.c_str(), get_key_values_callback, static_cast<void*>(&ctx));
}

auto AttributeSetView::to_owned() const -> AttributeSet {
    iox2_attribute_set_h handle = nullptr;
    iox2_attribute_set_new_clone(nullptr, m_handle, &handle);
    return AttributeSet(handle);
}

/////////////////////////////
/// END: AttributeSetView
/////////////////////////////

/////////////////////////////
/// BEGIN: AttributeSet
/////////////////////////////
AttributeSet::AttributeSet(iox2_attribute_set_h handle)
    : m_handle { handle }
    , m_view { AttributeSetView(iox2_cast_attribute_set_ptr(handle)) } {
}

AttributeSet::AttributeSet(AttributeSet&& rhs) noexcept
    : m_handle { std::move(rhs.m_handle) }
    , m_view { std::move(rhs.m_view) } {
    rhs.m_handle = nullptr;
    rhs.m_view.m_handle = nullptr;
}

auto AttributeSet::operator=(AttributeSet&& rhs) noexcept -> AttributeSet& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        m_view = std::move(rhs.m_view);

        rhs.m_handle = nullptr;
        rhs.m_view.m_handle = nullptr;
    }

    return *this;
}

AttributeSet::~AttributeSet() {
    drop();
}

void AttributeSet::drop() {
    if (m_handle != nullptr) {
        iox2_attribute_set_drop(m_handle);

        m_handle = nullptr;
        m_view.m_handle = nullptr;
    }
}

auto AttributeSet::len() const -> uint64_t {
    return m_view.len();
}

auto AttributeSet::at(const uint64_t index) const -> AttributeView {
    return m_view.at(index);
}

auto AttributeSet::get_key_value_len(const Attribute::Key& key) const -> uint64_t {
    return m_view.get_key_value_len(key);
}

auto AttributeSet::get_key_value_at(const Attribute::Key& key, const uint64_t idx) -> iox::optional<Attribute::Value> {
    return m_view.get_key_value_at(key, idx);
}

void AttributeSet::get_key_values(const Attribute::Key& key,
                                  const iox::function<CallbackProgression(const Attribute::Value&)>& callback) const {
    m_view.get_key_values(key, callback);
}
/////////////////////////////
/// END: AttributeSet
/////////////////////////////
} // namespace iox2

auto operator<<(std::ostream& stream, const iox2::AttributeSetView& value) -> std::ostream& {
    stream << "AttributeSetView { ";
    for (uint64_t idx = 0; idx < value.len(); ++idx) {
        auto attribute = value.at(idx);
        stream << attribute;
    }
    return stream;
}

auto operator<<(std::ostream& stream, const iox2::AttributeSet& value) -> std::ostream& {
    stream << "AttributeSet { ";
    for (uint64_t idx = 0; idx < value.len(); ++idx) {
        auto attribute = value.at(idx);
        stream << attribute;
    }
    return stream;
}
