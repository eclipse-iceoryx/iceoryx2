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

#include "iox2/monofier.hpp"

namespace iox2 {
MonofierView::MonofierView(iox2_monofier_ptr handle)
    : m_handle { handle } {
}

auto MonofierView::listener_key() const -> ListenerKey {
    iox2_listener_key_h key_handle = nullptr;
    iox2_monofier_listener_key(m_handle, nullptr, &key_handle);
    return ListenerKey { key_handle };
}

auto MonofierView::notify() const -> bb::Expected<void, NotifierNotifyError> {
    auto result = iox2_monofier_notify(m_handle);
    if (result == IOX2_OK) {
        return {};
    }
    return bb::err(bb::into<NotifierNotifyError>(result));
}

auto MonofierView::notify_with_custom_event_id(EventId event_id) const -> bb::Expected<void, NotifierNotifyError> {
    auto result = iox2_monofier_notify_with_custom_event_id(m_handle, &event_id.m_value);
    if (result == IOX2_OK) {
        return {};
    }
    return bb::err(bb::into<NotifierNotifyError>(result));
}
} // namespace iox2
