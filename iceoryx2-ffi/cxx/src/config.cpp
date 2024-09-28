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

#include "iox2/config.hpp"

namespace iox2 {
/////////////////////////
// BEGIN: ConfigView
/////////////////////////
ConfigView::ConfigView(iox2_config_ptr ptr)
    : m_ptr { ptr } {
}

auto ConfigView::to_owned() const -> Config {
    iox2_config_h handle = nullptr;
    iox2_config_from_ptr(m_ptr, nullptr, &handle);

    return Config(handle);
}

/////////////////////////
// END: ConfigView
/////////////////////////

/////////////////////////
// BEGIN: Config
/////////////////////////
Config::Config() {
    iox2_config_default(nullptr, &m_handle);
}

void Config::drop() {
    if (m_handle != nullptr) {
        iox2_config_drop(m_handle);
        m_handle = nullptr;
    }
}

Config::Config(const Config& rhs) {
    auto* ref_handle = iox2_cast_config_h_ref(rhs.m_handle);
    iox2_config_clone(ref_handle, nullptr, &m_handle);
}

Config::Config(Config&& rhs) noexcept
    : m_handle { std::move(rhs.m_handle) } {
    rhs.m_handle = nullptr;
}

Config::~Config() {
    drop();
}

auto Config::operator=(const Config& rhs) -> Config& {
    if (this != &rhs) {
        drop();
        auto* ref_handle = iox2_cast_config_h_ref(rhs.m_handle);
        iox2_config_clone(ref_handle, nullptr, &m_handle);
    }
    return *this;
}


auto Config::operator=(Config&& rhs) noexcept -> Config& {
    if (this != &rhs) {
        drop();
        m_handle = rhs.m_handle;
        rhs.m_handle = nullptr;
    }

    return *this;
}

Config::Config(iox2_config_h handle)
    : m_handle { handle } {
}

auto Config::from_file(const iox::FilePath& file) -> iox::expected<Config, ConfigCreationError> {
    iox2_config_h handle = nullptr;
    auto result = iox2_config_from_file(nullptr, &handle, file.as_string().c_str());
    if (result == IOX2_OK) {
        return iox::ok(Config(handle));
    }

    return iox::err(iox::into<ConfigCreationError>(result));
}

auto Config::global() -> config::Global {
    return config::Global(&this->m_handle);
}

auto Config::defaults() -> config::Defaults {
    return config::Defaults(&this->m_handle);
}

auto Config::global_config() -> ConfigView {
    return ConfigView { iox2_config_global_config() };
}

auto Config::view() -> ConfigView {
    return ConfigView { iox2_cast_config_ptr(m_handle) };
}
/////////////////////////
// END: Config
/////////////////////////

namespace config {
/////////////////////////
// BEGIN: Global
/////////////////////////
Global::Global(iox2_config_h* config)
    : m_config { config } {
}

auto Global::prefix() && -> const char* {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_global_prefix(ref_handle);
}

void Global::set_prefix(const iox::FileName& value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_global_set_prefix(ref_handle, value.as_string().c_str());
}

auto Global::root_path() && -> const char* {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_global_root_path(ref_handle);
}

void Global::set_root_path(const iox::Path& value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_global_set_root_path(ref_handle, value.as_string().c_str());
}

auto Global::service() -> Service {
    return Service(m_config);
}

auto Global::node() -> Node {
    return Node(m_config);
}
/////////////////////////
// END: Global
/////////////////////////

/////////////////////////
// BEGIN: Defaults
/////////////////////////
Defaults::Defaults(iox2_config_h* config)
    : m_config { config } {
}

auto Defaults::publish_subscribe() && -> PublishSubscribe {
    return PublishSubscribe(m_config);
}

auto Defaults::event() && -> Event {
    return Event(m_config);
}
/////////////////////////
// END: Defaults
/////////////////////////

/////////////////////////
// BEGIN: Event
/////////////////////////
Event::Event(iox2_config_h* config)
    : m_config { config } {
}

auto Event::max_listeners() && -> size_t {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_defaults_event_max_listeners(ref_handle);
}

void Event::set_max_listeners(size_t value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_defaults_event_set_max_listeners(ref_handle, value);
}

auto Event::max_notifiers() && -> size_t {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_defaults_event_max_notifiers(ref_handle);
}

void Event::set_max_notifiers(size_t value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_defaults_event_set_max_notifiers(ref_handle, value);
}

auto Event::max_nodes() && -> size_t {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_defaults_event_max_nodes(ref_handle);
}

void Event::set_max_nodes(size_t value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_defaults_event_set_max_nodes(ref_handle, value);
}

auto Event::event_id_max_value() && -> size_t {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_defaults_event_event_id_max_value(ref_handle);
}

void Event::set_event_id_max_value(size_t value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_defaults_event_set_event_id_max_value(ref_handle, value);
}
/////////////////////////
// END: Event
/////////////////////////

/////////////////////////
// BEGIN: PublishSubscribe
/////////////////////////
PublishSubscribe::PublishSubscribe(iox2_config_h* config)
    : m_config { config } {
}

auto PublishSubscribe::max_subscribers() && -> size_t {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_defaults_publish_subscribe_max_subscribers(ref_handle);
}

void PublishSubscribe::set_max_subscribers(size_t value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_defaults_publish_subscribe_set_max_subscribers(ref_handle, value);
}

auto PublishSubscribe::max_publishers() && -> size_t {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_defaults_publish_subscribe_max_publishers(ref_handle);
}

void PublishSubscribe::set_max_publishers(size_t value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_defaults_publish_subscribe_set_max_publishers(ref_handle, value);
}

auto PublishSubscribe::max_nodes() && -> size_t {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_defaults_publish_subscribe_max_nodes(ref_handle);
}

void PublishSubscribe::set_max_nodes(size_t value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_defaults_publish_subscribe_set_max_nodes(ref_handle, value);
}

auto PublishSubscribe::subscriber_max_buffer_size() && -> size_t {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_defaults_publish_subscribe_subscriber_max_buffer_size(ref_handle);
}

void PublishSubscribe::set_subscriber_max_buffer_size(size_t value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_defaults_publish_subscribe_set_subscriber_max_buffer_size(ref_handle, value);
}

auto PublishSubscribe::subscriber_max_borrowed_samples() && -> size_t {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_defaults_publish_subscribe_subscriber_max_borrowed_samples(ref_handle);
}

void PublishSubscribe::set_subscriber_max_borrowed_samples(size_t value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_defaults_publish_subscribe_set_subscriber_max_borrowed_samples(ref_handle, value);
}

auto PublishSubscribe::publisher_max_loaned_samples() && -> size_t {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_defaults_publish_subscribe_publisher_max_loaned_samples(ref_handle);
}

void PublishSubscribe::set_publisher_max_loaned_samples(size_t value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_defaults_publish_subscribe_set_publisher_max_loaned_samples(ref_handle, value);
}

auto PublishSubscribe::publisher_history_size() && -> size_t {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_defaults_publish_subscribe_publisher_history_size(ref_handle);
}

void PublishSubscribe::set_publisher_history_size(size_t value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_defaults_publish_subscribe_set_publisher_history_size(ref_handle, value);
}

auto PublishSubscribe::enable_safe_overflow() && -> bool {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_defaults_publish_subscribe_enable_safe_overflow(ref_handle);
}

void PublishSubscribe::set_enable_safe_overflow(bool value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_defaults_publish_subscribe_set_enable_safe_overflow(ref_handle, value);
}

auto PublishSubscribe::unable_to_deliver_strategy() && -> UnableToDeliverStrategy {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox::into<UnableToDeliverStrategy>(
        iox2_config_defaults_publish_subscribe_unable_to_deliver_strategy(ref_handle));
}

void PublishSubscribe::set_unable_to_deliver_strategy(UnableToDeliverStrategy value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_defaults_publish_subscribe_set_unable_to_deliver_strategy(
        ref_handle, static_cast<iox2_unable_to_deliver_strategy_e>(iox::into<int>(value)));
}

auto PublishSubscribe::subscriber_expired_connection_buffer() && -> size_t {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_defaults_publish_subscribe_subscriber_expired_connection_buffer(ref_handle);
}

void PublishSubscribe::set_subscriber_expired_connection_buffer(size_t value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_defaults_publish_subscribe_set_subscriber_expired_connection_buffer(ref_handle, value);
}
/////////////////////////
// END: PublishSubscribe
/////////////////////////

/////////////////////////
// BEGIN: Service
/////////////////////////
Service::Service(iox2_config_h* config)
    : m_config { config } {
}

auto Service::directory() && -> const char* {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_global_service_directory(ref_handle);
}

void Service::set_directory(const iox::Path& value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_global_service_set_directory(ref_handle, value.as_string().c_str());
}

auto Service::publisher_data_segment_suffix() && -> const char* {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_global_service_publisher_data_segment_suffix(ref_handle);
}

void Service::set_publisher_data_segment_suffix(const iox::FileName& value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_global_service_set_publisher_data_segment_suffix(ref_handle, value.as_string().c_str());
}

auto Service::static_config_storage_suffix() && -> const char* {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_global_service_static_config_storage_suffix(ref_handle);
}

void Service::set_static_config_storage_suffix(const iox::FileName& value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_global_service_set_static_config_storage_suffix(ref_handle, value.as_string().c_str());
}

auto Service::dynamic_config_storage_suffix() && -> const char* {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_global_service_dynamic_config_storage_suffix(ref_handle);
}

void Service::set_dynamic_config_storage_suffix(const iox::FileName& value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_global_service_set_dynamic_config_storage_suffix(ref_handle, value.as_string().c_str());
}

auto Service::creation_timeout() && -> iox::units::Duration {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    uint64_t secs = 0;
    uint32_t nsecs = 0;
    iox2_config_global_service_creation_timeout(ref_handle, &secs, &nsecs);

    return iox::units::Duration::fromSeconds(secs) + iox::units::Duration::fromNanoseconds(nsecs);
}

void Service::set_creation_timeout(const iox::units::Duration& value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    auto duration = value.timespec();
    iox2_config_global_service_set_creation_timeout(ref_handle, duration.tv_sec, duration.tv_nsec);
}

auto Service::connection_suffix() && -> const char* {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_global_service_connection_suffix(ref_handle);
}

void Service::set_connection_suffix(const iox::FileName& value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_global_service_set_connection_suffix(ref_handle, value.as_string().c_str());
}

auto Service::event_connection_suffix() && -> const char* {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_global_service_event_connection_suffix(ref_handle);
}

void Service::set_event_connection_suffix(const iox::FileName& value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_global_service_set_event_connection_suffix(ref_handle, value.as_string().c_str());
}
/////////////////////////
// END: Service
/////////////////////////

/////////////////////////
// BEGIN: Node
/////////////////////////
Node::Node(iox2_config_h* config)
    : m_config { config } {
}

auto Node::directory() && -> const char* {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_global_node_directory(ref_handle);
}

void Node::set_directory(const iox::Path& value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_global_node_set_directory(ref_handle, value.as_string().c_str());
}

auto Node::monitor_suffix() && -> const char* {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_global_node_monitor_suffix(ref_handle);
}

void Node::set_monitor_suffix(const iox::FileName& value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_global_node_set_monitor_suffix(ref_handle, value.as_string().c_str());
}

auto Node::static_config_suffix() && -> const char* {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_global_node_static_config_suffix(ref_handle);
}

void Node::set_static_config_suffix(const iox::FileName& value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_global_node_set_static_config_suffix(ref_handle, value.as_string().c_str());
}

auto Node::service_tag_suffix() && -> const char* {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_global_node_service_tag_suffix(ref_handle);
}

void Node::set_service_tag_suffix(const iox::FileName& value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_global_node_set_service_tag_suffix(ref_handle, value.as_string().c_str());
}

auto Node::cleanup_dead_nodes_on_creation() && -> bool {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_global_node_cleanup_dead_nodes_on_creation(ref_handle);
}

void Node::set_cleanup_dead_nodes_on_creation(bool value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_global_node_set_cleanup_dead_nodes_on_creation(ref_handle, value);
}

auto Node::cleanup_dead_nodes_on_destruction() && -> bool {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    return iox2_config_global_node_cleanup_dead_nodes_on_destruction(ref_handle);
}

void Node::set_cleanup_dead_nodes_on_destruction(bool value) && {
    auto* ref_handle = iox2_cast_config_h_ref(*m_config);
    iox2_config_global_node_set_cleanup_dead_nodes_on_destruction(ref_handle, value);
}
/////////////////////////
// END: Node
/////////////////////////
} // namespace config
} // namespace iox2
