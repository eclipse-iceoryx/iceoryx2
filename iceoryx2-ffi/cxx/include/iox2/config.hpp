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
#include "iox2/config_creation_error.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/unable_to_deliver_strategy.hpp"

namespace iox2 {
class Config;
class NodeBuilder;

namespace config {
class Global;

/// All configurable settings of a [`Node`].
class Node {
  public:
    /// The directory in which all node files are stored
    auto directory() && -> const char*;
    /// Set the directory in which all node files are stored
    void set_directory(const iox::Path& value) &&;
    /// The suffix of the monitor token
    auto monitor_suffix() && -> const char*;
    /// Set the suffix of the monitor token
    void set_monitor_suffix(const iox::FileName& value) &&;
    /// The suffix of the files where the node configuration is stored.
    auto static_config_suffix() && -> const char*;
    /// Set the suffix of the files where the node configuration is stored.
    void set_static_config_suffix(const iox::FileName& value) &&;
    /// The suffix of the service tags.
    auto service_tag_suffix() && -> const char*;
    /// Set the suffix of the service tags.
    void set_service_tag_suffix(const iox::FileName& value) &&;
    /// When true, the [`NodeBuilder`](NodeBuilder) checks for dead nodes and
    /// cleans up all their stale resources whenever a new [`Node`](Node) is
    /// created.
    auto cleanup_dead_nodes_on_creation() && -> bool;
    /// Enable/disable the cleanup dead nodes on creation
    void set_cleanup_dead_nodes_on_creation(bool value) &&;
    /// When true, the [`NodeBuilder`](NodeBuilder) checks for dead nodes and
    /// cleans up all their stale resources whenever an existing [`Node`](Node) is
    /// going out of scope.
    auto cleanup_dead_nodes_on_destruction() && -> bool;
    /// Enable/disable the cleanup dead nodes on destruction
    void set_cleanup_dead_nodes_on_destruction(bool value) &&;

  private:
    friend class Global;
    explicit Node(iox2_config_h* config);

    iox2_config_h* m_config = nullptr;
};

/// All configurable settings of a [`Service`].
class Service {
  public:
    /// The directory in which all service files are stored
    auto directory() && -> const char*;
    /// Set the directory in which all service files are stored
    void set_directory(const iox::Path& value) &&;
    /// The suffix of the ports data segment
    auto data_segment_suffix() && -> const char*;
    /// Set the suffix of the ports data segment
    void set_data_segment_suffix(const iox::FileName& value) &&;
    /// The suffix of the static config file
    auto static_config_storage_suffix() && -> const char*;
    /// Set the suffix of the static config file
    void set_static_config_storage_suffix(const iox::FileName& value) &&;
    /// The suffix of the dynamic config file
    auto dynamic_config_storage_suffix() && -> const char*;
    /// Set the suffix of the dynamic config file
    void set_dynamic_config_storage_suffix(const iox::FileName& value) &&;
    /// Defines the time of how long another process will wait until the service creation is
    /// finalized
    auto creation_timeout() && -> iox::units::Duration;
    /// Set the creation timeout
    void set_creation_timeout(const iox::units::Duration& value) &&;
    /// The suffix of a one-to-one connection
    auto connection_suffix() && -> const char*;
    /// Set the suffix of a one-to-one connection
    void set_connection_suffix(const iox::FileName& value) &&;
    /// The suffix of a one-to-one connection
    auto event_connection_suffix() && -> const char*;
    /// Set the suffix of a one-to-one connection
    void set_event_connection_suffix(const iox::FileName& value) &&;

  private:
    friend class Global;
    explicit Service(iox2_config_h* config);

    iox2_config_h* m_config = nullptr;
};

/// The global settings
class Global {
  public:
    /// Prefix used for all files created during runtime
    auto prefix() && -> const char*;
    /// Set the prefix used for all files created during runtime
    void set_prefix(const iox::FileName& value) &&;
    /// The path under which all other directories or files will be created
    auto root_path() && -> const char*;
    /// Defines the path under which all other directories or files will be created
    void set_root_path(const iox::Path& value) &&;

    /// Returns the service part of the global configuration
    auto service() -> Service;

    /// Returns the node part of the global configuration
    auto node() -> Node;

  private:
    friend class ::iox2::Config;
    explicit Global(iox2_config_h* config);

