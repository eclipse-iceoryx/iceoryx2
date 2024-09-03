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

#ifndef IOX2_CONFIG_HPP
#define IOX2_CONFIG_HPP

#include "iox/duration.hpp"
#include "iox/file_name.hpp"
#include "iox/path.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/unable_to_deliver_strategy.hpp"

namespace iox2 {
class Config;

namespace config {
class Global;

class Node {
  public:
    auto directory() && -> const char*;
    auto set_directory(const iox::Path& value) &&;
    auto monitor_suffix() && -> const char*;
    auto set_monitor_suffix(const iox::FileName& value) &&;
    auto static_config_suffix() && -> const char*;
    auto set_static_config_suffix(const iox::FileName& value) &&;
    auto service_tag_suffix() && -> const char*;
    auto set_service_tag_suffix(const iox::FileName& value) &&;
    auto cleanup_dead_nodes_on_creation() && -> bool;
    auto set_cleanup_dead_nodes_on_creation(bool value) &&;
    auto cleanup_dead_nodes_on_destruction() && -> bool;
    auto set_cleanup_dead_nodes_on_destruction(bool value) &&;

  private:
    friend class Global;
    explicit Node(Config* config);

    Config* m_config;
};

class Service {
  public:
    auto directory() && -> const char*;
    auto set_directory(const iox::Path& value) &&;
    auto publisher_data_segment_suffix() && -> const char*;
    auto set_publisher_data_segment_suffix(const iox::FileName& value) &&;
    auto static_config_storage_suffix() && -> const char*;
    auto set_static_config_storage_suffix(const iox::FileName& value) &&;
    auto dynamic_config_storage_suffix() && -> const char*;
    auto set_dynamic_config_storage_suffix(const iox::FileName& value) &&;
    auto creation_timeout() && -> iox::units::Duration;
    auto set_creation_timeout(const iox::units::Duration& value) &&;
    auto connection_suffix() && -> const char*;
    auto set_connection_suffix(const iox::FileName& value) &&;
    auto event_connection_suffix() && -> const char*;
    auto set_event_connection_suffix(const iox::FileName& value) &&;

  private:
    friend class Global;
    explicit Service(Config* config);

    Config* m_config;
};

class Global {
  public:
    auto prefix() && -> const char*;
    void set_prefix(const iox::FileName& value) &&;
    auto root_path() && -> const char*;
    void set_root_path(const iox::Path& value) &&;

    auto service() -> Service;
    auto node() -> Node;

  private:
    friend class ::iox2::Config;
    explicit Global(Config* config);

    Config* m_config;
};

class PublishSubscribe {
  public:
    auto max_subscribers() -> size_t;
    void set_max_subscribers(size_t value);
    auto max_publishers() -> size_t;
    void set_max_publishers(size_t value);
    auto max_nodes() -> size_t;
    void set_max_nodes(size_t value);
    auto subscriber_max_buffer_size() -> size_t;
    void set_subscriber_max_buffer_size(size_t value);
    auto subscriber_max_borrowed_samples() -> size_t;
    void set_subscriber_max_borrowed_samples(size_t value);
    auto publisher_max_loaned_samples() -> size_t;
    void set_publisher_max_loaned_samples(size_t value);
    auto publisher_history_size() -> size_t;
    void set_history_sizeed_samples(size_t value);
    auto enable_safe_overflow() -> bool;
    void set_enable_safe_overflow(bool value);
    auto unable_to_deliver_strategy() -> UnableToDeliverStrategy;
    void set_unable_to_deliver_strategy(UnableToDeliverStrategy value);
    auto subscriber_expired_connection_buffer() -> size_t;
    void set_subscriber_expired_connection_buffer(size_t value);

  private:
    friend class Defaults;
    explicit PublishSubscribe(Config* config);

    Config* m_config;
};

class Event {
  public:
    auto max_listeners() -> size_t;
    void set_max_listeners(size_t value);
    auto max_notifiers() -> size_t;
    void set_max_notifiers(size_t value);
    auto max_nodes() -> size_t;
    void set_max_nodes(size_t value);
    auto event_id_max_value() -> size_t;
    void set_event_id_max_value(size_t value);

  private:
    friend class Defaults;
    explicit Event(Config* config);

    Config* m_config;
};

class Defaults {
  public:
    auto publish_subscribe() -> PublishSubscribe;
    auto event() -> Event;

  private:
    friend class ::iox2::Config;
    explicit Defaults(Config* config);

    Config* m_config;
};
} // namespace config

/// Non-owning view of a [`Config`].
class ConfigView {
  public:
    ConfigView(const ConfigView&) = default;
    ConfigView(ConfigView&&) = default;
    auto operator=(const ConfigView&) -> ConfigView& = default;
    auto operator=(ConfigView&&) -> ConfigView& = default;
    ~ConfigView() = default;

    /// Creates a copy of the corresponding [`Config`] and returns it.
    auto to_owned() const -> Config;

  private:
    friend class Config;
    template <ServiceType>
    friend class Node;

    template <ServiceType>
    friend class Service;

    explicit ConfigView(iox2_config_ptr ptr);
    iox2_config_ptr m_ptr;
};

class Config {
  public:
    Config();
    Config(const Config& rhs);
    Config(Config&& rhs) noexcept;
    ~Config();

    auto operator=(const Config& rhs) -> Config&;
    auto operator=(Config&& rhs) noexcept -> Config&;

    auto global() -> config::Global;
    auto defaults() -> config::Defaults;

    /// Returns a [`ConfigView`] to the current global config.
    static auto global_config() -> ConfigView;

  private:
    friend class ConfigView;
    friend class config::Global;
    explicit Config(iox2_config_h handle);
    void drop();

    iox2_config_h m_handle = nullptr;
};
} // namespace iox2

#endif
