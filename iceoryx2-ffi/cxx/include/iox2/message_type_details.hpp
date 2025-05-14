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

#ifndef IOX2_MESSAGE_TYPE_DETAILS_HPP
#define IOX2_MESSAGE_TYPE_DETAILS_HPP

#include "iox2/internal/iceoryx2.hpp"
#include "iox2/type_variant.hpp"


namespace iox2 {
/// Contains all type details required to connect to a
/// [`crate::service::Service`]
class TypeDetail {
  public:
    /// The [`TypeVariant`] of the type
    auto variant() const -> TypeVariant;

    /// Contains the output of [`typeid().name`].
    auto type_name() const -> const char*;

    /// The size of the underlying type.
    auto size() const -> size_t;

    /// The alignment of the underlying type.
    auto alignment() const -> size_t;

  private:
    friend class MessageTypeDetails;
    explicit TypeDetail(iox2_type_detail_t value);

    iox2_type_detail_t m_value;
};

/// Contains all type information to the header and payload type.
class MessageTypeDetails {
  public:
    /// The [`TypeDetail`] of the header of a message, the first iceoryx2
    /// internal part.
    auto header() const -> TypeDetail;

    /// The [`TypeDetail`] of the user_header or the custom header, is located
    /// directly after the header.
    auto user_header() const -> TypeDetail;

    /// The [`TypeDetail`] of the payload of the message, the last part.
    auto payload() const -> TypeDetail;

  private:
    friend class StaticConfigPublishSubscribe;
    friend class StaticConfigRequestResponse;

    explicit MessageTypeDetails(iox2_message_type_details_t value);

    iox2_message_type_details_t m_value;
};
} // namespace iox2

#endif
