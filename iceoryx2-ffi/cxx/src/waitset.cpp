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

#include "iox2/waitset.hpp"

namespace iox2 {
////////////////////////////
// BEGIN: WaitSetBuilder
////////////////////////////
WaitSetBuilder::WaitSetBuilder()
    : m_handle([] {
        iox2_waitset_builder_h handle {};
        iox2_waitset_builder_new(nullptr, &handle);
        return handle;
    }()) {
}

WaitSetBuilder::~WaitSetBuilder() {
    iox2_waitset_builder_drop(m_handle);
}

template <ServiceType S>
auto WaitSetBuilder::create() const&& -> iox::expected<WaitSet<S>, WaitSetCreateError> {
    iox2_waitset_h waitset_handle {};
    auto result = iox2_waitset_builder_create(m_handle, iox::into<iox2_service_type_e>(S), nullptr, &waitset_handle);

    if (result == IOX2_OK) {
        return iox::ok(WaitSet<S>(waitset_handle));
    }

    return iox::err(iox::into<WaitSetCreateError>(result));
}
////////////////////////////
// END: WaitSetBuilder
////////////////////////////

////////////////////////////
// BEGIN: WaitSet
////////////////////////////
template <ServiceType S>
WaitSet<S>::WaitSet(iox2_waitset_h handle)
    : m_handle { handle } {
}

template <ServiceType S>
WaitSet<S>::WaitSet(WaitSet&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S>
auto WaitSet<S>::operator=(WaitSet&& rhs) noexcept -> WaitSet& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S>
WaitSet<S>::~WaitSet() {
    drop();
}

template <ServiceType S>
void WaitSet<S>::drop() {
    if (m_handle != nullptr) {
        iox2_waitset_drop(m_handle);
        m_handle = nullptr;
    }
}

template <ServiceType S>
auto WaitSet<S>::capacity() const -> uint64_t {
    return iox2_waitset_capacity(&m_handle);
}

template <ServiceType S>
auto WaitSet<S>::len() const -> uint64_t {
    return iox2_waitset_len(&m_handle);
}

template <ServiceType S>
auto WaitSet<S>::is_empty() const -> bool {
    return iox2_waitset_is_empty(&m_handle);
}
////////////////////////////
// END: WaitSet
////////////////////////////

template class WaitSet<ServiceType::Ipc>;
template class WaitSet<ServiceType::Local>;

template auto WaitSetBuilder::create() const&& -> iox::expected<WaitSet<ServiceType::Ipc>, WaitSetCreateError>;
template auto WaitSetBuilder::create() const&& -> iox::expected<WaitSet<ServiceType::Local>, WaitSetCreateError>;
} // namespace iox2
