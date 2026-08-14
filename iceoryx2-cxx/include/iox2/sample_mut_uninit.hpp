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

#ifndef IOX2_SAMPLE_MUT_UNINIT_HPP
#define IOX2_SAMPLE_MUT_UNINIT_HPP

#include "iox2/bb/slice.hpp"
#include "iox2/bb/static_function.hpp"
#include "iox2/deployment/iceoryx2_cxx_feature_flags.hpp"
#include "iox2/header_publish_subscribe.hpp"
#include "iox2/internal/helper.hpp"
#include "iox2/marker.hpp"
#include "iox2/sample_mut.hpp"
#include "iox2/service_type.hpp"

#if IOX2_FEATURE_FLATBUFFERS
#include "iox2/internal/resizable_memory_publish_subscribe.hpp"

#include <flatbuffers/buffer.h>
#include <flatbuffers/flatbuffer_builder.h>
#endif

namespace iox2 {
template <ServiceType S, typename Payload, typename UserHeader>
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init,hicpp-member-init) 'm_sample' is not used directly but only via the initialized 'm_handle'; furthermore, it will be initialized on the call site
class SampleMutUninit {
    using ValueType = typename PayloadInfo<Payload>::ValueType;

  public:
    SampleMutUninit(SampleMutUninit&& rhs) noexcept = default;
    auto operator=(SampleMutUninit&& rhs) noexcept -> SampleMutUninit& = default;
    ~SampleMutUninit() noexcept = default;

    SampleMutUninit(const SampleMutUninit&) = delete;
    auto operator=(const SampleMutUninit&) -> SampleMutUninit& = delete;

    /// Returns a reference to the [`Header`] of the [`Sample`].
    template <typename T = Payload, typename = std::enable_if_t<!has_flatbuffer_marker<T>(), void>>
    auto header() const -> HeaderPublishSubscribe;

    /// Returns a reference to the user_header of the [`Sample`]
    template <typename T = UserHeader,
              typename U = Payload,
              typename = std::enable_if_t<!std::is_same<void, UserHeader>::value && !has_flatbuffer_marker<U>(), T>>
    auto user_header() const -> const T&;

    /// Returns a mutable reference to the user_header of the [`Sample`].
    template <typename T = UserHeader,
              typename U = Payload,
              typename = std::enable_if_t<!std::is_same<void, UserHeader>::value && !has_flatbuffer_marker<U>(), T>>
    auto user_header_mut() -> T&;

    /// Returns a reference to the const payload of the sample.
    template <typename T = Payload,
              typename = std::enable_if_t<!bb::IsSlice<T>::VALUE && !has_flatbuffer_marker<T>(), void>>
    auto payload() const -> const ValueType&;

    /// Returns a reference to the payload of the sample.
    template <typename T = Payload,
              typename = std::enable_if_t<!bb::IsSlice<T>::VALUE && !has_flatbuffer_marker<T>(), void>>
    auto payload_mut() -> ValueType&;

    /// Returns an immutable slice to the payload of the sample.
    template <typename T = Payload, typename = std::enable_if_t<bb::IsSlice<T>::VALUE, void>>
    auto payload() const -> bb::ImmutableSlice<ValueType>;

    /// Returns an mutable slice to the payload of the sample.
    template <typename T = Payload, typename = std::enable_if_t<bb::IsSlice<T>::VALUE, void>>
    auto payload_mut() -> bb::MutableSlice<ValueType>;

    /// Writes the payload to the sample
    template <typename T = Payload,
              typename = std::enable_if_t<!bb::IsSlice<T>::VALUE && !has_flatbuffer_marker<T>(), T>>
    auto write_payload(T&& value) -> SampleMut<S, Payload, UserHeader>;

    /// Writes the payload to the sample
    template <typename T = Payload, typename = std::enable_if_t<bb::IsSlice<T>::VALUE, T>>
    auto write_from_fn(const iox2::bb::StaticFunction<typename T::ValueType(uint64_t)>& initializer)
        -> SampleMut<S, Payload, UserHeader>;

    /// mem copies the value to the sample
    template <typename T = Payload, typename = std::enable_if_t<bb::IsSlice<T>::VALUE, T>>
    auto write_from_slice(bb::ImmutableSlice<ValueType>& value) -> SampleMut<S, Payload, UserHeader>;

#if IOX2_FEATURE_FLATBUFFERS
    /// Returns the internal [`FlatBufferBuilder`] that was constructed with the internal iceoryx2
    /// allocator to enable true zero-copy data transfer.
    template <typename T = Payload, typename = std::enable_if_t<has_flatbuffer_marker<T>(), T>>
    auto flatbuffer_builder() -> flatbuffers::FlatBufferBuilder&;
#endif // IOX2_FEATURE_FLATBUFFERS

  private:
    template <ServiceType, typename, typename>
    friend class Publisher;

