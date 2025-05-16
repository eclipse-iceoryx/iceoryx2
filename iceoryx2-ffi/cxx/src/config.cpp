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
    iox2_config_clone(&rhs.m_handle, nullptr, &m_handle);
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
        iox2_config_clone(&rhs.m_handle, nullptr, &m_handle);
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
    return iox2_config_global_prefix(m_config);
}

void Global::set_prefix(const iox::FileName& value) && {
    iox2_config_global_set_prefix(m_config, value.as_string().c_str());
}

auto Global::root_path() && -> const char* {
    return iox2_config_global_root_path(m_config);
}

void Global::set_root_path(const iox::Path& value) && {
    iox2_config_global_set_root_path(m_config, value.as_string().c_str());
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

auto Defaults::request_response() && -> RequestResponse {
    return RequestResponse(m_config);
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
    return iox2_config_defaults_event_max_listeners(m_config);
}

void Event::set_max_listeners(size_t value) && {
    iox2_config_defaults_event_set_max_listeners(m_config, value);
}

auto Event::max_notifiers() && -> size_t {
    return iox2_config_defaults_event_max_notifiers(m_config);
}

void Event::set_max_notifiers(size_t value) && {
    iox2_config_defaults_event_set_max_notifiers(m_config, value);
}

auto Event::max_nodes() && -> size_t {
    return iox2_config_defaults_event_max_nodes(m_config);
}

void Event::set_max_nodes(size_t value) && {
    iox2_config_defaults_event_set_max_nodes(m_config, value);
}

auto Event::event_id_max_value() && -> size_t {
    return iox2_config_defaults_event_event_id_max_value(m_config);
}

void Event::set_event_id_max_value(size_t value) && {
    iox2_config_defaults_event_set_event_id_max_value(m_config, value);
}

auto Event::notifier_created_event() && -> iox::optional<size_t> {
    size_t value = 0;
    if (iox2_config_defaults_event_notifier_created_event(m_config, &value)) {
        return { value };
    }

    return iox::nullopt;
}

void Event::set_notifier_created_event(iox::optional<size_t> value) && {
    if (value.has_value()) {
        iox2_config_defaults_event_set_notifier_created_event(m_config, &*value);
    } else {
        iox2_config_defaults_event_set_notifier_created_event(m_config, nullptr);
    }
}

auto Event::notifier_dropped_event() && -> iox::optional<size_t> {
    size_t value = 0;
    if (iox2_config_defaults_event_notifier_dropped_event(m_config, &value)) {
        return { value };
    }

    return iox::nullopt;
}

void Event::set_notifier_dropped_event(iox::optional<size_t> value) && {
    if (value.has_value()) {
        iox2_config_defaults_event_set_notifier_dropped_event(m_config, &*value);
    } else {
        iox2_config_defaults_event_set_notifier_dropped_event(m_config, nullptr);
    }
}

auto Event::notifier_dead_event() && -> iox::optional<size_t> {
    size_t value = 0;
    if (iox2_config_defaults_event_notifier_dead_event(m_config, &value)) {
        return { value };
    }

    return iox::nullopt;
}

void Event::set_notifier_dead_event(iox::optional<size_t> value) && {
    if (value.has_value()) {
        iox2_config_defaults_event_set_notifier_dead_event(m_config, &*value);
    } else {
        iox2_config_defaults_event_set_notifier_dead_event(m_config, nullptr);
    }
}

auto Event::deadline() && -> iox::optional<iox::units::Duration> {
    uint64_t seconds = 0;
    uint32_t nanoseconds = 0;
    if (iox2_config_defaults_event_deadline(m_config, &seconds, &nanoseconds)) {
        return { iox::units::Duration::fromSeconds(seconds) + iox::units::Duration::fromNanoseconds(nanoseconds) };
    }

    return iox::nullopt;
}

void Event::set_deadline(iox::optional<iox::units::Duration> value) && {
    value
        .and_then([&](auto value) {
            const uint64_t seconds = value.toSeconds();
            const uint32_t nanoseconds =
                value.toNanoseconds() - (value.toSeconds() * iox::units::Duration::NANOSECS_PER_SEC);
            iox2_config_defaults_event_set_deadline(m_config, &seconds, &nanoseconds);
        })
        .or_else([&] { iox2_config_defaults_event_set_deadline(m_config, nullptr, nullptr); });
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
    return iox2_config_defaults_publish_subscribe_max_subscribers(m_config);
}

void PublishSubscribe::set_max_subscribers(size_t value) && {
    iox2_config_defaults_publish_subscribe_set_max_subscribers(m_config, value);
}

auto PublishSubscribe::max_publishers() && -> size_t {
    return iox2_config_defaults_publish_subscribe_max_publishers(m_config);
}

void PublishSubscribe::set_max_publishers(size_t value) && {
    iox2_config_defaults_publish_subscribe_set_max_publishers(m_config, value);
}

auto PublishSubscribe::max_nodes() && -> size_t {
    return iox2_config_defaults_publish_subscribe_max_nodes(m_config);
}

void PublishSubscribe::set_max_nodes(size_t value) && {
    iox2_config_defaults_publish_subscribe_set_max_nodes(m_config, value);
}

auto PublishSubscribe::subscriber_max_buffer_size() && -> size_t {
    return iox2_config_defaults_publish_subscribe_subscriber_max_buffer_size(m_config);
}

void PublishSubscribe::set_subscriber_max_buffer_size(size_t value) && {
    iox2_config_defaults_publish_subscribe_set_subscriber_max_buffer_size(m_config, value);
}

auto PublishSubscribe::subscriber_max_borrowed_samples() && -> size_t {
    return iox2_config_defaults_publish_subscribe_subscriber_max_borrowed_samples(m_config);
}

void PublishSubscribe::set_subscriber_max_borrowed_samples(size_t value) && {
    iox2_config_defaults_publish_subscribe_set_subscriber_max_borrowed_samples(m_config, value);
}

auto PublishSubscribe::publisher_max_loaned_samples() && -> size_t {
    return iox2_config_defaults_publish_subscribe_publisher_max_loaned_samples(m_config);
}

void PublishSubscribe::set_publisher_max_loaned_samples(size_t value) && {
    iox2_config_defaults_publish_subscribe_set_publisher_max_loaned_samples(m_config, value);
}

auto PublishSubscribe::publisher_history_size() && -> size_t {
    return iox2_config_defaults_publish_subscribe_publisher_history_size(m_config);
}

void PublishSubscribe::set_publisher_history_size(size_t value) && {
    iox2_config_defaults_publish_subscribe_set_publisher_history_size(m_config, value);
}

auto PublishSubscribe::enable_safe_overflow() && -> bool {
    return iox2_config_defaults_publish_subscribe_enable_safe_overflow(m_config);
}

void PublishSubscribe::set_enable_safe_overflow(bool value) && {
    iox2_config_defaults_publish_subscribe_set_enable_safe_overflow(m_config, value);
}

auto PublishSubscribe::unable_to_deliver_strategy() && -> UnableToDeliverStrategy {
    return iox::into<UnableToDeliverStrategy>(
        iox2_config_defaults_publish_subscribe_unable_to_deliver_strategy(m_config));
}

void PublishSubscribe::set_unable_to_deliver_strategy(UnableToDeliverStrategy value) && {
    iox2_config_defaults_publish_subscribe_set_unable_to_deliver_strategy(
        m_config, static_cast<iox2_unable_to_deliver_strategy_e>(iox::into<int>(value)));
}

auto PublishSubscribe::subscriber_expired_connection_buffer() && -> size_t {
    return iox2_config_defaults_publish_subscribe_subscriber_expired_connection_buffer(m_config);
}

void PublishSubscribe::set_subscriber_expired_connection_buffer(size_t value) && {
    iox2_config_defaults_publish_subscribe_set_subscriber_expired_connection_buffer(m_config, value);
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
    return iox2_config_global_service_directory(m_config);
}

void Service::set_directory(const iox::Path& value) && {
    iox2_config_global_service_set_directory(m_config, value.as_string().c_str());
}

auto Service::data_segment_suffix() && -> const char* {
    return iox2_config_global_service_data_segment_suffix(m_config);
}

void Service::set_data_segment_suffix(const iox::FileName& value) && {
    iox2_config_global_service_set_data_segment_suffix(m_config, value.as_string().c_str());
}

auto Service::static_config_storage_suffix() && -> const char* {
    return iox2_config_global_service_static_config_storage_suffix(m_config);
}

void Service::set_static_config_storage_suffix(const iox::FileName& value) && {
    iox2_config_global_service_set_static_config_storage_suffix(m_config, value.as_string().c_str());
}

auto Service::dynamic_config_storage_suffix() && -> const char* {
    return iox2_config_global_service_dynamic_config_storage_suffix(m_config);
}

void Service::set_dynamic_config_storage_suffix(const iox::FileName& value) && {
    iox2_config_global_service_set_dynamic_config_storage_suffix(m_config, value.as_string().c_str());
}

auto Service::creation_timeout() && -> iox::units::Duration {
    uint64_t secs = 0;
    uint32_t nsecs = 0;
    iox2_config_global_service_creation_timeout(m_config, &secs, &nsecs);

    return iox::units::Duration::fromSeconds(secs) + iox::units::Duration::fromNanoseconds(nsecs);
}

void Service::set_creation_timeout(const iox::units::Duration& value) && {
    auto duration = value.timespec();
    iox2_config_global_service_set_creation_timeout(m_config, duration.tv_sec, duration.tv_nsec);
}

auto Service::connection_suffix() && -> const char* {
    return iox2_config_global_service_connection_suffix(m_config);
}

void Service::set_connection_suffix(const iox::FileName& value) && {
    iox2_config_global_service_set_connection_suffix(m_config, value.as_string().c_str());
}

auto Service::event_connection_suffix() && -> const char* {
    return iox2_config_global_service_event_connection_suffix(m_config);
}

void Service::set_event_connection_suffix(const iox::FileName& value) && {
    iox2_config_global_service_set_event_connection_suffix(m_config, value.as_string().c_str());
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
    return iox2_config_global_node_directory(m_config);
}

void Node::set_directory(const iox::Path& value) && {
    iox2_config_global_node_set_directory(m_config, value.as_string().c_str());
}

auto Node::monitor_suffix() && -> const char* {
    return iox2_config_global_node_monitor_suffix(m_config);
}

void Node::set_monitor_suffix(const iox::FileName& value) && {
    iox2_config_global_node_set_monitor_suffix(m_config, value.as_string().c_str());
}

auto Node::static_config_suffix() && -> const char* {
    return iox2_config_global_node_static_config_suffix(m_config);
}

void Node::set_static_config_suffix(const iox::FileName& value) && {
    iox2_config_global_node_set_static_config_suffix(m_config, value.as_string().c_str());
}

auto Node::service_tag_suffix() && -> const char* {
    return iox2_config_global_node_service_tag_suffix(m_config);
}

void Node::set_service_tag_suffix(const iox::FileName& value) && {
    iox2_config_global_node_set_service_tag_suffix(m_config, value.as_string().c_str());
}

auto Node::cleanup_dead_nodes_on_creation() && -> bool {
    return iox2_config_global_node_cleanup_dead_nodes_on_creation(m_config);
}

void Node::set_cleanup_dead_nodes_on_creation(bool value) && {
    iox2_config_global_node_set_cleanup_dead_nodes_on_creation(m_config, value);
}

auto Node::cleanup_dead_nodes_on_destruction() && -> bool {
    return iox2_config_global_node_cleanup_dead_nodes_on_destruction(m_config);
}

void Node::set_cleanup_dead_nodes_on_destruction(bool value) && {
    iox2_config_global_node_set_cleanup_dead_nodes_on_destruction(m_config, value);
}
/////////////////////////
// END: Node
/////////////////////////

/////////////////////////
// BEGIN: RequestResponse
/////////////////////////
RequestResponse::RequestResponse(iox2_config_h* config)
    : m_config { config } {
}

auto RequestResponse::enable_safe_overflow_for_requests() && -> bool {
    return iox2_config_defaults_request_response_enable_safe_overflow_for_requests(m_config);
}

void RequestResponse::set_enable_safe_overflow_for_requests(bool value) && {
    iox2_config_defaults_request_response_set_enable_safe_overflow_for_requests(m_config, value);
}

auto RequestResponse::enable_safe_overflow_for_responses() && -> bool {
    return iox2_config_defaults_request_response_enable_safe_overflow_for_responses(m_config);
}

void RequestResponse::set_enable_safe_overflow_for_responses(bool value) && {
    iox2_config_defaults_request_response_set_enable_safe_overflow_for_responses(m_config, value);
}

auto RequestResponse::max_active_requests_per_client() && -> size_t {
    return iox2_config_defaults_request_response_max_active_requests_per_client(m_config);
}

void RequestResponse::set_max_active_requests_per_client(size_t value) && {
    iox2_config_defaults_request_response_set_max_active_requests_per_client(m_config, value);
}

auto RequestResponse::max_response_buffer_size() && -> size_t {
    return iox2_config_defaults_request_response_max_response_buffer_size(m_config);
}

void RequestResponse::set_max_response_buffer_size(size_t value) && {
    iox2_config_defaults_request_response_set_max_response_buffer_size(m_config, value);
}

auto RequestResponse::max_servers() && -> size_t {
    return iox2_config_defaults_request_response_max_servers(m_config);
}

void RequestResponse::set_max_servers(size_t value) && {
    iox2_config_defaults_request_response_set_max_servers(m_config, value);
}

auto RequestResponse::max_clients() && -> size_t {
    return iox2_config_defaults_request_response_max_clients(m_config);
}

void RequestResponse::set_max_clients(size_t value) && {
    iox2_config_defaults_request_response_set_max_clients(m_config, value);
}

auto RequestResponse::max_nodes() && -> size_t {
    return iox2_config_defaults_request_response_max_nodes(m_config);
}

void RequestResponse::set_max_nodes(size_t value) && {
    iox2_config_defaults_request_response_set_max_nodes(m_config, value);
}

auto RequestResponse::max_borrowed_responses_per_pending_response() && -> size_t {
    return iox2_config_defaults_request_response_max_borrowed_responses_per_pending_response(m_config);
}

void RequestResponse::set_max_borrowed_responses_per_pending_response(size_t value) && {
    iox2_config_defaults_request_response_set_max_borrowed_responses_per_pending_response(m_config, value);
}

auto RequestResponse::max_loaned_requests() && -> size_t {
    return iox2_config_defaults_request_response_max_loaned_requests(m_config);
}

void RequestResponse::set_max_loaned_requests(size_t value) && {
    iox2_config_defaults_request_response_set_max_loaned_requests(m_config, value);
}

auto RequestResponse::server_max_loaned_responses_per_request() && -> size_t {
    return iox2_config_defaults_request_response_server_max_loaned_responses_per_request(m_config);
}

void RequestResponse::set_server_max_loaned_responses_per_request(size_t value) && {
    iox2_config_defaults_request_response_set_server_max_loaned_responses_per_request(m_config, value);
}

auto RequestResponse::client_unable_to_deliver_strategy() && -> UnableToDeliverStrategy {
    return iox::into<UnableToDeliverStrategy>(
        iox2_config_defaults_request_response_client_unable_to_deliver_strategy(m_config));
}

void RequestResponse::set_client_unable_to_deliver_strategy(UnableToDeliverStrategy value) && {
    iox2_config_defaults_request_response_set_client_unable_to_deliver_strategy(
        m_config, static_cast<iox2_unable_to_deliver_strategy_e>(value));
}

auto RequestResponse::server_unable_to_deliver_strategy() && -> UnableToDeliverStrategy {
    return iox::into<UnableToDeliverStrategy>(
        iox2_config_defaults_request_response_server_unable_to_deliver_strategy(m_config));
}

void RequestResponse::set_server_unable_to_deliver_strategy(UnableToDeliverStrategy value) && {
    iox2_config_defaults_request_response_set_server_unable_to_deliver_strategy(
        m_config, static_cast<iox2_unable_to_deliver_strategy_e>(value));
}

auto RequestResponse::client_expired_connection_buffer() && -> size_t {
    return iox2_config_defaults_request_response_client_expired_connection_buffer(m_config);
}

void RequestResponse::set_client_expired_connection_buffer(size_t value) && {
    iox2_config_defaults_request_response_set_client_expired_connection_buffer(m_config, value);
}

auto RequestResponse::server_expired_connection_buffer() && -> size_t {
    return iox2_config_defaults_request_response_server_expired_connection_buffer(m_config);
}

void RequestResponse::set_server_expired_connection_buffer(size_t value) && {
    iox2_config_defaults_request_response_set_server_expired_connection_buffer(m_config, value);
}

auto RequestResponse::enable_fire_and_forget_requests() && -> bool {
    return iox2_config_defaults_request_response_has_fire_and_forget_requests(m_config);
}

void RequestResponse::set_enable_fire_and_forget_requests(bool value) && {
    iox2_config_defaults_request_response_set_fire_and_forget_requests(m_config, value);
}
/////////////////////////
// END: RequestResponse
/////////////////////////
} // namespace config
} // namespace iox2
