// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2023 by Apex.AI Inc. All rights reserved.
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
#ifndef IOX_HOOFS_CONTAINER_VECTOR_INL
#define IOX_HOOFS_CONTAINER_VECTOR_INL

#include "iox/assertions.hpp"
#include "iox/iceoryx_hoofs_deployment.hpp"
#include "iox/vector.hpp"

#include <cstring> // std::memcpy, std::memmove
#include <iostream>
#include <type_traits>

namespace iox
{
template <typename T, uint64_t Capacity>
inline vector<T, Capacity>::vector(const uint64_t count, const T& value) noexcept
{
    if (count > Capacity)
    {
        IOX_LOG(Error,
                "Attempting to initialize a vector of capacity "
                    << Capacity << " with " << count << " elements. This exceeds the capacity and only " << Capacity
                    << " elements will be created!");
    }

    for (uint64_t i{0U}; (i < count) && (i < Capacity); ++i)
    {
        IOX_DISCARD_RESULT(emplace_back(value));
    }
}

template <typename T, uint64_t Capacity>
inline vector<T, Capacity>::vector(const uint64_t count) noexcept
{
    if (count > Capacity)
    {
        IOX_LOG(Error,
                "Attempting to initialize a vector of capacity "
                    << Capacity << " with " << count << " elements. This exceeds the capacity and only " << Capacity
                    << " elements will be created!");
    }

    m_size = std::min(count, Capacity);
    for (uint64_t i{0U}; i < m_size; ++i)
    {
        // AXIVION Next Line AutosarC++19_03-A18.5.2, FaultDetection-IndirectAssignmentOverflow : False positive, it is a placement new. Size guaranteed by T.
        new (&at_unchecked(i)) T();
    }
}

template <typename T, uint64_t Capacity>
inline vector<T, Capacity>::vector(const vector& rhs) noexcept
{
    *this = rhs;
}

template <typename T, uint64_t Capacity>
inline vector<T, Capacity>::vector(vector&& rhs) noexcept
{
    *this = std::move(rhs);
}

template <typename T, uint64_t Capacity>
inline vector<T, Capacity>::~vector() noexcept
{
    clear();
}

template <typename T, uint64_t Capacity>
inline vector<T, Capacity>& vector<T, Capacity>::operator=(const vector& rhs) noexcept
{
    if (this != &rhs)
    {
        uint64_t i{0U};
        const uint64_t rhsSize{rhs.size()};

#if not IOX_HOOFS_SUBSET
        if constexpr (std::is_trivially_copyable<T>::value)
        {
            std::memcpy(data(), rhs.data(), static_cast<size_t>(rhsSize) * sizeof(T));
            i = rhsSize;
        }
        else
#endif
        {
            const uint64_t minSize{algorithm::minVal(m_size, rhsSize)};

            // copy using copy assignment
            for (; i < minSize; ++i)
            {
                // AXIVION Next Line AutosarC++19_03-A5.0.1 : Expands to basic variable assignment. Evaluation order is inconsequential.
                at(i) = rhs.at(i);
            }

            // copy using copy ctor
            for (; i < rhsSize; ++i)
            {
                IOX_DISCARD_RESULT(emplace_back(rhs.at(i)));
            }
        }

        // delete remaining elements
        clearFrom(i);

        m_size = rhsSize;
    }
    return *this;
}

template <typename T, uint64_t Capacity>
inline vector<T, Capacity>& vector<T, Capacity>::operator=(vector&& rhs) noexcept
{
    if (this != &rhs)
    {
        uint64_t i{0U};
        const uint64_t rhsSize{rhs.size()};

#if not IOX_HOOFS_SUBSET
        if constexpr (std::is_trivially_copyable<T>::value)
        {
            std::memcpy(data(), rhs.data(), static_cast<size_t>(rhsSize) * sizeof(T));
            i = rhsSize;
        }
        else
#endif
        {
            const uint64_t minSize{algorithm::minVal(m_size, rhsSize)};

            // move using move assignment
            for (; i < minSize; ++i)
            {
                // AXIVION Next Line AutosarC++19_03-A5.0.1 : Expands to basic variable assignment. Evaluation order is inconsequential.
                at(i) = std::move(rhs.at(i));
            }

            // move using move ctor
            for (; i < rhsSize; ++i)
            {
                IOX_DISCARD_RESULT(emplace_back(std::move(rhs.at(i))));
            }
        }

        // delete remaining elements
        clearFrom(i);

        m_size = rhsSize;
        rhs.clear();
    }
    return *this;
}

template <typename T, uint64_t Capacity>
inline bool vector<T, Capacity>::empty() const noexcept
{
    return m_size == 0U;
}

template <typename T, uint64_t Capacity>
inline uint64_t vector<T, Capacity>::size() const noexcept
{
    return m_size;
}

template <typename T, uint64_t Capacity>
inline constexpr uint64_t vector<T, Capacity>::capacity() noexcept
{
    return Capacity;
}

template <typename T, uint64_t Capacity>
inline void vector<T, Capacity>::clear() noexcept
{
    clearFrom(0);
}

template <typename T, uint64_t Capacity>
template <typename... Targs>
inline bool vector<T, Capacity>::emplace_back(Targs&&... args) noexcept
{
    if (m_size < Capacity)
    {
#if IOX_HOOFS_SUBSET
        if (std::is_trivial<T>::value)
#else
        if constexpr (std::is_trivial<T>::value)
#endif
        {
            at_unchecked(m_size++) = T{std::forward<Targs>(args)...};
        }
        else
        {
            // AXIVION Next Line AutosarC++19_03-A5.0.1, FaultDetection-IndirectAssignmentOverflow: Size guaranteed by T. Evaluation order is inconsequential.
            new (&at_unchecked(m_size++)) T{std::forward<Targs>(args)...};
        }
        return true;
    }
    return false;
}

template <typename T, uint64_t Capacity>
template <typename... Targs>
inline bool vector<T, Capacity>::emplace(const uint64_t position, Targs&&... args) noexcept
{
    const uint64_t sizeBeforeEmplace{m_size};
    if ((m_size >= Capacity) || ((position >= Capacity) || (position > sizeBeforeEmplace)))
    {
        return false;
    }

    if (position == sizeBeforeEmplace)
    {
        return emplace_back(std::forward<Targs>(args)...);
    }

#if not IOX_HOOFS_SUBSET
    if constexpr (std::is_trivial<T>::value)
    {
        resize(size() + 1U);
        const size_t dataLen{static_cast<size_t>(sizeBeforeEmplace) - static_cast<size_t>(position)};
        std::memmove(data() + position + 1U, data() + position, dataLen * sizeof(T));
        at_unchecked(position) = T{std::forward<Targs>(args)...};
    }
    else
#endif
    {
        IOX_DISCARD_RESULT(emplace_back(std::move(at_unchecked(sizeBeforeEmplace - 1U))));
        for (uint64_t i{sizeBeforeEmplace - 1U}; i > position; --i)
        {
            at_unchecked(i) = std::move(at_unchecked(i - 1U));
        }
#if IOX_HOOFS_SUBSET
        if (!std::is_trivially_destructible<T>::value)
#else
        if constexpr (!std::is_trivially_destructible<T>::value)
#endif
        {
            at_unchecked(position).~T();
        }
        new (&at_unchecked(position)) T(std::forward<Targs>(args)...);
    }
    return true;
}

template <typename T, uint64_t Capacity>
inline bool vector<T, Capacity>::push_back(const T& value) noexcept
{
    return emplace_back(value);
}

template <typename T, uint64_t Capacity>
// NOLINTNEXTLINE(cppcoreguidelines-rvalue-reference-param-not-moved) perfect forwarding is used
inline bool vector<T, Capacity>::push_back(T&& value) noexcept
{
    // AXIVION Next Construct AutosarC++19_03-A18.9.2: we use idiomatic perfect forwarding
    return emplace_back(std::forward<T>(value));
}

template <typename T, uint64_t Capacity>
inline bool vector<T, Capacity>::pop_back() noexcept
{
    if (m_size > 0U)
    {
#if IOX_HOOFS_SUBSET
        if (std::is_trivial<T>::value)
#else
        if constexpr (std::is_trivial<T>::value)
#endif
        {
            m_size--;
        }
        else
        {
            at_unchecked(--m_size).~T();
        }
        return true;
    }
    return false;
}

template <typename T, uint64_t Capacity>
template <typename... Targs>
inline bool vector<T, Capacity>::resize(const uint64_t count, const Targs&... args) noexcept
{
    if (count > Capacity)
    {
        return false;
    }

    if (count < m_size)
    {
        clearFrom(count);
    }
    else
    {
        while (count != m_size)
        {
            IOX_DISCARD_RESULT(emplace_back(args...));
        }
    }
    return true;
}

template <typename T, uint64_t Capacity>
inline T* vector<T, Capacity>::data() noexcept
{
    // AXIVION DISABLE STYLE AutosarC++19_03-A5.2.3 : const cast to avoid code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<T*>(const_cast<const vector<T, Capacity>*>(this)->data());
    // AXIVION ENABLE STYLE AutosarC++19_03-A5.2.3
}

template <typename T, uint64_t Capacity>
inline const T* vector<T, Capacity>::data() const noexcept
{
    return &at_unchecked(0);
}

template <typename T, uint64_t Capacity>
inline T& vector<T, Capacity>::at(const uint64_t index) noexcept
{
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : const cast to avoid code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<T&>(const_cast<const vector<T, Capacity>*>(this)->at(index));
}

template <typename T, uint64_t Capacity>
inline const T& vector<T, Capacity>::at(const uint64_t index) const noexcept
{
    IOX_ENFORCE(index < m_size, "Out of bounds access");
    return at_unchecked(index);
}

template <typename T, uint64_t Capacity>
inline T& vector<T, Capacity>::operator[](const uint64_t index) noexcept
{
    return at(index);
}

template <typename T, uint64_t Capacity>
inline const T& vector<T, Capacity>::operator[](const uint64_t index) const noexcept
{
    return at(index);
}

template <typename T, uint64_t Capacity>
inline T& vector<T, Capacity>::front() noexcept
{
    IOX_ENFORCE(!empty(), "Attempting to access the front of an empty vector");
    return at(0);
}

template <typename T, uint64_t Capacity>
inline const T& vector<T, Capacity>::front() const noexcept
{
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : const cast to avoid code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<vector<T, Capacity>*>(this)->front();
}

template <typename T, uint64_t Capacity>
inline T& vector<T, Capacity>::back() noexcept
{
    IOX_ENFORCE(!empty(), "Attempting to access the back of an empty vector");
    return at(size() - 1U);
}

template <typename T, uint64_t Capacity>
inline const T& vector<T, Capacity>::back() const noexcept
{
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : const cast to avoid code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<vector<T, Capacity>*>(this)->back();
}

template <typename T, uint64_t Capacity>
inline typename vector<T, Capacity>::iterator vector<T, Capacity>::begin() noexcept
{
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : const cast to avoid code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<iterator>(const_cast<const vector<T, Capacity>*>(this)->begin());
}

template <typename T, uint64_t Capacity>
inline typename vector<T, Capacity>::const_iterator vector<T, Capacity>::begin() const noexcept
{
    // AXIVION Next Construct AutosarC++19_03-A5.2.4 : Type-safety ensured by template parameter
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
    return reinterpret_cast<const_iterator>(&at_unchecked(0));
}

template <typename T, uint64_t Capacity>
inline typename vector<T, Capacity>::iterator vector<T, Capacity>::end() noexcept
{
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : const cast to avoid code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<iterator>(const_cast<const vector<T, Capacity>*>(this)->end());
}

template <typename T, uint64_t Capacity>
inline typename vector<T, Capacity>::const_iterator vector<T, Capacity>::end() const noexcept
{
    // AXIVION Next Construct AutosarC++19_03-A5.2.4, AutosarC++19_03-A5.0.4, AutosarC++19_03-M5.0.15 : Type-safety ensured by template parameter.
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
    return reinterpret_cast<const_iterator>(&(at_unchecked(0)) + m_size);
}

template <typename T, uint64_t Capacity>
inline bool vector<T, Capacity>::erase(iterator position) noexcept
{
    if ((begin() <= position) && (position < end()))
    {
        // AXIVION Next Line AutosarC++19_03-M5.0.9 : False positive. Pointer arithmetic occurs here.
        uint64_t index{static_cast<uint64_t>(position - begin())};
        uint64_t n{index};
#if not IOX_HOOFS_SUBSET
        if constexpr (std::is_trivially_copyable<T>::value)
        {
            if constexpr (!(std::is_trivially_destructible<T>::value))
            {
                at_unchecked(n).~T();
            }
            uint64_t dataLen{size() - n - 1U};
            std::memmove(data() + n, data() + n + 1U, static_cast<size_t>(dataLen) * sizeof(T));
        }
        else
#endif
        {
            while ((n + 1U) < size())
            {
                // AXIVION Next Line AutosarC++19_03-A5.0.1 : Expands to basic variable assignment. Evaluation order is inconsequential.
                at_unchecked(n) = std::move(at(n + 1U));
                ++n;
            }
            at_unchecked(n).~T();
        }

        m_size--;
        return true;
    }
    return false;
}

template <typename T, uint64_t Capacity>
inline T& vector<T, Capacity>::at_unchecked(const uint64_t index) noexcept
{
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : const cast to avoid code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<T&>(const_cast<const vector<T, Capacity>*>(this)->at_unchecked(index));
}

template <typename T, uint64_t Capacity>
inline const T& vector<T, Capacity>::at_unchecked(const uint64_t index) const noexcept
{
    // AXIVION Next Construct AutosarC++19_03-A5.2.4 : Type-safety ensured by template parameter
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
    return *reinterpret_cast<const T*>(&m_data[index]);
}

template <typename T, uint64_t Capacity>
inline void vector<T, Capacity>::clearFrom(const uint64_t startPosition) noexcept
{
#if IOX_HOOFS_SUBSET
    if (std::is_trivially_destructible<T>::value)
#else
    if constexpr (std::is_trivially_destructible<T>::value)
#endif
    {
        m_size = startPosition;
    }
    else
    {
        while (m_size > startPosition)
        {
            at_unchecked(--m_size).~T();
        }
    }
}

// AXIVION Next Construct AutosarC++19_03-A13.5.5 : intentional implementation with different parameters to enable
// comparison of vectors with different capacity
template <typename T, uint64_t CapacityLeft, uint64_t CapacityRight>
inline constexpr bool operator==(const vector<T, CapacityLeft>& lhs, const vector<T, CapacityRight>& rhs) noexcept
{
    uint64_t vectorSize{lhs.size()};
    if (vectorSize != rhs.size())
    {
        return false;
    }

    for (uint64_t i{0U}; i < vectorSize; ++i)
    {
        if (lhs[i] != rhs[i])
        {
            return false;
        }
    }
    return true;
}

// AXIVION Next Construct AutosarC++19_03-A13.5.5 : intentional implementation with different parameters to enable
// comparison of vectors with different capacity
template <typename T, uint64_t CapacityLeft, uint64_t CapacityRight>
inline bool constexpr operator!=(const vector<T, CapacityLeft>& lhs, const vector<T, CapacityRight>& rhs) noexcept
{
    return !(lhs == rhs);
}
} // namespace iox
#endif // IOX_HOOFS_CONTAINER_VECTOR_INL
