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

#ifndef IOX2_ENUM_TRANSLATION_HPP
#define IOX2_ENUM_TRANSLATION_HPP

#include "iox/assertions.hpp"
#include "iox/into.hpp"
#include "iox2/allocation_strategy.hpp"
#include "iox2/callback_progression.hpp"
#include "iox2/client_error.hpp"
#include "iox2/config_creation_error.hpp"
#include "iox2/connection_failure.hpp"
#include "iox2/iceoryx2.h"
#include "iox2/listener_error.hpp"
#include "iox2/log_level.hpp"
#include "iox2/messaging_pattern.hpp"
#include "iox2/node_failure_enums.hpp"
#include "iox2/node_wait_failure.hpp"
#include "iox2/notifier_error.hpp"
#include "iox2/port_error.hpp"
#include "iox2/publisher_error.hpp"
#include "iox2/semantic_string.hpp"
#include "iox2/server_error.hpp"
#include "iox2/service_builder_event_error.hpp"
#include "iox2/service_builder_publish_subscribe_error.hpp"
#include "iox2/service_builder_request_response_error.hpp"
#include "iox2/service_error_enums.hpp"
#include "iox2/service_type.hpp"
#include "iox2/signal_handling_mode.hpp"
#include "iox2/subscriber_error.hpp"
#include "iox2/type_variant.hpp"
#include "iox2/unable_to_deliver_strategy.hpp"
#include "iox2/waitset_enums.hpp"

