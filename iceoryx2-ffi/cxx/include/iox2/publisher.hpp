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

#ifndef IOX2_PUBLISHER_HPP
#define IOX2_PUBLISHER_HPP

#include "iox/assertions_addendum.hpp"
#include "iox/expected.hpp"
#include "iox/slice.hpp"
#include "iox2/connection_failure.hpp"
#include "iox2/iceoryx2.h"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/publisher_error.hpp"
#include "iox2/sample_mut.hpp"
#include "iox2/sample_mut_uninit.hpp"
#include "iox2/service_type.hpp"
#include "iox2/unique_port_id.hpp"

#include <cstdint>
#include <type_traits>

namespace iox2 {
/// Sending endpoint of a publish-subscriber based communication.
template <ServiceType S, typename Payload, typename UserHeader>
class Publisher {
    using ValueType = typename PayloadInfo<Payload>::ValueType;

  public:
    Publisher(Publisher&& rhs) noexcept;
    auto operator=(Publisher&& rhs) noexcept -> Publisher&;
    ~Publisher();

    Publisher(const Publisher&) = delete;
    auto operator=(const Publisher&) -> Publisher& = delete;

    /// Returns the [`UniquePublisherId`] of the [`Publisher`]
    auto id() const -> UniquePublisherId;

    /// Returns the strategy the [`Publisher`] follows when a [`SampleMut`] cannot be delivered
    /// since the [`Subscriber`]s buffer is full.
    auto unable_to_deliver_strategy() const -> UnableToDeliverStrategy;

    /// Returns the maximum number of elements that can be loaned in a slice.
    template <typename T = Payload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto max_slice_len() const -> uint64_t;

    /// Copies the input `value` into a [`SampleMut`] and delivers it.
    /// On success it returns the number of [`Subscriber`]s that received
    /// the data, otherwise a [`PublisherSendError`] describing the failure.
    template <typename T = Payload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto send_copy(const Payload& payload) const -> iox::expected<size_t, PublisherSendError>;

    template <typename T = Payload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto send_slice_copy(iox::ImmutableSlice<ValueType>& payload) const -> iox::expected<size_t, PublisherSendError>;

    /// Loans/allocates a [`SampleMutUninit`] from the underlying data segment of the [`Publisher`].
    /// The user has to initialize the payload before it can be sent.
    ///
    /// On failure it returns [`PublisherLoanError`] describing the failure.
    template <typename T = Payload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto loan_uninit() -> iox::expected<SampleMutUninit<S, Payload, UserHeader>, PublisherLoanError>;

    /// Loans/allocates a [`SampleMut`] from the underlying data segment of the [`Publisher`]
    /// and initialize it with the default value. This can be a performance hit and [`Publisher::loan_uninit`]
    /// can be used to loan an uninitalized [`SampleMut`].
    ///
    /// On failure it returns [`PublisherLoanError`] describing the failure.
    template <typename T = Payload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto loan() -> iox::expected<SampleMut<S, Payload, UserHeader>, PublisherLoanError>;

    /// Loans/allocates a [`SampleMut`] from the underlying data segment of the [`Publisher`]
    /// and initializes all slice elements with the default value. This can be a performance hit
    /// and [`Publisher::loan_slice_uninit()`] can be used to loan a slice of uninitialized
    /// [`Payload`].
    ///
    /// On failure it returns [`PublisherLoanError`] describing the failure.
    template <typename T = Payload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto loan_slice(uint64_t number_of_elements) -> iox::expected<SampleMut<S, T, UserHeader>, PublisherLoanError>;

    /// Loans/allocates a [`SampleMut`] from the underlying data segment of the [`Publisher`].
    /// The user has to initialize the payload before it can be sent.
    ///
    /// On failure it returns [`PublisherLoanError`] describing the failure.
    template <typename T = Payload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto loan_slice_uninit(uint64_t number_of_elements)
        -> iox::expected<SampleMutUninit<S, T, UserHeader>, PublisherLoanError>;

    /// Explicitly updates all connections to the [`Subscriber`]s. This is
    /// required to be called whenever a new [`Subscriber`] is connected to
    /// the service. It is called implicitly whenever [`SampleMut::send()`] or
    /// [`Publisher::send_copy()`] is called.
    /// When a [`Subscriber`] is connected that requires a history this
    /// call will deliver it.
    auto update_connections() -> iox::expected<void, ConnectionFailure>;

  private:
    template <ServiceType, typename, typename>
    friend class PortFactoryPublisher;

    explicit Publisher(iox2_publisher_h handle);
    void drop();

