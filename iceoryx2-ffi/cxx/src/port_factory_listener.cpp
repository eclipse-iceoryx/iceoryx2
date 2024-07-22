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

#include "iox2/port_factory_listener.hpp"
#include "iox/assertions_addendum.hpp"

namespace iox2 {
template <ServiceType S>
PortFactoryListener<S>::PortFactoryListener(iox2_port_factory_listener_builder_h handle)
    : m_handle { handle } {
}

template <ServiceType S>
auto PortFactoryListener<S>::create() && -> iox::expected<Listener<S>, ListenerCreateError> {
    IOX_TODO();
}

template class PortFactoryListener<ServiceType::Ipc>;
template class PortFactoryListener<ServiceType::Local>;
} // namespace iox2