    template <ServiceType ST, typename PayloadT, typename UserHeaderT>
    friend auto assume_init(SampleMutUninit<ST, PayloadT, UserHeaderT>&& self) -> SampleMut<ST, PayloadT, UserHeaderT>;

#if IOX2_FEATURE_FLATBUFFERS
    template <ServiceType ST, typename PayloadT, typename UserHeaderT>
    friend auto assume_init(SampleMutUninit<ST, Flatbuffer<PayloadT>, UserHeaderT>&& self,
                            flatbuffers::Offset<PayloadT>) -> SampleMut<ST, Flatbuffer<PayloadT>, UserHeaderT>;
#endif // IOX2_FEATURE_FLATBUFFERS

    // The sample is defaulted since both members are initialized in Publisher::loan_uninit() or
    // Publisher::loan_slice_uninit()
    explicit SampleMutUninit() = default;

    auto get_handle() -> iox2_sample_mut_h;

    SampleMut<S, Payload, UserHeader> m_sample;

#if IOX2_FEATURE_FLATBUFFERS
    internal::ResizableMemoryPublishSubscribe<S>* m_memory = nullptr;
    bb::Optional<flatbuffers::FlatBufferBuilder> m_flatbuffer_builder;
#endif // IOX2_FEATURE_FLATBUFFERS
};

/// Acquires the ownership and converts the uninitialized [`SampleMutUninit`] into the
/// initialized version [`SampleMut`].
template <ServiceType S, typename Payload, typename UserHeader>
inline auto assume_init(SampleMutUninit<S, Payload, UserHeader>&& self) -> SampleMut<S, Payload, UserHeader> {
    return std::move(self.m_sample);
}

#if IOX2_FEATURE_FLATBUFFERS

/// Acquires the ownership and converts the uninitialized [`SampleMutUninit`] into the
/// initialized version [`SampleMut`] containing the root payload.
template <ServiceType S, typename Payload, typename UserHeader>
inline auto assume_init(SampleMutUninit<S, Flatbuffer<Payload>, UserHeader>&& self, flatbuffers::Offset<Payload> root)
    -> SampleMut<S, Flatbuffer<Payload>, UserHeader> {
    self.flatbuffer_builder().Finish(root, nullptr);
    const auto* payload_ptr = self.flatbuffer_builder().GetBufferPointer();
    auto handle = self.get_handle();
    iox2_sample_mut_finish_serialized(&handle, payload_ptr);
    // must be the last statement since `iox2_sample_mut_finish_serialized` updates the
    // header and user header ptrs when the flatbuffer builder has resized the memory
    internal::PlacementDefault<UserHeader>::placement_default(self.m_sample);

    return std::move(self.m_sample);
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto SampleMutUninit<S, Payload, UserHeader>::flatbuffer_builder() -> flatbuffers::FlatBufferBuilder& {
    return m_flatbuffer_builder.value();
}

#endif // IOX2_FEATURE_FLATBUFFERS

template <ServiceType S, typename Payload, typename UserHeader>
inline auto SampleMutUninit<S, Payload, UserHeader>::get_handle() -> iox2_sample_mut_h {
    return m_sample.m_handle;
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto SampleMutUninit<S, Payload, UserHeader>::header() const -> HeaderPublishSubscribe {
    return m_sample.header();
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename U, typename>
inline auto SampleMutUninit<S, Payload, UserHeader>::user_header() const -> const T& {
    return m_sample.template user_header<T>();
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename U, typename>
inline auto SampleMutUninit<S, Payload, UserHeader>::user_header_mut() -> T& {
    return m_sample.template user_header_mut<T>();
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto SampleMutUninit<S, Payload, UserHeader>::payload() const -> const ValueType& {
    return m_sample.payload();
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto SampleMutUninit<S, Payload, UserHeader>::payload_mut() -> ValueType& {
    return m_sample.payload_mut();
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto SampleMutUninit<S, Payload, UserHeader>::payload() const -> bb::ImmutableSlice<ValueType> {
    return m_sample.payload();
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto SampleMutUninit<S, Payload, UserHeader>::payload_mut() -> bb::MutableSlice<ValueType> {
    return m_sample.payload_mut();
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto SampleMutUninit<S, Payload, UserHeader>::write_payload(T&& value) -> SampleMut<S, Payload, UserHeader> {
    new (&payload_mut()) Payload(std::forward<T>(value));
    return std::move(m_sample);
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto SampleMutUninit<S, Payload, UserHeader>::write_from_fn(
    const iox2::bb::StaticFunction<typename T::ValueType(uint64_t)>& initializer) -> SampleMut<S, Payload, UserHeader> {
    auto slice = payload_mut();
    for (uint64_t i = 0; i < slice.number_of_elements(); ++i) {
        new (&slice[i]) typename T::ValueType(initializer(i));
    }
    return std::move(m_sample);
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto SampleMutUninit<S, Payload, UserHeader>::write_from_slice(bb::ImmutableSlice<ValueType>& value)
    -> SampleMut<S, Payload, UserHeader> {
    auto dest = payload_mut();
    IOX2_ASSERT(dest.number_of_bytes() >= value.number_of_bytes(),
                "Destination payload size is smaller than source slice size");
    std::memcpy(dest.begin(), value.begin(), value.number_of_bytes());
    return std::move(m_sample);
}
} // namespace iox2

#endif
