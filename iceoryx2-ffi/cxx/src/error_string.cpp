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

#include "iox2/config_creation_error.hpp"
#include "iox2/connection_failure.hpp"
#include "iox2/enum_translation.hpp"
#include "iox2/iceoryx2.h"
#include "iox2/listener_error.hpp"
#include "iox2/node_failure_enums.hpp"
#include "iox2/node_wait_failure.hpp"
#include "iox2/notifier_error.hpp"
#include "iox2/publisher_error.hpp"
#include "iox2/semantic_string.hpp"
#include "iox2/service_builder_event_error.hpp"
#include "iox2/service_builder_publish_subscribe_error.hpp"
#include "iox2/service_error_enums.hpp"
#include "iox2/subscriber_error.hpp"
#include "iox2/waitset_enums.hpp"

namespace iox2 {

auto error_string(const iox2::ConfigCreationError& error) -> const char* {
    return iox2_config_creation_error_string(iox::into<iox2_config_creation_error_e>(error));
}

auto error_string(const iox2::ConnectionFailure& error) -> const char* {
    return iox2_connection_failure_string(iox::into<iox2_connection_failure_e>(error));
}

auto error_string(const iox2::ServiceDetailsError& error) -> const char* {
    return iox2_service_details_error_string(iox::into<iox2_service_details_error_e>(error));
}

auto error_string(const iox2::ServiceListError& error) -> const char* {
    return iox2_service_list_error_string(iox::into<iox2_service_list_error_e>(error));
}

auto error_string(const iox2::ListenerCreateError& error) -> const char* {
    return iox2_listener_create_error_string(iox::into<iox2_listener_create_error_e>(error));
}

auto error_string(const iox2::ListenerWaitError& error) -> const char* {
    return iox2_listener_wait_error_string(iox::into<iox2_listener_wait_error_e>(error));
}

auto error_string(const iox2::NodeListFailure& error) -> const char* {
    return iox2_node_list_failure_string(iox::into<iox2_node_list_failure_e>(error));
}

auto error_string(const iox2::NodeCreationFailure& error) -> const char* {
    return iox2_node_creation_failure_string(iox::into<iox2_node_creation_failure_e>(error));
}

auto error_string(const iox2::NodeWaitFailure& error) -> const char* {
    return iox2_node_wait_failure_string(iox::into<iox2_node_wait_failure_e>(error));
}

auto error_string(const iox2::NotifierCreateError& error) -> const char* {
    return iox2_notifier_create_error_string(iox::into<iox2_notifier_create_error_e>(error));
}

auto error_string(const iox2::NotifierNotifyError& error) -> const char* {
    return iox2_notifier_notify_error_string(iox::into<iox2_notifier_notify_error_e>(error));
}

auto error_string(const iox2::PublisherCreateError& error) -> const char* {
    return iox2_publisher_create_error_string(iox::into<iox2_publisher_create_error_e>(error));
}

auto error_string(const iox2::PublisherLoanError& error) -> const char* {
    return iox2_publisher_loan_error_string(iox::into<iox2_publisher_loan_error_e>(error));
}

auto error_string(const iox2::PublisherSendError& error) -> const char* {
    return iox2_publisher_send_error_string(iox::into<iox2_publisher_send_error_e>(error));
}

auto error_string(const iox2::PublishSubscribeOpenError& error) -> const char* {
    return iox2_pub_sub_open_or_create_error_string(iox::into<iox2_pub_sub_open_or_create_error_e>(error));
}

auto error_string(const iox2::PublishSubscribeCreateError& error) -> const char* {
    return iox2_pub_sub_open_or_create_error_string(iox::into<iox2_pub_sub_open_or_create_error_e>(error));
}

auto error_string(const iox2::PublishSubscribeOpenOrCreateError& error) -> const char* {
    return iox2_pub_sub_open_or_create_error_string(iox::into<iox2_pub_sub_open_or_create_error_e>(error));
}

auto error_string(const iox2::SemanticStringError& error) -> const char* {
    return iox2_semantic_string_error_string(iox::into<iox2_semantic_string_error_e>(error));
}

auto error_string(const iox2::EventOpenError& error) -> const char* {
    return iox2_event_open_or_create_error_string(iox::into<iox2_event_open_or_create_error_e>(error));
}

auto error_string(const iox2::EventCreateError& error) -> const char* {
    return iox2_event_open_or_create_error_string(iox::into<iox2_event_open_or_create_error_e>(error));
}

auto error_string(const iox2::EventOpenOrCreateError& error) -> const char* {
    return iox2_event_open_or_create_error_string(iox::into<iox2_event_open_or_create_error_e>(error));
}

auto error_string(const iox2::SubscriberCreateError& error) -> const char* {
    return iox2_subscriber_create_error_string(iox::into<iox2_subscriber_create_error_e>(error));
}

auto error_string(const iox2::SubscriberReceiveError& error) -> const char* {
    return iox2_subscriber_receive_error_string(iox::into<iox2_subscriber_receive_error_e>(error));
}

auto error_string(const iox2::WaitSetCreateError& error) -> const char* {
    return iox2_waitset_create_error_string(iox::into<iox2_waitset_create_error_e>(error));
}

auto error_string(const iox2::WaitSetAttachmentError& error) -> const char* {
    return iox2_waitset_attachment_error_string(iox::into<iox2_waitset_attachment_error_e>(error));
}

auto error_string(const iox2::WaitSetRunError& error) -> const char* {
    return iox2_waitset_run_error_string(iox::into<iox2_waitset_run_error_e>(error));
}

} // namespace iox2