    iox2_config_h* m_config = nullptr;
};

/// Default settings for the publish-subscribe messaging pattern. These settings are used unless
/// the user specifies custom QoS or port settings.
class PublishSubscribe {
  public:
    /// The maximum amount of supported [`Subscriber`]s
    auto max_subscribers() && -> size_t;
    /// Set the maximum amount of supported [`Subscriber`]s
    void set_max_subscribers(size_t value) &&;
    /// The maximum amount of supported [`Publisher`]s
    auto max_publishers() && -> size_t;
    /// Set the maximum amount of supported [`Publisher`]s
    void set_max_publishers(size_t value) &&;
    /// The maximum amount of supported [`Node`]s. Defines indirectly how many
    /// processes can open the service at the same time.
    auto max_nodes() && -> size_t;
    /// Set the maximum amount of supported [`Node`]s.
    void set_max_nodes(size_t value) &&;
    /// The maximum buffer size a [`Subscriber`] can have
    auto subscriber_max_buffer_size() && -> size_t;
    /// Set the maximum buffer size a [`Subscriber`] can have
    void set_subscriber_max_buffer_size(size_t value) &&;
    /// The maximum amount of [`Sample`]s a [`Subscriber`] can hold at the same time.
    auto subscriber_max_borrowed_samples() && -> size_t;
    /// Set the maximum amount of [`Sample`]s a [`Subscriber`] can hold at the same time.
    void set_subscriber_max_borrowed_samples(size_t value) &&;
    /// The maximum amount of [`SampleMut`]s a [`Publisher`] can loan at the same time.
    auto publisher_max_loaned_samples() && -> size_t;
    /// The maximum amount of [`SampleMut`]s a [`Publisher`] can loan at the same time.
    void set_publisher_max_loaned_samples(size_t value) &&;
    /// The maximum history size a [`Subscriber`] can request from a [`Publisher`].
    auto publisher_history_size() && -> size_t;
    /// Set the maximum history size a [`Subscriber`] can request from a [`Publisher`].
    void set_publisher_history_size(size_t value) &&;
    /// Defines how the [`Subscriber`] buffer behaves when it is
    /// full. When safe overflow is activated, the [`Publisher`] will
    /// replace the oldest [`Sample`] with the newest one.
    auto enable_safe_overflow() && -> bool;
    /// Enables/disables safe overflow
    void set_enable_safe_overflow(bool value) &&;
    /// If safe overflow is deactivated it defines the deliver strategy of the
    /// [`Publisher`] when the [`Subscriber`]s buffer is full.
    auto unable_to_deliver_strategy() && -> UnableToDeliverStrategy;
    /// Define the unable to deliver strategy
    void set_unable_to_deliver_strategy(UnableToDeliverStrategy value) &&;
    /// Defines the size of the internal [`Subscriber`]
    /// buffer that contains expired connections. An
    /// connection is expired when the [`Publisher`]
    /// disconnected from a service and the connection
    /// still contains unconsumed [`Sample`]s.
    auto subscriber_expired_connection_buffer() && -> size_t;
    /// Set the expired connection buffer size
    void set_subscriber_expired_connection_buffer(size_t value) &&;

  private:
    friend class Defaults;
    explicit PublishSubscribe(iox2_config_h* config);

    iox2_config_h* m_config = nullptr;
};

/// Default settings for the event messaging pattern. These settings are used unless
/// the user specifies custom QoS or port settings.
class Event {
  public:
    /// The maximum amount of supported [`Listener`]
    auto max_listeners() && -> size_t;
    /// Set the maximum amount of supported [`Listener`]
    void set_max_listeners(size_t value) &&;
    /// The maximum amount of supported [`Notifier`]
    auto max_notifiers() && -> size_t;
    /// Set the maximum amount of supported [`Notifier`]
    void set_max_notifiers(size_t value) &&;
    /// The maximum amount of supported [`Node`]s. Defines indirectly how many
    /// processes can open the service at the same time.
    auto max_nodes() && -> size_t;
    /// Set the maximum amount of supported [`Node`]s.
    void set_max_nodes(size_t value) &&;
    /// The largest event id supported by the event service
    auto event_id_max_value() && -> size_t;
    /// Set the largest event id supported by the event service
    void set_event_id_max_value(size_t value) &&;
    /// Defines the event id value that is emitted after a new notifier was created.
    auto notifier_created_event() && -> iox::optional<size_t>;
    /// Sets the event id value that is emitted after a new notifier was created.
    void set_notifier_created_event(iox::optional<size_t> value) &&;
    /// Defines the event id value that is emitted before a new notifier is dropped.
    auto notifier_dropped_event() && -> iox::optional<size_t>;
    /// Sets the event id value that is emitted before a new notifier is dropped.
    void set_notifier_dropped_event(iox::optional<size_t> value) &&;
    /// Defines the event id value that is emitted if a notifier was identified as dead.
    auto notifier_dead_event() && -> iox::optional<size_t>;
    /// Sets the event id value that is emitted if a notifier was identified as dead.
    void set_notifier_dead_event(iox::optional<size_t> value) &&;
    /// Defines the maximum allowed time between two consecutive notifications. If a notifiation
    /// is not sent after the defined time, every [`Listener`]
    /// that is attached to a [`WaitSet`] will be notified.
    auto deadline() && -> iox::optional<iox::units::Duration>;
    /// Sets the deadline of the event service.
    void set_deadline(iox::optional<iox::units::Duration> value) &&;

