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

#ifndef IOX2_PORTFACTORY_READER_HPP
#define IOX2_PORTFACTORY_READER_HPP

#include "iox/assertions_addendum.hpp"
#include "iox/expected.hpp"
#include "iox2/reader.hpp"
#include "iox2/reader_error.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
/// Factory to create a new [`Reader`] port/endpoint for [`MessagingPattern::Blackboard`]
/// based communication.
template <ServiceType S, typename KeyType>
class PortFactoryReader {
  public:
    PortFactoryReader(PortFactoryReader&&) noexcept = default;
    auto operator=(PortFactoryReader&&) noexcept -> PortFactoryReader& = default;
    ~PortFactoryReader() = default;

    PortFactoryReader(const PortFactoryReader&) = delete;
    auto operator=(const PortFactoryReader&) -> PortFactoryReader& = delete;

    /// Creates a new [`Reader`] port or returns a [`ReaderCreateError`] on failure.
    auto create() && -> iox::expected<Reader<S, KeyType>, ReaderCreateError>;

  private:
    template <ServiceType, typename>
    friend class PortFactoryBlackboard;

    explicit PortFactoryReader(/*iox2_port_factory_reader_builder_h handle*/);

    // iox2_port_factory_reader_builder_h m_handle = nullptr;
};

template <ServiceType S, typename KeyType>
inline PortFactoryReader<S, KeyType>::PortFactoryReader(/*iox2_port_factory_reader_builder_h handle*/) {
    IOX_TODO();
}

template <ServiceType S, typename KeyType>
inline auto PortFactoryReader<S, KeyType>::create() && -> iox::expected<Reader<S, KeyType>, ReaderCreateError> {
    IOX_TODO();
}
} // namespace iox2

#endif
