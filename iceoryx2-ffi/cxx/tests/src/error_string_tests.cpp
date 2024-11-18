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

using iox2::error_string;

TEST(ErrorStringTest, config_creation_error_string) {
    using Sut = iox2::ConfigCreationError;
    ASSERT_GT(strlen(error_string(Sut::FailedToOpenConfigFile)), 1U);
    ASSERT_GT(strlen(error_string(Sut::UnableToDeserializeContents)), 1U);
    ASSERT_GT(strlen(error_string(Sut::FailedToReadConfigFileContents)), 1U);
}

TEST(ErrorStringTest, connection_failure_string) {
    using Sut = iox2::ConnectionFailure;
    ASSERT_GT(strlen(error_string(Sut::FailedToEstablishConnection)), 1U);
    ASSERT_GT(strlen(error_string(Sut::UnableToMapPublishersDataSegment)), 1U);
}

TEST(ErrorStringTest, listener_create_error_string) {
    using Sut = iox2::ListenerCreateError;
    ASSERT_GT(strlen(error_string(Sut::ExceedsMaxSupportedListeners)), 1U);
    ASSERT_GT(strlen(error_string(Sut::ResourceCreationFailed)), 1U);
}

TEST(ErrorStringTest, listener_wait_error_string) {
    using Sut = iox2::ListenerWaitError;
    ASSERT_GT(strlen(error_string(Sut::ContractViolation)), 1U);
    ASSERT_GT(strlen(error_string(Sut::InterruptSignal)), 1U);
    ASSERT_GT(strlen(error_string(Sut::InternalFailure)), 1U);
}

TEST(ErrorStringTest, node_list_failure_string) {
    using Sut = iox2::NodeListFailure;
    ASSERT_GT(strlen(error_string(Sut::InsufficientPermissions)), 1U);
    ASSERT_GT(strlen(error_string(Sut::InternalError)), 1U);
    ASSERT_GT(strlen(error_string(Sut::Interrupt)), 1U);
}

TEST(ErrorStringTest, node_creation_failure_string) {
    using Sut = iox2::NodeCreationFailure;
    ASSERT_GT(strlen(error_string(Sut::InsufficientPermissions)), 1U);
    ASSERT_GT(strlen(error_string(Sut::InternalError)), 1U);
}

TEST(ErrorStringTest, node_wait_failure_string) {
    using Sut = iox2::NodeWaitFailure;
    ASSERT_GT(strlen(error_string(Sut::TerminationRequest)), 1U);
    ASSERT_GT(strlen(error_string(Sut::Interrupt)), 1U);
}

TEST(ErrorStringTest, notifier_create_error_string) {
    using Sut = iox2::NotifierCreateError;
    ASSERT_GT(strlen(error_string(Sut::ExceedsMaxSupportedNotifiers)), 1U);
}

TEST(ErrorStringTest, notifier_notify_error_string) {
    using Sut = iox2::NotifierNotifyError;
    ASSERT_GT(strlen(error_string(Sut::EventIdOutOfBounds)), 1U);
}

TEST(ErrorStringTest, publisher_create_error_string) {
    using Sut = iox2::PublisherCreateError;
    ASSERT_GT(strlen(error_string(Sut::ExceedsMaxSupportedPublishers)), 1U);
    ASSERT_GT(strlen(error_string(Sut::UnableToCreateDataSegment)), 1U);
}

TEST(ErrorStringTest, publisher_loan_error_string) {
    using Sut = iox2::PublisherLoanError;
    ASSERT_GT(strlen(error_string(Sut::OutOfMemory)), 1U);
    ASSERT_GT(strlen(error_string(Sut::ExceedsMaxLoanedSamples)), 1U);
    ASSERT_GT(strlen(error_string(Sut::ExceedsMaxLoanSize)), 1U);
    ASSERT_GT(strlen(error_string(Sut::InternalFailure)), 1U);
}

