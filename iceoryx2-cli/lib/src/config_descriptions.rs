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

use iceoryx2::config::Config;
pub struct CliEntry {
    pub key: &'static str,
    pub value_type: &'static str,
    pub default: String,
    pub description: &'static str,
}

pub struct CliSection {
    pub name: &'static str,
    pub entries: Vec<CliEntry>,
}

pub fn get_sections() -> Vec<CliSection> {
    let config = Config::default();

    vec![
        CliSection {
            name: "Global",
            entries: vec![
                CliEntry {
                    key: "global.root-path",
                    value_type: "string",
                    default: format!("\"{}\"", config.global.root_path().to_string()),
                    description: "Defines the path for all iceoryx2 files and directories.",
                },
                CliEntry {
                    key: "global.prefix",
                    value_type: "string",
                    default: format!("\"{}\"", config.global.prefix.to_string()),
                    description: "Prefix that is used for every file iceoryx2 creates.",
                },
            ],
        },
        CliSection {
            name: "Global: Service",
            entries: vec![
                CliEntry {
                    key: "global.service.directory",
                    value_type: "string",
                    default: format!("\"{}\"", config.global.service.directory.to_string()),
                    description: "Specifies the path for service-related files under `global.root-path`.",
                },
                CliEntry {
                    key: "global.service.data-segment-suffix",
                    value_type: "string",
                    default: format!("\"{}\"", config.global.service.data_segment_suffix.to_string()),
                    description: "Suffix added to the ports's data segment.",
                },
                CliEntry {
                    key: "global.service.static-config-storage-suffix",
                    value_type: "string",
                    default: format!("\"{}\"", config.global.service.static_config_storage_suffix.to_string()),
                    description: "Suffix for static service configuration files.",
                },
                CliEntry {
                    key: "global.service.dynamic-config-storage-suffix",
                    value_type: "string",
                    default: format!("\"{}\"", config.global.service.dynamic_config_storage_suffix.to_string()),
                    description: "Suffix for dynamic service configuration files.",
                },
                CliEntry {
                    key: "global.service.event-connection-suffix",
                    value_type: "string",
                    default: format!("\"{}\"", config.global.service.event_connection_suffix.to_string()),
                    description: "Suffix for event channel.",
                },
                CliEntry {
                    key: "global.service.connection-suffix",
                    value_type: "string",
                    default: format!("\"{}\"", config.global.service.connection_suffix.to_string()),
                    description: "Suffix for one-to-one connections.",
                },
                CliEntry {
                    key: "global.service.blackboard-mgmt-suffix",
                    value_type: "string",
                    default: format!("\"{}\"", config.global.service.blackboard_mgmt_suffix.to_string()),
                    description: "The suffix of the blackboard management data segment.",
                },
                CliEntry {
                    key: "global.service.blackboard-data-suffix",
                    value_type: "string",
                    default: format!("\"{}\"", config.global.service.blackboard_data_suffix.to_string()),
                    description: "The suffix of the blackboard payload data segment.",
                },
            ],
        },
        CliSection {
            name: "Global: Node",
            entries: vec![
                CliEntry {
                    key: "global.node.directory",
                    value_type: "string",
                    default: format!("\"{}\"", config.global.node.directory.to_string()),
                    description: "Specifies the path for node-related files under `global.root-path`.",
                },
                CliEntry {
                    key: "global.node.monitor-suffix",
                    value_type: "string",
                    default: format!("\"{}\"", config.global.node.monitor_suffix.to_string()),
                    description: "Suffix added to the node monitor.",
                },
                CliEntry {
                    key: "global.node.static-config-suffix",
                    value_type: "string",
                    default: format!("\"{}\"", config.global.node.static_config_suffix.to_string()),
                    description: "Suffix added to the static config of the node.",
                },
                CliEntry {
                    key: "global.node.service-tag-suffix",
                    value_type: "string",
                    default: format!("\"{}\"", config.global.node.service_tag_suffix.to_string()),
                    description: "Suffix added to the service tag of the node.",
                },
                CliEntry {
                    key: "global.node.cleanup-dead-nodes-on-creation",
                    value_type: "`true`|`false`",
                    default: config.global.node.cleanup_dead_nodes_on_creation.to_string(),
                    description: "Defines if there shall be a scan for dead nodes with a following stale resource cleanup whenever a new node is created.",
                },
                CliEntry {
                    key: "global.node.cleanup-dead-nodes-on-destruction",
                    value_type: "`true`|`false`",
                    default: config.global.node.cleanup_dead_nodes_on_destruction.to_string(),
                    description: "Defines if there shall be a scan for dead nodes with a following stale resource cleanup whenever a node is going out-of-scope.",
                },
            ],
        },
        CliSection{
            name: "Global: Service Creation Timeout",
            entries: vec![
                CliEntry {
                    key: "global.service.creation-timeout.secs",
                    value_type: "int",
                    default: config.global.service.creation_timeout.as_secs().to_string(),
                    description: "Maximum time for service setup in seconds. Uncreated services after this are marked as stalled.",
                },
                CliEntry {
                    key: "global.service.creation-timeout.nanos",
                    value_type: "int",
                    default: config.global.service.creation_timeout.subsec_nanos().to_string(),
                    description: "Additional nanoseconds for service setup timeout.",
                },
            ],
        },
        CliSection {
            name: "Defaults: Publish Subscribe Messaging Pattern",
            entries: vec![
                CliEntry {
                    key: "defaults.publish-subscribe.max-subscribers",
                    value_type: "int",
                    default: config.defaults.publish_subscribe.max_subscribers.to_string(),
                    description: "Maximum number of subscribers.",
                },
                CliEntry {
                    key: "defaults.publish-subscribe.max-publishers",
                    value_type: "int",
                    default: config.defaults.publish_subscribe.max_publishers.to_string(),
                    description: "Maximum number of publishers.",
                },
                CliEntry {
                    key: "defaults.publish-subscribe.max-nodes",
                    value_type: "int",
                    default: config.defaults.publish_subscribe.max_nodes.to_string(),
                    description: "Maximum number of nodes.",
                },
                CliEntry {
                    key: "defaults.publish-subscribe.subscriber-max-buffer-size",
                    value_type: "int",
                    default: config.defaults.publish_subscribe.subscriber_max_buffer_size.to_string(),
                    description: "Maximum buffer size of a subscriber.",
                },
                CliEntry {
                    key: "defaults.publish-subscribe.subscriber-max-borrowed-samples",
                    value_type: "int",
                    default: config.defaults.publish_subscribe.subscriber_max_borrowed_samples.to_string(),
                    description: "Maximum samples a subscriber can hold.",
                },
                CliEntry {
                    key: "defaults.publish-subscribe.publisher-max-loaned-samples",
                    value_type: "int",
                    default: config.defaults.publish_subscribe.publisher_max_loaned_samples.to_string(),
                    description: "Maximum samples a publisher can loan.",
                },
                CliEntry {
                    key: "defaults.publish-subscribe.publisher-history-size",
                    value_type: "int",
                    default: config.defaults.publish_subscribe.publisher_history_size.to_string(),
                    description: "Maximum history size a subscriber can request.",
                },
                CliEntry {
                    key: "defaults.publish-subscribe.enable-safe-overflow",
                    value_type: "`true`|`false`",
                    default: config.defaults.publish_subscribe.enable_safe_overflow.to_string(),
                    description: "Default overflow behavior.",
                },
                CliEntry {
                    key: "defaults.publish-subscribe.unable-to-deliver-strategy",
                    value_type: "`Block`|`DiscardSample`",
                    default: format!("{:?}", config.defaults.publish_subscribe.unable_to_deliver_strategy),
                    description: "Default strategy for non-overflowing setups when delivery fails.",
                },
                CliEntry {
                    key: "defaults.publish-subscribe.subscriber-expired-connection-buffer",
                    value_type: "int",
                    default: config.defaults.publish_subscribe.subscriber_expired_connection_buffer.to_string(),
                    description: "Expired connection buffer size of the subscriber. Connections to publishers are expired when the publisher disconnected from the service and the connection contains unconsumed samples.",
                },
            ],
        },
        CliSection {
            name: "Defaults: Event Messaging Pattern",
            entries: vec![
                CliEntry {
                    key: "defaults.event.max-listeners",
                    value_type: "int",
                    default: config.defaults.event.max_listeners.to_string(),
                    description: "Maximum number of listeners.",
                },
                CliEntry {
                    key: "defaults.event.max-notifiers",
                    value_type: "int",
                    default: config.defaults.event.max_notifiers.to_string(),
                    description: "Maximum number of notifiers.",
                },
                CliEntry {
                    key: "defaults.event.max-nodes",
                    value_type: "int",
                    default: config.defaults.event.max_nodes.to_string(),
                    description: "Maximum number of nodes.",
                },
                CliEntry {
                    key: "defaults.event.event-id-max-value",
                    value_type: "int",
                    default: config.defaults.event.event_id_max_value.to_string(),
                    description: "Greatest value an [`EventId`] can have.",
                },
                CliEntry {
                    key: "defaults.event.deadline",
                    value_type: "Option<Duration>",
                    default: config.defaults.event.deadline.map_or("None".to_string(), |e| format!("{e:?}")),
                    description: "Maximum allowed time between two consecutive notifications. If not sent after this time, all listeners attached to a WaitSet will be notified.",
                },
                CliEntry {
                    key: "defaults.event.notifier-created-event",
                    value_type: "Option<int>",
                    default: config.defaults.event.notifier_created_event.map_or("None".to_string(), |e| e.to_string()),
                    description: "Event id emitted after a new notifier is created.",
                },
                CliEntry {
                    key: "defaults.event.notifier-dropped-event",
                    value_type: "Option<int>",
                    default: config.defaults.event.notifier_dropped_event.map_or("None".to_string(), |e| e.to_string()),
                    description: "Event id emitted before a notifier is dropped.",
                },
                CliEntry {
                    key: "defaults.event.notifier-dead-event",
                    value_type: "Option<int>",
                    default: config.defaults.event.notifier_dead_event.map_or("None".to_string(), |e| e.to_string()),
                    description: "Event id emitted if a notifier is identified as dead.",
                },
            ],
        },
        CliSection {
            name: "Defaults: Request Response Messaging Pattern",
            entries: vec![
                CliEntry {
                    key: "defaults.request-response.enable-safe-overflow-for-requests",
                    value_type: "`true`|`false`",
                    default: config.defaults.request_response.enable_safe_overflow_for_requests.to_string(),
                    description: "Defines if the request buffer of the service safely overflows.",
                },
                CliEntry {
                    key: "defaults.request-response.enable-safe-overflow-for-responses",
                    value_type: "`true`|`false`",
                    default: config.defaults.request_response.enable_safe_overflow_for_responses.to_string(),
                    description: "Defines if the request buffer of the service safely overflows.",
                },
                CliEntry {
                    key: "defaults.request-response.max-active-requests-per-client",
                    value_type: "int",
                    default: config.defaults.request_response.max_active_requests_per_client.to_string(),
                    description: "The maximum of active requests a server can hold per client.",
                },
                CliEntry {
                    key: "defaults.request-response.max-response-buffer-size",
                    value_type: "int",
                    default: config.defaults.request_response.max_response_buffer_size.to_string(),
                    description: "The maximum buffer size for responses for an active request.",
                },
                CliEntry {
                    key: "defaults.request-response.max-servers",
                    value_type: "int",
                    default: config.defaults.request_response.max_servers.to_string(),
                    description: "The maximum amount of supported servers.",
                },
                CliEntry {
                    key: "defaults.request-response.max-clients",
                    value_type: "int",
                    default: config.defaults.request_response.max_clients.to_string(),
                    description: "The maximum amount of supported clients.",
                },
                CliEntry {
                    key: "defaults.request-response.max-nodes",
                    value_type: "int",
                    default: config.defaults.request_response.max_nodes.to_string(),
                    description: "The maximum amount of supported nodes. Defines indirectly how many processes can open the service at the same time.",
                },
                CliEntry {
                    key: "defaults.request-response.max-borrowed-responses-per-pending-response",
                    value_type: "int",
                    default: config.defaults.request_response.max_borrowed_responses_per_pending_response.to_string(),
                    description: "The maximum number of borrowed responses a client can hold in parallel per pending response.",
                },
                CliEntry {
                    key: "defaults.request-response.max-loaned-requests",
                    value_type: "int",
                    default: config.defaults.request_response.max_loaned_requests.to_string(),
                    description: "Maximum number of requests a client can loan in parallel.",
                },
                CliEntry {
                    key: "defaults.request-response.server-max-loaned-responses-per-request",
                    value_type: "int",
                    default: config.defaults.request_response.server_max_loaned_responses_per_request.to_string(),
                    description: "Maximum number of responses a server can loan per request.",
                },
                CliEntry {
                    key: "defaults.request-response.client-unable-to-deliver-strategy",
                    value_type: "`Block`|`DiscardSample`",
                    default: format!("{:?}", config.defaults.request_response.client_unable_to_deliver_strategy),
                    description: "Default strategy for non-overflowing setups when delivery fails.",
                },
                CliEntry {
                    key: "defaults.request-response.server-unable-to-deliver-strategy",
                    value_type: "`Block`|`DiscardSample`",
                    default: format!("{:?}", config.defaults.request_response.server_unable_to_deliver_strategy),
                    description: "Default strategy for non-overflowing setups when delivery fails.",
                },
                CliEntry {
                    key: "defaults.request-response.client-expired-connection-buffer",
                    value_type: "int",
                    default: config.defaults.request_response.client_expired_connection_buffer.to_string(),
                    description: "Expired connection buffer size of the client. Connections to servers are expired when the server disconnected from the service and the connection contains unconsumed responses.",
                },
                CliEntry {
                    key: "defaults.request-response.enable-fire-and-forget-requests",
                    value_type: "`true`|`false`",
                    default: config.defaults.request_response.enable_fire_and_forget_requests.to_string(),
                    description: "Enables the client to send requests without expecting a response.",
                },
                CliEntry {
                    key: "defaults.request-response.server-expired-connection-buffer",
                    value_type: "int",
                    default: config.defaults.request_response.server_expired_connection_buffer.to_string(),
                    description: "Expired connection buffer size of the server. Connections to clients are expired when the client disconnected from the service and the connection contains unconsumed active requests.",
                },
            ],
        },
        CliSection {
            name: "Defaults: Blackboard",
            entries: vec![
                CliEntry {
                    key: "defaults.blackboard.max-readers",
                    value_type: "int",
                    default: config.defaults.blackboard.max_readers.to_string(),
                    description: "The maximum amount of supported Readers.",
                },
                CliEntry {
                    key: "defaults.blackboard.max-nodes",
                    value_type: "int",
                    default: config.defaults.blackboard.max_nodes.to_string(),
                    description: "The maximum amount of supported Nodes. Defines indirectly how many processes can open the service at the same time.",
                },
            ],
        },
    ]
}
