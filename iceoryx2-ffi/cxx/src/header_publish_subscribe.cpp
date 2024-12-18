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

#include "iox2/header_publish_subscribe.hpp"

namespace iox2 {
HeaderPublishSubscribe::HeaderPublishSubscribe(iox2_publish_subscribe_header_h handle)
    : m_handle { handle } {
}

void HeaderPublishSubscribe::drop() {
    if (m_handle != nullptr) {
        iox2_publish_subscribe_header_drop(m_handle);
        m_handle = nullptr;
    }
}

HeaderPublishSubscribe::HeaderPublishSubscribe(HeaderPublishSubscribe&& rhs) noexcept {
    *this = std::move(rhs);
}

auto HeaderPublishSubscribe::operator=(HeaderPublishSubscribe&& rhs) noexcept -> HeaderPublishSubscribe& {
    if (this != &rhs) {
        drop();

        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

HeaderPublishSubscribe::~HeaderPublishSubscribe() {
    drop();
}

auto HeaderPublishSubscribe::publisher_id() const -> UniquePublisherId {
    iox2_unique_publisher_id_h id_handle = nullptr;

    iox2_publish_subscribe_header_publisher_id(&m_handle, nullptr, &id_handle);
    return UniquePublisherId { id_handle };
}

auto HeaderPublishSubscribe::number_of_elements() const -> uint64_t {
    return iox2_publish_subscribe_header_number_of_elements(&m_handle);
}
} // namespace iox2
