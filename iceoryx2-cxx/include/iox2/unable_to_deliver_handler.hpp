// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

#ifndef IOX2_UNABLE_TO_DELIVER_HANDLER_HPP
#define IOX2_UNABLE_TO_DELIVER_HANDLER_HPP

#include "iox2/bb/duration.hpp"
#include "iox2/bb/static_function.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/unable_to_deliver_action.hpp"
#include "iox2/unique_port_id.hpp"

namespace iox2 {
namespace detail {
// forward declaration in order to make it a friend in UnableToDeliverInfo
inline auto unable_to_deliver_handler_delegate(iox2_unable_to_deliver_info_h_ref info_handle,
                                               iox2_callback_context callback_cxt) -> iox2_unable_to_deliver_action_e;
} // namespace detail

/// The info passed to the [`UnableToDeliverHandler`]
class UnableToDeliverInfo {
  private:
    iox2_unable_to_deliver_info_h_ref m_info;

    explicit UnableToDeliverInfo(iox2_unable_to_deliver_info_h_ref info)
        : m_info(info) {
    }

    friend auto iox2::detail::unable_to_deliver_handler_delegate(iox2_unable_to_deliver_info_h_ref info_handle,
                                                                 iox2_callback_context callback_cxt)
        -> iox2_unable_to_deliver_action_e;

    static_assert(sizeof(iox2_buffer_16_align_4_t::data) == RawIdType::capacity(),
                  "RawIdType capacity must match iox2_buffer_16_align_4_t capacity");

  public:
    /// Returns the ServiceId of the involved ports
    auto service_id() const -> RawIdType {
        iox2_buffer_16_align_4_t buf;
        iox2_unable_to_deliver_info_service_id(m_info, &buf);
        return RawIdType::from_range_unchecked(buf.data).value();
    }
    /// Returns the ReceiverPortId of the involved ports
    auto receiver_port_id() const -> RawIdType {
        iox2_buffer_16_align_4_t buf;
        iox2_unable_to_deliver_info_receiver_port_id(m_info, &buf);
        return RawIdType::from_range_unchecked(buf.data).value();
    }
    /// Returns the ReceiverPortId of the involved ports
    auto sender_port_id() const -> RawIdType {
        iox2_buffer_16_align_4_t buf;
        iox2_unable_to_deliver_info_sender_port_id(m_info, &buf);
        return RawIdType::from_range_unchecked(buf.data).value();
    }
    /// Returns the number retries for the running delivery to the receiver port
    auto retries() const -> uint64_t {
        return iox2_unable_to_deliver_info_retries(m_info);
    }
    /// Returns the elapsed time since the initial retry
    auto elapsed_time() const -> bb::Duration {
        uint64_t seconds = 0;
        uint32_t nanoseconds = 0;
        iox2_unable_to_deliver_info_elapsed_time(m_info, &seconds, &nanoseconds);
        return bb::Duration::create_duration(seconds, nanoseconds);
    }
};

/// The unable to deliver handler invoked when a sample could not be delivered
///
/// @param[in] UnableToDeliverInfo is a reference to [`UnableToDeliverInfo`] with additional information for the user to
/// handle the incident
///
/// @eturn The [`UnableToDeliverAction`] to be taken to mitigate the incident
using UnableToDeliverHandler = iox2::bb::StaticFunction<UnableToDeliverAction(UnableToDeliverInfo&)>;

namespace detail {
inline auto unable_to_deliver_handler_delegate(iox2_unable_to_deliver_info_h_ref info_handle,
                                               iox2_callback_context callback_cxt) -> iox2_unable_to_deliver_action_e {
    auto* callback = static_cast<UnableToDeliverHandler*>(callback_cxt);

    auto info = UnableToDeliverInfo(info_handle);

    return bb::into<iox2_unable_to_deliver_action_e>((*callback)(info));
}
} // namespace detail

} // namespace iox2

#endif // IOX2_UNABLE_TO_DELIVER_HANDLER_HPP