  private:
    friend class Defaults;
    explicit Event(iox2_config_h* config);

    iox2_config_h* m_config = nullptr;
};

/// Default settings for the request response messaging pattern. These settings are used unless
/// the user specifies custom QoS or port settings.
class RequestResponse {
  public:
    /// Defines if the request buffer of the [`Service`] safely overflows.
    auto enable_safe_overflow_for_requests() && -> bool;
    /// Enables/disables safe overflow for the request buffer.
    void set_enable_safe_overflow_for_requests(bool value) &&;
    /// Defines if the response buffer of the [`Service`] safely overflows.
    auto enable_safe_overflow_for_responses() && -> bool;
    /// Enables/disables safe overflow for the response buffer.
    void set_enable_safe_overflow_for_responses(bool value) &&;
    /// The maximum of [`crate::active_request::ActiveRequest`]s a [`crate::port::server::Server`] can hold in
    /// parallel per [`crate::port::client::Client`].
    auto max_active_requests_per_client() && -> size_t;
    /// Set the maximum of [`crate::active_request::ActiveRequest`]s a [`crate::port::server::Server`] can hold in
    /// parallel per [`crate::port::client::Client`].
    void set_max_active_requests_per_client(size_t value) &&;
    /// The maximum buffer size for [`crate::response::Response`]s for a
    /// [`crate::pending_response::PendingResponse`].
    auto max_response_buffer_size() && -> size_t;
    /// Set the maximum buffer size for [`crate::response::Response`]s for a
    /// [`crate::pending_response::PendingResponse`].
    void set_max_response_buffer_size(size_t value) &&;
    /// The maximum amount of supported [`crate::port::server::Server`]
    auto max_servers() && -> size_t;
    /// Set the maximum amount of supported [`crate::port::server::Server`]
    void set_max_servers(size_t value) &&;
    /// The maximum amount of supported [`crate::port::client::Client`]
    auto max_clients() && -> size_t;
    /// Set the maximum amount of supported [`crate::port::client::Client`]
    void set_max_clients(size_t value) &&;
    /// The maximum amount of supported [`crate::node::Node`]s. Defines
    /// indirectly how many processes can open the service at the same time.
    auto max_nodes() && -> size_t;
    /// Set the maximum amount of supported [`crate::node::Node`]s. Defines
    /// indirectly how many processes can open the service at the same time.
    void set_max_nodes(size_t value) &&;
    /// The maximum amount of borrowed [`crate::response::Response`] per
    /// [`crate::pending_response::PendingResponse`] on the [`crate::port::client::Client`] side.
    auto max_borrowed_responses_per_pending_response() && -> size_t;
    /// Set the maximum amount of borrowed [`crate::response::Response`] per
    /// [`crate::pending_response::PendingResponse`] on the [`crate::port::client::Client`] side.
    void set_max_borrowed_responses_per_pending_response(size_t value) &&;
    /// Defines how many [`crate::request_mut::RequestMut`] a
    /// [`crate::port::client::Client`] can loan in parallel.
    auto max_loaned_requests() && -> size_t;
    /// Set how many [`crate::request_mut::RequestMut`] a
    /// [`crate::port::client::Client`] can loan in parallel.
    void set_max_loaned_requests(size_t value) &&;
    /// Defines how many [`crate::response_mut::ResponseMut`] a [`crate::port::server::Server`] can loan in
    /// parallel per [`crate::active_request::ActiveRequest`].
    auto server_max_loaned_responses_per_request() && -> size_t;
    /// Set how many [`crate::response_mut::ResponseMut`] a [`crate::port::server::Server`] can loan in
    /// parallel per [`crate::active_request::ActiveRequest`].
    void set_server_max_loaned_responses_per_request(size_t value) &&;
    /// Defines the [`UnableToDeliverStrategy`] when a [`Client`](crate::port::client::Client)
    /// could not deliver the request to the [`Server`](crate::port::server::Server).
    auto client_unable_to_deliver_strategy() && -> UnableToDeliverStrategy;
    /// Set the [`UnableToDeliverStrategy`] when a [`Client`](crate::port::client::Client)
    /// could not deliver the request to the [`Server`](crate::port::server::Server).
    void set_client_unable_to_deliver_strategy(UnableToDeliverStrategy value) &&;
    /// Defines the [`UnableToDeliverStrategy`] when a [`Server`](crate::port::server::Server)
    /// could not deliver the response to the [`Client`](crate::port::client::Client).
    auto server_unable_to_deliver_strategy() && -> UnableToDeliverStrategy;
    /// Set the [`UnableToDeliverStrategy`] when a [`Server`](crate::port::server::Server)
    /// could not deliver the response to the [`Client`](crate::port::client::Client).
    void set_server_unable_to_deliver_strategy(UnableToDeliverStrategy value) &&;
    /// Defines the size of the internal [`Client`](crate::port::client::Client)
    /// buffer that contains expired connections. An
    /// connection is expired when the [`Server`](crate::port::server::Server)
    /// disconnected from a service and the connection
    /// still contains unconsumed [`Response`](crate::response::Response)s.
    auto client_expired_connection_buffer() && -> size_t;
    /// Set the size of the internal [`Client`](crate::port::client::Client)
    /// buffer that contains expired connections. An
    /// connection is expired when the [`Server`](crate::port::server::Server)
    /// disconnected from a service and the connection
    /// still contains unconsumed [`Response`](crate::response::Response)s.
    void set_client_expired_connection_buffer(size_t value) &&;
    /// Defines the size of the internal [`Server`]
    /// buffer that contains expired connections. An
    /// connection is expired when the [`Client`]
    /// disconnected from a service and the connection
    /// still contains unconsumed [`ActiveRequest`]s.
    auto server_expired_connection_buffer() && -> size_t;
    /// Set the size of the internal [`Server`]
    /// buffer that contains expired connections. An
    /// connection is expired when the [`Client`]
    /// disconnected from a service and the connection
    /// still contains unconsumed [`ActiveRequest`]s.
    void set_server_expired_connection_buffer(size_t value) &&;
    /// Allows the [`Server`](crate::port::server::Server) to receive
    /// [`RequestMut`](crate::response_mut::ResponseMut)s of
    /// [`Client`](crate::port::client::Client)s that are not interested in a
    /// [`Response`](crate::response::Response), meaning that the
    /// [`Server`](crate::port::server::Server) will receive the
    /// [`RequestMut`](crate::response_mut::ResponseMut) despite the corresponding
    /// [`PendingResponse`](crate::pending_response::PendingResponse) already went out-of-scope.
    /// So any [`Response`](crate::response::Response) sent by the
    /// [`Server`](crate::port::server::Server) would not be received by the corresponding
    /// [`Client`](crate::port::client::Client)s
    /// [`PendingResponse`](crate::pending_response::PendingResponse).
    ///
    /// Consider enabling this feature if you do not want to loose any
    /// [`RequestMut`](crate::response_mut::ResponseMut).
    auto enable_fire_and_forget_requests() && -> bool;
    /// Set if fire-and-forget feature is enabled
    void set_enable_fire_and_forget_requests(bool value) &&;

