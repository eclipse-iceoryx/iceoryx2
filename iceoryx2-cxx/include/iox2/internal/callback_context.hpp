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

#include "iox/optional.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/node_details.hpp"
#include "iox2/node_id.hpp"
#include "iox2/node_name.hpp"
#include "iox2/node_state.hpp"
#include "iox2/service_type.hpp"

namespace iox2::internal {

/// Building block to provide a type-safe context pointer to a C callback
/// that has a `void*` context argument.
/// The context could be hereby a user provided clojure with capture or
/// any other C++ object.
///
/// # Example
///
/// ```
/// void some_c_callback(void* context) {
///    auto ctx = internal::ctx_cast<SomeType>(context);
///    ctx->value(); // access underlying my_context_object
/// }
///
/// SomeType my_context_object;
/// auto ctx = internal::ctx(my_context_object);
/// some_c_callback(static_cast<void*>(&ctx));
/// ```
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
inline auto ctx(const T& ptr) -> CallbackContext<T> {
    return CallbackContext<T>(ptr);
}

template <typename T>
inline auto ctx_cast(void* ptr) -> CallbackContext<T>* {
    return static_cast<CallbackContext<T>*>(ptr);
}

template <typename T, typename ViewType>
auto list_ports_callback(void* context, const T port_details_view) -> iox2_callback_progression_e {
    auto* callback = internal::ctx_cast<iox::function<CallbackProgression(ViewType)>>(context);
    return iox::into<iox2_callback_progression_e>(callback->value()(ViewType(port_details_view)));
}

template <ServiceType T>
// NOLINTBEGIN(readability-function-size)
auto list_callback(iox2_node_state_e node_state,
                   iox2_node_id_ptr node_id_ptr,
                   const char* executable,
                   iox2_node_name_ptr node_name,
                   iox2_config_ptr config,
                   iox2_callback_context context) -> iox2_callback_progression_e {
    auto node_details = [&] {
        if (node_id_ptr == nullptr || config == nullptr) {
            return iox::optional<NodeDetails>();
        }

        return iox::optional<NodeDetails>(NodeDetails {
            iox::FileName::create(iox::string<iox::FileName::capacity()>(iox::TruncateToCapacity, executable))
                .expect("The executable file name is always valid."),
            NodeNameView { node_name }.to_owned(),
            Config {} });
    }();

    iox2_node_id_h node_id_handle = nullptr;
    iox2_node_id_clone_from_ptr(nullptr, node_id_ptr, &node_id_handle);
    NodeId node_id { node_id_handle };

    auto node_state_object = [&] {
        switch (node_state) {
        case iox2_node_state_e_ALIVE:
            return NodeState<T> { AliveNodeView<T> { node_id, node_details } };
        case iox2_node_state_e_DEAD:
            return NodeState<T> { DeadNodeView<T> { AliveNodeView<T> { node_id, node_details } } };
        case iox2_node_state_e_UNDEFINED:
            return NodeState<T> { iox2_node_state_e_UNDEFINED, node_id };
        case iox2_node_state_e_INACCESSIBLE:
            return NodeState<T> { iox2_node_state_e_INACCESSIBLE, node_id };
        }

        IOX_UNREACHABLE();
    }();

    auto* callback = internal::ctx_cast<iox::function<CallbackProgression(NodeState<T>)>>(context);
    return iox::into<iox2_callback_progression_e>(callback->value()(node_state_object));
}
// NOLINTEND(readability-function-size)

} // namespace iox2::internal

#endif
