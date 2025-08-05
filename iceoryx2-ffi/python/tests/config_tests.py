# Copyright (c) 2025 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache Software License 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
# which is available at https://opensource.org/licenses/MIT.
#
# SPDX-License-Identifier: Apache-2.0 OR MIT

import iceoryx2 as iox2
import pytest


def test_global_root_path_can_be_set() -> None:
    sut = iox2.config.default()
    path = iox2.Path.new("/some/path")
    sut.global_cfg.root_path = path
    assert sut.global_cfg.root_path == path


def test_global_prefix_can_be_set() -> None:
    sut = iox2.config.default()
    path = iox2.FileName.new("prefix_")
    sut.global_cfg.prefix = path
    assert sut.global_cfg.prefix == path


def test_global_service_directory_can_be_set() -> None:
    sut = iox2.config.default()
    path = iox2.Path.new("/path/to/funky")
    sut.global_cfg.service.directory = path
    assert sut.global_cfg.service.directory == path


def test_global_service_data_segment_suffix_can_be_set() -> None:
    sut = iox2.config.default()
    path = iox2.FileName.new(".let_suf_the_fix")
    sut.global_cfg.service.data_segment_suffix = path
    assert sut.global_cfg.service.data_segment_suffix == path


def test_global_service_static_config_storage_suffix_can_be_set() -> None:
    sut = iox2.config.default()
    path = iox2.FileName.new(".its_a_fix_for_the_suf")
    sut.global_cfg.service.static_config_storage_suffix = path
    assert sut.global_cfg.service.static_config_storage_suffix == path


def test_global_service_dynamic_config_storage_suffix_can_be_set() -> None:
    sut = iox2.config.default()
    path = iox2.FileName.new(".flufn_fluff")
    sut.global_cfg.service.dynamic_config_storage_suffix = path
    assert sut.global_cfg.service.dynamic_config_storage_suffix == path


def test_global_service_creation_timeout_can_be_set() -> None:
    sut = iox2.config.default()
    path = iox2.Duration.from_millis(123)
    sut.global_cfg.service.creation_timeout = path
    assert sut.global_cfg.service.creation_timeout == path


def test_global_service_connection_suffix_can_be_set() -> None:
    sut = iox2.config.default()
    path = iox2.FileName.new(".fuldaba")
    sut.global_cfg.service.connection_suffix = path
    assert sut.global_cfg.service.connection_suffix == path


def test_global_service_event_connection_suffix_can_be_set() -> None:
    sut = iox2.config.default()
    path = iox2.FileName.new(".blarb")
    sut.global_cfg.service.event_connection_suffix = path
    assert sut.global_cfg.service.event_connection_suffix == path


def test_global_node_directory_can_be_set() -> None:
    sut = iox2.config.default()
    value = iox2.Path.new("/dir/to/somewher")
    sut.global_cfg.node.directory = value
    assert sut.global_cfg.node.directory == value


def test_global_node_monitor_suffix_can_be_set() -> None:
    sut = iox2.config.default()
    value = iox2.FileName.new(".suf_le_fix_le")
    sut.global_cfg.node.monitor_suffix = value
    assert sut.global_cfg.node.monitor_suffix == value


def test_global_node_static_config_suffix_can_be_set() -> None:
    sut = iox2.config.default()
    value = iox2.FileName.new(".wuff")
    sut.global_cfg.node.static_config_suffix = value
    assert sut.global_cfg.node.static_config_suffix == value


def test_global_node_service_tag_suffix_can_be_set() -> None:
    sut = iox2.config.default()
    value = iox2.FileName.new(".grogg")
    sut.global_cfg.node.service_tag_suffix = value
    assert sut.global_cfg.node.service_tag_suffix == value