TEST(ErrorStringTest, publisher_send_error_string) {
    using Sut = iox2::PublisherSendError;
    ASSERT_GT(strlen(error_string(Sut::ConnectionBrokenSincePublisherNoLongerExists)), 1U);
    ASSERT_GT(strlen(error_string(Sut::ConnectionCorrupted)), 1U);
    ASSERT_GT(strlen(error_string(Sut::LoanErrorOutOfMemory)), 1U);
    ASSERT_GT(strlen(error_string(Sut::LoanErrorExceedsMaxLoanedSamples)), 1U);
    ASSERT_GT(strlen(error_string(Sut::LoanErrorExceedsMaxLoanSize)), 1U);
    ASSERT_GT(strlen(error_string(Sut::LoanErrorInternalFailure)), 1U);
    ASSERT_GT(strlen(error_string(Sut::ConnectionError)), 1U);
}

TEST(ErrorStringTest, event_open_error_string) {
    using Sut = iox2::EventOpenError;
    ASSERT_GT(strlen(error_string(Sut::DoesNotExist)), 1U);
    ASSERT_GT(strlen(error_string(Sut::InsufficientPermissions)), 1U);
    ASSERT_GT(strlen(error_string(Sut::ServiceInCorruptedState)), 1U);
    ASSERT_GT(strlen(error_string(Sut::IncompatibleMessagingPattern)), 1U);
    ASSERT_GT(strlen(error_string(Sut::IncompatibleAttributes)), 1U);
    ASSERT_GT(strlen(error_string(Sut::InternalFailure)), 1U);
    ASSERT_GT(strlen(error_string(Sut::HangsInCreation)), 1U);
    ASSERT_GT(strlen(error_string(Sut::DoesNotSupportRequestedAmountOfNotifiers)), 1U);
    ASSERT_GT(strlen(error_string(Sut::DoesNotSupportRequestedAmountOfListeners)), 1U);
    ASSERT_GT(strlen(error_string(Sut::DoesNotSupportRequestedMaxEventId)), 1U);
    ASSERT_GT(strlen(error_string(Sut::DoesNotSupportRequestedAmountOfNodes)), 1U);
    ASSERT_GT(strlen(error_string(Sut::ExceedsMaxNumberOfNodes)), 1U);
    ASSERT_GT(strlen(error_string(Sut::IsMarkedForDestruction)), 1U);
}

TEST(ErrorStringTest, event_create_error_string) {
    using Sut = iox2::EventCreateError;
    ASSERT_GT(strlen(error_string(Sut::ServiceInCorruptedState)), 1U);
    ASSERT_GT(strlen(error_string(Sut::InternalFailure)), 1U);
    ASSERT_GT(strlen(error_string(Sut::IsBeingCreatedByAnotherInstance)), 1U);
    ASSERT_GT(strlen(error_string(Sut::AlreadyExists)), 1U);
    ASSERT_GT(strlen(error_string(Sut::HangsInCreation)), 1U);
    ASSERT_GT(strlen(error_string(Sut::InsufficientPermissions)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OldConnectionsStillActive)), 1U);
}

TEST(ErrorStringTest, event_open_or_create_error_string) {
    using Sut = iox2::EventOpenOrCreateError;
    ASSERT_GT(strlen(error_string(Sut::OpenDoesNotExist)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenInsufficientPermissions)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenServiceInCorruptedState)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenIncompatibleMessagingPattern)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenIncompatibleAttributes)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenInternalFailure)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenHangsInCreation)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenDoesNotSupportRequestedAmountOfNotifiers)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenDoesNotSupportRequestedAmountOfListeners)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenDoesNotSupportRequestedMaxEventId)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenDoesNotSupportRequestedAmountOfNodes)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenExceedsMaxNumberOfNodes)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenIsMarkedForDestruction)), 1U);
    ASSERT_GT(strlen(error_string(Sut::CreateServiceInCorruptedState)), 1U);
    ASSERT_GT(strlen(error_string(Sut::CreateInternalFailure)), 1U);
    ASSERT_GT(strlen(error_string(Sut::CreateIsBeingCreatedByAnotherInstance)), 1U);
    ASSERT_GT(strlen(error_string(Sut::CreateAlreadyExists)), 1U);
    ASSERT_GT(strlen(error_string(Sut::CreateHangsInCreation)), 1U);
    ASSERT_GT(strlen(error_string(Sut::CreateInsufficientPermissions)), 1U);
    ASSERT_GT(strlen(error_string(Sut::CreateOldConnectionsStillActive)), 1U);
}

