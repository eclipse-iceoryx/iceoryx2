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

#include "test.hpp"

#include "iox2/config_creation_error.hpp"
#include "iox2/connection_failure.hpp"
#include "iox2/listener_error.hpp"
#include "iox2/node_failure_enums.hpp"
#include "iox2/node_wait_failure.hpp"
#include "iox2/notifier_error.hpp"
#include "iox2/publisher_error.hpp"
#include "iox2/service_builder_event_error.hpp"
#include "iox2/service_builder_publish_subscribe_error.hpp"
#include "iox2/service_error_enums.hpp"
#include "iox2/subscriber_error.hpp"
#include "iox2/waitset_enums.hpp"

namespace {

TEST(EnumConversionTest, config_creation_into_c_str) {
    using Sut = iox2::ConfigCreationError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::UnableToDeserializeContents)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::FailedToReadConfigFileContents)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InsufficientPermissions)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ConfigFileDoesNotExist)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::UnableToOpenConfigFile)), 1U);
}

TEST(EnumConversionTest, connection_failure_into_c_str) {
    using Sut = iox2::ConnectionFailure;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::FailedToEstablishConnection)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::UnableToMapSendersDataSegment)), 1U);
}

TEST(EnumConversionTest, listener_create_into_c_str) {
    using Sut = iox2::ListenerCreateError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ExceedsMaxSupportedListeners)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ResourceCreationFailed)), 1U);
}

TEST(EnumConversionTest, listener_wait_into_c_str) {
    using Sut = iox2::ListenerWaitError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ContractViolation)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InterruptSignal)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InternalFailure)), 1U);
}

TEST(EnumConversionTest, node_list_failure_into_c_str) {
    using Sut = iox2::NodeListFailure;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InsufficientPermissions)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InternalError)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::Interrupt)), 1U);
}

TEST(EnumConversionTest, node_creation_failure_into_c_str) {
    using Sut = iox2::NodeCreationFailure;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InsufficientPermissions)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InternalError)), 1U);
}

TEST(EnumConversionTest, node_wait_failure_into_c_str) {
    using Sut = iox2::NodeWaitFailure;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::TerminationRequest)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::Interrupt)), 1U);
}

TEST(EnumConversionTest, notifier_create_into_c_str) {
    using Sut = iox2::NotifierCreateError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ExceedsMaxSupportedNotifiers)), 1U);
}

TEST(EnumConversionTest, notifier_notify_into_c_str) {
    using Sut = iox2::NotifierNotifyError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::EventIdOutOfBounds)), 1U);
}

TEST(EnumConversionTest, publisher_create_into_c_str) {
    using Sut = iox2::PublisherCreateError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ExceedsMaxSupportedPublishers)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::UnableToCreateDataSegment)), 1U);
}

TEST(EnumConversionTest, publisher_loan_into_c_str) {
    using Sut = iox2::LoanError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OutOfMemory)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ExceedsMaxLoanedSamples)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ExceedsMaxLoanSize)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InternalFailure)), 1U);
}

TEST(EnumConversionTest, publisher_send_into_c_str) {
    using Sut = iox2::SendError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ConnectionBrokenSinceSenderNoLongerExists)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ConnectionCorrupted)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::LoanErrorOutOfMemory)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::LoanErrorExceedsMaxLoans)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::LoanErrorExceedsMaxLoanSize)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::LoanErrorInternalFailure)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ConnectionError)), 1U);
}

TEST(EnumConversionTest, event_open_into_c_str) {
    using Sut = iox2::EventOpenError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::DoesNotExist)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InsufficientPermissions)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ServiceInCorruptedState)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::IncompatibleMessagingPattern)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::IncompatibleAttributes)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InternalFailure)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::HangsInCreation)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::DoesNotSupportRequestedAmountOfNotifiers)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::DoesNotSupportRequestedAmountOfListeners)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::DoesNotSupportRequestedMaxEventId)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::DoesNotSupportRequestedAmountOfNodes)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ExceedsMaxNumberOfNodes)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::IsMarkedForDestruction)), 1U);
}

TEST(EnumConversionTest, event_create_into_c_str) {
    using Sut = iox2::EventCreateError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ServiceInCorruptedState)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InternalFailure)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::IsBeingCreatedByAnotherInstance)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::AlreadyExists)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::HangsInCreation)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InsufficientPermissions)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OldConnectionsStillActive)), 1U);
}

TEST(EnumConversionTest, event_open_or_create_into_c_str) {
    using Sut = iox2::EventOpenOrCreateError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenDoesNotExist)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenInsufficientPermissions)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenServiceInCorruptedState)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenIncompatibleMessagingPattern)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenIncompatibleAttributes)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenInternalFailure)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenHangsInCreation)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenDoesNotSupportRequestedAmountOfNotifiers)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenDoesNotSupportRequestedAmountOfListeners)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenDoesNotSupportRequestedMaxEventId)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenDoesNotSupportRequestedAmountOfNodes)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenExceedsMaxNumberOfNodes)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenIsMarkedForDestruction)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::CreateServiceInCorruptedState)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::CreateInternalFailure)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::CreateIsBeingCreatedByAnotherInstance)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::CreateAlreadyExists)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::CreateHangsInCreation)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::CreateInsufficientPermissions)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::CreateOldConnectionsStillActive)), 1U);
}