def test_global_node_cleanup_dead_nodes_on_destruction_can_be_set() -> None:
    sut = iox2.config.default()
    sut.global_cfg.node.cleanup_dead_nodes_on_destruction = True
    assert sut.global_cfg.node.cleanup_dead_nodes_on_destruction
    sut.global_cfg.node.cleanup_dead_nodes_on_destruction = False
    assert not sut.global_cfg.node.cleanup_dead_nodes_on_destruction


def test_global_node_cleanup_dead_nodes_on_creation_can_be_set() -> None:
    sut = iox2.config.default()
    sut.global_cfg.node.cleanup_dead_nodes_on_creation = True
    assert sut.global_cfg.node.cleanup_dead_nodes_on_creation
    sut.global_cfg.node.cleanup_dead_nodes_on_creation = False
    assert not sut.global_cfg.node.cleanup_dead_nodes_on_creation


def test_defaults_request_response_safe_overflow_for_requests_can_be_set() -> None:
    sut = iox2.config.default()
    sut.defaults.request_response.enable_safe_overflow_for_requests = False
    assert not sut.defaults.request_response.enable_safe_overflow_for_requests
    sut.defaults.request_response.enable_safe_overflow_for_requests = True
    assert sut.defaults.request_response.enable_safe_overflow_for_requests


def test_defaults_request_response_safe_overflow_for_responses_can_be_set() -> None:
    sut = iox2.config.default()
    sut.defaults.request_response.enable_safe_overflow_for_responses = False
    assert not sut.defaults.request_response.enable_safe_overflow_for_responses
    sut.defaults.request_response.enable_safe_overflow_for_responses = True
    assert sut.defaults.request_response.enable_safe_overflow_for_responses


def test_defaults_request_response_max_active_requests_per_client_can_be_set() -> None:
    sut = iox2.config.default()
    value = 912
    sut.defaults.request_response.max_active_requests_per_client = value
    assert sut.defaults.request_response.max_active_requests_per_client == value


def test_defaults_request_response_max_response_buffer_size_can_be_set() -> None:
    sut = iox2.config.default()
    value = 281
    sut.defaults.request_response.max_response_buffer_size = value
    assert sut.defaults.request_response.max_response_buffer_size == value


def test_defaults_request_response_max_servers_can_be_set() -> None:
    sut = iox2.config.default()
    value = 91
    sut.defaults.request_response.max_servers = value
    assert sut.defaults.request_response.max_servers == value


def test_defaults_request_response_max_clients_can_be_set() -> None:
    sut = iox2.config.default()
    value = 661
    sut.defaults.request_response.max_clients = value
    assert sut.defaults.request_response.max_clients == value


def test_defaults_request_response_max_nodes_can_be_set() -> None:
    sut = iox2.config.default()
    value = 45
    sut.defaults.request_response.max_nodes = value
    assert sut.defaults.request_response.max_nodes == value


def test_defaults_request_response_max_borrowed_responses_per_pending_response_can_be_set() -> (
    None
):
    sut = iox2.config.default()
    value = 23
    sut.defaults.request_response.max_borrowed_responses_per_pending_response = value
    assert (
        sut.defaults.request_response.max_borrowed_responses_per_pending_response
        == value
    )


def test_defaults_request_response_max_loaned_requests_can_be_set() -> None:
    sut = iox2.config.default()
    value = 41
    sut.defaults.request_response.max_loaned_requests = value
    assert sut.defaults.request_response.max_loaned_requests == value


def test_defaults_request_response_server_max_loaned_responses_per_request_can_be_set() -> (
    None
):
    sut = iox2.config.default()
    value = 56
    sut.defaults.request_response.server_max_loaned_responses_per_request = value
    assert (
        sut.defaults.request_response.server_max_loaned_responses_per_request == value
    )