  private:
    friend class Defaults;
    explicit RequestResponse(iox2_config_h* config);

    iox2_config_h* m_config = nullptr;
};

/// Default settings. These values are used when the user in the code does not specify anything
/// else.
class Defaults {
  public:
    /// Returns the publish_subscribe part of the default settings
    auto publish_subscribe() && -> PublishSubscribe;
    /// Returns the event part of the default settings
    auto event() && -> Event;
    /// Returns the request_response part of the default settings
    auto request_response() && -> RequestResponse;

  private:
    friend class ::iox2::Config;
    explicit Defaults(iox2_config_h* config);

    iox2_config_h* m_config = nullptr;
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
    iox2_config_ptr m_ptr = nullptr;
};

/// Represents the configuration that iceoryx2 will utilize. It is divided into two sections:
/// the [Global] settings, which must align with the iceoryx2 instance the application intends to
/// join, and the [Defaults] for communication within that iceoryx2 instance. The user has the
/// flexibility to override both sections.
class Config {
  public:
    Config();
    Config(const Config& rhs);
    Config(Config&& rhs) noexcept;
    ~Config();

    auto operator=(const Config& rhs) -> Config&;
    auto operator=(Config&& rhs) noexcept -> Config&;

    /// Loads a configuration from a file. On success it returns a [`Config`] object otherwise a
    /// [`ConfigCreationError`] describing the failure.
    static auto from_file(const iox::FilePath& file) -> iox::expected<Config, ConfigCreationError>;

    /// Returns the [`config::Global`] part of the config
    auto global() -> config::Global;
    /// Returns the [`config::Defaults`] part of the config
    auto defaults() -> config::Defaults;

    /// Returns a [`ConfigView`] to the current global config.
    static auto global_config() -> ConfigView;

    /// Returns the [`ConfigView`] to this [`Config`]
    auto view() -> ConfigView;

  private:
    friend class ConfigView;
    friend class config::Global;
    friend class NodeBuilder;
    template <ServiceType>
    friend class DeadNodeView;

    explicit Config(iox2_config_h handle);
    void drop();

    iox2_config_h m_handle = nullptr;
};
} // namespace iox2

#endif
