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

#include "iox/builder_addendum.hpp"
#include "iox/duration.hpp"
#include "iox/expected.hpp"
#include "iox2/callback_progression.hpp"
#include "iox2/file_descriptor.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/listener.hpp"
#include "iox2/service_type.hpp"
#include "iox2/signal_handling_mode.hpp"
#include "iox2/waitset_enums.hpp"

namespace iox2 {
/// The [`WaitSetGuard`] is returned by [`WaitSet::attach_deadline()`], [`WaitSet::attach_notification()`]
/// or [`WaitSet::attach_interval()`]. As soon as it goes out-of-scope it detaches the attachment.
/// It can also be used to determine the origin of an event in [`WaitSet::wait_and_process()`] or
/// [`WaitSet::try_wait_and_process()`] via [`WaitSetAttachmentId::has_event_from()`] or
/// [`WaitSetAttachmentId::has_missed_deadline()`].
template <ServiceType S>
class WaitSetGuard {
  public:
    WaitSetGuard(WaitSetGuard&&) noexcept;
    auto operator=(WaitSetGuard&& rhs) noexcept -> WaitSetGuard&;
    ~WaitSetGuard();

    WaitSetGuard(const WaitSetGuard&) = delete;
    auto operator=(const WaitSetGuard&) = delete;

  private:
    template <ServiceType>
    friend class WaitSet;

    template <ServiceType>
    friend class WaitSetAttachmentId;
    explicit WaitSetGuard(iox2_waitset_guard_h handle);
    void drop();

    iox2_waitset_guard_h m_handle = nullptr;
};

/// Represents an attachment to the [`WaitSet`]
template <ServiceType S>
class WaitSetAttachmentId {
  public:
    WaitSetAttachmentId(const WaitSetAttachmentId& rhs) = delete;
    auto operator=(const WaitSetAttachmentId& rhs) -> WaitSetAttachmentId& = delete;

    WaitSetAttachmentId(WaitSetAttachmentId&& rhs) noexcept;
    auto operator=(WaitSetAttachmentId&& rhs) noexcept -> WaitSetAttachmentId&;
    ~WaitSetAttachmentId();

    /// Creates an [`WaitSetAttachmentId`] from a [`WaitSetGuard`] that was returned via
    /// [`WaitSet::attach_interval()`], [`WaitSet::attach_notification()`] or
    /// [`WaitSet::attach_deadline()`].
    static auto from_guard(const WaitSetGuard<S>& guard) -> WaitSetAttachmentId;

    /// Returns true if an event was emitted from a notification or deadline attachment
    /// corresponding to [`WaitSetGuard`].
    auto has_event_from(const WaitSetGuard<S>& guard) const -> bool;

    /// Returns true if the deadline for the attachment corresponding to [`WaitSetGuard`] was missed.
    auto has_missed_deadline(const WaitSetGuard<S>& guard) const -> bool;

    /// Returns the a non-secure hash for the [`WaitSetAttachmentId`].
    auto hash() const -> std::size_t;

  private:
    explicit WaitSetAttachmentId(iox2_waitset_attachment_id_h handle);
    template <ServiceType>
    friend auto run_callback(iox2_waitset_attachment_id_h, void*) -> iox2_callback_progression_e;
    template <ServiceType ST>
    friend auto operator==(const WaitSetAttachmentId<ST>&, const WaitSetAttachmentId<ST>&) -> bool;
    template <ServiceType ST>
    friend auto operator<(const WaitSetAttachmentId<ST>&, const WaitSetAttachmentId<ST>&) -> bool;
    template <ServiceType ST>
    friend auto operator<<(std::ostream& stream, const WaitSetAttachmentId<ST>& self) -> std::ostream&;

    void drop();

    iox2_waitset_attachment_id_h m_handle = nullptr;
};

template <ServiceType S>
auto operator==(const WaitSetAttachmentId<S>& lhs, const WaitSetAttachmentId<S>& rhs) -> bool;

template <ServiceType S>
auto operator<(const WaitSetAttachmentId<S>& lhs, const WaitSetAttachmentId<S>& rhs) -> bool;

template <ServiceType S>
auto operator<<(std::ostream& stream, const WaitSetAttachmentId<S>& self) -> std::ostream&;

/// The [`WaitSet`] implements a reactor pattern and allows to wait on multiple events in one
/// single call [`WaitSet::try_wait_and_process()`] until it wakes up or to run repeatedly with
/// [`WaitSet::wait_and_process()`] until the a interrupt or termination signal was received or the user
/// has explicitly requested to stop by returning [`CallbackProgression::Stop`] in the provided
/// callback.
///
/// The [`Listener`] can be attached as well as sockets or anything else that
/// can be packed into a [`FileDescriptorView`].
///
/// Can be created via the [`WaitSetBuilder`].
template <ServiceType S>
class WaitSet {
  public:
    WaitSet(const WaitSet&) = delete;
    auto operator=(const WaitSet&) -> WaitSet& = delete;
    WaitSet(WaitSet&&) noexcept;
    auto operator=(WaitSet&&) noexcept -> WaitSet&;
    ~WaitSet();