def test_defaults_request_response_client_unable_to_deliver_strategy_can_be_set() -> (
    None
):
    sut = iox2.config.default()
    sut.defaults.request_response.client_unable_to_deliver_strategy = (
        iox2.UnableToDeliverStrategy.Block
    )
    assert (
        sut.defaults.request_response.client_unable_to_deliver_strategy
        == iox2.UnableToDeliverStrategy.Block
    )
    sut.defaults.request_response.client_unable_to_deliver_strategy = (
        iox2.UnableToDeliverStrategy.DiscardSample
    )
    assert (
        sut.defaults.request_response.client_unable_to_deliver_strategy
        == iox2.UnableToDeliverStrategy.DiscardSample
    )


def test_defaults_request_response_server_unable_to_deliver_strategy_can_be_set() -> (
    None
):
    sut = iox2.config.default()
    sut.defaults.request_response.server_unable_to_deliver_strategy = (
        iox2.UnableToDeliverStrategy.Block
    )
    assert (
        sut.defaults.request_response.server_unable_to_deliver_strategy
        == iox2.UnableToDeliverStrategy.Block
    )
    sut.defaults.request_response.server_unable_to_deliver_strategy = (
        iox2.UnableToDeliverStrategy.DiscardSample
    )
    assert (
        sut.defaults.request_response.server_unable_to_deliver_strategy
        == iox2.UnableToDeliverStrategy.DiscardSample
    )


def test_defaults_request_response_client_expired_connection_buffer_can_be_set() -> (
    None
):
    sut = iox2.config.default()
    value = 78
    sut.defaults.request_response.client_expired_connection_buffer = value
    assert sut.defaults.request_response.client_expired_connection_buffer == value


def test_defaults_request_response_server_expired_connection_buffer_can_be_set() -> (
    None
):
    sut = iox2.config.default()
    value = 213
    sut.defaults.request_response.server_expired_connection_buffer = value
    assert sut.defaults.request_response.server_expired_connection_buffer == value


def test_defaults_request_response_enable_fire_and_forget_requests_can_be_set() -> None:
    sut = iox2.config.default()
    sut.defaults.request_response.enable_fire_and_forget_requests = True
    assert sut.defaults.request_response.enable_fire_and_forget_requests
    sut.defaults.request_response.enable_fire_and_forget_requests = False
    assert not sut.defaults.request_response.enable_fire_and_forget_requests


def test_defaults_event_max_listeners_can_be_set() -> None:
    sut = iox2.config.default()
    value = 8891
    sut.defaults.event.max_listeners = value
    assert sut.defaults.event.max_listeners == value


def test_defaults_event_max_notifiers_can_be_set() -> None:
    sut = iox2.config.default()
    value = 3321
    sut.defaults.event.max_notifiers = value
    assert sut.defaults.event.max_notifiers == value


def test_defaults_event_max_nodes_can_be_set() -> None:
    sut = iox2.config.default()
    value = 1121
    sut.defaults.event.max_nodes = value
    assert sut.defaults.event.max_nodes == value


def test_defaults_event_event_id_max_value_can_be_set() -> None:
    sut = iox2.config.default()
    value = 18
    sut.defaults.event.event_id_max_value = value
    assert sut.defaults.event.event_id_max_value == value


def test_defaults_event_deadline_can_be_set_can_be_set() -> None:
    sut = iox2.config.default()
    value = iox2.Duration.from_secs(2)
    sut.defaults.event.deadline = value
    assert sut.defaults.event.deadline == value


def test_defaults_event_notifier_created_event_can_be_set() -> None:
    sut = iox2.config.default()
    value = 941
    sut.defaults.event.notifier_created_event = value
    assert sut.defaults.event.notifier_created_event == value
    assert sut.defaults.event.has_notifier_created_event
    sut.defaults.event.disable_notifier_created_event()
    assert not sut.defaults.event.has_notifier_created_event


def test_defaults_event_notifier_dropped_event_can_be_set() -> None:
    sut = iox2.config.default()
    value = 9411
    sut.defaults.event.notifier_dropped_event = value
    assert sut.defaults.event.notifier_dropped_event == value
    assert sut.defaults.event.has_notifier_dropped_event
    sut.defaults.event.disable_notifier_dropped_event()
    assert not sut.defaults.event.has_notifier_dropped_event


