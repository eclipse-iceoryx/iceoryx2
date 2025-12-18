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

use anyhow::Result;
use colored::Colorize;

use iceoryx2::config::Config;

/// Represents a configuration field with its metadata.
///
/// A `Field` contains all the necessary information to describe a single
/// configuration option, including its key path, expected value type,
/// default value, and a human-readable description.
pub struct Field {
    /// The hierarchical key path for this configuration field (e.g., "global.root-path")
    pub key: &'static str,
    /// The expected data type for this field's value (e.g., "string", "int", "`true`|`false`")
    pub value_type: &'static str,
    /// The default value of this field as a formatted string
    pub default_value: String,
    /// A human-readable description explaining the purpose and usage of this field
    pub description: &'static str,
}

/// Represents a logical grouping of related configuration fields.
///
/// A `Section` organizes configuration fields into categories for better
/// readability and understanding of the configuration schema. Each section
/// contains a descriptive name and a collection of related fields.
pub struct Section {
    /// The display name of this configuration section
    pub name: &'static str,
    /// A collection of configuration fields that belong to this section
    pub fields: Vec<Field>,
}

/// Describes the configuration schema by extracting field defaults from
/// the provided config.
///
/// This function creates a comprehensive description of all configuration
/// sections and their fields. The schema is organized
/// into logical sections for better readability.
///
/// # Arguments
///
/// * `config` - A reference to the Config instance from which to extract
///   default values
///
/// # Returns
///
/// A vector of `Section` structs, each containing a collection of `Field`
/// structs that describe the configuration options.
pub(crate) fn describe_schema(config: &Config) -> Vec<Section> {
    vec![
        Section {
            name: "Global",
            fields: vec![
                Field {
                    key: "global.root-path",
                    value_type: "string",
                    default_value: format!("\"{}\"", config.global.root_path()),
                    description: "Defines the path for all iceoryx2 files and directories.",
                },
                Field {
                    key: "global.prefix",
                    value_type: "string",
                    default_value: format!("\"{}\"", config.global.prefix),
                    description: "Prefix that is used for every file iceoryx2 creates.",
                },
            ],
        },
        Section {
            name: "Global: Node",
            fields: vec![
                Field {
                    key: "global.node.directory",
                    value_type: "string",
                    default_value: format!("\"{}\"", config.global.node.directory),
                    description: "Specifies the path for node-related files under `global.root-path`.",
                },
                Field {
                    key: "global.node.monitor-suffix",
                    value_type: "string",
                    default_value: format!("\"{}\"", config.global.node.monitor_suffix),
                    description: "Suffix added to the node monitor.",
                },
                Field {
                    key: "global.node.static-config-suffix",
                    value_type: "string",
                    default_value: format!("\"{}\"", config.global.node.static_config_suffix),
                    description: "Suffix added to the static config of the node.",
                },
                Field {
                    key: "global.node.service-tag-suffix",
                    value_type: "string",
                    default_value: format!("\"{}\"", config.global.node.service_tag_suffix),
                    description: "Suffix added to the service tag of the node.",
                },
                Field {
                    key: "global.node.cleanup-dead-nodes-on-creation",
                    value_type: "`true`|`false`",
                    default_value: config.global.node.cleanup_dead_nodes_on_creation.to_string(),
                    description: "Defines if there shall be a scan for dead nodes with a following stale resource cleanup whenever a new node is created.",
                },
                Field {
                    key: "global.node.cleanup-dead-nodes-on-destruction",
                    value_type: "`true`|`false`",
                    default_value: config.global.node.cleanup_dead_nodes_on_destruction.to_string(),
                    description: "Defines if there shall be a scan for dead nodes with a following stale resource cleanup whenever a node is going out-of-scope.",
                },
            ],
        },
        Section {
            name: "Global: Service",
            fields: vec![
                Field {
                    key: "global.service.directory",
                    value_type: "string",
                    default_value: format!("\"{}\"", config.global.service.directory),
                    description: "Specifies the path for service-related files under `global.root-path`.",
                },
                Field {
                    key: "global.service.data-segment-suffix",
                    value_type: "string",
                    default_value: format!("\"{}\"", config.global.service.data_segment_suffix),
                    description: "Suffix added to the ports's data segment.",
                },
                Field {
                    key: "global.service.static-config-storage-suffix",
                    value_type: "string",
                    default_value: format!("\"{}\"", config.global.service.static_config_storage_suffix),
                    description: "Suffix for static service configuration files.",
                },
                Field {
                    key: "global.service.dynamic-config-storage-suffix",
                    value_type: "string",
                    default_value: format!("\"{}\"", config.global.service.dynamic_config_storage_suffix),
                    description: "Suffix for dynamic service configuration files.",
                },
                Field {
                    key: "global.service.event-connection-suffix",
                    value_type: "string",
                    default_value: format!("\"{}\"", config.global.service.event_connection_suffix),
                    description: "Suffix for event channel.",
                },
                Field {
                    key: "global.service.connection-suffix",
                    value_type: "string",
                    default_value: format!("\"{}\"", config.global.service.connection_suffix),
                    description: "Suffix for one-to-one connections.",
                },
                Field {
                    key: "global.service.blackboard-mgmt-suffix",
                    value_type: "string",
                    default_value: format!("\"{}\"", config.global.service.blackboard_mgmt_suffix),
                    description: "The suffix of the blackboard management data segment.",
                },
                Field {
                    key: "global.service.blackboard-data-suffix",
                    value_type: "string",
                    default_value: format!("\"{}\"", config.global.service.blackboard_data_suffix),
                    description: "The suffix of the blackboard payload data segment.",
                },
            ],
        },
        Section{
            name: "Global: Service Creation Timeout",
            fields: vec![
                Field {
                    key: "global.service.creation-timeout.secs",
                    value_type: "int",
                    default_value: config.global.service.creation_timeout.as_secs().to_string(),
                    description: "Maximum time for service setup in seconds. Uncreated services after this are marked as stalled.\n   \
                    Attention: Both 'secs' and 'nanos' must be set together; leaving one unset will cause the configuration to be invalid.",
                },
                Field {
                    key: "global.service.creation-timeout.nanos",
                    value_type: "int",
                    default_value: config.global.service.creation_timeout.subsec_nanos().to_string(),
                    description: "Additional nanoseconds for service setup timeout.\n   \
                    Attention: Both 'secs' and 'nanos' must be set together; leaving one unset will cause the configuration to be invalid.",
                },
            ],
        },
        Section {
            name: "Defaults: Publish Subscribe Messaging Pattern",
            fields: vec![
                Field {
                    key: "defaults.publish-subscribe.max-subscribers",
                    value_type: "int",
                    default_value: config.defaults.publish_subscribe.max_subscribers.to_string(),
                    description: "Maximum number of subscribers.",
                },
                Field {
                    key: "defaults.publish-subscribe.max-publishers",
                    value_type: "int",
                    default_value: config.defaults.publish_subscribe.max_publishers.to_string(),
                    description: "Maximum number of publishers.",
                },
                Field {
                    key: "defaults.publish-subscribe.max-nodes",
                    value_type: "int",
                    default_value: config.defaults.publish_subscribe.max_nodes.to_string(),
                    description: "Maximum number of nodes.",
                },
                Field {
                    key: "defaults.publish-subscribe.subscriber-max-buffer-size",
                    value_type: "int",
                    default_value: config.defaults.publish_subscribe.subscriber_max_buffer_size.to_string(),
                    description: "Maximum buffer size of a subscriber.",
                },
                Field {
                    key: "defaults.publish-subscribe.subscriber-max-borrowed-samples",
                    value_type: "int",
                    default_value: config.defaults.publish_subscribe.subscriber_max_borrowed_samples.to_string(),
                    description: "Maximum samples a subscriber can hold.",
                },
                Field {
                    key: "defaults.publish-subscribe.publisher-max-loaned-samples",
                    value_type: "int",
                    default_value: config.defaults.publish_subscribe.publisher_max_loaned_samples.to_string(),
                    description: "Maximum samples a publisher can loan.",
                },
                Field {
                    key: "defaults.publish-subscribe.publisher-history-size",
                    value_type: "int",
                    default_value: config.defaults.publish_subscribe.publisher_history_size.to_string(),
                    description: "Maximum history size a subscriber can request.",
                },
                Field {
                    key: "defaults.publish-subscribe.enable-safe-overflow",
                    value_type: "`true`|`false`",
                    default_value: config.defaults.publish_subscribe.enable_safe_overflow.to_string(),
                    description: "Default overflow behavior.",
                },
                Field {
                    key: "defaults.publish-subscribe.unable-to-deliver-strategy",
                    value_type: "`Block`|`DiscardSample`",
                    default_value: format!("{:?}", config.defaults.publish_subscribe.unable_to_deliver_strategy),
                    description: "Default strategy for non-overflowing setups when delivery fails.",
                },
                Field {
                    key: "defaults.publish-subscribe.subscriber-expired-connection-buffer",
                    value_type: "int",
                    default_value: config.defaults.publish_subscribe.subscriber_expired_connection_buffer.to_string(),
                    description: "Expired connection buffer size of the subscriber. Connections to publishers are expired when the publisher disconnected from the service and the connection contains unconsumed samples.",
                },
            ],
        },
        Section {
            name: "Defaults: Event Messaging Pattern",
            fields: vec![
                Field {
                    key: "defaults.event.max-listeners",
                    value_type: "int",
                    default_value: config.defaults.event.max_listeners.to_string(),
                    description: "Maximum number of listeners.",
                },
                Field {
                    key: "defaults.event.max-notifiers",
                    value_type: "int",
                    default_value: config.defaults.event.max_notifiers.to_string(),
                    description: "Maximum number of notifiers.",
                },
                Field {
                    key: "defaults.event.max-nodes",
                    value_type: "int",
                    default_value: config.defaults.event.max_nodes.to_string(),
                    description: "Maximum number of nodes.",
                },
                Field {
                    key: "defaults.event.event-id-max-value",
                    value_type: "int",
                    default_value: config.defaults.event.event_id_max_value.to_string(),
                    description: "Greatest value an [`EventId`] can have.",
                },
                Field {
                    key: "defaults.event.deadline",
                    value_type: "Option<Duration>",
                    default_value: config.defaults.event.deadline.map_or("None".to_string(), |e| format!("{e:?}")),
                    description: "\
                    Maximum allowed time between two consecutive notifications. If not sent after \
                    this time, all listeners attached to a WaitSet will be notified.\n   \
                    Due to a current limitation, the keys are actually \
                    `defaults.event.deadline.secs` and `defaults.event.deadline.nanos`.\n   \
                    Attention: Both 'secs' and 'nanos' must be set together; leaving one unset \
                    will cause the configuration to be invalid.",
                },
                Field {
                    key: "defaults.event.notifier-created-event",
                    value_type: "Option<int>",
                    default_value: config.defaults.event.notifier_created_event.map_or("None".to_string(), |e| e.to_string()),
                    description: "Event id emitted after a new notifier is created.",
                },
                Field {
                    key: "defaults.event.notifier-dropped-event",
                    value_type: "Option<int>",
                    default_value: config.defaults.event.notifier_dropped_event.map_or("None".to_string(), |e| e.to_string()),
                    description: "Event id emitted before a notifier is dropped.",
                },
                Field {
                    key: "defaults.event.notifier-dead-event",
                    value_type: "Option<int>",
                    default_value: config.defaults.event.notifier_dead_event.map_or("None".to_string(), |e| e.to_string()),
                    description: "Event id emitted if a notifier is identified as dead.",
                },
            ],
        },
        Section {
            name: "Defaults: Request Response Messaging Pattern",
            fields: vec![
                Field {
                    key: "defaults.request-response.enable-safe-overflow-for-requests",
                    value_type: "`true`|`false`",
                    default_value: config.defaults.request_response.enable_safe_overflow_for_requests.to_string(),
                    description: "Defines if the request buffer of the service safely overflows.",
                },
                Field {
                    key: "defaults.request-response.enable-safe-overflow-for-responses",
                    value_type: "`true`|`false`",
                    default_value: config.defaults.request_response.enable_safe_overflow_for_responses.to_string(),
                    description: "Defines if the request buffer of the service safely overflows.",
                },
                Field {
                    key: "defaults.request-response.max-active-requests-per-client",
                    value_type: "int",
                    default_value: config.defaults.request_response.max_active_requests_per_client.to_string(),
                    description: "The maximum of active requests a server can hold per client.",
                },
                Field {
                    key: "defaults.request-response.max-response-buffer-size",
                    value_type: "int",
                    default_value: config.defaults.request_response.max_response_buffer_size.to_string(),
                    description: "The maximum buffer size for responses for an active request.",
                },
                Field {
                    key: "defaults.request-response.max-servers",
                    value_type: "int",
                    default_value: config.defaults.request_response.max_servers.to_string(),
                    description: "The maximum amount of supported servers.",
                },
                Field {
                    key: "defaults.request-response.max-clients",
                    value_type: "int",
                    default_value: config.defaults.request_response.max_clients.to_string(),
                    description: "The maximum amount of supported clients.",
                },
                Field {
                    key: "defaults.request-response.max-nodes",
                    value_type: "int",
                    default_value: config.defaults.request_response.max_nodes.to_string(),
                    description: "The maximum amount of supported nodes. Defines indirectly how many processes can open the service at the same time.",
                },
                Field {
                    key: "defaults.request-response.max-borrowed-responses-per-pending-response",
                    value_type: "int",
                    default_value: config.defaults.request_response.max_borrowed_responses_per_pending_response.to_string(),
                    description: "The maximum number of borrowed responses a client can hold in parallel per pending response.",
                },
                Field {
                    key: "defaults.request-response.max-loaned-requests",
                    value_type: "int",
                    default_value: config.defaults.request_response.max_loaned_requests.to_string(),
                    description: "Maximum number of requests a client can loan in parallel.",
                },
                Field {
                    key: "defaults.request-response.server-max-loaned-responses-per-request",
                    value_type: "int",
                    default_value: config.defaults.request_response.server_max_loaned_responses_per_request.to_string(),
                    description: "Maximum number of responses a server can loan per request.",
                },
                Field {
                    key: "defaults.request-response.client-unable-to-deliver-strategy",
                    value_type: "`Block`|`DiscardSample`",
                    default_value: format!("{:?}", config.defaults.request_response.client_unable_to_deliver_strategy),
                    description: "Default strategy for non-overflowing setups when delivery fails.",
                },
                Field {
                    key: "defaults.request-response.server-unable-to-deliver-strategy",
                    value_type: "`Block`|`DiscardSample`",
                    default_value: format!("{:?}", config.defaults.request_response.server_unable_to_deliver_strategy),
                    description: "Default strategy for non-overflowing setups when delivery fails.",
                },
                Field {
                    key: "defaults.request-response.client-expired-connection-buffer",
                    value_type: "int",
                    default_value: config.defaults.request_response.client_expired_connection_buffer.to_string(),
                    description: "Expired connection buffer size of the client. Connections to servers are expired when the server disconnected from the service and the connection contains unconsumed responses.",
                },
                Field {
                    key: "defaults.request-response.enable-fire-and-forget-requests",
                    value_type: "`true`|`false`",
                    default_value: config.defaults.request_response.enable_fire_and_forget_requests.to_string(),
                    description: "Enables the client to send requests without expecting a response.",
                },
                Field {
                    key: "defaults.request-response.server-expired-connection-buffer",
                    value_type: "int",
                    default_value: config.defaults.request_response.server_expired_connection_buffer.to_string(),
                    description: "Expired connection buffer size of the server. Connections to clients are expired when the client disconnected from the service and the connection contains unconsumed active requests.",
                },
            ],
        },
        Section {
            name: "Defaults: Blackboard Messaging Pattern",
            fields: vec![
                Field {
                    key: "defaults.blackboard.max-readers",
                    value_type: "int",
                    default_value: config.defaults.blackboard.max_readers.to_string(),
                    description: "The maximum amount of supported Readers.",
                },
                Field {
                    key: "defaults.blackboard.max-nodes",
                    value_type: "int",
                    default_value: config.defaults.blackboard.max_nodes.to_string(),
                    description: "The maximum amount of supported Nodes. Defines indirectly how many processes can open the service at the same time.",
                },
            ],
        },
    ]
}

