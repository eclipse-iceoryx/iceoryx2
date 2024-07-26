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

#ifndef IOX2_INTERNAL_CALLBACK_CONTEXT_HPP
#define IOX2_INTERNAL_CALLBACK_CONTEXT_HPP

namespace iox2::internal {
template <typename T>
class CallbackContext {
  public:
    explicit CallbackContext(const T& ptr)
        : m_ptr { &ptr } {
    }

    auto value() const -> const T& {
        return *m_ptr;
    }

  private:
    const T* m_ptr;
};

template <typename T>
auto ctx(const T& ptr) -> CallbackContext<T> {
    return CallbackContext<T>(ptr);
}

template <typename T>
auto ctx_cast(void* ptr) -> CallbackContext<T>* {
    return static_cast<CallbackContext<T>*>(ptr);
}
} // namespace iox2::internal

#endif
