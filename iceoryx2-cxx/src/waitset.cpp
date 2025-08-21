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

#include "iox2/waitset.hpp"
#include "iox/into.hpp"
#include "iox2/internal/callback_context.hpp"

namespace iox2 {
////////////////////////////
// BEGIN: WaitSetAttachmentId
////////////////////////////
template <ServiceType S>
WaitSetAttachmentId<S>::WaitSetAttachmentId(iox2_waitset_attachment_id_h handle)
    : m_handle { handle } {
}

template <ServiceType S>
WaitSetAttachmentId<S>::WaitSetAttachmentId(WaitSetAttachmentId&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S>
auto WaitSetAttachmentId<S>::operator=(WaitSetAttachmentId&& rhs) noexcept -> WaitSetAttachmentId& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S>
WaitSetAttachmentId<S>::~WaitSetAttachmentId() {
    drop();
}

template <ServiceType S>
auto WaitSetAttachmentId<S>::from_guard(const WaitSetGuard<S>& guard) -> WaitSetAttachmentId {
    iox2_waitset_attachment_id_h handle {};
    iox2_waitset_attachment_id_from_guard(&guard.m_handle, nullptr, &handle);
    return WaitSetAttachmentId(handle);
}

template <ServiceType S>
auto WaitSetAttachmentId<S>::has_event_from(const WaitSetGuard<S>& guard) const -> bool {
    return iox2_waitset_attachment_id_has_event_from(&m_handle, &guard.m_handle);
}

template <ServiceType S>
auto WaitSetAttachmentId<S>::has_missed_deadline(const WaitSetGuard<S>& guard) const -> bool {
    return iox2_waitset_attachment_id_has_missed_deadline(&m_handle, &guard.m_handle);
}

template <ServiceType S>
void WaitSetAttachmentId<S>::drop() {
    if (m_handle != nullptr) {
        iox2_waitset_attachment_id_drop(m_handle);
        m_handle = nullptr;
    }
}

template <ServiceType S>
auto WaitSetAttachmentId<S>::hash() const -> std::size_t {
    auto len = iox2_waitset_attachment_id_debug_len(&m_handle);
    std::string empty(len, '\0');
    iox2_waitset_attachment_id_debug(&m_handle, empty.data(), len);
    return std::hash<std::string> {}(empty);
}

template <ServiceType S>
auto operator==(const WaitSetAttachmentId<S>& lhs, const WaitSetAttachmentId<S>& rhs) -> bool {
    return iox2_waitset_attachment_id_equal(&lhs.m_handle, &rhs.m_handle);
}

template <ServiceType S>
auto operator<(const WaitSetAttachmentId<S>& lhs, const WaitSetAttachmentId<S>& rhs) -> bool {
    return iox2_waitset_attachment_id_less(&lhs.m_handle, &rhs.m_handle);
}

template <ServiceType S>
auto operator<<(std::ostream& stream, const WaitSetAttachmentId<S>& self) -> std::ostream& {
    auto len = iox2_waitset_attachment_id_debug_len(&self.m_handle);
    std::string empty(len, '\0');
    iox2_waitset_attachment_id_debug(&self.m_handle, empty.data(), len);
    stream << empty;
    return stream;
}

////////////////////////////
// END: WaitSetAttachmentId
////////////////////////////

////////////////////////////
// BEGIN: WaitSetGuard
////////////////////////////
template <ServiceType S>
WaitSetGuard<S>::WaitSetGuard(WaitSetGuard&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S>
auto WaitSetGuard<S>::operator=(WaitSetGuard&& rhs) noexcept -> WaitSetGuard& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S>
WaitSetGuard<S>::~WaitSetGuard() {
    drop();
}

template <ServiceType S>
WaitSetGuard<S>::WaitSetGuard(iox2_waitset_guard_h handle)
    : m_handle { handle } {
}

template <ServiceType S>
void WaitSetGuard<S>::drop() {
    if (m_handle != nullptr) {
        iox2_waitset_guard_drop(m_handle);
        m_handle = nullptr;
    }
}

////////////////////////////
// END: WaitSetGuard
////////////////////////////

////////////////////////////
// BEGIN: WaitSetBuilder
////////////////////////////
WaitSetBuilder::WaitSetBuilder()
    : m_handle([] {
        iox2_waitset_builder_h handle {};
        iox2_waitset_builder_new(nullptr, &handle);
        return handle;
    }()) {
}

template <ServiceType S>
auto WaitSetBuilder::create() const&& -> iox::expected<WaitSet<S>, WaitSetCreateError> {
    if (m_signal_handling_mode.has_value()) {
        iox2_waitset_builder_set_signal_handling_mode(
            &m_handle, iox::into<iox2_signal_handling_mode_e>(m_signal_handling_mode.value()));
    }

    iox2_waitset_h waitset_handle {};
    auto result = iox2_waitset_builder_create(m_handle, iox::into<iox2_service_type_e>(S), nullptr, &waitset_handle);

    if (result == IOX2_OK) {
        return iox::ok(WaitSet<S>(waitset_handle));
    }

    return iox::err(iox::into<WaitSetCreateError>(result));
}
////////////////////////////
// END: WaitSetBuilder
////////////////////////////

////////////////////////////
// BEGIN: WaitSet
////////////////////////////
template <ServiceType S>
WaitSet<S>::WaitSet(iox2_waitset_h handle)
    : m_handle { handle } {
}

template <ServiceType S>
WaitSet<S>::WaitSet(WaitSet&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S>
auto WaitSet<S>::operator=(WaitSet&& rhs) noexcept -> WaitSet& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S>
WaitSet<S>::~WaitSet() {
    drop();
}

template <ServiceType S>
void WaitSet<S>::drop() {
    if (m_handle != nullptr) {
        iox2_waitset_drop(m_handle);
        m_handle = nullptr;
    }
}

template <ServiceType S>
auto WaitSet<S>::signal_handling_mode() const -> SignalHandlingMode {
    return iox::into<SignalHandlingMode>(static_cast<int>(iox2_waitset_signal_handling_mode(&m_handle)));
}

template <ServiceType S>
auto WaitSet<S>::capacity() const -> uint64_t {
    return iox2_waitset_capacity(&m_handle);
}

template <ServiceType S>
auto WaitSet<S>::len() const -> uint64_t {
    return iox2_waitset_len(&m_handle);
}

template <ServiceType S>
auto WaitSet<S>::is_empty() const -> bool {
    return iox2_waitset_is_empty(&m_handle);
}

template <ServiceType S>
auto WaitSet<S>::attach_interval(const iox::units::Duration deadline)
    -> iox::expected<WaitSetGuard<S>, WaitSetAttachmentError> {
    iox2_waitset_guard_h guard_handle {};
    auto result = iox2_waitset_attach_interval(&m_handle,
                                               deadline.toSeconds(),
                                               deadline.toNanoseconds()
                                                   - (deadline.toSeconds() * iox::units::Duration::NANOSECS_PER_SEC),
                                               nullptr,
                                               &guard_handle);

    if (result == IOX2_OK) {
        return iox::ok(WaitSetGuard<S>(guard_handle));
    }

    return iox::err(iox::into<WaitSetAttachmentError>(result));
}

template <ServiceType S>
auto WaitSet<S>::attach_deadline(const FileDescriptorBased& attachment, const iox::units::Duration deadline)
    -> iox::expected<WaitSetGuard<S>, WaitSetAttachmentError> {
    iox2_waitset_guard_h guard_handle {};
    auto result = iox2_waitset_attach_deadline(&m_handle,
                                               attachment.file_descriptor().m_handle,
                                               deadline.toSeconds(),
                                               deadline.toNanoseconds()
                                                   - (deadline.toSeconds() * iox::units::Duration::NANOSECS_PER_SEC),
                                               nullptr,
                                               &guard_handle);

    if (result == IOX2_OK) {
        return iox::ok(WaitSetGuard<S>(guard_handle));
    }

    return iox::err(iox::into<WaitSetAttachmentError>(result));
}

template <ServiceType S>
auto WaitSet<S>::attach_deadline(const Listener<S>& listener, const iox::units::Duration deadline)
    -> iox::expected<WaitSetGuard<S>, WaitSetAttachmentError> {
    return attach_deadline(FileDescriptorView(iox2_listener_get_file_descriptor(&listener.m_handle)), deadline);
}

template <ServiceType S>
auto WaitSet<S>::attach_notification(const FileDescriptorBased& attachment)
    -> iox::expected<WaitSetGuard<S>, WaitSetAttachmentError> {
    iox2_waitset_guard_h guard_handle {};
    auto result =
        iox2_waitset_attach_notification(&m_handle, attachment.file_descriptor().m_handle, nullptr, &guard_handle);

    if (result == IOX2_OK) {
        return iox::ok(WaitSetGuard<S>(guard_handle));
    }

    return iox::err(iox::into<WaitSetAttachmentError>(result));
}

template <ServiceType S>
auto WaitSet<S>::attach_notification(const Listener<S>& listener)
    -> iox::expected<WaitSetGuard<S>, WaitSetAttachmentError> {
    return attach_notification(FileDescriptorView(iox2_listener_get_file_descriptor(&listener.m_handle)));
}

template <ServiceType S>
auto run_callback(iox2_waitset_attachment_id_h attachment_id, void* context) -> iox2_callback_progression_e {
    auto* fn_call = internal::ctx_cast<iox::function<CallbackProgression(WaitSetAttachmentId<S>)>>(context);
    return iox::into<iox2_callback_progression_e>(fn_call->value()(WaitSetAttachmentId<S>(attachment_id)));
}

template <ServiceType S>
auto WaitSet<S>::wait_and_process(const iox::function<CallbackProgression(WaitSetAttachmentId<S>)>& fn_call)
    -> iox::expected<WaitSetRunResult, WaitSetRunError> {
    iox2_waitset_run_result_e run_result = iox2_waitset_run_result_e_STOP_REQUEST;
    auto ctx = internal::ctx(fn_call);
    auto result = iox2_waitset_wait_and_process(&m_handle, run_callback<S>, static_cast<void*>(&ctx), &run_result);

    if (result == IOX2_OK) {
        return iox::ok(iox::into<WaitSetRunResult>(static_cast<int>(run_result)));
    }

    return iox::err(iox::into<WaitSetRunError>(result));
}

template <ServiceType S>
auto WaitSet<S>::wait_and_process_once(const iox::function<CallbackProgression(WaitSetAttachmentId<S>)>& fn_call)
    -> iox::expected<WaitSetRunResult, WaitSetRunError> {
    iox2_waitset_run_result_e run_result = iox2_waitset_run_result_e_STOP_REQUEST;
    auto ctx = internal::ctx(fn_call);
    auto result = iox2_waitset_wait_and_process_once(&m_handle, run_callback<S>, static_cast<void*>(&ctx), &run_result);

    if (result == IOX2_OK) {
        return iox::ok(iox::into<WaitSetRunResult>(static_cast<int>(run_result)));
    }

    return iox::err(iox::into<WaitSetRunError>(result));
}

template <ServiceType S>
auto WaitSet<S>::wait_and_process_once_with_timeout(
    const iox::function<CallbackProgression(WaitSetAttachmentId<S>)>& fn_call, const iox::units::Duration timeout)
    -> iox::expected<WaitSetRunResult, WaitSetRunError> {
    iox2_waitset_run_result_e run_result = iox2_waitset_run_result_e_STOP_REQUEST;
    auto ctx = internal::ctx(fn_call);
    auto timeout_secs = timeout.toSeconds();
    auto timeout_nsecs = timeout.toNanoseconds() - (timeout.toSeconds() * iox::units::Duration::NANOSECS_PER_SEC);
    auto result = iox2_waitset_wait_and_process_once_with_timeout(
        &m_handle, run_callback<S>, static_cast<void*>(&ctx), timeout_secs, timeout_nsecs, &run_result);

    if (result == IOX2_OK) {
        return iox::ok(iox::into<WaitSetRunResult>(static_cast<int>(run_result)));
    }

    return iox::err(iox::into<WaitSetRunError>(result));
}

////////////////////////////
// END: WaitSet
////////////////////////////

template class WaitSetAttachmentId<ServiceType::Ipc>;
template class WaitSetAttachmentId<ServiceType::Local>;
template class WaitSetGuard<ServiceType::Ipc>;
template class WaitSetGuard<ServiceType::Local>;
template class WaitSet<ServiceType::Ipc>;
template class WaitSet<ServiceType::Local>;

template auto WaitSetBuilder::create() const&& -> iox::expected<WaitSet<ServiceType::Ipc>, WaitSetCreateError>;
template auto WaitSetBuilder::create() const&& -> iox::expected<WaitSet<ServiceType::Local>, WaitSetCreateError>;

template auto operator==(const WaitSetAttachmentId<ServiceType::Ipc>& lhs,
                         const WaitSetAttachmentId<ServiceType::Ipc>& rhs) -> bool;
template auto operator==(const WaitSetAttachmentId<ServiceType::Local>& lhs,
                         const WaitSetAttachmentId<ServiceType::Local>& rhs) -> bool;
template auto operator<(const WaitSetAttachmentId<ServiceType::Ipc>& lhs,
                        const WaitSetAttachmentId<ServiceType::Ipc>& rhs) -> bool;
template auto operator<(const WaitSetAttachmentId<ServiceType::Local>& lhs,
                        const WaitSetAttachmentId<ServiceType::Local>& rhs) -> bool;
template auto operator<<(std::ostream& stream, const WaitSetAttachmentId<ServiceType::Ipc>& self) -> std::ostream&;
template auto operator<<(std::ostream& stream, const WaitSetAttachmentId<ServiceType::Local>& self) -> std::ostream&;

} // namespace iox2
