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

#ifndef IOX2_PORTFACTORY_NOTIFIER_HPP
#define IOX2_PORTFACTORY_NOTIFIER_HPP

#include "iox/builder_addendum.hpp"
#include "iox/expected.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/notifier.hpp"
#include "iox2/notifier_error.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
/// Factory to create a new [`Notifier`] port/endpoint for [`MessagingPattern::Event`] based
/// communication.
template <ServiceType S>
class PortFactoryNotifier {
  public:
    /// Sets a default [`EventId`] for the [`Notifier`] that is used in
    /// [`Notifier::notify()`]
#ifdef DOXYGEN_MACRO_FIX
    auto default_event_id(const EventId value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(EventId, default_event_id);
#endif

  public:
    PortFactoryNotifier(PortFactoryNotifier&&) noexcept = default;
    auto operator=(PortFactoryNotifier&&) noexcept -> PortFactoryNotifier& = default;
    ~PortFactoryNotifier() = default;

    PortFactoryNotifier(const PortFactoryNotifier&) = delete;
    auto operator=(const PortFactoryNotifier&) -> PortFactoryNotifier& = delete;

    /// Creates a new [`Notifier`] port or returns a [`NotifierCreateError`] on failure.
    auto create() && -> iox::expected<Notifier<S>, NotifierCreateError>;

  private:
    template <ServiceType>
    friend class PortFactoryEvent;

    explicit PortFactoryNotifier(iox2_port_factory_notifier_builder_h handle);

    iox2_port_factory_notifier_builder_h m_handle = nullptr;
};
} // namespace iox2

#endif
