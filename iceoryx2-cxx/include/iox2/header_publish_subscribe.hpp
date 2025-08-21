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

#ifndef IOX2_HEADER_PUBLISH_SUBSCRIBE_HPP
#define IOX2_HEADER_PUBLISH_SUBSCRIBE_HPP

#include "iox2/internal/iceoryx2.hpp"
#include "unique_port_id.hpp"

namespace iox2 {
/// Sample header used by [`MessagingPattern::PublishSubscribe`]
class HeaderPublishSubscribe {
  public:
    HeaderPublishSubscribe(const HeaderPublishSubscribe&) = delete;
    HeaderPublishSubscribe(HeaderPublishSubscribe&& rhs) noexcept;
    auto operator=(const HeaderPublishSubscribe&) -> HeaderPublishSubscribe& = delete;
    auto operator=(HeaderPublishSubscribe&& rhs) noexcept -> HeaderPublishSubscribe&;
    ~HeaderPublishSubscribe();

    /// Returns the [`UniquePublisherId`] of the source [`Publisher`].
    auto publisher_id() const -> UniquePublisherId;

    /// Returns the number of [`Payload`] elements in the received [`Sample`].
    auto number_of_elements() const -> uint64_t;

  private:
    template <ServiceType, typename, typename>
    friend class Sample;
    template <ServiceType, typename, typename>
    friend class SampleMut;

    explicit HeaderPublishSubscribe(iox2_publish_subscribe_header_h handle);
    void drop();

    iox2_publish_subscribe_header_h m_handle = nullptr;
};
} // namespace iox2

#endif
