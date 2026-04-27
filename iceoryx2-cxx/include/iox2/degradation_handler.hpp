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

#ifndef IOX2_DEGRADATION_HANDLER_HPP
#define IOX2_DEGRADATION_HANDLER_HPP

#include "iox2/bb/static_function.hpp"
#include "iox2/degradation_action.hpp"
#include "iox2/degradation_cause.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/unique_port_id.hpp"

namespace iox2 {
namespace detail {
// forward declaration in order to make it a friend in DegradationInfo
inline auto degradation_handler_delegate(iox2_degradation_cause_e degradation_cause,
                                         iox2_degradation_info_h_ref info_handle,
                                         iox2_callback_context callback_cxt) -> iox2_degradation_action_e;
} // namespace detail

/// The degradation info passed to the [`DegradationHandler`]
class DegradationInfo {
  private:
    iox2_degradation_info_h_ref m_info;

    explicit DegradationInfo(iox2_degradation_info_h_ref info)
        : m_info(info) {
    }

    friend auto iox2::detail::degradation_handler_delegate(iox2_degradation_cause_e degradation_cause,
                                                           iox2_degradation_info_h_ref info_handle,
                                                           iox2_callback_context callback_cxt)
        -> iox2_degradation_action_e;

    static_assert(sizeof(iox2_buffer_16_align_4_t::data) == RawIdType::capacity(),
                  "RawIdType capacity must match iox2_buffer_16_align_4_t capacity");

  public:
    /// Returns the ServiceId of the involved ports
    auto service_id() const -> RawIdType {
        iox2_buffer_16_align_4_t buf;
        iox2_degradation_info_service_id(m_info, &buf);
        return RawIdType::from_range_unchecked(buf.data).value();
    }
    /// Returns the ReceiverPortId of the involved ports
    auto receiver_port_id() const -> RawIdType {
        iox2_buffer_16_align_4_t buf;
        iox2_degradation_info_receiver_port_id(m_info, &buf);
        return RawIdType::from_range_unchecked(buf.data).value();
    }
    /// Returns the ReceiverPortId of the involved ports
    auto sender_port_id() const -> RawIdType {
        iox2_buffer_16_align_4_t buf;
        iox2_degradation_info_sender_port_id(m_info, &buf);
        return RawIdType::from_range_unchecked(buf.data).value();
    }
};

/// The degradation handler invoked when a degradation is detected
///
/// @param[in] DegradationCause is the cause that triggered the handler
/// @param[in] DegradationInfo is a reference to [`DegradationInfo`] with additional information for the user to handle
/// the incident
///
/// @eturn The [`DegradationAction`] to be taken to mitigate the degradation
using DegradationHandler = iox2::bb::StaticFunction<DegradationAction(DegradationCause, DegradationInfo&)>;

namespace detail {
inline auto degradation_handler_delegate(iox2_degradation_cause_e degradation_cause,
                                         iox2_degradation_info_h_ref info_handle,
                                         iox2_callback_context callback_cxt) -> iox2_degradation_action_e {
    auto* handler = static_cast<DegradationHandler*>(callback_cxt);

    auto cause = bb::into<DegradationCause>(degradation_cause);
    auto info = DegradationInfo(info_handle);

    return bb::into<iox2_degradation_action_e>((*handler)(cause, info));
}
} // namespace detail

} // namespace iox2

#endif // IOX2_DEGRADATION_HANDLER_HPP
