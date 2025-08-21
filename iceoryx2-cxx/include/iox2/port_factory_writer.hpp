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

#ifndef IOX2_PORTFACTORY_WRITER_HPP
#define IOX2_PORTFACTORY_WRITER_HPP

#include "iox/assertions_addendum.hpp"
#include "iox/expected.hpp"
#include "iox2/service_type.hpp"
#include "iox2/writer.hpp"
#include "iox2/writer_error.hpp"

namespace iox2 {
/// Factory to create a new [`Writer`] port/endpoint for [`MessagingPattern::Blackboard`]
/// based communication.
template <ServiceType S, typename KeyType>
class PortFactoryWriter {
  public:
    PortFactoryWriter(PortFactoryWriter&&) noexcept = default;
    auto operator=(PortFactoryWriter&&) noexcept -> PortFactoryWriter& = default;
    ~PortFactoryWriter() = default;

    PortFactoryWriter(const PortFactoryWriter&) = delete;
    auto operator=(const PortFactoryWriter&) -> PortFactoryWriter& = delete;

    /// Creates a new [`Writer`] port or returns a [`WriterCreateError`] on failure.
    auto create() && -> iox::expected<Writer<S, KeyType>, WriterCreateError>;

  private:
    template <ServiceType, typename>
    friend class PortFactoryBlackboard;

    explicit PortFactoryWriter(/*iox2_port_factory_writer_builder_h handle*/);

    // iox2_port_factory_writer_builder_h m_handle = nullptr;
};

template <ServiceType S, typename KeyType>
inline PortFactoryWriter<S, KeyType>::PortFactoryWriter(/*iox2_port_factory_writer_builder_h handle*/) {
    IOX_TODO();
}

template <ServiceType S, typename KeyType>
inline auto PortFactoryWriter<S, KeyType>::create() && -> iox::expected<Writer<S, KeyType>, WriterCreateError> {
    IOX_TODO();
}
} // namespace iox2

#endif
