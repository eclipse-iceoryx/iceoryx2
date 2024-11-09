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
#include "iox2/callback_progression.hpp"
#include "iox2/config_creation_error.hpp"
#include "iox2/connection_failure.hpp"
#include "iox2/iceoryx2.h"
#include "iox2/listener_error.hpp"
#include "iox2/log_level.hpp"
#include "iox2/messaging_pattern.hpp"
#include "iox2/node_failure_enums.hpp"
#include "iox2/node_wait_failure.hpp"
#include "iox2/notifier_error.hpp"
#include "iox2/publisher_error.hpp"
#include "iox2/semantic_string.hpp"
#include "iox2/service_builder_event_error.hpp"
#include "iox2/service_builder_publish_subscribe_error.hpp"
#include "iox2/service_error_enums.hpp"
#include "iox2/service_type.hpp"
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
constexpr auto
from<iox2::ServiceType, iox2_service_type_e>(const iox2::ServiceType value) noexcept -> iox2_service_type_e {
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
constexpr auto from<iox2::CallbackProgression, iox2_callback_progression_e>(
    const iox2::CallbackProgression value) noexcept -> iox2_callback_progression_e {
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
constexpr auto from<iox2::MessagingPattern, iox2_messaging_pattern_e>(const iox2::MessagingPattern value) noexcept
    -> iox2_messaging_pattern_e {
    switch (value) {
    case iox2::MessagingPattern::PublishSubscribe:
        return iox2_messaging_pattern_e_PUBLISH_SUBSCRIBE;
    case iox2::MessagingPattern::Event:
        return iox2_messaging_pattern_e_EVENT;
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
    }

    IOX_UNREACHABLE();
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
    case iox2_event_open_or_create_error_e_C_OLD_CONNECTION_STILL_ACTIVE:
        return iox2::EventCreateError::OldConnectionsStillActive;
    default:
        IOX_UNREACHABLE();
    }
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
    case iox2_pub_sub_open_or_create_error_e_C_OLD_CONNECTION_STILL_ACTIVE:
        return iox2::PublishSubscribeOpenOrCreateError::CreateOldConnectionsStillActive;
    case iox2_pub_sub_open_or_create_error_e_C_HANGS_IN_CREATION:
        return iox2::PublishSubscribeOpenOrCreateError::CreateHangsInCreation;
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
constexpr auto
from<int, iox2::PublishSubscribeCreateError>(const int value) noexcept -> iox2::PublishSubscribeCreateError {
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
    case iox2_pub_sub_open_or_create_error_e_C_OLD_CONNECTION_STILL_ACTIVE:
        return iox2::PublishSubscribeCreateError::OldConnectionsStillActive;
    case iox2_pub_sub_open_or_create_error_e_C_HANGS_IN_CREATION:
        return iox2::PublishSubscribeCreateError::HangsInCreation;
    default:
        IOX_UNREACHABLE();
    }
}

template <>
constexpr auto from<int, iox2::NotifierCreateError>(const int value) noexcept -> iox2::NotifierCreateError {
    const auto error = static_cast<iox2_notifier_create_error_e>(value);
    switch (error) {
    case iox2_notifier_create_error_e_EXCEEDS_MAX_SUPPORTED_NOTIFIERS:
        return iox2::NotifierCreateError::ExceedsMaxSupportedNotifiers;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<int, iox2::ListenerCreateError>(const int value) noexcept -> iox2::ListenerCreateError {
    const auto error = static_cast<iox2_listener_create_error_e>(value);
    switch (error) {
    case iox2_listener_create_error_e_EXCEEDS_MAX_SUPPORTED_LISTENERS:
        return iox2::ListenerCreateError::ExceedsMaxSupportedListeners;
    case iox2_listener_create_error_e_RESOURCE_CREATION_FAILED:
        return iox2::ListenerCreateError::ResourceCreationFailed;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<int, iox2::NotifierNotifyError>(const int value) noexcept -> iox2::NotifierNotifyError {
    const auto error = static_cast<iox2_notifier_notify_error_e>(value);
    switch (error) {
    case iox2_notifier_notify_error_e_EVENT_ID_OUT_OF_BOUNDS:
        return iox2::NotifierNotifyError::EventIdOutOfBounds;
    }

    IOX_UNREACHABLE();
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
constexpr auto from<int, iox2::PublisherCreateError>(const int value) noexcept -> iox2::PublisherCreateError {
    const auto error = static_cast<iox2_publisher_create_error_e>(value);
    switch (error) {
    case iox2_publisher_create_error_e_EXCEEDS_MAX_SUPPORTED_PUBLISHERS:
        return iox2::PublisherCreateError::ExceedsMaxSupportedPublishers;
    case iox2_publisher_create_error_e_UNABLE_TO_CREATE_DATA_SEGMENT:
        return iox2::PublisherCreateError::UnableToCreateDataSegment;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<int, iox2::SubscriberCreateError>(const int value) noexcept -> iox2::SubscriberCreateError {
    const auto error = static_cast<iox2_subscriber_create_error_e>(value);
    switch (error) {
    case iox2_subscriber_create_error_e_BUFFER_SIZE_EXCEEDS_MAX_SUPPORTED_BUFFER_SIZE_OF_SERVICE:
        return iox2::SubscriberCreateError::BufferSizeExceedsMaxSupportedBufferSizeOfService;
    case iox2_subscriber_create_error_e_EXCEEDS_MAX_SUPPORTED_SUBSCRIBERS:
        return iox2::SubscriberCreateError::ExceedsMaxSupportedSubscribers;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<int, iox2::PublisherSendError>(const int value) noexcept -> iox2::PublisherSendError {
    const auto error = static_cast<iox2_publisher_send_error_e>(value);
    switch (error) {
    case iox2_publisher_send_error_e_CONNECTION_BROKEN_SINCE_PUBLISHER_NO_LONGER_EXISTS:
        return iox2::PublisherSendError::ConnectionBrokenSincePublisherNoLongerExists;
    case iox2_publisher_send_error_e_CONNECTION_CORRUPTED:
        return iox2::PublisherSendError::ConnectionCorrupted;
    case iox2_publisher_send_error_e_LOAN_ERROR_OUT_OF_MEMORY:
        return iox2::PublisherSendError::LoanErrorOutOfMemory;
    case iox2_publisher_send_error_e_LOAN_ERROR_EXCEEDS_MAX_LOANED_SAMPLES:
        return iox2::PublisherSendError::LoanErrorExceedsMaxLoanedSamples;
    case iox2_publisher_send_error_e_LOAN_ERROR_EXCEEDS_MAX_LOAN_SIZE:
        return iox2::PublisherSendError::LoanErrorExceedsMaxLoanSize;
    case iox2_publisher_send_error_e_LOAN_ERROR_INTERNAL_FAILURE:
        return iox2::PublisherSendError::LoanErrorInternalFailure;
    case iox2_publisher_send_error_e_CONNECTION_ERROR:
        return iox2::PublisherSendError::ConnectionError;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<int, iox2::SubscriberReceiveError>(const int value) noexcept -> iox2::SubscriberReceiveError {
    const auto error = static_cast<iox2_subscriber_receive_error_e>(value);
    switch (error) {
    case iox2_subscriber_receive_error_e_FAILED_TO_ESTABLISH_CONNECTION:
        return iox2::SubscriberReceiveError::FailedToEstablishConnection;
    case iox2_subscriber_receive_error_e_UNABLE_TO_MAP_PUBLISHERS_DATA_SEGMENT:
        return iox2::SubscriberReceiveError::UnableToMapPublishersDataSegment;
    case iox2_subscriber_receive_error_e_EXCEEDS_MAX_BORROWED_SAMPLES:
        return iox2::SubscriberReceiveError::ExceedsMaxBorrowedSamples;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<int, iox2::PublisherLoanError>(const int value) noexcept -> iox2::PublisherLoanError {
    const auto error = static_cast<iox2_publisher_loan_error_e>(value);
    switch (error) {
    case iox2_publisher_loan_error_e_EXCEEDS_MAX_LOANED_SAMPLES:
        return iox2::PublisherLoanError::ExceedsMaxLoanedSamples;
    case iox2_publisher_loan_error_e_OUT_OF_MEMORY:
        return iox2::PublisherLoanError::OutOfMemory;
    case iox2_publisher_loan_error_e_EXCEEDS_MAX_LOAN_SIZE:
        return iox2::PublisherLoanError::ExceedsMaxLoanSize;
    case iox2_publisher_loan_error_e_INTERNAL_FAILURE:
        return iox2::PublisherLoanError::InternalFailure;
    }

    IOX_UNREACHABLE();
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
constexpr auto from<int, iox2::MessagingPattern>(const int value) noexcept -> iox2::MessagingPattern {
    const auto variant = static_cast<iox2_messaging_pattern_e>(value);
    switch (variant) {
    case iox2_messaging_pattern_e_EVENT:
        return iox2::MessagingPattern::Event;
    case iox2_messaging_pattern_e_PUBLISH_SUBSCRIBE:
        return iox2::MessagingPattern::PublishSubscribe;
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
    case iox2_connection_failure_e_UNABLE_TO_MAP_PUBLISHERS_DATA_SEGMENT:
        return iox2::ConnectionFailure::UnableToMapPublishersDataSegment;
    }

    IOX_UNREACHABLE();
}

template <>
constexpr auto from<int, iox2::ConfigCreationError>(const int value) noexcept -> iox2::ConfigCreationError {
    const auto variant = static_cast<iox2_config_creation_error_e>(value);
    switch (variant) {
    case iox2_config_creation_error_e_FAILED_TO_OPEN_CONFIG_FILE:
        return iox2::ConfigCreationError::FailedToOpenConfigFile;
    case iox2_config_creation_error_e_FAILED_TO_READ_CONFIG_FILE_CONTENTS:
        return iox2::ConfigCreationError::FailedToReadConfigFileContents;
    case iox2_config_creation_error_e_UNABLE_TO_DESERIALIZE_CONTENTS:
        return iox2::ConfigCreationError::UnableToDeserializeContents;
    case iox2_config_creation_error_e_INVALID_FILE_PATH:
        // unreachable since this error case is excluded by using the strong type iox::FilePath
        IOX_UNREACHABLE();
    }

    IOX_UNREACHABLE();
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

} // namespace iox

#endif