TEST(ErrorStringTest, publish_subscribe_open_error_string) {
    using Sut = iox2::PublishSubscribeOpenError;
    ASSERT_GT(strlen(error_string(Sut::DoesNotExist)), 1U);
    ASSERT_GT(strlen(error_string(Sut::InternalFailure)), 1U);
    ASSERT_GT(strlen(error_string(Sut::IncompatibleTypes)), 1U);
    ASSERT_GT(strlen(error_string(Sut::IncompatibleMessagingPattern)), 1U);
    ASSERT_GT(strlen(error_string(Sut::IncompatibleAttributes)), 1U);
    ASSERT_GT(strlen(error_string(Sut::DoesNotSupportRequestedMinBufferSize)), 1U);
    ASSERT_GT(strlen(error_string(Sut::DoesNotSupportRequestedMinHistorySize)), 1U);
    ASSERT_GT(strlen(error_string(Sut::DoesNotSupportRequestedMinSubscriberBorrowedSamples)), 1U);
    ASSERT_GT(strlen(error_string(Sut::DoesNotSupportRequestedAmountOfPublishers)), 1U);
    ASSERT_GT(strlen(error_string(Sut::DoesNotSupportRequestedAmountOfSubscribers)), 1U);
    ASSERT_GT(strlen(error_string(Sut::DoesNotSupportRequestedAmountOfNodes)), 1U);
    ASSERT_GT(strlen(error_string(Sut::IncompatibleOverflowBehavior)), 1U);
    ASSERT_GT(strlen(error_string(Sut::InsufficientPermissions)), 1U);
    ASSERT_GT(strlen(error_string(Sut::ServiceInCorruptedState)), 1U);
    ASSERT_GT(strlen(error_string(Sut::HangsInCreation)), 1U);
    ASSERT_GT(strlen(error_string(Sut::ExceedsMaxNumberOfNodes)), 1U);
    ASSERT_GT(strlen(error_string(Sut::IsMarkedForDestruction)), 1U);
}

TEST(ErrorStringTest, publish_subscribe_create_error_string) {
    using Sut = iox2::PublishSubscribeCreateError;
    ASSERT_GT(strlen(error_string(Sut::ServiceInCorruptedState)), 1U);
    ASSERT_GT(strlen(error_string(Sut::SubscriberBufferMustBeLargerThanHistorySize)), 1U);
    ASSERT_GT(strlen(error_string(Sut::AlreadyExists)), 1U);
    ASSERT_GT(strlen(error_string(Sut::InsufficientPermissions)), 1U);
    ASSERT_GT(strlen(error_string(Sut::InternalFailure)), 1U);
    ASSERT_GT(strlen(error_string(Sut::IsBeingCreatedByAnotherInstance)), 1U);
    ASSERT_GT(strlen(error_string(Sut::HangsInCreation)), 1U);
}

TEST(ErrorStringTest, publish_subscribe_open_or_create_error_string) {
    using Sut = iox2::PublishSubscribeOpenOrCreateError;
    ASSERT_GT(strlen(error_string(Sut::OpenDoesNotExist)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenInternalFailure)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenIncompatibleTypes)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenIncompatibleMessagingPattern)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenIncompatibleAttributes)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenDoesNotSupportRequestedMinBufferSize)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenDoesNotSupportRequestedMinHistorySize)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenDoesNotSupportRequestedMinSubscriberBorrowedSamples)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenDoesNotSupportRequestedAmountOfPublishers)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenDoesNotSupportRequestedAmountOfSubscribers)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenDoesNotSupportRequestedAmountOfNodes)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenIncompatibleOverflowBehavior)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenInsufficientPermissions)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenServiceInCorruptedState)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenHangsInCreation)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenExceedsMaxNumberOfNodes)), 1U);
    ASSERT_GT(strlen(error_string(Sut::OpenIsMarkedForDestruction)), 1U);
    ASSERT_GT(strlen(error_string(Sut::CreateServiceInCorruptedState)), 1U);
    ASSERT_GT(strlen(error_string(Sut::CreateSubscriberBufferMustBeLargerThanHistorySize)), 1U);
    ASSERT_GT(strlen(error_string(Sut::CreateAlreadyExists)), 1U);
    ASSERT_GT(strlen(error_string(Sut::CreateInsufficientPermissions)), 1U);
    ASSERT_GT(strlen(error_string(Sut::CreateInternalFailure)), 1U);
    ASSERT_GT(strlen(error_string(Sut::CreateIsBeingCreatedByAnotherInstance)), 1U);
    ASSERT_GT(strlen(error_string(Sut::CreateOldConnectionsStillActive)), 1U);
    ASSERT_GT(strlen(error_string(Sut::CreateHangsInCreation)), 1U);
}