pub fn explain() -> Result<()> {
    let schema = describe_schema(&Config::default());
    for section in schema {
        println!("\n{}", format!("== {} ==", section.name).bright_green());

        for entry in section.fields {
            println!(
                "-> {} [{}]",
                entry.key.bright_blue(),
                entry.value_type.bright_red()
            );
            println!(
                "   {}",
                format!("(Default value: {})", entry.default_value.bright_white()).bright_yellow()
            );
            println!("   {}", entry.description.italic());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use ron::de::from_str;
    use ron::ser::to_string;
    use ron::Value;

    use iceoryx2::config::Config;
    use iceoryx2_bb_testing::assert_that;

    use super::describe_schema;

    // Recursively walk through ron::Value to flatten all keys into a HashSet
    fn collect_keys_ron(value: &Value, prefix: String, keys: &mut HashSet<String>) {
        match value {
            Value::Map(map) => {
                for (k, v) in map.iter() {
                    let k_str = match k {
                        Value::String(s) => s.clone(),
                        _ => continue, // skip non-string keys
                    };

                    let new_prefix = if prefix.is_empty() {
                        k_str
                    } else {
                        format!("{}.{}", prefix, k_str)
                    };

                    collect_keys_ron(v, new_prefix, keys);
                }
            }
            _ => {
                keys.insert(prefix);
            }
        }
    }

    #[test]
    fn check_description_is_present_for_all_fields() {
        let config = Config::default();

        let ron_string = to_string(&config).expect("Failed to serialize config to RON");
        let parsed: Value = from_str(&ron_string).expect("Invalid RON");

        let mut ron_keys = HashSet::new();
        collect_keys_ron(&parsed, "".to_string(), &mut ron_keys);

        let field_keys: HashSet<String> = describe_schema(&config)
            .iter()
            .flat_map(|section| section.fields.iter())
            .map(|field| field.key.to_string())
            .collect();

        let missing_in_config = field_keys.difference(&ron_keys).collect::<Vec<_>>();
        let extra_in_config = ron_keys.difference(&field_keys).collect::<Vec<_>>();
        println!("Missing in config: {:?}", missing_in_config);
        println!("Extra in config: {:?}", extra_in_config);
        assert_that!(missing_in_config.len(), eq 0);
        assert_that!(extra_in_config.len(), eq 0);
    }
}