def test_defaults_event_notifier_dead_event_can_be_set() -> None:
    sut = iox2.config.default()
    value = 9411
    sut.defaults.event.notifier_dead_event = value
    assert sut.defaults.event.notifier_dead_event == value
    assert sut.defaults.event.has_notifier_dead_event
    sut.defaults.event.disable_notifier_dead_event()
    assert not sut.defaults.event.has_notifier_dead_event


def test_defaults_publish_subscribe_max_subscribers_can_be_set() -> None:
    sut = iox2.config.default()
    value = 1818
    sut.defaults.publish_subscribe.max_subscribers = value
    assert sut.defaults.publish_subscribe.max_subscribers == value


def test_defaults_publish_subscribe_max_publishers_can_be_set() -> None:
    sut = iox2.config.default()
    value = 8181
    sut.defaults.publish_subscribe.max_publishers = value
    assert sut.defaults.publish_subscribe.max_publishers == value


def test_defaults_publish_subscribe_max_nodes_can_be_set() -> None:
    sut = iox2.config.default()
    value = 5454
    sut.defaults.publish_subscribe.max_nodes = value
    assert sut.defaults.publish_subscribe.max_nodes == value


def test_defaults_publish_subscribe_subscriber_max_buffer_size_can_be_set() -> None:
    sut = iox2.config.default()
    value = 9918
    sut.defaults.publish_subscribe.subscriber_max_buffer_size = value
    assert sut.defaults.publish_subscribe.subscriber_max_buffer_size == value


def test_defaults_publish_subscribe_subscriber_max_borrowed_samples_can_be_set() -> (
    None
):
    sut = iox2.config.default()
    value = 23786
    sut.defaults.publish_subscribe.subscriber_max_borrowed_samples = value
    assert sut.defaults.publish_subscribe.subscriber_max_borrowed_samples == value


def test_defaults_publish_subscribe_publisher_max_loaned_samples_can_be_set() -> None:
    sut = iox2.config.default()
    value = 182
    sut.defaults.publish_subscribe.publisher_max_loaned_samples = value
    assert sut.defaults.publish_subscribe.publisher_max_loaned_samples == value


def test_defaults_publish_subscribe_publisher_history_size_can_be_set() -> None:
    sut = iox2.config.default()
    value = 18221
    sut.defaults.publish_subscribe.publisher_history_size = value
    assert sut.defaults.publish_subscribe.publisher_history_size == value


def test_defaults_publish_subscribe_enable_safe_overflow_can_be_set() -> None:
    sut = iox2.config.default()
    sut.defaults.publish_subscribe.enable_safe_overflow = True
    assert sut.defaults.publish_subscribe.enable_safe_overflow
    sut.defaults.publish_subscribe.enable_safe_overflow = False
    assert not sut.defaults.publish_subscribe.enable_safe_overflow


def test_defaults_publish_subscribe_unable_to_deliver_strategy_can_be_set() -> None:
    sut = iox2.config.default()
    sut.defaults.publish_subscribe.unable_to_deliver_strategy = (
        iox2.UnableToDeliverStrategy.Block
    )
    assert (
        sut.defaults.publish_subscribe.unable_to_deliver_strategy
        == iox2.UnableToDeliverStrategy.Block
    )
    sut.defaults.publish_subscribe.unable_to_deliver_strategy = (
        iox2.UnableToDeliverStrategy.DiscardSample
    )
    assert (
        sut.defaults.publish_subscribe.unable_to_deliver_strategy
        == iox2.UnableToDeliverStrategy.DiscardSample
    )


def test_defaults_publish_subscribe_subscriber_expired_connection_buffer_can_be_set() -> (
    None
):
    sut = iox2.config.default()
    value = 56273
    sut.defaults.publish_subscribe.subscriber_expired_connection_buffer = value
    assert sut.defaults.publish_subscribe.subscriber_expired_connection_buffer == value
