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

#ifndef IOX2_STATIC_CONFIG_PUBLISH_SUBSCRIBE_HPP
#define IOX2_STATIC_CONFIG_PUBLISH_SUBSCRIBE_HPP

#include "iox2/attribute_set.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/message_type_details.hpp"

#include <cstdint>

namespace iox2 {
/// The static configuration of an [`MessagingPattern::PublishSubscribe`]
/// based service. Contains all parameters that do not change during the lifetime of a
/// [`Service`].
class StaticConfigPublishSubscribe {
  public:
    /// Returns the maximum supported amount of [`Node`]s that can open the
    /// [`Service`] in parallel.
    auto max_nodes() const -> uint64_t;

    /// Returns the maximum supported amount of [`Publisher`] ports
    auto max_publishers() const -> uint64_t;

    /// Returns the maximum supported amount of [`Subscriber`] ports
    auto max_subscribers() const -> uint64_t;

    /// Returns the maximum history size that can be requested on connect.
    auto history_size() const -> uint64_t;

    /// Returns the maximum supported buffer size for [`Subscriber`] port
    auto subscriber_max_buffer_size() const -> uint64_t;

    /// Returns how many [`Sample`]s a [`Subscriber`] port can borrow in parallel at most.
    auto subscriber_max_borrowed_samples() const -> uint64_t;

    /// Returns true if the [`Service`] safely overflows, otherwise false. Safe
    /// overflow means that the [`Publisher`] will recycle the oldest
    /// [`Sample`] from the [`Subscriber`] when its buffer is full.
    auto has_safe_overflow() const -> bool;

    /// Returns the type details of the [`Service`].
    auto message_type_details() const -> MessageTypeDetails;

  private:
    template <ServiceType, typename, typename>
    friend class PortFactoryPublishSubscribe;

    explicit StaticConfigPublishSubscribe(iox2_static_config_publish_subscribe_t value);

    iox2_static_config_publish_subscribe_t m_value;
};
} // namespace iox2

#endif
