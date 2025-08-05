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

#ifndef IOX2_STATIC_CONFIG_BLACKBOARD_HPP
#define IOX2_STATIC_CONFIG_BLACKBOARD_HPP

#include "iox2/iceoryx2.h"
#include "iox2/message_type_details.hpp"

namespace iox2 {
/// The static configuration of an [`MessagingPattern::Blackboard`]
/// based service. Contains all parameters that do not change during the lifetime of a
/// [`Service`].
class StaticConfigBlackboard {
  public:
    /// Returns the maximum supported amount of [`Node`]s that can open the
    /// [`Service`] in parallel.
    auto max_nodes() const -> size_t;

    /// Returns the maximum supported amount of [`Reader`] ports
    auto max_readers() const -> size_t;

    /// Returns the type details of the [`Service`].
    auto type_details() const -> TypeDetail&;

  private:
    template <ServiceType, typename>
    friend class PortFactoryBlackboard;

    explicit StaticConfigBlackboard(/*iox2_static_config_blackboard_t value*/);

    // iox2_static_config_blackboard_t m_value;
};
} // namespace iox2

#endif