namespace iox {
template <>
constexpr auto from<int, iox2::SemanticStringError>(const int value) noexcept -> iox2::SemanticStringError {
    const auto error = static_cast<iox2_semantic_string_error_e>(value);
    switch (error) {
    case iox2_semantic_string_error_e_INVALID_CONTENT:
        return iox2::SemanticStringError::InvalidContent;
    case iox2_semantic_string_error_e_EXCEEDS_MAXIMUM_LENGTH:
        return iox2::SemanticStringError::ExceedsMaximumLength;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto
from<iox2::SemanticStringError, iox2_semantic_string_error_e>(const iox2::SemanticStringError value) noexcept
    -> iox2_semantic_string_error_e {
    switch (value) {
    case iox2::SemanticStringError::InvalidContent:
        return iox2_semantic_string_error_e_INVALID_CONTENT;
    case iox2::SemanticStringError::ExceedsMaximumLength:
        return iox2_semantic_string_error_e_EXCEEDS_MAXIMUM_LENGTH;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::SemanticStringError, const char*>(const iox2::SemanticStringError value) noexcept -> const
    char* {
    return iox2_semantic_string_error_string(iox::into<iox2_semantic_string_error_e>(value));
}

template <>
constexpr auto from<int, iox2::ServiceType>(const int value) noexcept -> iox2::ServiceType {
    const auto service_type = static_cast<iox2_service_type_e>(value);
    switch (service_type) {
    case iox2_service_type_e_IPC:
        return iox2::ServiceType::Ipc;
    case iox2_service_type_e_LOCAL:
        return iox2::ServiceType::Local;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<iox2::ServiceType, iox2_service_type_e>(const iox2::ServiceType value) noexcept
    -> iox2_service_type_e {
    switch (value) {
    case iox2::ServiceType::Ipc:
        return iox2_service_type_e_IPC;
    case iox2::ServiceType::Local:
        return iox2_service_type_e_LOCAL;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<int, iox2::NodeCreationFailure>(const int value) noexcept -> iox2::NodeCreationFailure {
    const auto error = static_cast<iox2_node_creation_failure_e>(value);
    switch (error) {
    case iox2_node_creation_failure_e_INSUFFICIENT_PERMISSIONS:
        return iox2::NodeCreationFailure::InsufficientPermissions;
    case iox2_node_creation_failure_e_INTERNAL_ERROR:
        return iox2::NodeCreationFailure::InternalError;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto
from<iox2::NodeCreationFailure, iox2_node_creation_failure_e>(const iox2::NodeCreationFailure value) noexcept
    -> iox2_node_creation_failure_e {
    switch (value) {
    case iox2::NodeCreationFailure::InsufficientPermissions:
        return iox2_node_creation_failure_e_INSUFFICIENT_PERMISSIONS;
    case iox2::NodeCreationFailure::InternalError:
        return iox2_node_creation_failure_e_INTERNAL_ERROR;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::NodeCreationFailure, const char*>(const iox2::NodeCreationFailure value) noexcept -> const
    char* {
    return iox2_node_creation_failure_string(iox::into<iox2_node_creation_failure_e>(value));
}

template <>
constexpr auto from<int, iox2::CallbackProgression>(const int value) noexcept -> iox2::CallbackProgression {
    const auto error = static_cast<iox2_callback_progression_e>(value);
    switch (error) {
    case iox2_callback_progression_e_CONTINUE:
        return iox2::CallbackProgression::Continue;
    case iox2_callback_progression_e_STOP:
        return iox2::CallbackProgression::Stop;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto
from<iox2::CallbackProgression, iox2_callback_progression_e>(const iox2::CallbackProgression value) noexcept
    -> iox2_callback_progression_e {
    switch (value) {
    case iox2::CallbackProgression::Continue:
        return iox2_callback_progression_e_CONTINUE;
    case iox2::CallbackProgression::Stop:
        return iox2_callback_progression_e_STOP;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<int, iox2::NodeListFailure>(const int value) noexcept -> iox2::NodeListFailure {
    const auto error = static_cast<iox2_node_list_failure_e>(value);
    switch (error) {
    case iox2_node_list_failure_e_INSUFFICIENT_PERMISSIONS:
        return iox2::NodeListFailure::InsufficientPermissions;
    case iox2_node_list_failure_e_INTERNAL_ERROR:
        return iox2::NodeListFailure::InternalError;
    case iox2_node_list_failure_e_INTERRUPT:
        return iox2::NodeListFailure::Interrupt;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<iox2::NodeListFailure, iox2_node_list_failure_e>(const iox2::NodeListFailure value) noexcept
    -> iox2_node_list_failure_e {
    switch (value) {
    case iox2::NodeListFailure::InsufficientPermissions:
        return iox2_node_list_failure_e_INSUFFICIENT_PERMISSIONS;
    case iox2::NodeListFailure::InternalError:
        return iox2_node_list_failure_e_INTERNAL_ERROR;
    case iox2::NodeListFailure::Interrupt:
        return iox2_node_list_failure_e_INTERRUPT;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::NodeListFailure, const char*>(const iox2::NodeListFailure value) noexcept -> const char* {
    return iox2_node_list_failure_string(iox::into<iox2_node_list_failure_e>(value));
}

template <>
constexpr auto from<int, iox2::NodeWaitFailure>(const int value) noexcept -> iox2::NodeWaitFailure {
    const auto error = static_cast<iox2_node_wait_failure_e>(value);
    switch (error) {
    case iox2_node_wait_failure_e_TERMINATION_REQUEST:
        return iox2::NodeWaitFailure::TerminationRequest;
    case iox2_node_wait_failure_e_INTERRUPT:
        return iox2::NodeWaitFailure::Interrupt;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<iox2::NodeWaitFailure, iox2_node_wait_failure_e>(const iox2::NodeWaitFailure value) noexcept
    -> iox2_node_wait_failure_e {
    switch (value) {
    case iox2::NodeWaitFailure::TerminationRequest:
        return iox2_node_wait_failure_e_TERMINATION_REQUEST;
    case iox2::NodeWaitFailure::Interrupt:
        return iox2_node_wait_failure_e_INTERRUPT;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::NodeWaitFailure, const char*>(const iox2::NodeWaitFailure value) noexcept -> const char* {
    return iox2_node_wait_failure_string(iox::into<iox2_node_wait_failure_e>(value));
}

template <>
constexpr auto from<iox2::MessagingPattern, iox2_messaging_pattern_e>(const iox2::MessagingPattern value) noexcept
    -> iox2_messaging_pattern_e {
    switch (value) {
    case iox2::MessagingPattern::PublishSubscribe:
        return iox2_messaging_pattern_e_PUBLISH_SUBSCRIBE;
    case iox2::MessagingPattern::Event:
        return iox2_messaging_pattern_e_EVENT;
    case iox2::MessagingPattern::RequestResponse:
        return iox2_messaging_pattern_e_REQUEST_RESPONSE;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<int, iox2::ServiceDetailsError>(const int value) noexcept -> iox2::ServiceDetailsError {
    const auto error = static_cast<iox2_service_details_error_e>(value);
    switch (error) {
    case iox2_service_details_error_e_FAILED_TO_OPEN_STATIC_SERVICE_INFO:
        return iox2::ServiceDetailsError::FailedToOpenStaticServiceInfo;
    case iox2_service_details_error_e_FAILED_TO_READ_STATIC_SERVICE_INFO:
        return iox2::ServiceDetailsError::FailedToReadStaticServiceInfo;
    case iox2_service_details_error_e_FAILED_TO_ACQUIRE_NODE_STATE:
        return iox2::ServiceDetailsError::FailedToAcquireNodeState;
    case iox2_service_details_error_e_FAILED_TO_DESERIALIZE_STATIC_SERVICE_INFO:
        return iox2::ServiceDetailsError::FailedToDeserializeStaticServiceInfo;
    case iox2_service_details_error_e_INTERNAL_ERROR:
        return iox2::ServiceDetailsError::InternalError;
    case iox2_service_details_error_e_SERVICE_IN_INCONSISTENT_STATE:
        return iox2::ServiceDetailsError::ServiceInInconsistentState;
    case iox2_service_details_error_e_VERSION_MISMATCH:
        return iox2::ServiceDetailsError::VersionMismatch;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto
from<iox2::ServiceDetailsError, iox2_service_details_error_e>(const iox2::ServiceDetailsError value) noexcept
    -> iox2_service_details_error_e {
    switch (value) {
    case iox2::ServiceDetailsError::FailedToOpenStaticServiceInfo:
        return iox2_service_details_error_e_FAILED_TO_OPEN_STATIC_SERVICE_INFO;
    case iox2::ServiceDetailsError::FailedToReadStaticServiceInfo:
        return iox2_service_details_error_e_FAILED_TO_READ_STATIC_SERVICE_INFO;
    case iox2::ServiceDetailsError::FailedToAcquireNodeState:
        return iox2_service_details_error_e_FAILED_TO_ACQUIRE_NODE_STATE;
    case iox2::ServiceDetailsError::FailedToDeserializeStaticServiceInfo:
        return iox2_service_details_error_e_FAILED_TO_DESERIALIZE_STATIC_SERVICE_INFO;
    case iox2::ServiceDetailsError::InternalError:
        return iox2_service_details_error_e_INTERNAL_ERROR;
    case iox2::ServiceDetailsError::ServiceInInconsistentState:
        return iox2_service_details_error_e_SERVICE_IN_INCONSISTENT_STATE;
    case iox2::ServiceDetailsError::VersionMismatch:
        return iox2_service_details_error_e_VERSION_MISMATCH;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::ServiceDetailsError, const char*>(const iox2::ServiceDetailsError value) noexcept -> const
    char* {
    return iox2_service_details_error_string(iox::into<iox2_service_details_error_e>(value));
}

template <>
constexpr auto from<int, iox2::EventOpenOrCreateError>(const int value) noexcept -> iox2::EventOpenOrCreateError {
    const auto error = static_cast<iox2_event_open_or_create_error_e>(value);
    switch (error) {
    case iox2_event_open_or_create_error_e_O_DOES_NOT_EXIST:
        return iox2::EventOpenOrCreateError::OpenDoesNotExist;
    case iox2_event_open_or_create_error_e_O_INSUFFICIENT_PERMISSIONS:
        return iox2::EventOpenOrCreateError::OpenInsufficientPermissions;
    case iox2_event_open_or_create_error_e_O_SERVICE_IN_CORRUPTED_STATE:
        return iox2::EventOpenOrCreateError::OpenServiceInCorruptedState;
    case iox2_event_open_or_create_error_e_O_INCOMPATIBLE_MESSAGING_PATTERN:
        return iox2::EventOpenOrCreateError::OpenIncompatibleMessagingPattern;
    case iox2_event_open_or_create_error_e_O_INCOMPATIBLE_ATTRIBUTES:
        return iox2::EventOpenOrCreateError::OpenIncompatibleAttributes;
    case iox2_event_open_or_create_error_e_O_INCOMPATIBLE_DEADLINE:
        return iox2::EventOpenOrCreateError::OpenIncompatibleDeadline;
    case iox2_event_open_or_create_error_e_O_INTERNAL_FAILURE:
        return iox2::EventOpenOrCreateError::OpenInternalFailure;
    case iox2_event_open_or_create_error_e_O_HANGS_IN_CREATION:
        return iox2::EventOpenOrCreateError::OpenHangsInCreation;
    case iox2_event_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NOTIFIERS:
        return iox2::EventOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfNotifiers;
    case iox2_event_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_LISTENERS:
        return iox2::EventOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfListeners;
    case iox2_event_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_MAX_EVENT_ID:
        return iox2::EventOpenOrCreateError::OpenDoesNotSupportRequestedMaxEventId;
    case iox2_event_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES:
        return iox2::EventOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfNodes;
    case iox2_event_open_or_create_error_e_O_EXCEEDS_MAX_NUMBER_OF_NODES:
        return iox2::EventOpenOrCreateError::OpenExceedsMaxNumberOfNodes;
    case iox2_event_open_or_create_error_e_O_IS_MARKED_FOR_DESTRUCTION:
        return iox2::EventOpenOrCreateError::OpenIsMarkedForDestruction;
    case iox2_event_open_or_create_error_e_O_INCOMPATIBLE_NOTIFIER_CREATED_EVENT:
        return iox2::EventOpenOrCreateError::OpenIncompatibleNotifierCreatedEvent;
    case iox2_event_open_or_create_error_e_O_INCOMPATIBLE_NOTIFIER_DROPPED_EVENT:
        return iox2::EventOpenOrCreateError::OpenIncompatibleNotifierDroppedEvent;
    case iox2_event_open_or_create_error_e_O_INCOMPATIBLE_NOTIFIER_DEAD_EVENT:
        return iox2::EventOpenOrCreateError::OpenIncompatibleNotifierDeadEvent;
    case iox2_event_open_or_create_error_e_C_SERVICE_IN_CORRUPTED_STATE:
        return iox2::EventOpenOrCreateError::CreateServiceInCorruptedState;
    case iox2_event_open_or_create_error_e_C_INTERNAL_FAILURE:
        return iox2::EventOpenOrCreateError::CreateInternalFailure;
    case iox2_event_open_or_create_error_e_C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE:
        return iox2::EventOpenOrCreateError::CreateIsBeingCreatedByAnotherInstance;
    case iox2_event_open_or_create_error_e_C_ALREADY_EXISTS:
        return iox2::EventOpenOrCreateError::CreateAlreadyExists;
    case iox2_event_open_or_create_error_e_C_HANGS_IN_CREATION:
        return iox2::EventOpenOrCreateError::CreateHangsInCreation;
    case iox2_event_open_or_create_error_e_C_INSUFFICIENT_PERMISSIONS:
        return iox2::EventOpenOrCreateError::CreateInsufficientPermissions;
    case iox2_event_open_or_create_error_e_C_OLD_CONNECTION_STILL_ACTIVE:
        return iox2::EventOpenOrCreateError::CreateOldConnectionsStillActive;
    case iox2_event_open_or_create_error_e_SYSTEM_IN_FLUX:
        return iox2::EventOpenOrCreateError::SystemInFlux;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto
from<iox2::EventOpenOrCreateError, iox2_event_open_or_create_error_e>(const iox2::EventOpenOrCreateError value) noexcept
    -> iox2_event_open_or_create_error_e {
    switch (value) {
    case iox2::EventOpenOrCreateError::OpenDoesNotExist:
        return iox2_event_open_or_create_error_e_O_DOES_NOT_EXIST;
    case iox2::EventOpenOrCreateError::OpenInsufficientPermissions:
        return iox2_event_open_or_create_error_e_O_INSUFFICIENT_PERMISSIONS;
    case iox2::EventOpenOrCreateError::OpenServiceInCorruptedState:
        return iox2_event_open_or_create_error_e_O_SERVICE_IN_CORRUPTED_STATE;
    case iox2::EventOpenOrCreateError::OpenIncompatibleMessagingPattern:
        return iox2_event_open_or_create_error_e_O_INCOMPATIBLE_MESSAGING_PATTERN;
    case iox2::EventOpenOrCreateError::OpenIncompatibleAttributes:
        return iox2_event_open_or_create_error_e_O_INCOMPATIBLE_ATTRIBUTES;
    case iox2::EventOpenOrCreateError::OpenInternalFailure:
        return iox2_event_open_or_create_error_e_O_INTERNAL_FAILURE;
    case iox2::EventOpenOrCreateError::OpenHangsInCreation:
        return iox2_event_open_or_create_error_e_O_HANGS_IN_CREATION;
    case iox2::EventOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfNotifiers:
        return iox2_event_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NOTIFIERS;
    case iox2::EventOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfListeners:
        return iox2_event_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_LISTENERS;
    case iox2::EventOpenOrCreateError::OpenDoesNotSupportRequestedMaxEventId:
        return iox2_event_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_MAX_EVENT_ID;
    case iox2::EventOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfNodes:
        return iox2_event_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES;
    case iox2::EventOpenOrCreateError::OpenExceedsMaxNumberOfNodes:
        return iox2_event_open_or_create_error_e_O_EXCEEDS_MAX_NUMBER_OF_NODES;
    case iox2::EventOpenOrCreateError::OpenIsMarkedForDestruction:
        return iox2_event_open_or_create_error_e_O_IS_MARKED_FOR_DESTRUCTION;

    case iox2::EventOpenOrCreateError::CreateServiceInCorruptedState:
        return iox2_event_open_or_create_error_e_C_SERVICE_IN_CORRUPTED_STATE;
    case iox2::EventOpenOrCreateError::CreateInternalFailure:
        return iox2_event_open_or_create_error_e_C_INTERNAL_FAILURE;
    case iox2::EventOpenOrCreateError::CreateIsBeingCreatedByAnotherInstance:
        return iox2_event_open_or_create_error_e_C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE;
    case iox2::EventOpenOrCreateError::CreateAlreadyExists:
        return iox2_event_open_or_create_error_e_C_ALREADY_EXISTS;
    case iox2::EventOpenOrCreateError::CreateHangsInCreation:
        return iox2_event_open_or_create_error_e_C_HANGS_IN_CREATION;
    case iox2::EventOpenOrCreateError::CreateInsufficientPermissions:
        return iox2_event_open_or_create_error_e_C_INSUFFICIENT_PERMISSIONS;
    case iox2::EventOpenOrCreateError::CreateOldConnectionsStillActive:
        return iox2_event_open_or_create_error_e_C_OLD_CONNECTION_STILL_ACTIVE;
    default:
        IOX_UNREACHABLE();
    }
}

template <>
inline auto from<iox2::EventOpenOrCreateError, const char*>(const iox2::EventOpenOrCreateError value) noexcept -> const
    char* {
    return iox2_event_open_or_create_error_string(iox::into<iox2_event_open_or_create_error_e>(value));
}

template <>
constexpr auto from<int, iox2::EventOpenError>(const int value) noexcept -> iox2::EventOpenError {
    const auto error = static_cast<iox2_event_open_or_create_error_e>(value);
    switch (error) {
    case iox2_event_open_or_create_error_e_O_DOES_NOT_EXIST:
        return iox2::EventOpenError::DoesNotExist;
    case iox2_event_open_or_create_error_e_O_INSUFFICIENT_PERMISSIONS:
        return iox2::EventOpenError::InsufficientPermissions;
    case iox2_event_open_or_create_error_e_O_SERVICE_IN_CORRUPTED_STATE:
        return iox2::EventOpenError::ServiceInCorruptedState;
    case iox2_event_open_or_create_error_e_O_INCOMPATIBLE_MESSAGING_PATTERN:
        return iox2::EventOpenError::IncompatibleMessagingPattern;
    case iox2_event_open_or_create_error_e_O_INCOMPATIBLE_ATTRIBUTES:
        return iox2::EventOpenError::IncompatibleAttributes;
    case iox2_event_open_or_create_error_e_O_INTERNAL_FAILURE:
        return iox2::EventOpenError::InternalFailure;
    case iox2_event_open_or_create_error_e_O_HANGS_IN_CREATION:
        return iox2::EventOpenError::HangsInCreation;
    case iox2_event_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NOTIFIERS:
        return iox2::EventOpenError::DoesNotSupportRequestedAmountOfNotifiers;
    case iox2_event_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_LISTENERS:
        return iox2::EventOpenError::DoesNotSupportRequestedAmountOfListeners;
    case iox2_event_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_MAX_EVENT_ID:
        return iox2::EventOpenError::DoesNotSupportRequestedMaxEventId;
    case iox2_event_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES:
        return iox2::EventOpenError::DoesNotSupportRequestedAmountOfNodes;
    case iox2_event_open_or_create_error_e_O_EXCEEDS_MAX_NUMBER_OF_NODES:
        return iox2::EventOpenError::ExceedsMaxNumberOfNodes;
    case iox2_event_open_or_create_error_e_O_IS_MARKED_FOR_DESTRUCTION:
        return iox2::EventOpenError::IsMarkedForDestruction;
    default:
        IOX_UNREACHABLE();
    }
}

template <>
constexpr auto from<iox2::EventOpenError, iox2_event_open_or_create_error_e>(const iox2::EventOpenError value) noexcept
    -> iox2_event_open_or_create_error_e {
    switch (value) {
    case iox2::EventOpenError::DoesNotExist:
        return iox2_event_open_or_create_error_e_O_DOES_NOT_EXIST;
    case iox2::EventOpenError::InsufficientPermissions:
        return iox2_event_open_or_create_error_e_O_INSUFFICIENT_PERMISSIONS;
    case iox2::EventOpenError::ServiceInCorruptedState:
        return iox2_event_open_or_create_error_e_O_SERVICE_IN_CORRUPTED_STATE;
    case iox2::EventOpenError::IncompatibleMessagingPattern:
        return iox2_event_open_or_create_error_e_O_INCOMPATIBLE_MESSAGING_PATTERN;
    case iox2::EventOpenError::IncompatibleAttributes:
        return iox2_event_open_or_create_error_e_O_INCOMPATIBLE_ATTRIBUTES;
    case iox2::EventOpenError::InternalFailure:
        return iox2_event_open_or_create_error_e_O_INTERNAL_FAILURE;
    case iox2::EventOpenError::HangsInCreation:
        return iox2_event_open_or_create_error_e_O_HANGS_IN_CREATION;
    case iox2::EventOpenError::DoesNotSupportRequestedAmountOfNotifiers:
        return iox2_event_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NOTIFIERS;
    case iox2::EventOpenError::DoesNotSupportRequestedAmountOfListeners:
        return iox2_event_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_LISTENERS;
    case iox2::EventOpenError::DoesNotSupportRequestedMaxEventId:
        return iox2_event_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_MAX_EVENT_ID;
    case iox2::EventOpenError::DoesNotSupportRequestedAmountOfNodes:
        return iox2_event_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES;
    case iox2::EventOpenError::ExceedsMaxNumberOfNodes:
        return iox2_event_open_or_create_error_e_O_EXCEEDS_MAX_NUMBER_OF_NODES;
    case iox2::EventOpenError::IsMarkedForDestruction:
        return iox2_event_open_or_create_error_e_O_IS_MARKED_FOR_DESTRUCTION;
    default:
        IOX_UNREACHABLE();
    }
}

template <>
inline auto from<iox2::EventOpenError, const char*>(const iox2::EventOpenError value) noexcept -> const char* {
    return iox2_event_open_or_create_error_string(iox::into<iox2_event_open_or_create_error_e>(value));
}

template <>
constexpr auto from<int, iox2::EventCreateError>(const int value) noexcept -> iox2::EventCreateError {
    const auto error = static_cast<iox2_event_open_or_create_error_e>(value);
    switch (error) {
    case iox2_event_open_or_create_error_e_C_SERVICE_IN_CORRUPTED_STATE:
        return iox2::EventCreateError::ServiceInCorruptedState;
    case iox2_event_open_or_create_error_e_C_INTERNAL_FAILURE:
        return iox2::EventCreateError::InternalFailure;
    case iox2_event_open_or_create_error_e_C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE:
        return iox2::EventCreateError::IsBeingCreatedByAnotherInstance;
    case iox2_event_open_or_create_error_e_C_ALREADY_EXISTS:
        return iox2::EventCreateError::AlreadyExists;
    case iox2_event_open_or_create_error_e_C_HANGS_IN_CREATION:
        return iox2::EventCreateError::HangsInCreation;
    case iox2_event_open_or_create_error_e_C_INSUFFICIENT_PERMISSIONS:
        return iox2::EventCreateError::InsufficientPermissions;
    default:
        IOX_UNREACHABLE();
    }
}

template <>
constexpr auto
from<iox2::EventCreateError, iox2_event_open_or_create_error_e>(const iox2::EventCreateError value) noexcept
    -> iox2_event_open_or_create_error_e {
    switch (value) {
    case iox2::EventCreateError::InsufficientPermissions:
        return iox2_event_open_or_create_error_e_C_INSUFFICIENT_PERMISSIONS;
    case iox2::EventCreateError::HangsInCreation:
        return iox2_event_open_or_create_error_e_C_HANGS_IN_CREATION;
    case iox2::EventCreateError::AlreadyExists:
        return iox2_event_open_or_create_error_e_C_ALREADY_EXISTS;
    case iox2::EventCreateError::IsBeingCreatedByAnotherInstance:
        return iox2_event_open_or_create_error_e_C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE;
    case iox2::EventCreateError::InternalFailure:
        return iox2_event_open_or_create_error_e_C_INTERNAL_FAILURE;
    case iox2::EventCreateError::ServiceInCorruptedState:
        return iox2_event_open_or_create_error_e_C_SERVICE_IN_CORRUPTED_STATE;
    case iox2::EventCreateError::OldConnectionsStillActive:
        return iox2_event_open_or_create_error_e_C_OLD_CONNECTION_STILL_ACTIVE;
    default:
        IOX_UNREACHABLE();
    }
}

template <>
inline auto from<iox2::EventCreateError, const char*>(const iox2::EventCreateError value) noexcept -> const char* {
    return iox2_event_open_or_create_error_string(iox::into<iox2_event_open_or_create_error_e>(value));
}

template <>
constexpr auto from<int, iox2::PublishSubscribeOpenOrCreateError>(const int value) noexcept
    -> iox2::PublishSubscribeOpenOrCreateError {
    const auto error = static_cast<iox2_pub_sub_open_or_create_error_e>(value);
    switch (error) {
    case iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_EXIST:
        return iox2::PublishSubscribeOpenOrCreateError::OpenDoesNotExist;
    case iox2_pub_sub_open_or_create_error_e_O_INTERNAL_FAILURE:
        return iox2::PublishSubscribeOpenOrCreateError::OpenInternalFailure;
    case iox2_pub_sub_open_or_create_error_e_O_INCOMPATIBLE_TYPES:
        return iox2::PublishSubscribeOpenOrCreateError::OpenIncompatibleTypes;
    case iox2_pub_sub_open_or_create_error_e_O_INCOMPATIBLE_MESSAGING_PATTERN:
        return iox2::PublishSubscribeOpenOrCreateError::OpenIncompatibleMessagingPattern;
    case iox2_pub_sub_open_or_create_error_e_O_INCOMPATIBLE_ATTRIBUTES:
        return iox2::PublishSubscribeOpenOrCreateError::OpenIncompatibleAttributes;
    case iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_MIN_BUFFER_SIZE:
        return iox2::PublishSubscribeOpenOrCreateError::OpenDoesNotSupportRequestedMinBufferSize;
    case iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_MIN_HISTORY_SIZE:
        return iox2::PublishSubscribeOpenOrCreateError::OpenDoesNotSupportRequestedMinHistorySize;
    case iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_MIN_SUBSCRIBER_BORROWED_SAMPLES:
        return iox2::PublishSubscribeOpenOrCreateError::OpenDoesNotSupportRequestedMinSubscriberBorrowedSamples;
    case iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_PUBLISHERS:
        return iox2::PublishSubscribeOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfPublishers;
    case iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_SUBSCRIBERS:
        return iox2::PublishSubscribeOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfSubscribers;
    case iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES:
        return iox2::PublishSubscribeOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfNodes;
    case iox2_pub_sub_open_or_create_error_e_O_INCOMPATIBLE_OVERFLOW_BEHAVIOR:
        return iox2::PublishSubscribeOpenOrCreateError::OpenIncompatibleOverflowBehavior;
    case iox2_pub_sub_open_or_create_error_e_O_INSUFFICIENT_PERMISSIONS:
        return iox2::PublishSubscribeOpenOrCreateError::OpenInsufficientPermissions;
    case iox2_pub_sub_open_or_create_error_e_O_SERVICE_IN_CORRUPTED_STATE:
        return iox2::PublishSubscribeOpenOrCreateError::OpenServiceInCorruptedState;
    case iox2_pub_sub_open_or_create_error_e_O_HANGS_IN_CREATION:
        return iox2::PublishSubscribeOpenOrCreateError::OpenHangsInCreation;
    case iox2_pub_sub_open_or_create_error_e_O_EXCEEDS_MAX_NUMBER_OF_NODES:
        return iox2::PublishSubscribeOpenOrCreateError::OpenExceedsMaxNumberOfNodes;
    case iox2_pub_sub_open_or_create_error_e_O_IS_MARKED_FOR_DESTRUCTION:
        return iox2::PublishSubscribeOpenOrCreateError::OpenIsMarkedForDestruction;

    case iox2_pub_sub_open_or_create_error_e_C_SERVICE_IN_CORRUPTED_STATE:
        return iox2::PublishSubscribeOpenOrCreateError::CreateServiceInCorruptedState;
    case iox2_pub_sub_open_or_create_error_e_C_SUBSCRIBER_BUFFER_MUST_BE_LARGER_THAN_HISTORY_SIZE:
        return iox2::PublishSubscribeOpenOrCreateError::CreateSubscriberBufferMustBeLargerThanHistorySize;
    case iox2_pub_sub_open_or_create_error_e_C_ALREADY_EXISTS:
        return iox2::PublishSubscribeOpenOrCreateError::CreateAlreadyExists;
    case iox2_pub_sub_open_or_create_error_e_C_INSUFFICIENT_PERMISSIONS:
        return iox2::PublishSubscribeOpenOrCreateError::CreateInsufficientPermissions;
    case iox2_pub_sub_open_or_create_error_e_C_INTERNAL_FAILURE:
        return iox2::PublishSubscribeOpenOrCreateError::CreateInternalFailure;
    case iox2_pub_sub_open_or_create_error_e_C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE:
        return iox2::PublishSubscribeOpenOrCreateError::CreateIsBeingCreatedByAnotherInstance;
    case iox2_pub_sub_open_or_create_error_e_C_HANGS_IN_CREATION:
        return iox2::PublishSubscribeOpenOrCreateError::CreateHangsInCreation;
    case iox2_pub_sub_open_or_create_error_e_C_OLD_CONNECTION_STILL_ACTIVE:
        return iox2::PublishSubscribeOpenOrCreateError::CreateOldConnectionsStillActive;
    case iox2_pub_sub_open_or_create_error_e_SYSTEM_IN_FLUX:
        return iox2::PublishSubscribeOpenOrCreateError::SystemInFlux;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<int, iox2::PublishSubscribeOpenError>(const int value) noexcept -> iox2::PublishSubscribeOpenError {
    const auto error = static_cast<iox2_pub_sub_open_or_create_error_e>(value);
    switch (error) {
    case iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_EXIST:
        return iox2::PublishSubscribeOpenError::DoesNotExist;
    case iox2_pub_sub_open_or_create_error_e_O_INTERNAL_FAILURE:
        return iox2::PublishSubscribeOpenError::InternalFailure;
    case iox2_pub_sub_open_or_create_error_e_O_INCOMPATIBLE_TYPES:
        return iox2::PublishSubscribeOpenError::IncompatibleTypes;
    case iox2_pub_sub_open_or_create_error_e_O_INCOMPATIBLE_MESSAGING_PATTERN:
        return iox2::PublishSubscribeOpenError::IncompatibleMessagingPattern;
    case iox2_pub_sub_open_or_create_error_e_O_INCOMPATIBLE_ATTRIBUTES:
        return iox2::PublishSubscribeOpenError::IncompatibleAttributes;
    case iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_MIN_BUFFER_SIZE:
        return iox2::PublishSubscribeOpenError::DoesNotSupportRequestedMinBufferSize;
    case iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_MIN_HISTORY_SIZE:
        return iox2::PublishSubscribeOpenError::DoesNotSupportRequestedMinHistorySize;
    case iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_MIN_SUBSCRIBER_BORROWED_SAMPLES:
        return iox2::PublishSubscribeOpenError::DoesNotSupportRequestedMinSubscriberBorrowedSamples;
    case iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_PUBLISHERS:
        return iox2::PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfPublishers;
    case iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_SUBSCRIBERS:
        return iox2::PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfSubscribers;
    case iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES:
        return iox2::PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfNodes;
    case iox2_pub_sub_open_or_create_error_e_O_INCOMPATIBLE_OVERFLOW_BEHAVIOR:
        return iox2::PublishSubscribeOpenError::IncompatibleOverflowBehavior;
    case iox2_pub_sub_open_or_create_error_e_O_INSUFFICIENT_PERMISSIONS:
        return iox2::PublishSubscribeOpenError::InsufficientPermissions;
    case iox2_pub_sub_open_or_create_error_e_O_SERVICE_IN_CORRUPTED_STATE:
        return iox2::PublishSubscribeOpenError::ServiceInCorruptedState;
    case iox2_pub_sub_open_or_create_error_e_O_HANGS_IN_CREATION:
        return iox2::PublishSubscribeOpenError::HangsInCreation;
    case iox2_pub_sub_open_or_create_error_e_O_EXCEEDS_MAX_NUMBER_OF_NODES:
        return iox2::PublishSubscribeOpenError::ExceedsMaxNumberOfNodes;
    case iox2_pub_sub_open_or_create_error_e_O_IS_MARKED_FOR_DESTRUCTION:
        return iox2::PublishSubscribeOpenError::IsMarkedForDestruction;
    default:
        IOX_UNREACHABLE();
    }
}

template <>
constexpr auto from<iox2::PublishSubscribeOpenError, iox2_pub_sub_open_or_create_error_e>(
    const iox2::PublishSubscribeOpenError value) noexcept -> iox2_pub_sub_open_or_create_error_e {
    switch (value) {
    case iox2::PublishSubscribeOpenError::DoesNotExist:
        return iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_EXIST;
    case iox2::PublishSubscribeOpenError::InternalFailure:
        return iox2_pub_sub_open_or_create_error_e_O_INTERNAL_FAILURE;
    case iox2::PublishSubscribeOpenError::IncompatibleTypes:
        return iox2_pub_sub_open_or_create_error_e_O_INCOMPATIBLE_TYPES;
    case iox2::PublishSubscribeOpenError::IncompatibleMessagingPattern:
        return iox2_pub_sub_open_or_create_error_e_O_INCOMPATIBLE_MESSAGING_PATTERN;
    case iox2::PublishSubscribeOpenError::IncompatibleAttributes:
        return iox2_pub_sub_open_or_create_error_e_O_INCOMPATIBLE_ATTRIBUTES;
    case iox2::PublishSubscribeOpenError::DoesNotSupportRequestedMinBufferSize:
        return iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_MIN_BUFFER_SIZE;
    case iox2::PublishSubscribeOpenError::DoesNotSupportRequestedMinHistorySize:
        return iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_MIN_HISTORY_SIZE;
    case iox2::PublishSubscribeOpenError::DoesNotSupportRequestedMinSubscriberBorrowedSamples:
        return iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_MIN_SUBSCRIBER_BORROWED_SAMPLES;
    case iox2::PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfPublishers:
        return iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_PUBLISHERS;
    case iox2::PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfSubscribers:
        return iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_SUBSCRIBERS;
    case iox2::PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfNodes:
        return iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES;
    case iox2::PublishSubscribeOpenError::IncompatibleOverflowBehavior:
        return iox2_pub_sub_open_or_create_error_e_O_INCOMPATIBLE_OVERFLOW_BEHAVIOR;
    case iox2::PublishSubscribeOpenError::InsufficientPermissions:
        return iox2_pub_sub_open_or_create_error_e_O_INSUFFICIENT_PERMISSIONS;
    case iox2::PublishSubscribeOpenError::ServiceInCorruptedState:
        return iox2_pub_sub_open_or_create_error_e_O_SERVICE_IN_CORRUPTED_STATE;
    case iox2::PublishSubscribeOpenError::HangsInCreation:
        return iox2_pub_sub_open_or_create_error_e_O_HANGS_IN_CREATION;
    case iox2::PublishSubscribeOpenError::ExceedsMaxNumberOfNodes:
        return iox2_pub_sub_open_or_create_error_e_O_EXCEEDS_MAX_NUMBER_OF_NODES;
    case iox2::PublishSubscribeOpenError::IsMarkedForDestruction:
        return iox2_pub_sub_open_or_create_error_e_O_IS_MARKED_FOR_DESTRUCTION;
    default:
        IOX_UNREACHABLE();
    }
}

template <>
inline auto from<iox2::PublishSubscribeOpenError, const char*>(const iox2::PublishSubscribeOpenError value) noexcept
    -> const char* {
    return iox2_pub_sub_open_or_create_error_string(iox::into<iox2_pub_sub_open_or_create_error_e>(value));
}

template <>
constexpr auto from<int, iox2::PublishSubscribeCreateError>(const int value) noexcept
    -> iox2::PublishSubscribeCreateError {
    const auto error = static_cast<iox2_pub_sub_open_or_create_error_e>(value);
    switch (error) {
    case iox2_pub_sub_open_or_create_error_e_C_SERVICE_IN_CORRUPTED_STATE:
        return iox2::PublishSubscribeCreateError::ServiceInCorruptedState;
    case iox2_pub_sub_open_or_create_error_e_C_SUBSCRIBER_BUFFER_MUST_BE_LARGER_THAN_HISTORY_SIZE:
        return iox2::PublishSubscribeCreateError::SubscriberBufferMustBeLargerThanHistorySize;
    case iox2_pub_sub_open_or_create_error_e_C_ALREADY_EXISTS:
        return iox2::PublishSubscribeCreateError::AlreadyExists;
    case iox2_pub_sub_open_or_create_error_e_C_INSUFFICIENT_PERMISSIONS:
        return iox2::PublishSubscribeCreateError::InsufficientPermissions;
    case iox2_pub_sub_open_or_create_error_e_C_INTERNAL_FAILURE:
        return iox2::PublishSubscribeCreateError::InternalFailure;
    case iox2_pub_sub_open_or_create_error_e_C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE:
        return iox2::PublishSubscribeCreateError::IsBeingCreatedByAnotherInstance;
    case iox2_pub_sub_open_or_create_error_e_C_HANGS_IN_CREATION:
        return iox2::PublishSubscribeCreateError::HangsInCreation;
    default:
        IOX_UNREACHABLE();
    }
}

template <>
constexpr auto from<iox2::PublishSubscribeCreateError, iox2_pub_sub_open_or_create_error_e>(
    const iox2::PublishSubscribeCreateError value) noexcept -> iox2_pub_sub_open_or_create_error_e {
    switch (value) {
    case iox2::PublishSubscribeCreateError::ServiceInCorruptedState:
        return iox2_pub_sub_open_or_create_error_e_C_SERVICE_IN_CORRUPTED_STATE;
    case iox2::PublishSubscribeCreateError::SubscriberBufferMustBeLargerThanHistorySize:
        return iox2_pub_sub_open_or_create_error_e_C_SUBSCRIBER_BUFFER_MUST_BE_LARGER_THAN_HISTORY_SIZE;
    case iox2::PublishSubscribeCreateError::AlreadyExists:
        return iox2_pub_sub_open_or_create_error_e_C_ALREADY_EXISTS;
    case iox2::PublishSubscribeCreateError::InsufficientPermissions:
        return iox2_pub_sub_open_or_create_error_e_C_INSUFFICIENT_PERMISSIONS;
    case iox2::PublishSubscribeCreateError::InternalFailure:
        return iox2_pub_sub_open_or_create_error_e_C_INTERNAL_FAILURE;
    case iox2::PublishSubscribeCreateError::IsBeingCreatedByAnotherInstance:
        return iox2_pub_sub_open_or_create_error_e_C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE;
    case iox2::PublishSubscribeCreateError::HangsInCreation:
        return iox2_pub_sub_open_or_create_error_e_C_HANGS_IN_CREATION;
    default:
        IOX_UNREACHABLE();
    }
}

template <>
inline auto from<iox2::PublishSubscribeCreateError, const char*>(const iox2::PublishSubscribeCreateError value) noexcept
    -> const char* {
    return iox2_pub_sub_open_or_create_error_string(iox::into<iox2_pub_sub_open_or_create_error_e>(value));
}

template <>
constexpr auto from<iox2::PublishSubscribeOpenOrCreateError, iox2_pub_sub_open_or_create_error_e>(
    const iox2::PublishSubscribeOpenOrCreateError value) noexcept -> iox2_pub_sub_open_or_create_error_e {
    switch (value) {
    case iox2::PublishSubscribeOpenOrCreateError::OpenDoesNotExist:
        return iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_EXIST;
    case iox2::PublishSubscribeOpenOrCreateError::OpenInternalFailure:
        return iox2_pub_sub_open_or_create_error_e_O_INTERNAL_FAILURE;
    case iox2::PublishSubscribeOpenOrCreateError::OpenIncompatibleTypes:
        return iox2_pub_sub_open_or_create_error_e_O_INCOMPATIBLE_TYPES;
    case iox2::PublishSubscribeOpenOrCreateError::OpenIncompatibleMessagingPattern:
        return iox2_pub_sub_open_or_create_error_e_O_INCOMPATIBLE_MESSAGING_PATTERN;
    case iox2::PublishSubscribeOpenOrCreateError::OpenIncompatibleAttributes:
        return iox2_pub_sub_open_or_create_error_e_O_INCOMPATIBLE_ATTRIBUTES;
    case iox2::PublishSubscribeOpenOrCreateError::OpenDoesNotSupportRequestedMinBufferSize:
        return iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_MIN_BUFFER_SIZE;
    case iox2::PublishSubscribeOpenOrCreateError::OpenDoesNotSupportRequestedMinHistorySize:
        return iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_MIN_HISTORY_SIZE;
    case iox2::PublishSubscribeOpenOrCreateError::OpenDoesNotSupportRequestedMinSubscriberBorrowedSamples:
        return iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_MIN_SUBSCRIBER_BORROWED_SAMPLES;
    case iox2::PublishSubscribeOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfPublishers:
        return iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_PUBLISHERS;
    case iox2::PublishSubscribeOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfSubscribers:
        return iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_SUBSCRIBERS;
    case iox2::PublishSubscribeOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfNodes:
        return iox2_pub_sub_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES;
    case iox2::PublishSubscribeOpenOrCreateError::OpenIncompatibleOverflowBehavior:
        return iox2_pub_sub_open_or_create_error_e_O_INCOMPATIBLE_OVERFLOW_BEHAVIOR;
    case iox2::PublishSubscribeOpenOrCreateError::OpenInsufficientPermissions:
        return iox2_pub_sub_open_or_create_error_e_O_INSUFFICIENT_PERMISSIONS;
    case iox2::PublishSubscribeOpenOrCreateError::OpenServiceInCorruptedState:
        return iox2_pub_sub_open_or_create_error_e_O_SERVICE_IN_CORRUPTED_STATE;
    case iox2::PublishSubscribeOpenOrCreateError::OpenHangsInCreation:
        return iox2_pub_sub_open_or_create_error_e_O_HANGS_IN_CREATION;
    case iox2::PublishSubscribeOpenOrCreateError::OpenExceedsMaxNumberOfNodes:
        return iox2_pub_sub_open_or_create_error_e_O_EXCEEDS_MAX_NUMBER_OF_NODES;
    case iox2::PublishSubscribeOpenOrCreateError::OpenIsMarkedForDestruction:
        return iox2_pub_sub_open_or_create_error_e_O_IS_MARKED_FOR_DESTRUCTION;
    case iox2::PublishSubscribeOpenOrCreateError::CreateServiceInCorruptedState:
        return iox2_pub_sub_open_or_create_error_e_C_SERVICE_IN_CORRUPTED_STATE;
    case iox2::PublishSubscribeOpenOrCreateError::CreateSubscriberBufferMustBeLargerThanHistorySize:
        return iox2_pub_sub_open_or_create_error_e_C_SUBSCRIBER_BUFFER_MUST_BE_LARGER_THAN_HISTORY_SIZE;
    case iox2::PublishSubscribeOpenOrCreateError::CreateAlreadyExists:
        return iox2_pub_sub_open_or_create_error_e_C_ALREADY_EXISTS;
    case iox2::PublishSubscribeOpenOrCreateError::CreateInsufficientPermissions:
        return iox2_pub_sub_open_or_create_error_e_C_INSUFFICIENT_PERMISSIONS;
    case iox2::PublishSubscribeOpenOrCreateError::CreateInternalFailure:
        return iox2_pub_sub_open_or_create_error_e_C_INTERNAL_FAILURE;
    case iox2::PublishSubscribeOpenOrCreateError::CreateIsBeingCreatedByAnotherInstance:
        return iox2_pub_sub_open_or_create_error_e_C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE;
    case iox2::PublishSubscribeOpenOrCreateError::CreateHangsInCreation:
        return iox2_pub_sub_open_or_create_error_e_C_HANGS_IN_CREATION;
    case iox2::PublishSubscribeOpenOrCreateError::CreateOldConnectionsStillActive:
        return iox2_pub_sub_open_or_create_error_e_C_OLD_CONNECTION_STILL_ACTIVE;
    default:
        IOX_UNREACHABLE();
    }
}

template <>
inline auto
from<iox2::PublishSubscribeOpenOrCreateError, const char*>(const iox2::PublishSubscribeOpenOrCreateError value) noexcept
    -> const char* {
    return iox2_pub_sub_open_or_create_error_string(iox::into<iox2_pub_sub_open_or_create_error_e>(value));
}

template <>
constexpr auto from<int, iox2::RequestResponseCreateError>(const int value) noexcept
    -> iox2::RequestResponseCreateError {
    const auto error = static_cast<iox2_request_response_open_or_create_error_e>(value);
    switch (error) {
    case iox2_request_response_open_or_create_error_e_C_ALREADY_EXISTS:
        return iox2::RequestResponseCreateError::AlreadyExists;
    case iox2_request_response_open_or_create_error_e_C_INTERNAL_FAILURE:
        return iox2::RequestResponseCreateError::InternalFailure;
    case iox2_request_response_open_or_create_error_e_C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE:
        return iox2::RequestResponseCreateError::IsBeingCreatedByAnotherInstance;
    case iox2_request_response_open_or_create_error_e_C_INSUFFICIENT_PERMISSIONS:
        return iox2::RequestResponseCreateError::InsufficientPermissions;
    case iox2_request_response_open_or_create_error_e_C_HANGS_IN_CREATION:
        return iox2::RequestResponseCreateError::HangsInCreation;
    case iox2_request_response_open_or_create_error_e_C_SERVICE_IN_CORRUPTED_STATE:
        return iox2::RequestResponseCreateError::ServiceInCorruptedState;
    default:
        IOX_UNREACHABLE();
    }
}

template <>
constexpr auto from<iox2::RequestResponseCreateError, iox2_request_response_open_or_create_error_e>(
    const iox2::RequestResponseCreateError value) noexcept -> iox2_request_response_open_or_create_error_e {
    switch (value) {
    case iox2::RequestResponseCreateError::AlreadyExists:
        return iox2_request_response_open_or_create_error_e_C_ALREADY_EXISTS;
    case iox2::RequestResponseCreateError::InternalFailure:
        return iox2_request_response_open_or_create_error_e_C_INTERNAL_FAILURE;
    case iox2::RequestResponseCreateError::IsBeingCreatedByAnotherInstance:
        return iox2_request_response_open_or_create_error_e_C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE;
    case iox2::RequestResponseCreateError::InsufficientPermissions:
        return iox2_request_response_open_or_create_error_e_C_INSUFFICIENT_PERMISSIONS;
    case iox2::RequestResponseCreateError::HangsInCreation:
        return iox2_request_response_open_or_create_error_e_C_HANGS_IN_CREATION;
    case iox2::RequestResponseCreateError::ServiceInCorruptedState:
        return iox2_request_response_open_or_create_error_e_C_SERVICE_IN_CORRUPTED_STATE;
    default:
        IOX_UNREACHABLE();
    }
}

template <>
inline auto from<iox2::RequestResponseCreateError, const char*>(const iox2::RequestResponseCreateError value) noexcept
    -> const char* {
    return iox2_request_response_open_or_create_error_string(
        iox::into<iox2_request_response_open_or_create_error_e>(value));
}

template <>
constexpr auto from<int, iox2::RequestResponseOpenError>(const int value) noexcept -> iox2::RequestResponseOpenError {
    const auto error = static_cast<iox2_request_response_open_or_create_error_e>(value);
    switch (error) {
    case iox2_request_response_open_or_create_error_e_O_DOES_NOT_EXIST:
        return iox2::RequestResponseOpenError::DoesNotExist;
    case iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_CLIENT_REQUEST_LOANS:
        return iox2::RequestResponseOpenError::DoesNotSupportRequestedAmountOfClientRequestLoans;
    case iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_ACTIVE_REQUESTS_PER_CLIENT:
        return iox2::RequestResponseOpenError::DoesNotSupportRequestedAmountOfActiveRequestsPerClient;
    case iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_RESPONSE_BUFFER_SIZE:
        return iox2::RequestResponseOpenError::DoesNotSupportRequestedResponseBufferSize;
    case iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_SERVERS:
        return iox2::RequestResponseOpenError::DoesNotSupportRequestedAmountOfServers;
    case iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_CLIENTS:
        return iox2::RequestResponseOpenError::DoesNotSupportRequestedAmountOfClients;
    case iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES:
        return iox2::RequestResponseOpenError::DoesNotSupportRequestedAmountOfNodes;
    case iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_BORROWED_RESPONSES_PER_PENDING_RESPONSE:
        return iox2::RequestResponseOpenError::DoesNotSupportRequestedAmountOfBorrowedResponsesPerPendingResponse;
    case iox2_request_response_open_or_create_error_e_O_EXCEEDS_MAX_NUMBER_OF_NODES:
        return iox2::RequestResponseOpenError::ExceedsMaxNumberOfNodes;
    case iox2_request_response_open_or_create_error_e_O_HANGS_IN_CREATION:
        return iox2::RequestResponseOpenError::HangsInCreation;
    case iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_REQUEST_TYPE:
        return iox2::RequestResponseOpenError::IncompatibleRequestType;
    case iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_RESPONSE_TYPE:
        return iox2::RequestResponseOpenError::IncompatibleResponseType;
    case iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_ATTRIBUTES:
        return iox2::RequestResponseOpenError::IncompatibleAttributes;
    case iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_MESSAGING_PATTERN:
        return iox2::RequestResponseOpenError::IncompatibleMessagingPattern;
    case iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_OVERFLOW_BEHAVIOR_FOR_REQUESTS:
        return iox2::RequestResponseOpenError::IncompatibleOverflowBehaviorForRequests;
    case iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_OVERFLOW_BEHAVIOR_FOR_RESPONSES:
        return iox2::RequestResponseOpenError::IncompatibleOverflowBehaviorForResponses;
    case iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_BEHAVIOR_FOR_FIRE_AND_FORGET_REQUESTS:
        return iox2::RequestResponseOpenError::IncompatibleBehaviorForFireAndForgetRequests;
    case iox2_request_response_open_or_create_error_e_O_INSUFFICIENT_PERMISSIONS:
        return iox2::RequestResponseOpenError::InsufficientPermissions;
    case iox2_request_response_open_or_create_error_e_O_INTERNAL_FAILURE:
        return iox2::RequestResponseOpenError::InternalFailure;
    case iox2_request_response_open_or_create_error_e_O_IS_MARKED_FOR_DESTRUCTION:
        return iox2::RequestResponseOpenError::IsMarkedForDestruction;
    case iox2_request_response_open_or_create_error_e_O_SERVICE_IN_CORRUPTED_STATE:
        return iox2::RequestResponseOpenError::ServiceInCorruptedState;
    default:
        IOX_UNREACHABLE();
    }
}

template <>
constexpr auto from<iox2::RequestResponseOpenError, iox2_request_response_open_or_create_error_e>(
    const iox2::RequestResponseOpenError value) noexcept -> iox2_request_response_open_or_create_error_e {
    switch (value) {
    case iox2::RequestResponseOpenError::DoesNotExist:
        return iox2_request_response_open_or_create_error_e_O_DOES_NOT_EXIST;
    case iox2::RequestResponseOpenError::DoesNotSupportRequestedAmountOfClientRequestLoans:
        return iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_CLIENT_REQUEST_LOANS;
    case iox2::RequestResponseOpenError::DoesNotSupportRequestedAmountOfActiveRequestsPerClient:
        return iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_ACTIVE_REQUESTS_PER_CLIENT;
    case iox2::RequestResponseOpenError::DoesNotSupportRequestedResponseBufferSize:
        return iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_RESPONSE_BUFFER_SIZE;
    case iox2::RequestResponseOpenError::DoesNotSupportRequestedAmountOfServers:
        return iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_SERVERS;
    case iox2::RequestResponseOpenError::DoesNotSupportRequestedAmountOfClients:
        return iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_CLIENTS;
    case iox2::RequestResponseOpenError::DoesNotSupportRequestedAmountOfNodes:
        return iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES;
    case iox2::RequestResponseOpenError::DoesNotSupportRequestedAmountOfBorrowedResponsesPerPendingResponse:
        return iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_BORROWED_RESPONSES_PER_PENDING_RESPONSE;
    case iox2::RequestResponseOpenError::ExceedsMaxNumberOfNodes:
        return iox2_request_response_open_or_create_error_e_O_EXCEEDS_MAX_NUMBER_OF_NODES;
    case iox2::RequestResponseOpenError::HangsInCreation:
        return iox2_request_response_open_or_create_error_e_O_HANGS_IN_CREATION;
    case iox2::RequestResponseOpenError::IncompatibleRequestType:
        return iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_REQUEST_TYPE;
    case iox2::RequestResponseOpenError::IncompatibleResponseType:
        return iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_RESPONSE_TYPE;
    case iox2::RequestResponseOpenError::IncompatibleAttributes:
        return iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_ATTRIBUTES;
    case iox2::RequestResponseOpenError::IncompatibleMessagingPattern:
        return iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_MESSAGING_PATTERN;
    case iox2::RequestResponseOpenError::IncompatibleOverflowBehaviorForRequests:
        return iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_OVERFLOW_BEHAVIOR_FOR_REQUESTS;
    case iox2::RequestResponseOpenError::IncompatibleOverflowBehaviorForResponses:
        return iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_OVERFLOW_BEHAVIOR_FOR_RESPONSES;
    case iox2::RequestResponseOpenError::IncompatibleBehaviorForFireAndForgetRequests:
        return iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_BEHAVIOR_FOR_FIRE_AND_FORGET_REQUESTS;
    case iox2::RequestResponseOpenError::InsufficientPermissions:
        return iox2_request_response_open_or_create_error_e_O_INSUFFICIENT_PERMISSIONS;
    case iox2::RequestResponseOpenError::InternalFailure:
        return iox2_request_response_open_or_create_error_e_O_INTERNAL_FAILURE;
    case iox2::RequestResponseOpenError::IsMarkedForDestruction:
        return iox2_request_response_open_or_create_error_e_O_IS_MARKED_FOR_DESTRUCTION;
    case iox2::RequestResponseOpenError::ServiceInCorruptedState:
        return iox2_request_response_open_or_create_error_e_O_SERVICE_IN_CORRUPTED_STATE;
    default:
        IOX_UNREACHABLE();
    }
}

template <>
inline auto from<iox2::RequestResponseOpenError, const char*>(const iox2::RequestResponseOpenError value) noexcept
    -> const char* {
    return iox2_request_response_open_or_create_error_string(
        iox::into<iox2_request_response_open_or_create_error_e>(value));
}

template <>
constexpr auto from<int, iox2::RequestResponseOpenOrCreateError>(const int value) noexcept
    -> iox2::RequestResponseOpenOrCreateError {
    const auto error = static_cast<iox2_request_response_open_or_create_error_e>(value);
    switch (error) {
    case iox2_request_response_open_or_create_error_e_O_DOES_NOT_EXIST:
        return iox2::RequestResponseOpenOrCreateError::OpenDoesNotExist;
    case iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_CLIENT_REQUEST_LOANS:
        return iox2::RequestResponseOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfClientRequestLoans;
    case iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_ACTIVE_REQUESTS_PER_CLIENT:
        return iox2::RequestResponseOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfActiveRequestsPerClient;
    case iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_RESPONSE_BUFFER_SIZE:
        return iox2::RequestResponseOpenOrCreateError::OpenDoesNotSupportRequestedResponseBufferSize;
    case iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_SERVERS:
        return iox2::RequestResponseOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfServers;
    case iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_CLIENTS:
        return iox2::RequestResponseOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfClients;
    case iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES:
        return iox2::RequestResponseOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfNodes;
    case iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_BORROWED_RESPONSES_PER_PENDING_RESPONSE:
        return iox2::RequestResponseOpenOrCreateError::
            OpenDoesNotSupportRequestedAmountOfBorrowedResponsesPerPendingResponse;
    case iox2_request_response_open_or_create_error_e_O_EXCEEDS_MAX_NUMBER_OF_NODES:
        return iox2::RequestResponseOpenOrCreateError::OpenExceedsMaxNumberOfNodes;
    case iox2_request_response_open_or_create_error_e_O_HANGS_IN_CREATION:
        return iox2::RequestResponseOpenOrCreateError::OpenHangsInCreation;
    case iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_REQUEST_TYPE:
        return iox2::RequestResponseOpenOrCreateError::OpenIncompatibleRequestType;
    case iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_RESPONSE_TYPE:
        return iox2::RequestResponseOpenOrCreateError::OpenIncompatibleResponseType;
    case iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_ATTRIBUTES:
        return iox2::RequestResponseOpenOrCreateError::OpenIncompatibleAttributes;
    case iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_MESSAGING_PATTERN:
        return iox2::RequestResponseOpenOrCreateError::OpenIncompatibleMessagingPattern;
    case iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_OVERFLOW_BEHAVIOR_FOR_REQUESTS:
        return iox2::RequestResponseOpenOrCreateError::OpenIncompatibleOverflowBehaviorForRequests;
    case iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_OVERFLOW_BEHAVIOR_FOR_RESPONSES:
        return iox2::RequestResponseOpenOrCreateError::OpenIncompatibleOverflowBehaviorForResponses;
    case iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_BEHAVIOR_FOR_FIRE_AND_FORGET_REQUESTS:
        return iox2::RequestResponseOpenOrCreateError::OpenIncompatibleBehaviorForFireAndForgetRequests;
    case iox2_request_response_open_or_create_error_e_O_INSUFFICIENT_PERMISSIONS:
        return iox2::RequestResponseOpenOrCreateError::OpenInsufficientPermissions;
    case iox2_request_response_open_or_create_error_e_O_INTERNAL_FAILURE:
        return iox2::RequestResponseOpenOrCreateError::OpenInternalFailure;
    case iox2_request_response_open_or_create_error_e_O_IS_MARKED_FOR_DESTRUCTION:
        return iox2::RequestResponseOpenOrCreateError::OpenIsMarkedForDestruction;
    case iox2_request_response_open_or_create_error_e_O_SERVICE_IN_CORRUPTED_STATE:
        return iox2::RequestResponseOpenOrCreateError::OpenServiceInCorruptedState;

    case iox2_request_response_open_or_create_error_e_C_ALREADY_EXISTS:
        return iox2::RequestResponseOpenOrCreateError::CreateAlreadyExists;
    case iox2_request_response_open_or_create_error_e_C_INTERNAL_FAILURE:
        return iox2::RequestResponseOpenOrCreateError::CreateInternalFailure;
    case iox2_request_response_open_or_create_error_e_C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE:
        return iox2::RequestResponseOpenOrCreateError::CreateIsBeingCreatedByAnotherInstance;
    case iox2_request_response_open_or_create_error_e_C_INSUFFICIENT_PERMISSIONS:
        return iox2::RequestResponseOpenOrCreateError::CreateInsufficientPermissions;
    case iox2_request_response_open_or_create_error_e_C_HANGS_IN_CREATION:
        return iox2::RequestResponseOpenOrCreateError::CreateHangsInCreation;
    case iox2_request_response_open_or_create_error_e_C_SERVICE_IN_CORRUPTED_STATE:
        return iox2::RequestResponseOpenOrCreateError::CreateServiceInCorruptedState;
    case iox2_request_response_open_or_create_error_e_SYSTEM_IN_FLUX:
        return iox2::RequestResponseOpenOrCreateError::SystemInFlux;

    default:
        IOX_UNREACHABLE();
    }
}

template <>
constexpr auto from<iox2::RequestResponseOpenOrCreateError, iox2_request_response_open_or_create_error_e>(
    const iox2::RequestResponseOpenOrCreateError value) noexcept -> iox2_request_response_open_or_create_error_e {
    switch (value) {
    case iox2::RequestResponseOpenOrCreateError::OpenDoesNotExist:
        return iox2_request_response_open_or_create_error_e_O_DOES_NOT_EXIST;
    case iox2::RequestResponseOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfClientRequestLoans:
        return iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_CLIENT_REQUEST_LOANS;
    case iox2::RequestResponseOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfActiveRequestsPerClient:
        return iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_ACTIVE_REQUESTS_PER_CLIENT;
    case iox2::RequestResponseOpenOrCreateError::OpenDoesNotSupportRequestedResponseBufferSize:
        return iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_RESPONSE_BUFFER_SIZE;
    case iox2::RequestResponseOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfServers:
        return iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_SERVERS;
    case iox2::RequestResponseOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfClients:
        return iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_CLIENTS;
    case iox2::RequestResponseOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfNodes:
        return iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES;
    case iox2::RequestResponseOpenOrCreateError::OpenDoesNotSupportRequestedAmountOfBorrowedResponsesPerPendingResponse:
        return iox2_request_response_open_or_create_error_e_O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_BORROWED_RESPONSES_PER_PENDING_RESPONSE;
    case iox2::RequestResponseOpenOrCreateError::OpenExceedsMaxNumberOfNodes:
        return iox2_request_response_open_or_create_error_e_O_EXCEEDS_MAX_NUMBER_OF_NODES;
    case iox2::RequestResponseOpenOrCreateError::OpenHangsInCreation:
        return iox2_request_response_open_or_create_error_e_O_HANGS_IN_CREATION;
    case iox2::RequestResponseOpenOrCreateError::OpenIncompatibleRequestType:
        return iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_REQUEST_TYPE;
    case iox2::RequestResponseOpenOrCreateError::OpenIncompatibleResponseType:
        return iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_RESPONSE_TYPE;
    case iox2::RequestResponseOpenOrCreateError::OpenIncompatibleAttributes:
        return iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_ATTRIBUTES;
    case iox2::RequestResponseOpenOrCreateError::OpenIncompatibleMessagingPattern:
        return iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_MESSAGING_PATTERN;
    case iox2::RequestResponseOpenOrCreateError::OpenIncompatibleOverflowBehaviorForRequests:
        return iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_OVERFLOW_BEHAVIOR_FOR_REQUESTS;
    case iox2::RequestResponseOpenOrCreateError::OpenIncompatibleOverflowBehaviorForResponses:
        return iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_OVERFLOW_BEHAVIOR_FOR_RESPONSES;
    case iox2::RequestResponseOpenOrCreateError::OpenIncompatibleBehaviorForFireAndForgetRequests:
        return iox2_request_response_open_or_create_error_e_O_INCOMPATIBLE_BEHAVIOR_FOR_FIRE_AND_FORGET_REQUESTS;
    case iox2::RequestResponseOpenOrCreateError::OpenInsufficientPermissions:
        return iox2_request_response_open_or_create_error_e_O_INSUFFICIENT_PERMISSIONS;
    case iox2::RequestResponseOpenOrCreateError::OpenInternalFailure:
        return iox2_request_response_open_or_create_error_e_O_INTERNAL_FAILURE;
    case iox2::RequestResponseOpenOrCreateError::OpenIsMarkedForDestruction:
        return iox2_request_response_open_or_create_error_e_O_IS_MARKED_FOR_DESTRUCTION;
    case iox2::RequestResponseOpenOrCreateError::OpenServiceInCorruptedState:
        return iox2_request_response_open_or_create_error_e_O_SERVICE_IN_CORRUPTED_STATE;

    case iox2::RequestResponseOpenOrCreateError::CreateAlreadyExists:
        return iox2_request_response_open_or_create_error_e_C_ALREADY_EXISTS;
    case iox2::RequestResponseOpenOrCreateError::CreateInternalFailure:
        return iox2_request_response_open_or_create_error_e_C_INTERNAL_FAILURE;
    case iox2::RequestResponseOpenOrCreateError::CreateIsBeingCreatedByAnotherInstance:
        return iox2_request_response_open_or_create_error_e_C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE;
    case iox2::RequestResponseOpenOrCreateError::CreateInsufficientPermissions:
        return iox2_request_response_open_or_create_error_e_C_INSUFFICIENT_PERMISSIONS;
    case iox2::RequestResponseOpenOrCreateError::CreateHangsInCreation:
        return iox2_request_response_open_or_create_error_e_C_HANGS_IN_CREATION;
    case iox2::RequestResponseOpenOrCreateError::CreateServiceInCorruptedState:
        return iox2_request_response_open_or_create_error_e_C_SERVICE_IN_CORRUPTED_STATE;
    case iox2::RequestResponseOpenOrCreateError::SystemInFlux:
        return iox2_request_response_open_or_create_error_e_SYSTEM_IN_FLUX;
    default:
        IOX_UNREACHABLE();
    }
}

template <>
inline auto
from<iox2::RequestResponseOpenOrCreateError, const char*>(const iox2::RequestResponseOpenOrCreateError value) noexcept
    -> const char* {
    return iox2_request_response_open_or_create_error_string(
        iox::into<iox2_request_response_open_or_create_error_e>(value));
}

template <>
constexpr auto from<int, iox2::ClientCreateError>(const int value) noexcept -> iox2::ClientCreateError {
    const auto error = static_cast<iox2_client_create_error_e>(value);
    switch (error) {
    case iox2_client_create_error_e_EXCEEDS_MAX_SUPPORTED_CLIENTS:
        return iox2::ClientCreateError::ExceedsMaxSupportedClients;
    case iox2_client_create_error_e_UNABLE_TO_CREATE_DATA_SEGMENT:
        return iox2::ClientCreateError::UnableToCreateDataSegment;
    case iox2_client_create_error_e_FAILED_TO_DEPLOY_THREAD_SAFETY_POLICY:
        return iox2::ClientCreateError::FailedToDeployThreadsafetyPolicy;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<iox2::ClientCreateError, iox2_client_create_error_e>(const iox2::ClientCreateError value) noexcept
    -> iox2_client_create_error_e {
    switch (value) {
    case iox2::ClientCreateError::ExceedsMaxSupportedClients:
        return iox2_client_create_error_e_EXCEEDS_MAX_SUPPORTED_CLIENTS;
    case iox2::ClientCreateError::UnableToCreateDataSegment:
        return iox2_client_create_error_e_UNABLE_TO_CREATE_DATA_SEGMENT;
    case iox2::ClientCreateError::FailedToDeployThreadsafetyPolicy:
        return iox2_client_create_error_e_FAILED_TO_DEPLOY_THREAD_SAFETY_POLICY;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::ClientCreateError, const char*>(const iox2::ClientCreateError value) noexcept -> const char* {
    return iox2_client_create_error_string(iox::into<iox2_client_create_error_e>(value));
}

template <>
constexpr auto from<int, iox2::ServerCreateError>(const int value) noexcept -> iox2::ServerCreateError {
    const auto error = static_cast<iox2_server_create_error_e>(value);
    switch (error) {
    case iox2_server_create_error_e_EXCEEDS_MAX_SUPPORTED_SERVERS:
        return iox2::ServerCreateError::ExceedsMaxSupportedServers;
    case iox2_server_create_error_e_UNABLE_TO_CREATE_DATA_SEGMENT:
        return iox2::ServerCreateError::UnableToCreateDataSegment;
    case iox2_server_create_error_e_FAILED_TO_DEPLOY_THREAD_SAFETY_POLICY:
        return iox2::ServerCreateError::FailedToDeployThreadsafetyPolicy;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<iox2::ServerCreateError, iox2_server_create_error_e>(const iox2::ServerCreateError value) noexcept
    -> iox2_server_create_error_e {
    switch (value) {
    case iox2::ServerCreateError::ExceedsMaxSupportedServers:
        return iox2_server_create_error_e_EXCEEDS_MAX_SUPPORTED_SERVERS;
    case iox2::ServerCreateError::UnableToCreateDataSegment:
        return iox2_server_create_error_e_UNABLE_TO_CREATE_DATA_SEGMENT;
    case iox2::ServerCreateError::FailedToDeployThreadsafetyPolicy:
        return iox2_server_create_error_e_FAILED_TO_DEPLOY_THREAD_SAFETY_POLICY;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::ServerCreateError, const char*>(const iox2::ServerCreateError value) noexcept -> const char* {
    return iox2_server_create_error_string(iox::into<iox2_server_create_error_e>(value));
}

template <>
constexpr auto from<int, iox2::NotifierCreateError>(const int value) noexcept -> iox2::NotifierCreateError {
    const auto error = static_cast<iox2_notifier_create_error_e>(value);
    switch (error) {
    case iox2_notifier_create_error_e_EXCEEDS_MAX_SUPPORTED_NOTIFIERS:
        return iox2::NotifierCreateError::ExceedsMaxSupportedNotifiers;
    case iox2_notifier_create_error_e_FAILED_TO_DEPLOY_THREAD_SAFETY_POLICY:
        return iox2::NotifierCreateError::FailedToDeployThreadsafetyPolicy;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto
from<iox2::NotifierCreateError, iox2_notifier_create_error_e>(const iox2::NotifierCreateError value) noexcept
    -> iox2_notifier_create_error_e {
    switch (value) {
    case iox2::NotifierCreateError::ExceedsMaxSupportedNotifiers:
        return iox2_notifier_create_error_e_EXCEEDS_MAX_SUPPORTED_NOTIFIERS;
    case iox2::NotifierCreateError::FailedToDeployThreadsafetyPolicy:
        return iox2_notifier_create_error_e_FAILED_TO_DEPLOY_THREAD_SAFETY_POLICY;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::NotifierCreateError, const char*>(const iox2::NotifierCreateError value) noexcept -> const
    char* {
    return iox2_notifier_create_error_string(iox::into<iox2_notifier_create_error_e>(value));
}

template <>
constexpr auto from<int, iox2::ListenerCreateError>(const int value) noexcept -> iox2::ListenerCreateError {
    const auto error = static_cast<iox2_listener_create_error_e>(value);
    switch (error) {
    case iox2_listener_create_error_e_EXCEEDS_MAX_SUPPORTED_LISTENERS:
        return iox2::ListenerCreateError::ExceedsMaxSupportedListeners;
    case iox2_listener_create_error_e_RESOURCE_CREATION_FAILED:
        return iox2::ListenerCreateError::ResourceCreationFailed;
    case iox2_listener_create_error_e_FAILED_TO_DEPLOY_THREAD_SAFETY_POLICY:
        return iox2::ListenerCreateError::FailedToDeployThreadsafetyPolicy;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto
from<iox2::ListenerCreateError, iox2_listener_create_error_e>(const iox2::ListenerCreateError value) noexcept
    -> iox2_listener_create_error_e {
    switch (value) {
    case iox2::ListenerCreateError::ExceedsMaxSupportedListeners:
        return iox2_listener_create_error_e_EXCEEDS_MAX_SUPPORTED_LISTENERS;
    case iox2::ListenerCreateError::ResourceCreationFailed:
        return iox2_listener_create_error_e_RESOURCE_CREATION_FAILED;
    case iox2::ListenerCreateError::FailedToDeployThreadsafetyPolicy:
        return iox2_listener_create_error_e_FAILED_TO_DEPLOY_THREAD_SAFETY_POLICY;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::ListenerCreateError, const char*>(const iox2::ListenerCreateError value) noexcept -> const
    char* {
    return iox2_listener_create_error_string(iox::into<iox2_listener_create_error_e>(value));
}

template <>
constexpr auto from<int, iox2::NotifierNotifyError>(const int value) noexcept -> iox2::NotifierNotifyError {
    const auto error = static_cast<iox2_notifier_notify_error_e>(value);
    switch (error) {
    case iox2_notifier_notify_error_e_EVENT_ID_OUT_OF_BOUNDS:
        return iox2::NotifierNotifyError::EventIdOutOfBounds;
    case iox2_notifier_notify_error_e_MISSED_DEADLINE:
        return iox2::NotifierNotifyError::MissedDeadline;
    case iox2_notifier_notify_error_e_UNABLE_TO_ACQUIRE_ELAPSED_TIME:
        return iox2::NotifierNotifyError::UnableToAcquireElapsedTime;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto
from<iox2::NotifierNotifyError, iox2_notifier_notify_error_e>(const iox2::NotifierNotifyError value) noexcept
    -> iox2_notifier_notify_error_e {
    switch (value) {
    case iox2::NotifierNotifyError::EventIdOutOfBounds:
        return iox2_notifier_notify_error_e_EVENT_ID_OUT_OF_BOUNDS;
    case iox2::NotifierNotifyError::MissedDeadline:
        return iox2_notifier_notify_error_e_MISSED_DEADLINE;
    case iox2::NotifierNotifyError::UnableToAcquireElapsedTime:
        return iox2_notifier_notify_error_e_UNABLE_TO_ACQUIRE_ELAPSED_TIME;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::NotifierNotifyError, const char*>(const iox2::NotifierNotifyError value) noexcept -> const
    char* {
    return iox2_notifier_notify_error_string(iox::into<iox2_notifier_notify_error_e>(value));
}

template <>
constexpr auto from<int, iox2::ListenerWaitError>(const int value) noexcept -> iox2::ListenerWaitError {
    const auto error = static_cast<iox2_listener_wait_error_e>(value);
    switch (error) {
    case iox2_listener_wait_error_e_CONTRACT_VIOLATION:
        return iox2::ListenerWaitError::ContractViolation;
    case iox2_listener_wait_error_e_INTERRUPT_SIGNAL:
        return iox2::ListenerWaitError::InterruptSignal;
    case iox2_listener_wait_error_e_INTERNAL_FAILURE:
        return iox2::ListenerWaitError::InternalFailure;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<iox2::ListenerWaitError, iox2_listener_wait_error_e>(const iox2::ListenerWaitError value) noexcept
    -> iox2_listener_wait_error_e {
    switch (value) {
    case iox2::ListenerWaitError::ContractViolation:
        return iox2_listener_wait_error_e_CONTRACT_VIOLATION;
    case iox2::ListenerWaitError::InterruptSignal:
        return iox2_listener_wait_error_e_INTERRUPT_SIGNAL;
    case iox2::ListenerWaitError::InternalFailure:
        return iox2_listener_wait_error_e_INTERNAL_FAILURE;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::ListenerWaitError, const char*>(const iox2::ListenerWaitError value) noexcept -> const char* {
    return iox2_listener_wait_error_string(iox::into<iox2_listener_wait_error_e>(value));
}

template <>
constexpr auto from<int, iox2::PublisherCreateError>(const int value) noexcept -> iox2::PublisherCreateError {
    const auto error = static_cast<iox2_publisher_create_error_e>(value);
    switch (error) {
    case iox2_publisher_create_error_e_EXCEEDS_MAX_SUPPORTED_PUBLISHERS:
        return iox2::PublisherCreateError::ExceedsMaxSupportedPublishers;
    case iox2_publisher_create_error_e_UNABLE_TO_CREATE_DATA_SEGMENT:
        return iox2::PublisherCreateError::UnableToCreateDataSegment;
    case iox2_publisher_create_error_e_FAILED_TO_DEPLOY_THREAD_SAFETY_POLICY:
        return iox2::PublisherCreateError::FailedToDeployThreadsafetyPolicy;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto
from<iox2::PublisherCreateError, iox2_publisher_create_error_e>(const iox2::PublisherCreateError value) noexcept
    -> iox2_publisher_create_error_e {
    switch (value) {
    case iox2::PublisherCreateError::ExceedsMaxSupportedPublishers:
        return iox2_publisher_create_error_e_EXCEEDS_MAX_SUPPORTED_PUBLISHERS;
    case iox2::PublisherCreateError::UnableToCreateDataSegment:
        return iox2_publisher_create_error_e_UNABLE_TO_CREATE_DATA_SEGMENT;
    case iox2::PublisherCreateError::FailedToDeployThreadsafetyPolicy:
        return iox2_publisher_create_error_e_FAILED_TO_DEPLOY_THREAD_SAFETY_POLICY;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::PublisherCreateError, const char*>(const iox2::PublisherCreateError value) noexcept -> const
    char* {
    return iox2_publisher_create_error_string(iox::into<iox2_publisher_create_error_e>(value));
}

template <>
constexpr auto from<int, iox2::SubscriberCreateError>(const int value) noexcept -> iox2::SubscriberCreateError {
    const auto error = static_cast<iox2_subscriber_create_error_e>(value);
    switch (error) {
    case iox2_subscriber_create_error_e_BUFFER_SIZE_EXCEEDS_MAX_SUPPORTED_BUFFER_SIZE_OF_SERVICE:
        return iox2::SubscriberCreateError::BufferSizeExceedsMaxSupportedBufferSizeOfService;
    case iox2_subscriber_create_error_e_EXCEEDS_MAX_SUPPORTED_SUBSCRIBERS:
        return iox2::SubscriberCreateError::ExceedsMaxSupportedSubscribers;
    case iox2_subscriber_create_error_e_FAILED_TO_DEPLOY_THREAD_SAFETY_POLICY:
        return iox2::SubscriberCreateError::FailedToDeployThreadsafetyPolicy;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto
from<iox2::SubscriberCreateError, iox2_subscriber_create_error_e>(const iox2::SubscriberCreateError value) noexcept
    -> iox2_subscriber_create_error_e {
    switch (value) {
    case iox2::SubscriberCreateError::BufferSizeExceedsMaxSupportedBufferSizeOfService:
        return iox2_subscriber_create_error_e_BUFFER_SIZE_EXCEEDS_MAX_SUPPORTED_BUFFER_SIZE_OF_SERVICE;
    case iox2::SubscriberCreateError::ExceedsMaxSupportedSubscribers:
        return iox2_subscriber_create_error_e_EXCEEDS_MAX_SUPPORTED_SUBSCRIBERS;
    case iox2::SubscriberCreateError::FailedToDeployThreadsafetyPolicy:
        return iox2_subscriber_create_error_e_FAILED_TO_DEPLOY_THREAD_SAFETY_POLICY;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::SubscriberCreateError, const char*>(const iox2::SubscriberCreateError value) noexcept -> const
    char* {
    return iox2_subscriber_create_error_string(iox::into<iox2_subscriber_create_error_e>(value));
}

template <>
constexpr auto from<int, iox2::SendError>(const int value) noexcept -> iox2::SendError {
    const auto error = static_cast<iox2_send_error_e>(value);
    switch (error) {
    case iox2_send_error_e_CONNECTION_BROKEN_SINCE_SENDER_NO_LONGER_EXISTS:
        return iox2::SendError::ConnectionBrokenSinceSenderNoLongerExists;
    case iox2_send_error_e_CONNECTION_CORRUPTED:
        return iox2::SendError::ConnectionCorrupted;
    case iox2_send_error_e_LOAN_ERROR_OUT_OF_MEMORY:
        return iox2::SendError::LoanErrorOutOfMemory;
    case iox2_send_error_e_LOAN_ERROR_EXCEEDS_MAX_LOANS:
        return iox2::SendError::LoanErrorExceedsMaxLoans;
    case iox2_send_error_e_LOAN_ERROR_EXCEEDS_MAX_LOAN_SIZE:
        return iox2::SendError::LoanErrorExceedsMaxLoanSize;
    case iox2_send_error_e_LOAN_ERROR_INTERNAL_FAILURE:
        return iox2::SendError::LoanErrorInternalFailure;
    case iox2_send_error_e_CONNECTION_ERROR:
        return iox2::SendError::ConnectionError;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<iox2::SendError, iox2_send_error_e>(const iox2::SendError value) noexcept -> iox2_send_error_e {
    switch (value) {
    case iox2::SendError::ConnectionBrokenSinceSenderNoLongerExists:
        return iox2_send_error_e_CONNECTION_BROKEN_SINCE_SENDER_NO_LONGER_EXISTS;
    case iox2::SendError::ConnectionCorrupted:
        return iox2_send_error_e_CONNECTION_CORRUPTED;
    case iox2::SendError::LoanErrorOutOfMemory:
        return iox2_send_error_e_LOAN_ERROR_OUT_OF_MEMORY;
    case iox2::SendError::LoanErrorExceedsMaxLoans:
        return iox2_send_error_e_LOAN_ERROR_EXCEEDS_MAX_LOANS;
    case iox2::SendError::LoanErrorExceedsMaxLoanSize:
        return iox2_send_error_e_LOAN_ERROR_EXCEEDS_MAX_LOAN_SIZE;
    case iox2::SendError::LoanErrorInternalFailure:
        return iox2_send_error_e_LOAN_ERROR_INTERNAL_FAILURE;
    case iox2::SendError::ConnectionError:
        return iox2_send_error_e_CONNECTION_ERROR;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::SendError, const char*>(const iox2::SendError value) noexcept -> const char* {
    return iox2_send_error_string(iox::into<iox2_send_error_e>(value));
}

template <>
constexpr auto from<int, iox2::ReceiveError>(const int value) noexcept -> iox2::ReceiveError {
    const auto error = static_cast<iox2_receive_error_e>(value);
    switch (error) {
    case iox2_receive_error_e_FAILED_TO_ESTABLISH_CONNECTION:
        return iox2::ReceiveError::FailedToEstablishConnection;
    case iox2_receive_error_e_UNABLE_TO_MAP_SENDERS_DATA_SEGMENT:
        return iox2::ReceiveError::UnableToMapSendersDataSegment;
    case iox2_receive_error_e_EXCEEDS_MAX_BORROWS:
        return iox2::ReceiveError::ExceedsMaxBorrows;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<iox2::ReceiveError, iox2_receive_error_e>(const iox2::ReceiveError value) noexcept
    -> iox2_receive_error_e {
    switch (value) {
    case iox2::ReceiveError::FailedToEstablishConnection:
        return iox2_receive_error_e_FAILED_TO_ESTABLISH_CONNECTION;
    case iox2::ReceiveError::UnableToMapSendersDataSegment:
        return iox2_receive_error_e_UNABLE_TO_MAP_SENDERS_DATA_SEGMENT;
    case iox2::ReceiveError::ExceedsMaxBorrows:
        return iox2_receive_error_e_EXCEEDS_MAX_BORROWS;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::ReceiveError, const char*>(const iox2::ReceiveError value) noexcept -> const char* {
    return iox2_receive_error_string(iox::into<iox2_receive_error_e>(value));
}

template <>
constexpr auto from<int, iox2::LoanError>(const int value) noexcept -> iox2::LoanError {
    const auto error = static_cast<iox2_loan_error_e>(value);
    switch (error) {
    case iox2_loan_error_e_EXCEEDS_MAX_LOANED_SAMPLES:
        return iox2::LoanError::ExceedsMaxLoanedSamples;
    case iox2_loan_error_e_OUT_OF_MEMORY:
        return iox2::LoanError::OutOfMemory;
    case iox2_loan_error_e_EXCEEDS_MAX_LOAN_SIZE:
        return iox2::LoanError::ExceedsMaxLoanSize;
    case iox2_loan_error_e_INTERNAL_FAILURE:
        return iox2::LoanError::InternalFailure;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<iox2::LoanError, iox2_loan_error_e>(const iox2::LoanError value) noexcept -> iox2_loan_error_e {
    switch (value) {
    case iox2::LoanError::ExceedsMaxLoanedSamples:
        return iox2_loan_error_e_EXCEEDS_MAX_LOANED_SAMPLES;
    case iox2::LoanError::OutOfMemory:
        return iox2_loan_error_e_OUT_OF_MEMORY;
    case iox2::LoanError::ExceedsMaxLoanSize:
        return iox2_loan_error_e_EXCEEDS_MAX_LOAN_SIZE;
    case iox2::LoanError::InternalFailure:
        return iox2_loan_error_e_INTERNAL_FAILURE;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::LoanError, const char*>(const iox2::LoanError value) noexcept -> const char* {
    return iox2_loan_error_string(iox::into<iox2_loan_error_e>(value));
}

template <>
constexpr auto from<int, iox2::RequestSendError>(const int value) noexcept -> iox2::RequestSendError {
    const auto error = static_cast<iox2_request_send_error_e>(value);
    switch (error) {
    case iox2_request_send_error_e_EXCEEDS_MAX_ACTIVE_REQUESTS:
        return iox2::RequestSendError::ExceedsMaxActiveRequests;
    case iox2_request_send_error_e_CONNECTION_BROKEN_SINCE_SENDER_NO_LONGER_EXISTS:
        return iox2::RequestSendError::ConnectionBrokenSinceSenderNoLongerExists;
    case iox2_request_send_error_e_CONNECTION_CORRUPTED:
        return iox2::RequestSendError::ConnectionCorrupted;
    case iox2_request_send_error_e_LOAN_ERROR_OUT_OF_MEMORY:
        return iox2::RequestSendError::LoanErrorOutOfMemory;
    case iox2_request_send_error_e_LOAN_ERROR_EXCEEDS_MAX_LOANS:
        return iox2::RequestSendError::LoanErrorExceedsMaxLoans;
    case iox2_request_send_error_e_LOAN_ERROR_EXCEEDS_MAX_LOAN_SIZE:
        return iox2::RequestSendError::LoanErrorExceedsMaxLoanSize;
    case iox2_request_send_error_e_LOAN_ERROR_INTERNAL_FAILURE:
        return iox2::RequestSendError::LoanErrorInternalFailure;
    case iox2_request_send_error_e_CONNECTION_ERROR:
        return iox2::RequestSendError::ConnectionError;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<iox2::RequestSendError, iox2_request_send_error_e>(const iox2::RequestSendError value) noexcept
    -> iox2_request_send_error_e {
    switch (value) {
    case iox2::RequestSendError::ExceedsMaxActiveRequests:
        return iox2_request_send_error_e_EXCEEDS_MAX_ACTIVE_REQUESTS;
    case iox2::RequestSendError::ConnectionBrokenSinceSenderNoLongerExists:
        return iox2_request_send_error_e_CONNECTION_BROKEN_SINCE_SENDER_NO_LONGER_EXISTS;
    case iox2::RequestSendError::ConnectionCorrupted:
        return iox2_request_send_error_e_CONNECTION_CORRUPTED;
    case iox2::RequestSendError::LoanErrorOutOfMemory:
        return iox2_request_send_error_e_LOAN_ERROR_OUT_OF_MEMORY;
    case iox2::RequestSendError::LoanErrorExceedsMaxLoans:
        return iox2_request_send_error_e_LOAN_ERROR_EXCEEDS_MAX_LOANS;
    case iox2::RequestSendError::LoanErrorExceedsMaxLoanSize:
        return iox2_request_send_error_e_LOAN_ERROR_EXCEEDS_MAX_LOAN_SIZE;
    case iox2::RequestSendError::LoanErrorInternalFailure:
        return iox2_request_send_error_e_LOAN_ERROR_INTERNAL_FAILURE;
    case iox2::RequestSendError::ConnectionError:
        return iox2_request_send_error_e_CONNECTION_ERROR;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::RequestSendError, const char*>(const iox2::RequestSendError value) noexcept -> const char* {
    return iox2_request_send_error_string(iox::into<iox2_request_send_error_e>(value));
}

template <>
constexpr auto from<int, iox2::TypeVariant>(const int value) noexcept -> iox2::TypeVariant {
    const auto variant = static_cast<iox2_type_variant_e>(value);
    switch (variant) {
    case iox2_type_variant_e_DYNAMIC:
        return iox2::TypeVariant::Dynamic;
    case iox2_type_variant_e_FIXED_SIZE:
        return iox2::TypeVariant::FixedSize;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<int, iox2::ServiceListError>(const int value) noexcept -> iox2::ServiceListError {
    const auto variant = static_cast<iox2_service_list_error_e>(value);
    switch (variant) {
    case iox2_service_list_error_e_INSUFFICIENT_PERMISSIONS:
        return iox2::ServiceListError::InsufficientPermissions;
    case iox2_service_list_error_e_INTERNAL_ERROR:
        return iox2::ServiceListError::InternalError;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<iox2::ServiceListError, iox2_service_list_error_e>(const iox2::ServiceListError value) noexcept
    -> iox2_service_list_error_e {
    switch (value) {
    case iox2::ServiceListError::InsufficientPermissions:
        return iox2_service_list_error_e_INSUFFICIENT_PERMISSIONS;
    case iox2::ServiceListError::InternalError:
        return iox2_service_list_error_e_INTERNAL_ERROR;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::ServiceListError, const char*>(const iox2::ServiceListError value) noexcept -> const char* {
    return iox2_service_list_error_string(iox::into<iox2_service_list_error_e>(value));
}

template <>
constexpr auto from<int, iox2::MessagingPattern>(const int value) noexcept -> iox2::MessagingPattern {
    const auto variant = static_cast<iox2_messaging_pattern_e>(value);
    switch (variant) {
    case iox2_messaging_pattern_e_EVENT:
        return iox2::MessagingPattern::Event;
    case iox2_messaging_pattern_e_PUBLISH_SUBSCRIBE:
        return iox2::MessagingPattern::PublishSubscribe;
    case iox2_messaging_pattern_e_REQUEST_RESPONSE:
        return iox2::MessagingPattern::RequestResponse;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<int, iox2::UnableToDeliverStrategy>(const int value) noexcept -> iox2::UnableToDeliverStrategy {
    const auto variant = static_cast<iox2_unable_to_deliver_strategy_e>(value);
    switch (variant) {
    case iox2_unable_to_deliver_strategy_e_BLOCK:
        return iox2::UnableToDeliverStrategy::Block;
    case iox2_unable_to_deliver_strategy_e_DISCARD_SAMPLE:
        return iox2::UnableToDeliverStrategy::DiscardSample;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<iox2::UnableToDeliverStrategy, int>(const iox2::UnableToDeliverStrategy value) noexcept -> int {
    switch (value) {
    case iox2::UnableToDeliverStrategy::DiscardSample:
        return iox2_unable_to_deliver_strategy_e_DISCARD_SAMPLE;
    case iox2::UnableToDeliverStrategy::Block:
        return iox2_unable_to_deliver_strategy_e_BLOCK;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<int, iox2::ConnectionFailure>(const int value) noexcept -> iox2::ConnectionFailure {
    const auto variant = static_cast<iox2_connection_failure_e>(value);
    switch (variant) {
    case iox2_connection_failure_e_FAILED_TO_ESTABLISH_CONNECTION:
        return iox2::ConnectionFailure::FailedToEstablishConnection;
    case iox2_connection_failure_e_UNABLE_TO_MAP_SENDERS_DATA_SEGMENT:
        return iox2::ConnectionFailure::UnableToMapSendersDataSegment;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<iox2::ConnectionFailure, iox2_connection_failure_e>(const iox2::ConnectionFailure value) noexcept
    -> iox2_connection_failure_e {
    switch (value) {
    case iox2::ConnectionFailure::FailedToEstablishConnection:
        return iox2_connection_failure_e_FAILED_TO_ESTABLISH_CONNECTION;
    case iox2::ConnectionFailure::UnableToMapSendersDataSegment:
        return iox2_connection_failure_e_UNABLE_TO_MAP_SENDERS_DATA_SEGMENT;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::ConnectionFailure, const char*>(const iox2::ConnectionFailure value) noexcept -> const char* {
    return iox2_connection_failure_string(iox::into<iox2_connection_failure_e>(value));
}

template <>
constexpr auto from<int, iox2::ConfigCreationError>(const int value) noexcept -> iox2::ConfigCreationError {
    const auto variant = static_cast<iox2_config_creation_error_e>(value);
    switch (variant) {
    case iox2_config_creation_error_e_FAILED_TO_READ_CONFIG_FILE_CONTENTS:
        return iox2::ConfigCreationError::FailedToReadConfigFileContents;
    case iox2_config_creation_error_e_UNABLE_TO_DESERIALIZE_CONTENTS:
        return iox2::ConfigCreationError::UnableToDeserializeContents;
    case iox2_config_creation_error_e_INSUFFICIENT_PERMISSIONS:
        return iox2::ConfigCreationError::InsufficientPermissions;
    case iox2_config_creation_error_e_CONFIG_FILE_DOES_NOT_EXIST:
        return iox2::ConfigCreationError::ConfigFileDoesNotExist;
    case iox2_config_creation_error_e_UNABLE_TO_OPEN_CONFIG_FILE:
        return iox2::ConfigCreationError::UnableToOpenConfigFile;
    case iox2_config_creation_error_e_INVALID_FILE_PATH:
        // unreachable since this error case is excluded by using the strong type iox::FilePath
        IOX_UNREACHABLE();
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto
from<iox2::ConfigCreationError, iox2_config_creation_error_e>(const iox2::ConfigCreationError value) noexcept
    -> iox2_config_creation_error_e {
    switch (value) {
    case iox2::ConfigCreationError::FailedToReadConfigFileContents:
        return iox2_config_creation_error_e_FAILED_TO_READ_CONFIG_FILE_CONTENTS;
    case iox2::ConfigCreationError::UnableToDeserializeContents:
        return iox2_config_creation_error_e_UNABLE_TO_DESERIALIZE_CONTENTS;
    case iox2::ConfigCreationError::InsufficientPermissions:
        return iox2_config_creation_error_e_INSUFFICIENT_PERMISSIONS;
    case iox2::ConfigCreationError::ConfigFileDoesNotExist:
        return iox2_config_creation_error_e_CONFIG_FILE_DOES_NOT_EXIST;
    case iox2::ConfigCreationError::UnableToOpenConfigFile:
        return iox2_config_creation_error_e_UNABLE_TO_OPEN_CONFIG_FILE;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::ConfigCreationError, const char*>(const iox2::ConfigCreationError value) noexcept -> const
    char* {
    return iox2_config_creation_error_string(iox::into<iox2_config_creation_error_e>(value));
}

template <>
constexpr auto from<iox2::LogLevel, iox2_log_level_e>(iox2::LogLevel value) noexcept -> iox2_log_level_e {
    switch (value) {
    case iox2::LogLevel::Trace:
        return iox2_log_level_e_TRACE;
    case iox2::LogLevel::Debug:
        return iox2_log_level_e_DEBUG;
    case iox2::LogLevel::Info:
        return iox2_log_level_e_INFO;
    case iox2::LogLevel::Warn:
        return iox2_log_level_e_WARN;
    case iox2::LogLevel::Error:
        return iox2_log_level_e_ERROR;
    case iox2::LogLevel::Fatal:
        return iox2_log_level_e_FATAL;
    }
    IOX_UNREACHABLE();
}

template <>
constexpr auto from<int, iox2::LogLevel>(int value) noexcept -> iox2::LogLevel {
    const auto variant = static_cast<iox2_log_level_e>(value);
    switch (variant) {
    case iox2_log_level_e_TRACE:
        return iox2::LogLevel::Trace;
    case iox2_log_level_e_DEBUG:
        return iox2::LogLevel::Debug;
    case iox2_log_level_e_INFO:
        return iox2::LogLevel::Info;
    case iox2_log_level_e_WARN:
        return iox2::LogLevel::Warn;
    case iox2_log_level_e_ERROR:
        return iox2::LogLevel::Error;
    case iox2_log_level_e_FATAL:
        return iox2::LogLevel::Fatal;
    default:
        IOX_UNREACHABLE();
    }
}

template <>
constexpr auto from<int, iox2::WaitSetCreateError>(const int value) noexcept -> iox2::WaitSetCreateError {
    const auto variant = static_cast<iox2_waitset_create_error_e>(value);
    switch (variant) {
    case iox2_waitset_create_error_e_INTERNAL_ERROR:
        return iox2::WaitSetCreateError::InternalError;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto
from<iox2::WaitSetCreateError, iox2_waitset_create_error_e>(const iox2::WaitSetCreateError value) noexcept
    -> iox2_waitset_create_error_e {
    switch (value) {
    case iox2::WaitSetCreateError::InternalError:
        return iox2_waitset_create_error_e_INTERNAL_ERROR;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::WaitSetCreateError, const char*>(const iox2::WaitSetCreateError value) noexcept -> const char* {
    return iox2_waitset_create_error_string(iox::into<iox2_waitset_create_error_e>(value));
}

template <>
constexpr auto from<int, iox2::WaitSetRunResult>(const int value) noexcept -> iox2::WaitSetRunResult {
    const auto variant = static_cast<iox2_waitset_run_result_e>(value);
    switch (variant) {
    case iox2_waitset_run_result_e_INTERRUPT:
        return iox2::WaitSetRunResult::Interrupt;
    case iox2_waitset_run_result_e_TERMINATION_REQUEST:
        return iox2::WaitSetRunResult::TerminationRequest;
    case iox2_waitset_run_result_e_STOP_REQUEST:
        return iox2::WaitSetRunResult::StopRequest;
    case iox2_waitset_run_result_e_ALL_EVENTS_HANDLED:
        return iox2::WaitSetRunResult::AllEventsHandled;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<iox2::WaitSetRunResult, iox2_waitset_run_result_e>(const iox2::WaitSetRunResult value) noexcept
    -> iox2_waitset_run_result_e {
    switch (value) {
    case iox2::WaitSetRunResult::Interrupt:
        return iox2_waitset_run_result_e_INTERRUPT;
    case iox2::WaitSetRunResult::TerminationRequest:
        return iox2_waitset_run_result_e_TERMINATION_REQUEST;
    case iox2::WaitSetRunResult::StopRequest:
        return iox2_waitset_run_result_e_STOP_REQUEST;
    case iox2::WaitSetRunResult::AllEventsHandled:
        return iox2_waitset_run_result_e_ALL_EVENTS_HANDLED;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<int, iox2::WaitSetAttachmentError>(const int value) noexcept -> iox2::WaitSetAttachmentError {
    const auto variant = static_cast<iox2_waitset_attachment_error_e>(value);
    switch (variant) {
    case iox2_waitset_attachment_error_e_ALREADY_ATTACHED:
        return iox2::WaitSetAttachmentError::AlreadyAttached;
    case iox2_waitset_attachment_error_e_INSUFFICIENT_CAPACITY:
        return iox2::WaitSetAttachmentError::InsufficientCapacity;
    case iox2_waitset_attachment_error_e_INTERNAL_ERROR:
        return iox2::WaitSetAttachmentError::InternalError;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto
from<iox2::WaitSetAttachmentError, iox2_waitset_attachment_error_e>(const iox2::WaitSetAttachmentError value) noexcept
    -> iox2_waitset_attachment_error_e {
    switch (value) {
    case iox2::WaitSetAttachmentError::AlreadyAttached:
        return iox2_waitset_attachment_error_e_ALREADY_ATTACHED;
    case iox2::WaitSetAttachmentError::InsufficientCapacity:
        return iox2_waitset_attachment_error_e_INSUFFICIENT_CAPACITY;
    case iox2::WaitSetAttachmentError::InternalError:
        return iox2_waitset_attachment_error_e_INTERNAL_ERROR;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::WaitSetAttachmentError, const char*>(const iox2::WaitSetAttachmentError value) noexcept -> const
    char* {
    return iox2_waitset_attachment_error_string(iox::into<iox2_waitset_attachment_error_e>(value));
}

template <>
constexpr auto from<int, iox2::WaitSetRunError>(const int value) noexcept -> iox2::WaitSetRunError {
    const auto variant = static_cast<iox2_waitset_run_error_e>(value);
    switch (variant) {
    case iox2_waitset_run_error_e_INSUFFICIENT_PERMISSIONS:
        return iox2::WaitSetRunError::InsufficientPermissions;
    case iox2_waitset_run_error_e_INTERNAL_ERROR:
        return iox2::WaitSetRunError::InternalError;
    case iox2_waitset_run_error_e_NO_ATTACHMENTS:
        return iox2::WaitSetRunError::NoAttachments;
    case iox2_waitset_run_error_e_TERMINATION_REQUEST:
        return iox2::WaitSetRunError::TerminationRequest;
    case iox2_waitset_run_error_e_INTERRUPT:
        return iox2::WaitSetRunError::Interrupt;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<iox2::WaitSetRunError, iox2_waitset_run_error_e>(const iox2::WaitSetRunError value) noexcept
    -> iox2_waitset_run_error_e {
    switch (value) {
    case iox2::WaitSetRunError::InsufficientPermissions:
        return iox2_waitset_run_error_e_INSUFFICIENT_PERMISSIONS;
    case iox2::WaitSetRunError::InternalError:
        return iox2_waitset_run_error_e_INTERNAL_ERROR;
    case iox2::WaitSetRunError::NoAttachments:
        return iox2_waitset_run_error_e_NO_ATTACHMENTS;
    case iox2::WaitSetRunError::TerminationRequest:
        return iox2_waitset_run_error_e_TERMINATION_REQUEST;
    case iox2::WaitSetRunError::Interrupt:
        return iox2_waitset_run_error_e_INTERRUPT;
    }

    IOX_UNREACHABLE();
}

template <>
inline auto from<iox2::WaitSetRunError, const char*>(const iox2::WaitSetRunError value) noexcept -> const char* {
    return iox2_waitset_run_error_string(iox::into<iox2_waitset_run_error_e>(value));
}

template <>
constexpr auto
from<iox2::SignalHandlingMode, iox2_signal_handling_mode_e>(const iox2::SignalHandlingMode value) noexcept
    -> iox2_signal_handling_mode_e {
    switch (value) {
    case iox2::SignalHandlingMode::Disabled:
        return iox2_signal_handling_mode_e_DISABLED;
    case iox2::SignalHandlingMode::HandleTerminationRequests:
        return iox2_signal_handling_mode_e_HANDLE_TERMINATION_REQUESTS;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<int, iox2::SignalHandlingMode>(const int value) noexcept -> iox2::SignalHandlingMode {
    const auto variant = static_cast<iox2_signal_handling_mode_e>(value);

    switch (variant) {
    case iox2_signal_handling_mode_e_DISABLED:
        return iox2::SignalHandlingMode::Disabled;
    case iox2_signal_handling_mode_e_HANDLE_TERMINATION_REQUESTS:
        return iox2::SignalHandlingMode::HandleTerminationRequests;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<iox2::AllocationStrategy, iox2_allocation_strategy_e>(const iox2::AllocationStrategy value) noexcept
    -> iox2_allocation_strategy_e {
    switch (value) {
    case iox2::AllocationStrategy::BestFit:
        return iox2_allocation_strategy_e_BEST_FIT;
    case iox2::AllocationStrategy::PowerOfTwo:
        return iox2_allocation_strategy_e_POWER_OF_TWO;
    case iox2::AllocationStrategy::Static:
        return iox2_allocation_strategy_e_STATIC;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<int, iox2::NodeCleanupFailure>(const int value) noexcept -> iox2::NodeCleanupFailure {
    const auto variant = static_cast<iox2_node_cleanup_failure_e>(value);

    switch (variant) {
    case iox2_node_cleanup_failure_e_INTERRUPT:
        return iox2::NodeCleanupFailure::Interrupt;
    case iox2_node_cleanup_failure_e_INTERNAL_ERROR:
        return iox2::NodeCleanupFailure::InternalError;
    case iox2_node_cleanup_failure_e_INSUFFICIENT_PERMISSIONS:
        return iox2::NodeCleanupFailure::InsufficientPermissions;
    case iox2_node_cleanup_failure_e_VERSION_MISMATCH:
        return iox2::NodeCleanupFailure::VersionMismatch;
    }

    IOX_UNREACHABLE();
}
} // namespace iox

#endif
