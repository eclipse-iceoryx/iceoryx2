// Copyright (c) 2022 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_TESTUTILS_BARRIER_HPP
#define IOX2_BB_TESTUTILS_BARRIER_HPP

#include <condition_variable>
#include <cstdint>
#include <mutex>

namespace iox2 {
namespace legacy {
namespace testing {

class Barrier {
  public:
    explicit Barrier(uint32_t requiredCount = 0)
        : m_requiredCount(requiredCount) {
    }

    void notify() {
        {
            std::lock_guard<std::mutex> lock(m_mutex);
            ++m_count;
        }
        if (m_count >= m_requiredCount) {
            m_condVar.notify_all();
        }
    }

    void wait() {
        std::unique_lock<std::mutex> lock(m_mutex);
        auto cond = [&]() { return m_count >= m_requiredCount; };
        m_condVar.wait(lock, cond);
    }

    void reset(uint32_t requiredCount) {
        {
            std::lock_guard<std::mutex> lock(m_mutex);
            m_requiredCount = requiredCount;
            m_count = 0;
        }

        // notify regardless of count, the threads woken up need to check the condition
        m_condVar.notify_all();
    }

  private:
    uint32_t m_count { 0 };
    std::mutex m_mutex;
    std::condition_variable m_condVar;
    uint32_t m_requiredCount;
};

} // namespace testing
} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_TESTUTILS_BARRIER_HPP
