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

#ifndef IOX2_SERVICE_HASH_HPP
#define IOX2_SERVICE_HASH_HPP

#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {
/// Represents the unique if of a [`Service`].
class ServiceHash {
  public:
    /// Returns the maximum string length of a [`ServiceHash`]
    auto max_number_of_characters() -> uint64_t;

    /// Returns the string value of the [`ServiceHash`]
    auto c_str() const -> const char*;

  private:
    explicit ServiceHash(const iox2::bb::StaticString<IOX2_SERVICE_HASH_LENGTH>& value);

    template <ServiceType>
    friend class PortFactoryEvent;
    template <ServiceType, typename, typename>
    friend class PortFactoryPublishSubscribe;
    template <ServiceType, typename, typename, typename, typename>
    friend class PortFactoryRequestResponse;
    template <ServiceType, typename>
    friend class PortFactoryBlackboard;

    iox2::bb::StaticString<IOX2_SERVICE_HASH_LENGTH> m_value;
};

} // namespace iox2

#endif
