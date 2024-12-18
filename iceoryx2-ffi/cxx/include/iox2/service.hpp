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

#ifndef IOX2_SERVICE_HPP
#define IOX2_SERVICE_HPP

#include "iox/expected.hpp"
#include "iox/function.hpp"
#include "iox/optional.hpp"
#include "iox2/callback_progression.hpp"
#include "iox2/config.hpp"
#include "iox2/messaging_pattern.hpp"
#include "iox2/service_details.hpp"
#include "iox2/service_error_enums.hpp"
#include "iox2/service_name.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
/// Represents a service. Used to create or open new services with the
/// [`crate::node::Node::service_builder()`].
/// Contains the building blocks a [`Service`] requires to create the underlying resources and
/// establish communication.
template <ServiceType S>
class Service {
  public:
    /// Checks if a service under a given [`ConfigView`] does exist.
    static auto does_exist(const ServiceName& service_name, ConfigView config, MessagingPattern messaging_pattern)
        -> iox::expected<bool, ServiceDetailsError>;

    /// Acquires the [`ServiceDetails`] of a [`Service`].
    static auto details(const ServiceName& service_name, ConfigView config, MessagingPattern messaging_pattern)
        -> iox::expected<iox::optional<ServiceDetails<S>>, ServiceDetailsError>;

    /// Returns a list of all services created under a given [`config::Config`].
    static auto list(ConfigView config, const iox::function<CallbackProgression(ServiceDetails<S>)>& callback)
        -> iox::expected<void, ServiceListError>;
};
} // namespace iox2

#endif
