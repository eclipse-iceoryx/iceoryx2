// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

#include "iox2/legacy/memory.hpp"

#include <cstdlib>

namespace iox2 {
namespace legacy {
void* alignedAlloc(const size_t alignment, const size_t size) noexcept {
    // -1 == since the max alignment addition is alignment - 1 otherwise the
    // memory is already aligned and we have to do nothing
    // low-level memory management, no other approach then to use malloc to acquire heap memory
    // NOLINTNEXTLINE(cppcoreguidelines-owning-memory,cppcoreguidelines-pro-type-reinterpret-cast,hicpp-no-malloc,cppcoreguidelines-no-malloc)
    auto memory = reinterpret_cast<size_t>(std::malloc(size + alignment + sizeof(void*) - 1));
    if (memory == 0) {
        return nullptr;
    }
    size_t alignedMemory = align(memory + sizeof(void*), alignment);
    assert(alignedMemory >= memory + 1);
    // low-level memory management, we have to store the actual start of the memory a position before the
    // returned aligned address to be able to release the actual memory address again with free when we
    // only get the aligned address
    // NOLINTNEXTLINE(performance-no-int-to-ptr,cppcoreguidelines-pro-type-reinterpret-cast,cppcoreguidelines-pro-bounds-pointer-arithmetic)
    reinterpret_cast<void**>(alignedMemory)[-1] = reinterpret_cast<void*>(memory);

    // we have to return a void pointer to the aligned memory address
    // NOLINTNEXTLINE(performance-no-int-to-ptr,cppcoreguidelines-pro-type-reinterpret-cast)
    return reinterpret_cast<void*>(alignedMemory);
}

void alignedFree(void* const memory) noexcept {
    if (memory != nullptr) {
        // low-level memory management
        // NOLINTNEXTLINE(cppcoreguidelines-owning-memory, cppcoreguidelines-no-malloc,
        // cppcoreguidelines-pro-bounds-pointer-arithmetic)
        // NOLINTNEXTLINE
        std::free(reinterpret_cast<void**>(memory)[-1]);
    }
}
} // namespace legacy
} // namespace iox2
