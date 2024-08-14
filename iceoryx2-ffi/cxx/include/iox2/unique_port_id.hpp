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

#ifndef IOX2_UNIQUE_PORT_ID_HPP
#define IOX2_UNIQUE_PORT_ID_HPP

#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {
class UniquePublisherId {
  public:
    auto operator==(const UniquePublisherId& rhs) -> bool;
    auto operator<(const UniquePublisherId& rhs) -> bool;

  private:
    friend class HeaderPublishSubscribe;
    explicit UniquePublisherId(iox2_unique_publisher_id_h handle);
    iox2_unique_publisher_id_h m_handle;
};
class UniqueSubscriberId { };
class UniqueNotifierId { };
class UniqueListenerId { };
} // namespace iox2

#endif
