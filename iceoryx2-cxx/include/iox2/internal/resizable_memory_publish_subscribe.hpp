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

#ifndef IOX2_RESIZABLE_MEMORY_PUBLISH_SUBSCRIBE_HPP
#define IOX2_RESIZABLE_MEMORY_PUBLISH_SUBSCRIBE_HPP

#include <flatbuffers/allocator.h>

#include "iox2/internal/iceoryx2.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
template <ServiceType, typename, typename>
class Publisher;

namespace internal {
/// Internal helper class to handle dynamic memory allocations used in e.g. flatbuffers
template <ServiceType Service>
class ResizableMemoryPublishSubscribe : public flatbuffers::Allocator {
  public:
    ResizableMemoryPublishSubscribe(const ResizableMemoryPublishSubscribe&) = delete;
    ResizableMemoryPublishSubscribe(ResizableMemoryPublishSubscribe&&) noexcept;
    ~ResizableMemoryPublishSubscribe() override;

    auto operator=(const ResizableMemoryPublishSubscribe&) noexcept -> ResizableMemoryPublishSubscribe& = delete;
    auto operator=(ResizableMemoryPublishSubscribe&&) noexcept -> ResizableMemoryPublishSubscribe&;

    auto allocate(size_t size) -> uint8_t* override;

    void deallocate(uint8_t* ptr, size_t size) override;

    auto reallocate_downward(uint8_t* old_p, size_t old_size, size_t new_size, size_t in_use_back, size_t in_use_front)
        -> uint8_t* override;

    auto len() const -> size_t;
    auto reserved_header_len() const -> size_t;
    auto as_ptr() const -> const uint8_t*;

  private:
    template <ServiceType, typename, typename>
    friend class iox2::Publisher;

    explicit ResizableMemoryPublishSubscribe(iox2_resizable_memory_publish_subscribe_h handle);
    void drop();

    iox2_resizable_memory_publish_subscribe_h m_handle = nullptr;
    bool m_has_allocated = false;
    uint8_t* m_ptr = nullptr;
};
} // namespace internal
} // namespace iox2
#endif
