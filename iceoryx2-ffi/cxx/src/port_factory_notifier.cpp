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

#include "iox2/port_factory_notifier.hpp"

namespace iox2 {
template <ServiceType S>
PortFactoryNotifier<S>::PortFactoryNotifier(iox2_port_factory_notifier_builder_h handle)
    : m_handle { handle } {
}

template <ServiceType S>
auto PortFactoryNotifier<S>::create() && -> iox::expected<Notifier<S>, NotifierCreateError> {
    m_default_event_id.and_then(
        [&](auto value) { iox2_port_factory_notifier_builder_set_default_event_id(&m_handle, &value.m_value); });

    iox2_notifier_h notifier_handle {};
    auto result = iox2_port_factory_notifier_builder_create(m_handle, nullptr, &notifier_handle);

    if (result == IOX2_OK) {
        return iox::ok(Notifier<S> { notifier_handle });
    }

    return iox::err(iox::into<NotifierCreateError>(result));
}

template class PortFactoryNotifier<ServiceType::Ipc>;
template class PortFactoryNotifier<ServiceType::Local>;
} // namespace iox2
