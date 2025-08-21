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

#include "iox2/dynamic_config_publish_subscribe.hpp"
#include "iox2/internal/callback_context.hpp"

namespace iox2 {
auto DynamicConfigPublishSubscribe::number_of_publishers() const -> uint64_t {
    return iox2_port_factory_pub_sub_dynamic_config_number_of_publishers(&m_handle);
}

auto DynamicConfigPublishSubscribe::number_of_subscribers() const -> uint64_t {
    return iox2_port_factory_pub_sub_dynamic_config_number_of_subscribers(&m_handle);
}

DynamicConfigPublishSubscribe::DynamicConfigPublishSubscribe(iox2_port_factory_pub_sub_h handle)
    : m_handle { handle } {
}

void DynamicConfigPublishSubscribe::list_publishers(
    const iox::function<CallbackProgression(PublisherDetailsView)>& callback) const {
    auto ctx = internal::ctx(callback);
    iox2_port_factory_pub_sub_dynamic_config_list_publishers(
        &m_handle,
        internal::list_ports_callback<iox2_publisher_details_ptr, PublisherDetailsView>,
        static_cast<void*>(&ctx));
}

void DynamicConfigPublishSubscribe::list_subscribers(
    const iox::function<CallbackProgression(SubscriberDetailsView)>& callback) const {
    auto ctx = internal::ctx(callback);
    iox2_port_factory_pub_sub_dynamic_config_list_subscribers(
        &m_handle,
        internal::list_ports_callback<iox2_subscriber_details_ptr, SubscriberDetailsView>,
        static_cast<void*>(&ctx));
}
} // namespace iox2
