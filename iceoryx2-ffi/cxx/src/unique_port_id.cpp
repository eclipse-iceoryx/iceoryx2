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

#include "iox2/unique_port_id.hpp"

namespace iox2 {
UniquePublisherId::UniquePublisherId(UniquePublisherId&& rhs) noexcept {
    *this = std::move(rhs);
}

auto UniquePublisherId::operator=(UniquePublisherId&& rhs) noexcept -> UniquePublisherId& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

UniquePublisherId::~UniquePublisherId() {
    drop();
}

auto operator==(const UniquePublisherId& lhs, const UniquePublisherId& rhs) -> bool {
    auto* lhs_ref = iox2_cast_unique_publisher_id_ref_h(lhs.m_handle);
    auto* rhs_ref = iox2_cast_unique_publisher_id_ref_h(rhs.m_handle);
    return iox2_unique_publisher_id_eq(lhs_ref, rhs_ref);
}

auto operator<(const UniquePublisherId& lhs, const UniquePublisherId& rhs) -> bool {
    auto* lhs_ref = iox2_cast_unique_publisher_id_ref_h(lhs.m_handle);
    auto* rhs_ref = iox2_cast_unique_publisher_id_ref_h(rhs.m_handle);
    return iox2_unique_publisher_id_less(lhs_ref, rhs_ref);
}

UniquePublisherId::UniquePublisherId(iox2_unique_publisher_id_h handle)
    : m_handle { handle } {
}

void UniquePublisherId::drop() {
    if (m_handle != nullptr) {
        iox2_unique_publisher_id_drop(m_handle);
        m_handle = nullptr;
    }
}

UniqueSubscriberId::UniqueSubscriberId(UniqueSubscriberId&& rhs) noexcept {
    *this = std::move(rhs);
}

auto UniqueSubscriberId::operator=(UniqueSubscriberId&& rhs) noexcept -> UniqueSubscriberId& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

UniqueSubscriberId::~UniqueSubscriberId() {
    drop();
}

auto operator==(const UniqueSubscriberId& lhs, const UniqueSubscriberId& rhs) -> bool {
    auto* lhs_ref = iox2_cast_unique_subscriber_id_ref_h(lhs.m_handle);
    auto* rhs_ref = iox2_cast_unique_subscriber_id_ref_h(rhs.m_handle);
    return iox2_unique_subscriber_id_eq(lhs_ref, rhs_ref);
}

auto operator<(const UniqueSubscriberId& lhs, const UniqueSubscriberId& rhs) -> bool {
    auto* lhs_ref = iox2_cast_unique_subscriber_id_ref_h(lhs.m_handle);
    auto* rhs_ref = iox2_cast_unique_subscriber_id_ref_h(rhs.m_handle);
    return iox2_unique_subscriber_id_less(lhs_ref, rhs_ref);
}

UniqueSubscriberId::UniqueSubscriberId(iox2_unique_subscriber_id_h handle)

    : m_handle { handle } {
}

void UniqueSubscriberId::drop() {
    if (m_handle != nullptr) {
        iox2_unique_subscriber_id_drop(m_handle);
        m_handle = nullptr;
    }
}

UniqueNotifierId::UniqueNotifierId(UniqueNotifierId&& rhs) noexcept {
    *this = std::move(rhs);
}

auto UniqueNotifierId::operator=(UniqueNotifierId&& rhs) noexcept -> UniqueNotifierId& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

UniqueNotifierId::~UniqueNotifierId() {
    drop();
}

auto operator==(const UniqueNotifierId& lhs, const UniqueNotifierId& rhs) -> bool {
    auto* lhs_ref = iox2_cast_unique_notifier_id_ref_h(lhs.m_handle);
    auto* rhs_ref = iox2_cast_unique_notifier_id_ref_h(rhs.m_handle);
    return iox2_unique_notifier_id_eq(lhs_ref, rhs_ref);
}

auto operator<(const UniqueNotifierId& lhs, const UniqueNotifierId& rhs) -> bool {
    auto* lhs_ref = iox2_cast_unique_notifier_id_ref_h(lhs.m_handle);
    auto* rhs_ref = iox2_cast_unique_notifier_id_ref_h(rhs.m_handle);
    return iox2_unique_notifier_id_less(lhs_ref, rhs_ref);
}

UniqueNotifierId::UniqueNotifierId(iox2_unique_notifier_id_h handle)
    : m_handle { handle } {
}

void UniqueNotifierId::drop() {
    if (m_handle != nullptr) {
        iox2_unique_notifier_id_drop(m_handle);
        m_handle = nullptr;
    }
}

UniqueListenerId::UniqueListenerId(UniqueListenerId&& rhs) noexcept {
    *this = std::move(rhs);
}

auto UniqueListenerId::operator=(UniqueListenerId&& rhs) noexcept -> UniqueListenerId& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

UniqueListenerId::~UniqueListenerId() {
    drop();
}

auto operator==(const UniqueListenerId& lhs, const UniqueListenerId& rhs) -> bool {
    auto* lhs_ref = iox2_cast_unique_listener_id_ref_h(lhs.m_handle);
    auto* rhs_ref = iox2_cast_unique_listener_id_ref_h(rhs.m_handle);
    return iox2_unique_listener_id_eq(lhs_ref, rhs_ref);
}

auto operator<(const UniqueListenerId& lhs, const UniqueListenerId& rhs) -> bool {
    auto* lhs_ref = iox2_cast_unique_listener_id_ref_h(lhs.m_handle);
    auto* rhs_ref = iox2_cast_unique_listener_id_ref_h(rhs.m_handle);
    return iox2_unique_listener_id_less(lhs_ref, rhs_ref);
}

UniqueListenerId::UniqueListenerId(iox2_unique_listener_id_h handle)
    : m_handle { handle } {
}

void UniqueListenerId::drop() {
    if (m_handle != nullptr) {
        iox2_unique_listener_id_drop(m_handle);
        m_handle = nullptr;
    }
}
} // namespace iox2
