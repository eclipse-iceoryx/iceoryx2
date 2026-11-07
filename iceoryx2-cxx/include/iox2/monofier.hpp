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

#ifndef IOX2_MONOFIER_HPP
#define IOX2_MONOFIER_HPP

#include "iox2/bb/expected.hpp"
#include "iox2/event_id.hpp"
#include "iox2/listener_key.hpp"
#include "iox2/notifier_error.hpp"

namespace iox2 {
/// View of the current listener's notification endpoint. It is valid only on
/// the current callback thread and expires when that callback returns.
class MonofierView {
  public:
    MonofierView(const MonofierView&) = delete;
    MonofierView(MonofierView&&) = default;
    auto operator=(const MonofierView&) -> MonofierView& = delete;
    auto operator=(MonofierView&&) -> MonofierView& = default;
    ~MonofierView() = default;

    auto listener_key() const -> ListenerKey;
    auto notify() const -> bb::Expected<void, NotifierNotifyError>;
    auto notify_with_custom_event_id(EventId event_id) const -> bb::Expected<void, NotifierNotifyError>;

  private:
    template <ServiceType>
    friend class Notifier;

    explicit MonofierView(iox2_monofier_ptr handle);
    iox2_monofier_ptr m_handle = nullptr;
};
} // namespace iox2

#endif