TEST(EnumConversionTest, publish_subscribe_open_into_c_str) {
    using Sut = iox2::PublishSubscribeOpenError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::DoesNotExist)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InternalFailure)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::IncompatibleTypes)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::IncompatibleMessagingPattern)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::IncompatibleAttributes)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::DoesNotSupportRequestedMinBufferSize)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::DoesNotSupportRequestedMinHistorySize)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::DoesNotSupportRequestedMinSubscriberBorrowedSamples)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::DoesNotSupportRequestedAmountOfPublishers)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::DoesNotSupportRequestedAmountOfSubscribers)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::DoesNotSupportRequestedAmountOfNodes)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::IncompatibleOverflowBehavior)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InsufficientPermissions)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ServiceInCorruptedState)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::HangsInCreation)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ExceedsMaxNumberOfNodes)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::IsMarkedForDestruction)), 1U);
}

TEST(EnumConversionTest, publish_subscribe_create_into_c_str) {
    using Sut = iox2::PublishSubscribeCreateError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ServiceInCorruptedState)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::SubscriberBufferMustBeLargerThanHistorySize)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::AlreadyExists)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InsufficientPermissions)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InternalFailure)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::IsBeingCreatedByAnotherInstance)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::HangsInCreation)), 1U);
}

TEST(EnumConversionTest, publish_subscribe_open_or_create_into_c_str) {
    using Sut = iox2::PublishSubscribeOpenOrCreateError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenDoesNotExist)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenInternalFailure)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenIncompatibleTypes)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenIncompatibleMessagingPattern)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenIncompatibleAttributes)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenDoesNotSupportRequestedMinBufferSize)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenDoesNotSupportRequestedMinHistorySize)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenDoesNotSupportRequestedMinSubscriberBorrowedSamples)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenDoesNotSupportRequestedAmountOfPublishers)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenDoesNotSupportRequestedAmountOfSubscribers)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenDoesNotSupportRequestedAmountOfNodes)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenIncompatibleOverflowBehavior)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenInsufficientPermissions)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenServiceInCorruptedState)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenHangsInCreation)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenExceedsMaxNumberOfNodes)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::OpenIsMarkedForDestruction)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::CreateServiceInCorruptedState)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::CreateSubscriberBufferMustBeLargerThanHistorySize)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::CreateAlreadyExists)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::CreateInsufficientPermissions)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::CreateInternalFailure)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::CreateIsBeingCreatedByAnotherInstance)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::CreateOldConnectionsStillActive)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::CreateHangsInCreation)), 1U);
}

TEST(EnumConversionTest, service_details_into_c_str) {
    using Sut = iox2::ServiceDetailsError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::FailedToOpenStaticServiceInfo)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::FailedToReadStaticServiceInfo)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::FailedToDeserializeStaticServiceInfo)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ServiceInInconsistentState)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::VersionMismatch)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InternalError)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::FailedToAcquireNodeState)), 1U);
}

TEST(EnumConversionTest, service_list_into_c_str) {
    using Sut = iox2::ServiceListError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InsufficientPermissions)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InternalError)), 1U);
}

TEST(EnumConversionTest, subscriber_receive_into_c_str) {
    using Sut = iox2::ReceiveError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ExceedsMaxBorrows)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::FailedToEstablishConnection)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::UnableToMapSendersDataSegment)), 1U);
}

TEST(EnumConversionTest, subscriber_create_into_c_str) {
    using Sut = iox2::SubscriberCreateError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ExceedsMaxSupportedSubscribers)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::BufferSizeExceedsMaxSupportedBufferSizeOfService)), 1U);
}

TEST(EnumConversionTest, waitset_create_into_c_str) {
    using Sut = iox2::WaitSetCreateError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InternalError)), 1U);
}

TEST(EnumConversionTest, waitset_attachment_into_c_str) {
    using Sut = iox2::WaitSetAttachmentError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InsufficientCapacity)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::AlreadyAttached)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InternalError)), 1U);
}

TEST(EnumConversionTest, waitset_run_into_c_str) {
    using Sut = iox2::WaitSetRunError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InsufficientPermissions)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InternalError)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::NoAttachments)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::TerminationRequest)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::Interrupt)), 1U);
}

TEST(EnumConversionTest, semantic_string_into_c_str) {
    using Sut = iox2::SemanticStringError;
    ASSERT_GT(strlen(iox::into<const char*>(Sut::InvalidContent)), 1U);
    ASSERT_GT(strlen(iox::into<const char*>(Sut::ExceedsMaximumLength)), 1U);
}

} // namespace