    /// Waits until an event arrives on the [`WaitSet`], then collects all events by calling the
    /// provided `fn_call` callback with the corresponding [`WaitSetAttachmentId`]. In contrast
    /// to [`WaitSet::wait_and_process_once()`] it will never return until the user explicitly
    /// requests it by returning [`CallbackProgression::Stop`] or by receiving a signal.
    ///
    /// The provided callback must return [`CallbackProgression::Continue`] to continue the event
    /// processing and handle the next event or [`CallbackProgression::Stop`] to return from this
    /// call immediately. All unhandled events will be lost forever and the call will return
    /// [`WaitSetRunResult::StopRequest`].
    ///
    /// If an interrupt- (`SIGINT`) or a termination-signal (`SIGTERM`) was received, it will exit
    /// the loop and inform the user with [`WaitSetRunResult::Interrupt`] or
    /// [`WaitSetRunResult::TerminationRequest`].
    auto wait_and_process(const iox::function<CallbackProgression(WaitSetAttachmentId<S>)>& fn_call)
        -> iox::expected<WaitSetRunResult, WaitSetRunError>;

    /// Waits until an event arrives on the [`WaitSet`], then
    /// collects all events by calling the provided `fn_call` callback with the corresponding
    /// [`WaitSetAttachmentId`] and then returns. This makes it ideal to be called in some kind of
    /// event-loop.
    ///
    /// The provided callback must return [`CallbackProgression::Continue`] to continue the event
    /// processing and handle the next event or [`CallbackProgression::Stop`] to return from this
    /// call immediately. All unhandled events will be lost forever and the call will return
    /// [`WaitSetRunResult::StopRequest`].
    ///
    /// If an interrupt- (`SIGINT`) or a termination-signal (`SIGTERM`) was received, it will exit
    /// the loop and inform the user with [`WaitSetRunResult::Interrupt`] or
    /// [`WaitSetRunResult::TerminationRequest`].
    ///
    /// When no signal was received and all events were handled, it will return
    /// [`WaitSetRunResult::AllEventsHandled`].
    auto wait_and_process_once(const iox::function<CallbackProgression(WaitSetAttachmentId<S>)>& fn_call)
        -> iox::expected<WaitSetRunResult, WaitSetRunError>;

    /// Waits until an event arrives on the [`WaitSet`] or the provided timeout has passed, then
    /// collects all events by calling the provided `fn_call` callback with the corresponding
    /// [`WaitSetAttachmentId`] and then returns. This makes it ideal to be called in some kind of
    /// event-loop.
    ///
    /// The provided callback must return [`CallbackProgression::Continue`] to continue the event
    /// processing and handle the next event or [`CallbackProgression::Stop`] to return from this
    /// call immediately. All unhandled events will be lost forever and the call will return
    /// [`WaitSetRunResult::StopRequest`].
    ///
    /// If an interrupt- (`SIGINT`) or a termination-signal (`SIGTERM`) was received, it will exit
    /// the loop and inform the user with [`WaitSetRunResult::Interrupt`] or
    /// [`WaitSetRunResult::TerminationRequest`].
    ///
    /// When no signal was received and all events were handled, it will return
    /// [`WaitSetRunResult::AllEventsHandled`].
    auto wait_and_process_once_with_timeout(const iox::function<CallbackProgression(WaitSetAttachmentId<S>)>& fn_call,
                                            iox::units::Duration timeout)
        -> iox::expected<WaitSetRunResult, WaitSetRunError>;

    /// Returns the capacity of the [`WaitSet`]
    auto capacity() const -> uint64_t;

    /// Returns the number of attachments.
    auto len() const -> uint64_t;

    /// Returns true if the [`WaitSet`] has no attachments, otherwise false.
    auto is_empty() const -> bool;

    /// Attaches a [`Listener`] as notification to the [`WaitSet`]. Whenever an event is received on the
    /// object the [`WaitSet`] informs the user in [`WaitSet::wait_and_process()`] to handle the event.
    /// The object cannot be attached twice and the
    /// [`WaitSet::capacity()`] is limited by the underlying implementation.
    ///
    /// # Safety
    ///
    /// * The [`Listener`] must life at least as long as the returned [`WaitSetGuard`].
    /// * The [`WaitSetGuard`] must life at least as long as the [`WaitsSet`].
    auto attach_notification(const Listener<S>& listener) -> iox::expected<WaitSetGuard<S>, WaitSetAttachmentError>;

