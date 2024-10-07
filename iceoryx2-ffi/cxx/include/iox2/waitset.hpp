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

#ifndef IOX2_WAITSET_HPP
#define IOX2_WAITSET_HPP

#include "iox/duration.hpp"
#include "iox/expected.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/listener.hpp"
#include "iox2/service_type.hpp"
#include "iox2/waitset_enums.hpp"

namespace iox2 {
template <ServiceType S>
class Guard {
  public:
    Guard(Guard&&) noexcept;
    auto operator=(Guard&& rhs) noexcept -> Guard&;
    ~Guard();

    Guard(const Guard&) = delete;
    auto operator=(const Guard&) = delete;

  private:
    template <ServiceType>
    friend class WaitSet;

    template <ServiceType>
    friend class AttachmentId;
    explicit Guard(iox2_guard_h handle);
    void drop();

    iox2_guard_h m_handle = nullptr;
};

template <ServiceType S>
class AttachmentId {
  public:
    AttachmentId(const AttachmentId& rhs) = delete;
    auto operator=(const AttachmentId& rhs) -> AttachmentId& = delete;

    AttachmentId(AttachmentId&& rhs) noexcept;
    auto operator=(AttachmentId&& rhs) noexcept -> AttachmentId&;
    ~AttachmentId();

    static auto from_guard(const Guard<S>& guard) -> AttachmentId;

    auto event_from(const Guard<S>& guard) const -> bool;
    auto deadline_from(const Guard<S>& guard) const -> bool;

  private:
    explicit AttachmentId(iox2_attachment_id_h handle);

    void drop();

    iox2_attachment_id_h m_handle = nullptr;
};

template <ServiceType S>
class WaitSet {
  public:
    WaitSet(const WaitSet&) = delete;
    auto operator=(const WaitSet&) -> WaitSet& = delete;
    WaitSet(WaitSet&&) noexcept;
    auto operator=(WaitSet&&) noexcept -> WaitSet&;
    ~WaitSet();

    void stop();
    auto run(const iox::function<void(AttachmentId<S>)>& fn_call) -> iox::expected<WaitSetRunResult, WaitSetRunError>;
    auto run_once(const iox::function<void(AttachmentId<S>)>& fn_call) -> iox::expected<void, WaitSetRunError>;

    auto capacity() const -> uint64_t;
    auto len() const -> uint64_t;
    auto is_empty() const -> bool;

    auto attach_notification(const Listener<S>& listener) -> iox::expected<Guard<S>, WaitSetAttachmentError>;
    auto attach_notification(int32_t file_descriptor) -> iox::expected<Guard<S>, WaitSetAttachmentError>;
    auto attach_deadline(const Listener<S>& listener,
                         iox::units::Duration deadline) -> iox::expected<Guard<S>, WaitSetAttachmentError>;
    auto attach_deadline(int32_t file_descriptor,
                         iox::units::Duration deadline) -> iox::expected<Guard<S>, WaitSetAttachmentError>;
    auto attach_interval(iox::units::Duration deadline) -> iox::expected<Guard<S>, WaitSetAttachmentError>;

  private:
    friend class WaitSetBuilder;
    explicit WaitSet(iox2_waitset_h handle);
    void drop();

    iox2_waitset_h m_handle {};
};

class WaitSetBuilder {
  public:
    WaitSetBuilder();
    ~WaitSetBuilder();

    WaitSetBuilder(const WaitSetBuilder&) = delete;
    WaitSetBuilder(WaitSetBuilder&&) = delete;
    auto operator=(const WaitSetBuilder&) -> WaitSetBuilder& = delete;
    auto operator=(WaitSetBuilder&&) -> WaitSetBuilder& = delete;

    template <ServiceType S>
    auto create() const&& -> iox::expected<WaitSet<S>, WaitSetCreateError>;

  private:
    iox2_waitset_builder_h m_handle;
};
} // namespace iox2

#endif
