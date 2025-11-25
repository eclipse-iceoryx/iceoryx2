// Copyright (c) 2022 by Apex.AI Inc. All rights reserved.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#ifndef IOX_HOOFS_TESTUTILS_BARRIER_HPP
#define IOX_HOOFS_TESTUTILS_BARRIER_HPP

#include <condition_variable>
#include <cstdint>
#include <mutex>


class Barrier
{
  public:
    explicit Barrier(uint32_t requiredCount = 0)
        : m_requiredCount(requiredCount)
    {
    }

    void notify()
    {
        {
            std::lock_guard<std::mutex> lock(m_mutex);
            ++m_count;
        }
        if (m_count >= m_requiredCount)
        {
            m_condVar.notify_all();
        }
    }

    void wait()
    {
        std::unique_lock<std::mutex> lock(m_mutex);
        auto cond = [&]() { return m_count >= m_requiredCount; };
        m_condVar.wait(lock, cond);
    }

    void reset(uint32_t requiredCount)
    {
        {
            std::lock_guard<std::mutex> lock(m_mutex);
            m_requiredCount = requiredCount;
            m_count = 0;
        }

        // notify regardless of count, the threads woken up need to check the condition
        m_condVar.notify_all();
    }

  private:
    uint32_t m_count{0};
    std::mutex m_mutex;
    std::condition_variable m_condVar;
    uint32_t m_requiredCount;
};

#endif // IOX_HOOFS_TESTUTILS_BARRIER_HPP
