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

#ifndef IOX2_SERVICE_ID_HPP
#define IOX2_SERVICE_ID_HPP

#include "iox/string.hpp"
#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {
/// Represents the unique if of a [`Service`].
class ServiceId {
  public:
    /// Returns the maximum string length of a [`ServiceId`]
    auto max_number_of_characters() -> uint64_t;

    /// Returns the string value of the [`ServiceId`]
    auto c_str() const -> const char*;

  private:
    explicit ServiceId(const iox::string<IOX2_SERVICE_ID_LENGTH>& value);

    template <ServiceType>
    friend class PortFactoryEvent;
    template <ServiceType, typename, typename>
    friend class PortFactoryPublishSubscribe;
    template <ServiceType, typename, typename, typename, typename>
    friend class PortFactoryRequestResponse;

    iox::string<IOX2_SERVICE_ID_LENGTH> m_value;
};

} // namespace iox2

#endif