    /// Attaches a [`FileDescriptorBased`] object as notification to the [`WaitSet`]. Whenever an event is received on
    /// the object the [`WaitSet`] informs the user in [`WaitSet::wait_and_process()`] to handle the event. The object
    /// cannot be attached twice and the
    /// [`WaitSet::capacity()`] is limited by the underlying implementation.
    ///
    /// # Safety
    ///
    /// * The corresponding [`FileDescriptor`] must life at least as long as the returned [`WaitSetGuard`].
    /// * The [`WaitSetGuard`] must life at least as long as the [`WaitsSet`].
    auto attach_notification(const FileDescriptorBased& attachment)
        -> iox::expected<WaitSetGuard<S>, WaitSetAttachmentError>;

    /// Attaches a [`Listener`] as deadline to the [`WaitSet`]. Whenever the event is received or the
    /// deadline is hit, the user is informed in [`WaitSet::wait_and_process()`].
    /// The object cannot be attached twice and the
    /// [`WaitSet::capacity()`] is limited by the underlying implementation.
    /// Whenever the object emits an event the deadline is reset by the [`WaitSet`].
    ///
    /// # Safety
    ///
    /// * The corresponding [`Listener`] must life at least as long as the returned [`WaitSetGuard`].
    /// * The [`WaitSetGuard`] must life at least as long as the [`WaitsSet`].
    auto attach_deadline(const Listener<S>& listener, iox::units::Duration deadline)
        -> iox::expected<WaitSetGuard<S>, WaitSetAttachmentError>;

    /// Attaches a [`FileDescriptorBased`] object as deadline to the [`WaitSet`]. Whenever the event is received or the
    /// deadline is hit, the user is informed in [`WaitSet::wait_and_process()`].
    /// The object cannot be attached twice and the
    /// [`WaitSet::capacity()`] is limited by the underlying implementation.
    /// Whenever the object emits an event the deadline is reset by the [`WaitSet`].
    ///
    /// # Safety
    ///
    /// * The corresponding [`FileDescriptor`] must life at least as long as the returned [`WaitSetGuard`].
    /// * The [`WaitSetGuard`] must life at least as long as the [`WaitsSet`].
    auto attach_deadline(const FileDescriptorBased& attachment, iox::units::Duration deadline)
        -> iox::expected<WaitSetGuard<S>, WaitSetAttachmentError>;

    /// Attaches a tick event to the [`WaitSet`]. Whenever the timeout is reached the [`WaitSet`]
    /// informs the user in [`WaitSet::wait_and_process()`].
    ///
    /// # Safety
    ///
    /// * The [`WaitSetGuard`] must life at least as long as the [`WaitsSet`].
    auto attach_interval(iox::units::Duration deadline) -> iox::expected<WaitSetGuard<S>, WaitSetAttachmentError>;

    /// Returns the [`SignalHandlingMode`] with which the [`WaitSet`] was created.
    auto signal_handling_mode() const -> SignalHandlingMode;

  private:
    friend class WaitSetBuilder;
    explicit WaitSet(iox2_waitset_h handle);
    void drop();

    iox2_waitset_h m_handle = nullptr;
};

/// The builder for the [`WaitSet`].
class WaitSetBuilder {
  public:
    /// Defines the [`SignalHandlingMode`] for the [`WaitSet`]. It affects the
    /// [`WaitSet::wait_and_process()`] and [`WaitSet::wait_and_process_once()`] calls
    /// that returns any received [`Signal`] via its [`WaitSetRunResult`] return value.
#ifdef DOXYGEN_MACRO_FIX
    auto signal_handling_mode(const SignalHandlingMode value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(SignalHandlingMode, signal_handling_mode);
#endif

  public:
    WaitSetBuilder();
    ~WaitSetBuilder() = default;

    WaitSetBuilder(const WaitSetBuilder&) = delete;
    WaitSetBuilder(WaitSetBuilder&&) = delete;
    auto operator=(const WaitSetBuilder&) -> WaitSetBuilder& = delete;
    auto operator=(WaitSetBuilder&&) -> WaitSetBuilder& = delete;

    /// Creates the [`WaitSet`].
    template <ServiceType S>
    auto create() const&& -> iox::expected<WaitSet<S>, WaitSetCreateError>;

  private:
    iox2_waitset_builder_h m_handle = nullptr;
};
} // namespace iox2
#endif
