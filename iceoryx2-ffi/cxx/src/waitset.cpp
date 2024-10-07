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
#include "iox2/internal/callback_context.hpp"

namespace iox2 {
////////////////////////////
// BEGIN: AttachmentId
////////////////////////////
template <ServiceType S>
AttachmentId<S>::AttachmentId(iox2_attachment_id_h handle)
    : m_handle { handle } {
}

template <ServiceType S>
AttachmentId<S>::AttachmentId(AttachmentId&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S>
auto AttachmentId<S>::operator=(AttachmentId&& rhs) noexcept -> AttachmentId& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S>
AttachmentId<S>::~AttachmentId() {
    drop();
}

template <ServiceType S>
auto AttachmentId<S>::from_guard(const Guard<S>& guard) -> AttachmentId {
    iox2_attachment_id_h handle {};
    iox2_attachment_id_from_guard(&guard.m_handle, nullptr, &handle);
    return AttachmentId(handle);
}

template <ServiceType S>
auto AttachmentId<S>::event_from(const Guard<S>& guard) const -> bool {
    return iox2_attachment_id_event_from(&m_handle, &guard.m_handle);
}

template <ServiceType S>
auto AttachmentId<S>::deadline_from(const Guard<S>& guard) const -> bool {
    return iox2_attachment_id_deadline_from(&m_handle, &guard.m_handle);
}

template <ServiceType S>
void AttachmentId<S>::drop() {
    if (m_handle != nullptr) {
        iox2_attachment_id_drop(m_handle);
        m_handle = nullptr;
    }
}

template <ServiceType S>
auto operator==(const AttachmentId<S>& lhs, const AttachmentId<S>& rhs) -> bool {
    return iox2_attachment_id_equal(&lhs.m_handle, &rhs.m_handle);
}

template <ServiceType S>
auto operator<(const AttachmentId<S>& lhs, const AttachmentId<S>& rhs) -> bool {
    return iox2_attachment_id_less(&lhs.m_handle, &rhs.m_handle);
}


////////////////////////////
// END: AttachmentId
////////////////////////////

////////////////////////////
// BEGIN: Guard
////////////////////////////
template <ServiceType S>
Guard<S>::Guard(Guard&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S>
auto Guard<S>::operator=(Guard&& rhs) noexcept -> Guard& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S>
Guard<S>::~Guard() {
    drop();
}

template <ServiceType S>
Guard<S>::Guard(iox2_guard_h handle)
    : m_handle { handle } {
}

template <ServiceType S>
void Guard<S>::drop() {
    if (m_handle != nullptr) {
        iox2_guard_drop(m_handle);
        m_handle = nullptr;
    }
}

////////////////////////////
// END: Guard
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

WaitSetBuilder::~WaitSetBuilder() {
    iox2_waitset_builder_drop(m_handle);
}

template <ServiceType S>
auto WaitSetBuilder::create() const&& -> iox::expected<WaitSet<S>, WaitSetCreateError> {
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
void WaitSet<S>::stop() {
    iox2_waitset_stop(&m_handle);
}

template <ServiceType S>
auto WaitSet<S>::attach_interval(const iox::units::Duration deadline)
    -> iox::expected<Guard<S>, WaitSetAttachmentError> {
    iox2_guard_h guard_handle {};
    auto result = iox2_waitset_attach_interval(&m_handle,
                                               deadline.toSeconds(),
                                               deadline.toNanoseconds()
                                                   - deadline.toSeconds() * iox::units::Duration::NANOSECS_PER_SEC,
                                               nullptr,
                                               &guard_handle);

    if (result == IOX2_OK) {
        return iox::ok(Guard<S>(guard_handle));
    }

    return iox::err(iox::into<WaitSetAttachmentError>(result));
}

template <ServiceType S>
auto WaitSet<S>::attach_deadline(FileDescriptorView file_descriptor, const iox::units::Duration deadline)
    -> iox::expected<Guard<S>, WaitSetAttachmentError> {
    iox2_guard_h guard_handle {};
    auto result = iox2_waitset_attach_deadline(&m_handle,
                                               file_descriptor.m_handle,
                                               deadline.toSeconds(),
                                               deadline.toNanoseconds()
                                                   - deadline.toSeconds() * iox::units::Duration::NANOSECS_PER_SEC,
                                               nullptr,
                                               &guard_handle);

    if (result == IOX2_OK) {
        return iox::ok(Guard<S>(guard_handle));
    }

    return iox::err(iox::into<WaitSetAttachmentError>(result));
}

template <ServiceType S>
auto WaitSet<S>::attach_deadline(const Listener<S>& listener, const iox::units::Duration deadline)
    -> iox::expected<Guard<S>, WaitSetAttachmentError> {
    return attach_deadline(FileDescriptorView(iox2_listener_get_file_descriptor(&listener.m_handle)), deadline);
}

template <ServiceType S>
auto WaitSet<S>::attach_notification(const FileDescriptorView file_descriptor)
    -> iox::expected<Guard<S>, WaitSetAttachmentError> {
    iox2_guard_h guard_handle {};
    auto result = iox2_waitset_attach_notification(&m_handle, file_descriptor.m_handle, nullptr, &guard_handle);

    if (result == IOX2_OK) {
        return iox::ok(Guard<S>(guard_handle));
    }

    return iox::err(iox::into<WaitSetAttachmentError>(result));
}

template <ServiceType S>
auto WaitSet<S>::attach_notification(const Listener<S>& listener) -> iox::expected<Guard<S>, WaitSetAttachmentError> {
    return attach_notification(FileDescriptorView(iox2_listener_get_file_descriptor(&listener.m_handle)));
}

template <ServiceType S>
auto run_callback(iox2_attachment_id_h attachment_id, void* context) {
    auto* fn_call = internal::ctx_cast<iox::function<void(AttachmentId<S>)>>(context);
    fn_call->value()(AttachmentId<S>(attachment_id));
}

template <ServiceType S>
auto WaitSet<S>::run(const iox::function<void(AttachmentId<S>)>& fn_call)
    -> iox::expected<WaitSetRunResult, WaitSetRunError> {
    iox2_waitset_run_result_e run_result = iox2_waitset_run_result_e_STOP_REQUEST;
    auto ctx = internal::ctx(fn_call);
    auto result = iox2_waitset_run(&m_handle, run_callback<S>, static_cast<void*>(&ctx), &run_result);

    if (result == IOX2_OK) {
        return iox::ok(iox::into<WaitSetRunResult>(static_cast<int>(run_result)));
    }

    return iox::err(iox::into<WaitSetRunError>(result));
}

template <ServiceType S>
auto WaitSet<S>::run_once(const iox::function<void(AttachmentId<S>)>& fn_call) -> iox::expected<void, WaitSetRunError> {
    auto ctx = internal::ctx(fn_call);
    auto result = iox2_waitset_run_once(&m_handle, run_callback<S>, static_cast<void*>(&ctx));

    if (result == IOX2_OK) {
        return iox::ok();
    }

    return iox::err(iox::into<WaitSetRunError>(result));
}

////////////////////////////
// END: WaitSet
////////////////////////////

template class AttachmentId<ServiceType::Ipc>;
template class AttachmentId<ServiceType::Local>;
template class Guard<ServiceType::Ipc>;
template class Guard<ServiceType::Local>;
template class WaitSet<ServiceType::Ipc>;
template class WaitSet<ServiceType::Local>;

template auto WaitSetBuilder::create() const&& -> iox::expected<WaitSet<ServiceType::Ipc>, WaitSetCreateError>;
template auto WaitSetBuilder::create() const&& -> iox::expected<WaitSet<ServiceType::Local>, WaitSetCreateError>;

template auto operator==(const AttachmentId<ServiceType::Ipc>& lhs, const AttachmentId<ServiceType::Ipc>& rhs) -> bool;
template auto operator==(const AttachmentId<ServiceType::Local>& lhs,
                         const AttachmentId<ServiceType::Local>& rhs) -> bool;
template auto operator<(const AttachmentId<ServiceType::Ipc>& lhs, const AttachmentId<ServiceType::Ipc>& rhs) -> bool;
template auto operator<(const AttachmentId<ServiceType::Local>& lhs,
                        const AttachmentId<ServiceType::Local>& rhs) -> bool;
} // namespace iox2
