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

pub struct CliEntry {
    pub key: &'static str,
    pub value_type: &'static str,
    pub description: &'static str,
}

pub struct CliSection {
    pub name: &'static str,
    pub entries: Vec<CliEntry>,
}

pub fn get_sections() -> Vec<CliSection> {
    vec![
        CliSection {
            name: "Global",
            entries: vec![
                CliEntry {
                    key: "global.root-path-unix",
                    value_type: "[string]",
                    description: "Path used as root for IPC-related files on Unix-based systems.",
                },
                CliEntry {
                    key: "global.root-path-windows",
                    value_type: "[string]",
                    description: "Path used as root for IPC-related files on Windows systems.",
                },
                CliEntry {
                    key: "global.prefix",
                    value_type: "[string]",
                    description: "Prefix that is used for every file iceoryx2 creates.",
                },
            ],
        },
        CliSection {
            name: "Global: Service",
            entries: vec![
                CliEntry {
                    key: "global.service.directory",
                    value_type: "[string]",
                    description: "Specifies the path for service-related files under `global.root-path`.",
                },
                CliEntry {
                    key: "global.service.data-segment-suffix",
                    value_type: "[string]",
                    description: "Suffix added to the ports's data segment.",
                },
                CliEntry {
                    key: "global.service.static-config-storage-suffix",
                    value_type: "[string]",
                    description: "Suffix for static service configuration files.",
                },
                CliEntry {
                    key: "global.service.dynamic-config-storage-suffix",
                    value_type: "[string]",
                    description: "Suffix for dynamic service configuration files.",
                },
                CliEntry {
                    key: "global.service.event-connection-suffix",
                    value_type: "[string]",
                    description: "Suffix for event channel.",
                },
                CliEntry {
                    key: "global.service.connection-suffix",
                    value_type: "[string]",
                    description: "Suffix for one-to-one connections.",
                },
            ],
        },
                CliSection {
            name: "Global: Node",
            entries: vec![
                CliEntry {
                    key: "global.node.directory",
                    value_type: "[string]",
                    description: "Specifies the path for node-related files under `global.root-path`.",
                },
                CliEntry {
                    key: "global.node.monitor-suffix",
                    value_type: "[string]",
                    description: "Suffix added to the node monitor.",
                },
                CliEntry {
                    key: "global.node.static-config-suffix",
                    value_type: "[string]",
                    description: "Suffix added to the static config of the node.",
                },
                CliEntry {
                    key: "global.node.service-tag-suffix",
                    value_type: "[string]",
                    description: "Suffix added to the service tag of the node.",
                },
                CliEntry {
                    key: "global.node.cleanup-dead-nodes-on-creation",
                    value_type: "[`true`|`false`]",
                    description: "Defines if there shall be a scan for dead nodes with a following stale resource cleanup whenever a new node is created.",
                },
                CliEntry {
                    key: "global.node.cleanup-dead-nodes-on-destruction",
                    value_type: "[`true`|`false`]",
                    description: "Defines if there shall be a scan for dead nodes with a following stale resource cleanup whenever a node is going out-of-scope.",
                },
            ],
        },
        CliSection{
            name: "Global: Service Creation Timeout",
            entries: vec![
                CliEntry {
                    key: "global.service.creation-timeout.secs",
                    value_type: "[int]",
                    description: "Maximum time for service setup in seconds. Uncreated services after this are marked as stalled.",
                },
                CliEntry {
                    key: "global.service.creation-timeout.nanos",
                    value_type: "[int]",
                    description: "Additional nanoseconds for service setup timeout.",
                },
            ],
        },
        CliSection {
            name: "Defaults: Publish Subscribe Messaging Pattern",
            entries: vec![
                CliEntry {
                    key: "defaults.publish-subscribe.max-subscribers",
                    value_type: "[int]",
                    description: "Maximum number of subscribers.",
                },
                CliEntry {
                    key: "defaults.publish-subscribe.max-publishers",
                    value_type: "[int]",
                    description: "Maximum number of publishers.",
                },
                CliEntry {
                    key: "defaults.publish-subscribe.max-nodes",
                    value_type: "[int]",
                    description: "Maximum number of nodes.",
                },
                CliEntry {
                    key: "defaults.publish-subscribe.subscriber-max-buffer-size",
                    value_type: "[int]",
                    description: "Maximum buffer size of a subscriber.",
                },
                CliEntry {
                    key: "defaults.publish-subscribe.subscriber-max-borrowed-samples",
                    value_type: "[int]",
                    description: "Maximum samples a subscriber can hold.",
                },
                CliEntry {
                    key: "defaults.publish-subscribe.publisher-max-loaned-samples",
                    value_type: "[int]",
                    description: "Maximum samples a publisher can loan.",
                },
                CliEntry {
                    key: "defaults.publish-subscribe.publisher-history-size",
                    value_type: "[int]",
                    description: "Maximum history size a subscriber can request.",
                },
                CliEntry {
                    key: "defaults.publish-subscribe.enable-safe-overflow",
                    value_type: "[`true`|`false`]",
                    description: "Default overflow behavior.",
                },
                CliEntry {
                    key: "defaults.publish-subscribe.unable-to-deliver-strategy",
                    value_type: "[`Block`|`DiscardSample`]",
                    description: "Default strategy for non-overflowing setups when delivery fails.",
                },
                CliEntry {
                    key: "defaults.publish-subscribe.subscriber-expired-connection-buffer",
                    value_type: "[int]",
                    description: "Expired connection buffer size of the subscriber. Connections to publishers are expired when the publisher disconnected from the service and the connection contains unconsumed samples.",
                },
            ],
        },
        CliSection {
            name: "Defaults: Event Messaging Pattern",
            entries: vec![
                CliEntry {
                    key: "defaults.event.max-listeners",
                    value_type: "[int]",
                    description: "Maximum number of listeners.",
                },
                CliEntry {
                    key: "defaults.event.max-notifiers",
                    value_type: "[int]",
                    description: "Maximum number of notifiers.",
                },
                CliEntry {
                    key: "defaults.event.max-nodes",
                    value_type: "[int]",
                    description: "Maximum number of nodes.",
                },
                CliEntry {
                    key: "defaults.event.event-id-max-value",
                    value_type: "[int]",
                    description: "Greatest value an [`EventId`] can have.",
                },
                CliEntry {
                    key: "defaults.event.deadline",
                    value_type: "[int]",
                    description: "Maximum allowed time between two consecutive notifications. If not sent after this time, all listeners attached to a WaitSet will be notified.",
                },
                CliEntry {
                    key: "defaults.event.notifier-created-event",
                    value_type: "[Option<int>]",
                    description: "Event id emitted after a new notifier is created.",
                },
                CliEntry {
                    key: "defaults.event.notifier-dropped-event",
                    value_type: "[Option<int>]",
                    description: "Event id emitted before a notifier is dropped.",
                },
                CliEntry {
                    key: "defaults.event.notifier-dead-event",
                    value_type: "[Option<int>]",
                    description: "Event id emitted if a notifier is identified as dead.",
                },
            ],
        },
        CliSection {
            name: "Defaults: Request Response Messaging Pattern",
            entries: vec![
                CliEntry {
                    key: "defaults.request-response.enable-safe-overflow-for-requests",
                    value_type: "[`true`|`false`]",
                    description: "Defines if the request buffer of the service safely overflows.",
                },
                CliEntry {
                    key: "defaults.request-response.enable-safe-overflow-for-responses",
                    value_type: "[`true`|`false`]",
                    description: "Defines if the request buffer of the service safely overflows.",
                },
                CliEntry {
                    key: "defaults.request-response.max-active-requests-per-client",
                    value_type: "[int]",
                    description: "The maximum of active requests a server can hold per client.",
                },
                CliEntry {
                    key: "defaults.request-response.max-response-buffer-size",
                    value_type: "[int]",
                    description: "The maximum buffer size for responses for an active request.",
                },
                CliEntry {
                    key: "defaults.request-response.max-servers",
                    value_type: "[int]",
                    description: "The maximum amount of supported servers.",
                },
                CliEntry {
                    key: "defaults.request-response.max-clients",
                    value_type: "[int]",
                    description: "The maximum amount of supported clients.",
                },
                CliEntry {
                    key: "defaults.request-response.max-nodes",
                    value_type: "[int]",
                    description: "The maximum amount of supported nodes. Defines indirectly how many processes can open the service at the same time.",
                },
                CliEntry {
                    key: "defaults.request-response.max-borrowed-responses-per-pending-response",
                    value_type: "[int]",
                    description: "The maximum number of borrowed responses a client can hold in parallel per pending response.",
                },
                CliEntry {
                    key: "defaults.request-response.max-loaned-requests",
                    value_type: "[int]",
                    description: "Maximum number of requests a client can loan in parallel.",
                },
                CliEntry {
                    key: "defaults.request-response.server-max-loaned-responses-per-request",
                    value_type: "[int]",
                    description: "Maximum number of responses a server can loan per request.",
                },
                CliEntry {
                    key: "defaults.request-response.client-unable-to-deliver-strategy",
                    value_type: "[`Block`|`DiscardSample`]",
                    description: "Default strategy for non-overflowing setups when delivery fails.",
                },
                CliEntry {
                    key: "defaults.request-response.server-unable-to-deliver-strategy",
                    value_type: "[`Block`|`DiscardSample`]",
                    description: "Default strategy for non-overflowing setups when delivery fails.",
                },
                CliEntry {
                    key: "defaults.request-response.client-expired-connection-buffer",
                    value_type: "[int]",
                    description: "Expired connection buffer size of the client. Connections to servers are expired when the server disconnected from the service and the connection contains unconsumed responses.",
                },
                CliEntry {
                    key: "defaults.request-response.enable-fire-and-forget-requests",
                    value_type: "[`true`|`false`]",
                    description: "Enables the client to send requests without expecting a response.",
                },
                CliEntry {
                    key: "defaults.request-response.server-expired-connection-buffer",
                    value_type: "[int]",
                    description: "Expired connection buffer size of the server. Connections to clients are expired when the client disconnected from the service and the connection contains unconsumed active requests.",
                },
            ],
        },
        CliSection {
            name: "Defaults: Blackboard",
            entries: vec![
                CliEntry {
                    key: "defaults.blackboard.max-readers",
                    value_type: "[int]",
                    description: "The maximum amount of supported Readers.",
                },
                CliEntry {
                    key: "defaults.blackboard.max-nodes",
                    value_type: "[int]",
                    description: "The maximum amount of supported Nodes. Defines indirectly how many processes can open the service at the same time.",
                },
            ],
        },
    ]
}