    iox2_publisher_h m_handle { nullptr };
};

template <ServiceType S, typename Payload, typename UserHeader>
inline Publisher<S, Payload, UserHeader>::Publisher(iox2_publisher_h handle)
    : m_handle { handle } {
}

template <ServiceType S, typename Payload, typename UserHeader>
inline void Publisher<S, Payload, UserHeader>::drop() {
    if (m_handle != nullptr) {
        iox2_publisher_drop(m_handle);
        m_handle = nullptr;
    }
}

template <ServiceType S, typename Payload, typename UserHeader>
inline Publisher<S, Payload, UserHeader>::Publisher(Publisher&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Publisher<S, Payload, UserHeader>::operator=(Publisher&& rhs) noexcept -> Publisher& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S, typename Payload, typename UserHeader>
inline Publisher<S, Payload, UserHeader>::~Publisher() {
    drop();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Publisher<S, Payload, UserHeader>::unable_to_deliver_strategy() const -> UnableToDeliverStrategy {
    return iox::into<UnableToDeliverStrategy>(static_cast<int>(iox2_publisher_unable_to_deliver_strategy(&m_handle)));
}


template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto Publisher<S, Payload, UserHeader>::max_slice_len() const -> uint64_t {
    return iox2_publisher_max_slice_len(&m_handle);
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Publisher<S, Payload, UserHeader>::id() const -> UniquePublisherId {
    iox2_unique_publisher_id_h id_handle = nullptr;

    iox2_publisher_id(&m_handle, nullptr, &id_handle);
    return UniquePublisherId { id_handle };
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto Publisher<S, Payload, UserHeader>::send_copy(const Payload& payload) const
    -> iox::expected<size_t, PublisherSendError> {
    static_assert(std::is_trivially_copyable<Payload>::value);

    size_t number_of_recipients = 0;
    auto result =
        iox2_publisher_send_copy(&m_handle, static_cast<const void*>(&payload), sizeof(Payload), &number_of_recipients);

    if (result == IOX2_OK) {
        return iox::ok(number_of_recipients);
    }

    return iox::err(iox::into<PublisherSendError>(result));
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto Publisher<S, Payload, UserHeader>::send_slice_copy(iox::ImmutableSlice<ValueType>& payload) const
    -> iox::expected<size_t, PublisherSendError> {
    size_t number_of_recipients = 0;
    auto result = iox2_publisher_send_slice_copy(&m_handle,
                                                 payload.data(),
                                                 sizeof(typename Payload::ValueType),
                                                 payload.number_of_elements(),
                                                 &number_of_recipients);

    if (result == IOX2_OK) {
        return iox::ok(number_of_recipients);
    }

    return iox::err(iox::into<PublisherSendError>(result));
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto Publisher<S, Payload, UserHeader>::loan_uninit()
    -> iox::expected<SampleMutUninit<S, Payload, UserHeader>, PublisherLoanError> {
    SampleMutUninit<S, Payload, UserHeader> sample;

    auto result = iox2_publisher_loan_slice_uninit(&m_handle, &sample.m_sample.m_sample, &sample.m_sample.m_handle, 1);

    if (result == IOX2_OK) {
        return iox::ok(std::move(sample));
    }

    return iox::err(iox::into<PublisherLoanError>(result));
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto
Publisher<S, Payload, UserHeader>::loan() -> iox::expected<SampleMut<S, Payload, UserHeader>, PublisherLoanError> {
    auto sample = loan_uninit();

    if (sample.has_error()) {
        return iox::err(sample.error());
    }

    new (&sample->payload_mut()) Payload();

    return iox::ok(assume_init(std::move(*sample)));
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto Publisher<S, Payload, UserHeader>::loan_slice(const uint64_t number_of_elements)
    -> iox::expected<SampleMut<S, T, UserHeader>, PublisherLoanError> {
    auto sample_uninit = loan_slice_uninit(number_of_elements);

    if (sample_uninit.has_error()) {
        return iox::err(sample_uninit.error());
    }
    auto sample_init = std::move(sample_uninit.value());

    for (auto& item : sample_init.payload_mut()) {
        new (&item) ValueType();
    }

    return iox::ok(assume_init(std::move(sample_init)));
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto Publisher<S, Payload, UserHeader>::loan_slice_uninit(const uint64_t number_of_elements)
    -> iox::expected<SampleMutUninit<S, T, UserHeader>, PublisherLoanError> {
    SampleMutUninit<S, Payload, UserHeader> sample;

    auto result = iox2_publisher_loan_slice_uninit(
        &m_handle, &sample.m_sample.m_sample, &sample.m_sample.m_handle, number_of_elements);

    if (result == IOX2_OK) {
        return iox::ok(std::move(sample));
    }

    return iox::err(iox::into<PublisherLoanError>(result));
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Publisher<S, Payload, UserHeader>::update_connections() -> iox::expected<void, ConnectionFailure> {
    auto result = iox2_publisher_update_connections(&m_handle);
    if (result != IOX2_OK) {
        return iox::err(iox::into<ConnectionFailure>(result));
    }

    return iox::ok();
}
} // namespace iox2

#endif
