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

#include "iox2/iceoryx2_cxx_deployment.hpp"

#if IOX2_FEATURE_FLATBUFFERS

#include "iox2/bb/detail/attributes.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/internal/resizable_memory_request.hpp"

namespace iox2 {
namespace internal {
template <ServiceType Service>
ResizableMemoryRequest<Service>::ResizableMemoryRequest(iox2_resizable_memory_request_h handle)
    : m_handle { handle }
    , m_ptr { iox2_resizable_memory_request_ptr(&handle) } {
}

template <ServiceType Service>
void ResizableMemoryRequest<Service>::drop() {
    if (m_handle != nullptr) {
        iox2_resizable_memory_request_drop(m_handle);
        m_handle = nullptr;
        m_ptr = nullptr;
    }
}

template <ServiceType Service>
ResizableMemoryRequest<Service>::ResizableMemoryRequest(ResizableMemoryRequest&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType Service>
ResizableMemoryRequest<Service>::~ResizableMemoryRequest() {
    drop();
}

template <ServiceType Service>
auto ResizableMemoryRequest<Service>::operator=(ResizableMemoryRequest&& rhs) noexcept -> ResizableMemoryRequest& {
    if (this != &rhs) {
        drop();
        m_handle = rhs.m_handle;
        m_ptr = rhs.m_ptr;
        m_has_allocated = rhs.m_has_allocated;

        rhs.m_handle = nullptr;
        rhs.m_ptr = nullptr;
        rhs.m_has_allocated = false;
    }

    return *this;
}

template <ServiceType Service>
auto ResizableMemoryRequest<Service>::allocate(size_t size) -> uint8_t* {
    if (m_has_allocated) {
        return nullptr;
    }

    m_has_allocated = true;

    auto current_size = len();

    if (size > current_size) {
        reallocate_downward(m_ptr, current_size, size, 0, 0);
    }

    return m_ptr;
}

template <ServiceType Service>
auto ResizableMemoryRequest<Service>::len() const -> size_t {
    return iox2_resizable_memory_request_len(&m_handle);
}

template <ServiceType Service>
auto ResizableMemoryRequest<Service>::as_ptr() const -> const uint8_t* {
    return m_ptr;
}

template <ServiceType Service>
void ResizableMemoryRequest<Service>::deallocate(uint8_t* ptr IOX2_MAYBE_UNUSED, size_t size IOX2_MAYBE_UNUSED) {
    IOX2_ASSERT(m_has_allocated == true, "Deallocating ptr that was not allocated with this instance.");
    m_has_allocated = false;
}

template <ServiceType Service>
// NOLINTNEXTLINE(readability-function-size) implementing external flatbuffers interface
auto ResizableMemoryRequest<Service>::reallocate_downward(
    uint8_t* old_p,
    // NOLINTBEGIN(bugprone-easily-swappable-parameters) implementing external flatbuffers interface
    size_t old_size IOX2_MAYBE_UNUSED,
    size_t new_size,
    size_t in_use_back IOX2_MAYBE_UNUSED,
    size_t in_use_front
    // NOLINTEND(bugprone-easily-swappable-parameters)
    ) -> uint8_t* {
    IOX2_ASSERT(old_p == m_ptr, "Growing ptr that was not allocated with this instance.");
    IOX2_ASSERT(iox2_resizable_memory_request_grow_downwards(&m_handle, new_size, in_use_front, &m_ptr) == IOX2_OK,
                "Failed to grow underlying data segment.");
    return m_ptr;
}

template class ResizableMemoryRequest<ServiceType::Ipc>;
template class ResizableMemoryRequest<ServiceType::Local>;
} // namespace internal
} // namespace iox2

#endif // IOX2_FEATURE_FLATBUFFERS
