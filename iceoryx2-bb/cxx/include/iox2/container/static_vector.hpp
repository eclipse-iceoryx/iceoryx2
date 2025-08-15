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

#ifndef INCLUDE_GUARD_IOX2_CONTAINER_STATIC_VECTOR_HPP
#define INCLUDE_GUARD_IOX2_CONTAINER_STATIC_VECTOR_HPP

#include "iox2/container/config.hpp"
#include "iox2/container/optional.hpp"

#include <cstddef>
#include <cstdint>
#include <cstring>
#include <functional>
#include <memory>
#include <type_traits>

namespace iox2 {
namespace container {

namespace detail {
    template<typename T, uint64_t N>
    struct StaticVectorStorage {
        static_assert(std::is_standard_layout<T>::value, "Storage is only valid for standard layout types.");

        alignas(T) char bytes[sizeof(T) * N];
        uint64_t size;

        constexpr StaticVectorStorage() noexcept
        :bytes{}, size(0)
        {}

        template<size_t M, typename std::enable_if<(N >= M), bool>::type = true>
        constexpr StaticVectorStorage(StaticVectorStorage<T, M> const& rhs)
        :bytes{}, size(rhs.size)
        {
            for (uint64_t index = 0; index < size; ++index) {
                new (pointer_from_index(index)) T(*rhs.pointer_from_index(index));
            }
        }

        template<size_t M, typename std::enable_if<(N >= M), bool>::type = true>
        constexpr StaticVectorStorage(StaticVectorStorage<T, M>&& rhs)
        :bytes{}, size(rhs.size)
        {
            for (uint64_t index = 0; index < size; ++index) {
                new (pointer_from_index(index)) T(std::move(*rhs.pointer_from_index(index)));
            }
        }

#       if __cplusplus >= 202002L
        constexpr
#       endif
        ~StaticVectorStorage() {
            for (uint64_t i = size; i != 0; --i) {
                uint64_t const index = i - 1;
                pointer_from_index(index)->~T();
            }
        }

        T* pointer_from_index(size_t idx) {
            return reinterpret_cast<T*>(bytes + idx * sizeof(T));
        }

        T const* pointer_from_index(size_t idx) const {
            return reinterpret_cast<T const*>(bytes + idx * sizeof(T));
        }
    };
} // namespace detail

/// A resizable container with compile-time fixed static capacity and contiguous inplace storage.
template<typename T, uint64_t N>
class StaticVector {
    static_assert(N > 0, "Static container with capacity 0 is not allowed.");
    static_assert(std::is_standard_layout<T>::value, "Containers can only be used with standard layout types.");
public:
    using ValueType = T;
    using SizeType = size_t;
    using DifferenceType = ptrdiff_t;
    using Reference = T&;
    using ConstReference = T const&;
    using Pointer = T*;
    using ConstPointer = T const*;
    using Iterator = Pointer;
    using ConstIterator = ConstPointer;
    using OptionalReference = Optional<std::reference_wrapper<T>>;
    using OptionalConstReference = Optional<std::reference_wrapper<T const>>;

    // Unchecked element access
    class UncheckedConstAccessor {
        friend class StaticVector;
    private:
        StaticVector const* m_parent;
    private:
        constexpr UncheckedConstAccessor(StaticVector const& parent)
        :m_parent(&parent)
        {}
    public:
        UncheckedConstAccessor& operator=(UncheckedConstAccessor&&) = delete;

        constexpr ConstReference operator[](SizeType index) const {
            return *m_parent->m_storage.pointer_from_index(index);
        }

        constexpr ConstIterator begin() const noexcept {
            return m_parent->m_storage.pointer_from_index(0);
        }

        constexpr ConstIterator end() const noexcept {
            return m_parent->m_storage.pointer_from_index(m_parent->m_storage.size);
        }

        constexpr ConstPointer data() const noexcept {
            return m_parent->m_storage.pointer_from_index(0);
        }
    };

    class UncheckedAccessor {
        friend class StaticVector;
    private:
        StaticVector* m_parent;
    private:
        constexpr UncheckedAccessor(StaticVector& parent)
        :m_parent(&parent)
        {}
    public:
        UncheckedAccessor& operator=(UncheckedAccessor&&) = delete;

        constexpr Reference operator[](SizeType index) {
            return *m_parent->m_storage.pointer_from_index(index);
        }

        constexpr ConstReference operator[](SizeType index) const {
            return *m_parent->m_storage.pointer_from_index(index);
        }

        constexpr Iterator begin() noexcept {
            return m_parent->m_storage.pointer_from_index(0);
        }

        constexpr ConstIterator begin() const noexcept {
            return m_parent->m_storage.pointer_from_index(0);
        }

        constexpr Iterator end() noexcept {
            return m_parent->m_storage.pointer_from_index(this->m_parent->m_storage.size);
        }
        
        constexpr ConstIterator end() const noexcept {
            return m_parent->m_storage.pointer_from_index(this->m_parent->m_storage.size);
        }

        constexpr Pointer data() noexcept {
            return m_parent->m_storage.pointer_from_index(0);
        }

        constexpr ConstPointer data() const noexcept {
            return m_parent->m_storage.pointer_from_index(0);
        }
    };
private:
    template<uint64_t M> friend class StaticVector<T, M>;
    detail::StaticVectorStorage<T, N> m_storage;
public:
    // constructors
    constexpr StaticVector() noexcept = default;
    constexpr StaticVector(StaticVector const&) = default;
    constexpr StaticVector(StaticVector&&) = default;
    
    template<uint64_t M, typename std::enable_if<(N >= M), bool>::type = true>
    constexpr StaticVector(StaticVector<T, M> const& rhs)
    :m_storage(rhs.m_storage)
    {}

    // destructor
#if __cplusplus >= 202002L
    constexpr
#endif
    ~StaticVector() = default;

    template<typename... Args>
    constexpr
    typename std::enable_if<std::is_constructible<T, Args...>::value, bool>::type
    try_emplace_back(Args&&... args) {
        if (m_storage.size < N) {
            new (m_storage.pointer_from_index(m_storage.size)) T(std::forward<Args>(args)...);
            ++m_storage.size;
            return true;
        } else {
            return false;
        }
    }
    
    constexpr bool try_push_back(T const& v) {
        if (m_storage.size < N) {
            new (m_storage.pointer_from_index(m_storage.size)) T(v);
            ++m_storage.size;
            return true;
        } else {
            return false;
        }
    }

    constexpr bool try_push_back(T&& v) {
        if (m_storage.size < N) {
            new (m_storage.pointer_from_index(m_storage.size)) T(std::move(v));
            ++m_storage.size;
            return true;
        } else {
            return false;
        }
    }

    static constexpr SizeType capacity() noexcept {
        return N;
    }

    constexpr SizeType size() const noexcept {
        return m_storage.size;
    }

    bool empty() const {
        return size() == 0;
    }

    OptionalReference element_at(SizeType index) {
        if (index < m_storage.size) {
            return *m_storage.pointer_from_index(index);
        } else {
            return NulloptT{};
        }
    }

    OptionalConstReference element_at(SizeType index) const {
        if (index < m_storage.size) {
            return *m_storage.pointer_from_index(index);
        } else {
            return NulloptT{};
        }
    }

    UncheckedAccessor unchecked_access() {
        return UncheckedAccessor{ *this };
    }
    
    UncheckedConstAccessor unchecked_access() const {
        return UncheckedConstAccessor{ *this };
    }
};

} // namespace container
} // namespace iox2

#endif
