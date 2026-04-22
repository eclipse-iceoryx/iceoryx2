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
#include "iox2/degradation_handler_enums.hpp"
#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {
namespace detail {
// forward declaration in order to make it a friend in DegradationContext
inline auto degradation_callback_delegate(iox2_degradation_cause_e degradation_cause,
                                          iox2_degradation_context_h_ref context_handle,
                                          iox2_callback_context callback_cxt) -> iox2_degradation_action_e;
} // namespace detail

/// The degradation context passed to the [`DegradationCallback`]
class DegradationContext {
  private:
    iox2_degradation_context_h_ref m_context;

    explicit DegradationContext(iox2_degradation_context_h_ref context)
        : m_context(context) {
    }

    friend auto iox2::detail::degradation_callback_delegate(iox2_degradation_cause_e degradation_cause,
                                                            iox2_degradation_context_h_ref context_handle,
                                                            iox2_callback_context callback_cxt)
        -> iox2_degradation_action_e;

  public:
    auto service_id() const -> uint64_t {
        return iox2_degradation_context_service_id(m_context);
    }
    auto receiver_port_id() const -> uint64_t {
        return iox2_degradation_context_receiver_port_id(m_context);
    }
    auto sender_port_id() const -> uint64_t {
        return iox2_degradation_context_sender_port_id(m_context);
    }
};

using DegradationCallback = iox2::bb::StaticFunction<DegradationAction(DegradationCause, DegradationContext&)>;

namespace detail {
inline auto degradation_callback_delegate(iox2_degradation_cause_e degradation_cause,
                                          iox2_degradation_context_h_ref context_handle,
                                          iox2_callback_context callback_cxt) -> iox2_degradation_action_e {
    auto* callback = static_cast<DegradationCallback*>(callback_cxt);

    auto cause = bb::into<DegradationCause>(degradation_cause);
    auto context = DegradationContext(context_handle);

    return bb::into<iox2_degradation_action_e>((*callback)(cause, context));
}
} // namespace detail

} // namespace iox2

#endif // IOX2_DEGRADATION_HANDLER_HPP