TEST(ErrorStringTest, service_details_error_string) {
    using Sut = iox2::ServiceDetailsError;
    ASSERT_GT(strlen(error_string(Sut::FailedToOpenStaticServiceInfo)), 1U);
    ASSERT_GT(strlen(error_string(Sut::FailedToReadStaticServiceInfo)), 1U);
    ASSERT_GT(strlen(error_string(Sut::FailedToDeserializeStaticServiceInfo)), 1U);
    ASSERT_GT(strlen(error_string(Sut::ServiceInInconsistentState)), 1U);
    ASSERT_GT(strlen(error_string(Sut::VersionMismatch)), 1U);
    ASSERT_GT(strlen(error_string(Sut::InternalError)), 1U);
    ASSERT_GT(strlen(error_string(Sut::FailedToAcquireNodeState)), 1U);
}

TEST(ErrorStringTest, service_list_error_string) {
    using Sut = iox2::ServiceListError;
    ASSERT_GT(strlen(error_string(Sut::InsufficientPermissions)), 1U);
    ASSERT_GT(strlen(error_string(Sut::InternalError)), 1U);
}

TEST(ErrorStringTest, subscriber_receive_error_string) {
    using Sut = iox2::SubscriberReceiveError;
    ASSERT_GT(strlen(error_string(Sut::ExceedsMaxBorrowedSamples)), 1U);
    ASSERT_GT(strlen(error_string(Sut::FailedToEstablishConnection)), 1U);
    ASSERT_GT(strlen(error_string(Sut::UnableToMapPublishersDataSegment)), 1U);
}

TEST(ErrorStringTest, subscriber_create_error_string) {
    using Sut = iox2::SubscriberCreateError;
    ASSERT_GT(strlen(error_string(Sut::ExceedsMaxSupportedSubscribers)), 1U);
    ASSERT_GT(strlen(error_string(Sut::BufferSizeExceedsMaxSupportedBufferSizeOfService)), 1U);
}

TEST(ErrorStringTest, waitset_create_error_string) {
    using Sut = iox2::WaitSetCreateError;
    ASSERT_GT(strlen(error_string(Sut::InternalError)), 1U);
}

TEST(ErrorStringTest, waitset_attachment_error_string) {
    using Sut = iox2::WaitSetAttachmentError;
    ASSERT_GT(strlen(error_string(Sut::InsufficientCapacity)), 1U);
    ASSERT_GT(strlen(error_string(Sut::AlreadyAttached)), 1U);
    ASSERT_GT(strlen(error_string(Sut::InternalError)), 1U);
}

TEST(ErrorStringTest, waitset_run_error_string) {
    using Sut = iox2::WaitSetRunError;
    ASSERT_GT(strlen(error_string(Sut::InsufficientPermissions)), 1U);
    ASSERT_GT(strlen(error_string(Sut::InternalError)), 1U);
    ASSERT_GT(strlen(error_string(Sut::NoAttachments)), 1U);
    ASSERT_GT(strlen(error_string(Sut::TerminationRequest)), 1U);
    ASSERT_GT(strlen(error_string(Sut::Interrupt)), 1U);
}

} // namespace
