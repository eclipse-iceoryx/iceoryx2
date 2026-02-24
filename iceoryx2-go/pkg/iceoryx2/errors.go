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

package iceoryx2

/*
#include "iox2/iceoryx2.h"
*/
import "C"
import (
	"errors"
	"fmt"
)

// ContextualError wraps an error with additional context about the operation.
// It implements the Unwrap() method for use with errors.Is() and errors.As().
type ContextualError struct {
	Op  string // The operation that failed (e.g., "Subscriber.Receive")
	Err error  // The underlying error
}

// Error returns the error message with context.
func (e *ContextualError) Error() string {
	if e.Op != "" {
		return fmt.Sprintf("%s: %v", e.Op, e.Err)
	}
	return e.Err.Error()
}

// Unwrap returns the underlying error for use with errors.Is() and errors.As().
func (e *ContextualError) Unwrap() error {
	return e.Err
}

// WrapError wraps an error with operation context.
// Returns nil if err is nil.
func WrapError(op string, err error) error {
	if err == nil {
		return nil
	}
	return &ContextualError{Op: op, Err: err}
}

// Sentinel errors for common conditions.
// Use errors.Is() to check for these errors.
var (
	// ErrNodeClosed indicates the node has already been closed.
	ErrNodeClosed = errors.New("iceoryx2: node is closed")

	// ErrNodeBuilderConsumed indicates the node builder has already been used.
	ErrNodeBuilderConsumed = errors.New("iceoryx2: node builder already consumed")

	// ErrPublisherClosed indicates the publisher has already been closed.
	ErrPublisherClosed = errors.New("iceoryx2: publisher is closed")

	// ErrSubscriberClosed indicates the subscriber has already been closed.
	ErrSubscriberClosed = errors.New("iceoryx2: subscriber is closed")

	// ErrSampleClosed indicates the sample has already been closed.
	ErrSampleClosed = errors.New("iceoryx2: sample is closed")

	// ErrServiceClosed indicates the service has already been closed.
	ErrServiceClosed = errors.New("iceoryx2: service is closed")

	// ErrListenerClosed indicates the listener has already been closed.
	ErrListenerClosed = errors.New("iceoryx2: listener is closed")

	// ErrNotifierClosed indicates the notifier has already been closed.
	ErrNotifierClosed = errors.New("iceoryx2: notifier is closed")

	// ErrClientClosed indicates the client has already been closed.
	ErrClientClosed = errors.New("iceoryx2: client is closed")

	// ErrServerClosed indicates the server has already been closed.
	ErrServerClosed = errors.New("iceoryx2: server is closed")

	// ErrWaitSetClosed indicates the waitset has already been closed.
	ErrWaitSetClosed = errors.New("iceoryx2: waitset is closed")

	// ErrBuilderConsumed indicates a builder has already been consumed.
	ErrBuilderConsumed = errors.New("iceoryx2: builder already consumed")

	// ErrNilHandle indicates an unexpected nil handle.
	ErrNilHandle = errors.New("iceoryx2: nil handle")

	// ErrNoData indicates no data is available (e.g., no sample, no event).
	// This is not an error condition but indicates the absence of data.
	ErrNoData = errors.New("iceoryx2: no data available")
)

// NodeCreationError represents errors that can occur when creating a node.
type NodeCreationError int

const (
	NodeCreationErrorInsufficientPermissions NodeCreationError = C.iox2_node_creation_failure_e_INSUFFICIENT_PERMISSIONS
	NodeCreationErrorInternalError           NodeCreationError = C.iox2_node_creation_failure_e_INTERNAL_ERROR
)

func (e NodeCreationError) Error() string {
	switch e {
	case NodeCreationErrorInsufficientPermissions:
		return "node creation failed: insufficient permissions"
	case NodeCreationErrorInternalError:
		return "node creation failed: internal error"
	default:
		return fmt.Sprintf("node creation failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for NodeCreationError.
func (e NodeCreationError) Is(target error) bool {
	if t, ok := target.(NodeCreationError); ok {
		return e == t
	}
	return false
}

// NodeWaitError represents errors that can occur when waiting on a node.
type NodeWaitError int

const (
	NodeWaitErrorInterrupt          NodeWaitError = C.iox2_node_wait_failure_e_INTERRUPT
	NodeWaitErrorTerminationRequest NodeWaitError = C.iox2_node_wait_failure_e_TERMINATION_REQUEST
)

func (e NodeWaitError) Error() string {
	switch e {
	case NodeWaitErrorInterrupt:
		return "node wait failed: interrupted"
	case NodeWaitErrorTerminationRequest:
		return "node wait failed: termination requested"
	default:
		return fmt.Sprintf("node wait failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for NodeWaitError.
func (e NodeWaitError) Is(target error) bool {
	if t, ok := target.(NodeWaitError); ok {
		return e == t
	}
	return false
}

// SemanticStringError represents errors related to semantic string validation.
type SemanticStringError int

const (
	SemanticStringErrorInvalidContent       SemanticStringError = C.iox2_semantic_string_error_e_INVALID_CONTENT
	SemanticStringErrorExceedsMaximumLength SemanticStringError = C.iox2_semantic_string_error_e_EXCEEDS_MAXIMUM_LENGTH
)

func (e SemanticStringError) Error() string {
	switch e {
	case SemanticStringErrorInvalidContent:
		return "semantic string error: invalid content"
	case SemanticStringErrorExceedsMaximumLength:
		return "semantic string error: exceeds maximum length"
	default:
		return fmt.Sprintf("semantic string error: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for SemanticStringError.
func (e SemanticStringError) Is(target error) bool {
	if t, ok := target.(SemanticStringError); ok {
		return e == t
	}
	return false
}

// PublisherCreateError represents errors that can occur when creating a publisher.
type PublisherCreateError int

const (
	PublisherCreateErrorExceedsMaxSupportedPublishers PublisherCreateError = C.iox2_publisher_create_error_e_EXCEEDS_MAX_SUPPORTED_PUBLISHERS
	PublisherCreateErrorUnableToCreateDataSegment     PublisherCreateError = C.iox2_publisher_create_error_e_UNABLE_TO_CREATE_DATA_SEGMENT
)

func (e PublisherCreateError) Error() string {
	switch e {
	case PublisherCreateErrorExceedsMaxSupportedPublishers:
		return "publisher creation failed: exceeds max supported publishers"
	case PublisherCreateErrorUnableToCreateDataSegment:
		return "publisher creation failed: unable to create data segment"
	default:
		return fmt.Sprintf("publisher creation failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for PublisherCreateError.
func (e PublisherCreateError) Is(target error) bool {
	if t, ok := target.(PublisherCreateError); ok {
		return e == t
	}
	return false
}

// SubscriberCreateError represents errors that can occur when creating a subscriber.
type SubscriberCreateError int

const (
	SubscriberCreateErrorExceedsMaxSupportedSubscribers          SubscriberCreateError = C.iox2_subscriber_create_error_e_EXCEEDS_MAX_SUPPORTED_SUBSCRIBERS
	SubscriberCreateErrorBufferSizeExceedsMaxSupportedBufferSize SubscriberCreateError = C.iox2_subscriber_create_error_e_BUFFER_SIZE_EXCEEDS_MAX_SUPPORTED_BUFFER_SIZE_OF_SERVICE
)

func (e SubscriberCreateError) Error() string {
	switch e {
	case SubscriberCreateErrorExceedsMaxSupportedSubscribers:
		return "subscriber creation failed: exceeds max supported subscribers"
	case SubscriberCreateErrorBufferSizeExceedsMaxSupportedBufferSize:
		return "subscriber creation failed: buffer size exceeds max supported buffer size"
	default:
		return fmt.Sprintf("subscriber creation failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for SubscriberCreateError.
func (e SubscriberCreateError) Is(target error) bool {
	if t, ok := target.(SubscriberCreateError); ok {
		return e == t
	}
	return false
}

// LoanError represents errors that can occur when loaning a sample.
type LoanError int

const (
	LoanErrorOutOfMemory             LoanError = C.iox2_loan_error_e_OUT_OF_MEMORY
	LoanErrorExceedsMaxLoanedSamples LoanError = C.iox2_loan_error_e_EXCEEDS_MAX_LOANED_SAMPLES
	LoanErrorExceedsMaxLoanSize      LoanError = C.iox2_loan_error_e_EXCEEDS_MAX_LOAN_SIZE
	LoanErrorInternalFailure         LoanError = C.iox2_loan_error_e_INTERNAL_FAILURE
)

func (e LoanError) Error() string {
	switch e {
	case LoanErrorOutOfMemory:
		return "loan failed: out of memory"
	case LoanErrorExceedsMaxLoanedSamples:
		return "loan failed: exceeds max loaned samples"
	case LoanErrorExceedsMaxLoanSize:
		return "loan failed: exceeds max loan size"
	case LoanErrorInternalFailure:
		return "loan failed: internal failure"
	default:
		return fmt.Sprintf("loan failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for LoanError.
func (e LoanError) Is(target error) bool {
	if t, ok := target.(LoanError); ok {
		return e == t
	}
	return false
}

// SendError represents errors that can occur when sending a sample.
type SendError int

const (
	SendErrorConnectionBroken    SendError = C.iox2_send_error_e_CONNECTION_BROKEN_SINCE_SENDER_NO_LONGER_EXISTS
	SendErrorConnectionCorrupted SendError = C.iox2_send_error_e_CONNECTION_CORRUPTED
	SendErrorLoanOutOfMemory     SendError = C.iox2_send_error_e_LOAN_ERROR_OUT_OF_MEMORY
	SendErrorLoanExceedsMaxLoans SendError = C.iox2_send_error_e_LOAN_ERROR_EXCEEDS_MAX_LOANS
	SendErrorConnectionError     SendError = C.iox2_send_error_e_CONNECTION_ERROR
)

func (e SendError) Error() string {
	switch e {
	case SendErrorConnectionBroken:
		return "send failed: connection broken"
	case SendErrorConnectionCorrupted:
		return "send failed: connection corrupted"
	case SendErrorLoanOutOfMemory:
		return "send failed: loan out of memory"
	case SendErrorLoanExceedsMaxLoans:
		return "send failed: loan exceeds max loans"
	case SendErrorConnectionError:
		return "send failed: connection error"
	default:
		return fmt.Sprintf("send failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for SendError.
func (e SendError) Is(target error) bool {
	if t, ok := target.(SendError); ok {
		return e == t
	}
	return false
}

// ReceiveError represents errors that can occur when receiving a sample.
type ReceiveError int

const (
	ReceiveErrorExceedsMaxBorrows             ReceiveError = C.iox2_receive_error_e_EXCEEDS_MAX_BORROWS
	ReceiveErrorFailedToEstablishConnection   ReceiveError = C.iox2_receive_error_e_FAILED_TO_ESTABLISH_CONNECTION
	ReceiveErrorUnableToMapSendersDataSegment ReceiveError = C.iox2_receive_error_e_UNABLE_TO_MAP_SENDERS_DATA_SEGMENT
)

func (e ReceiveError) Error() string {
	switch e {
	case ReceiveErrorExceedsMaxBorrows:
		return "receive failed: exceeds max borrows"
	case ReceiveErrorFailedToEstablishConnection:
		return "receive failed: failed to establish connection"
	case ReceiveErrorUnableToMapSendersDataSegment:
		return "receive failed: unable to map sender's data segment"
	default:
		return fmt.Sprintf("receive failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for ReceiveError.
func (e ReceiveError) Is(target error) bool {
	if t, ok := target.(ReceiveError); ok {
		return e == t
	}
	return false
}

// NotifierCreateError represents errors that can occur when creating a notifier.
type NotifierCreateError int

const (
	NotifierCreateErrorExceedsMaxSupportedNotifiers NotifierCreateError = C.iox2_notifier_create_error_e_EXCEEDS_MAX_SUPPORTED_NOTIFIERS
)

func (e NotifierCreateError) Error() string {
	switch e {
	case NotifierCreateErrorExceedsMaxSupportedNotifiers:
		return "notifier creation failed: exceeds max supported notifiers"
	default:
		return fmt.Sprintf("notifier creation failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for NotifierCreateError.
func (e NotifierCreateError) Is(target error) bool {
	if t, ok := target.(NotifierCreateError); ok {
		return e == t
	}
	return false
}

// ListenerCreateError represents errors that can occur when creating a listener.
type ListenerCreateError int

const (
	ListenerCreateErrorExceedsMaxSupportedListeners ListenerCreateError = C.iox2_listener_create_error_e_EXCEEDS_MAX_SUPPORTED_LISTENERS
	ListenerCreateErrorResourceCreationFailed       ListenerCreateError = C.iox2_listener_create_error_e_RESOURCE_CREATION_FAILED
)

func (e ListenerCreateError) Error() string {
	switch e {
	case ListenerCreateErrorExceedsMaxSupportedListeners:
		return "listener creation failed: exceeds max supported listeners"
	case ListenerCreateErrorResourceCreationFailed:
		return "listener creation failed: resource creation failed"
	default:
		return fmt.Sprintf("listener creation failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for ListenerCreateError.
func (e ListenerCreateError) Is(target error) bool {
	if t, ok := target.(ListenerCreateError); ok {
		return e == t
	}
	return false
}

// NotifierNotifyError represents errors that can occur when notifying.
type NotifierNotifyError int

const (
	NotifierNotifyErrorEventIdOutOfBounds NotifierNotifyError = C.iox2_notifier_notify_error_e_EVENT_ID_OUT_OF_BOUNDS
)

func (e NotifierNotifyError) Error() string {
	switch e {
	case NotifierNotifyErrorEventIdOutOfBounds:
		return "notify failed: event id out of bounds"
	default:
		return fmt.Sprintf("notify failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for NotifierNotifyError.
func (e NotifierNotifyError) Is(target error) bool {
	if t, ok := target.(NotifierNotifyError); ok {
		return e == t
	}
	return false
}

// ListenerWaitError represents errors that can occur when waiting on a listener.
type ListenerWaitError int

const (
	ListenerWaitErrorContractViolation ListenerWaitError = C.iox2_listener_wait_error_e_CONTRACT_VIOLATION
	ListenerWaitErrorInternalFailure   ListenerWaitError = C.iox2_listener_wait_error_e_INTERNAL_FAILURE
	ListenerWaitErrorInterruptSignal   ListenerWaitError = C.iox2_listener_wait_error_e_INTERRUPT_SIGNAL
)

func (e ListenerWaitError) Error() string {
	switch e {
	case ListenerWaitErrorContractViolation:
		return "listener wait failed: contract violation"
	case ListenerWaitErrorInternalFailure:
		return "listener wait failed: internal failure"
	case ListenerWaitErrorInterruptSignal:
		return "listener wait failed: interrupt signal"
	default:
		return fmt.Sprintf("listener wait failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for ListenerWaitError.
func (e ListenerWaitError) Is(target error) bool {
	if t, ok := target.(ListenerWaitError); ok {
		return e == t
	}
	return false
}

// PubSubOpenOrCreateError represents errors when opening or creating a pub-sub service.
type PubSubOpenOrCreateError int

func (e PubSubOpenOrCreateError) Error() string {
	return fmt.Sprintf("pub-sub service open/create failed: error code %d", int(e))
}

// Is implements errors.Is support for PubSubOpenOrCreateError.
func (e PubSubOpenOrCreateError) Is(target error) bool {
	if t, ok := target.(PubSubOpenOrCreateError); ok {
		return e == t
	}
	return false
}

// EventOpenOrCreateError represents errors when opening or creating an event service.
type EventOpenOrCreateError int

func (e EventOpenOrCreateError) Error() string {
	return fmt.Sprintf("event service open/create failed: error code %d", int(e))
}

// Is implements errors.Is support for EventOpenOrCreateError.
func (e EventOpenOrCreateError) Is(target error) bool {
	if t, ok := target.(EventOpenOrCreateError); ok {
		return e == t
	}
	return false
}

// TypeDetailError represents errors related to type details.
type TypeDetailError int

const (
	TypeDetailErrorInvalidTypeName             TypeDetailError = C.iox2_type_detail_error_e_INVALID_TYPE_NAME
	TypeDetailErrorInvalidSizeOrAlignmentValue TypeDetailError = C.iox2_type_detail_error_e_INVALID_SIZE_OR_ALIGNMENT_VALUE
)

func (e TypeDetailError) Error() string {
	switch e {
	case TypeDetailErrorInvalidTypeName:
		return "type detail error: invalid type name"
	case TypeDetailErrorInvalidSizeOrAlignmentValue:
		return "type detail error: invalid size or alignment value"
	default:
		return fmt.Sprintf("type detail error: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for TypeDetailError.
func (e TypeDetailError) Is(target error) bool {
	if t, ok := target.(TypeDetailError); ok {
		return e == t
	}
	return false
}

// RequestResponseOpenOrCreateError represents errors when opening or creating a request-response service.
type RequestResponseOpenOrCreateError int

func (e RequestResponseOpenOrCreateError) Error() string {
	return fmt.Sprintf("request-response service open/create failed: error code %d", int(e))
}

// Is implements errors.Is support for RequestResponseOpenOrCreateError.
func (e RequestResponseOpenOrCreateError) Is(target error) bool {
	if t, ok := target.(RequestResponseOpenOrCreateError); ok {
		return e == t
	}
	return false
}

// ClientCreateError represents errors that can occur when creating a client.
type ClientCreateError int

const (
	ClientCreateErrorExceedsMaxSupportedClients ClientCreateError = C.iox2_client_create_error_e_EXCEEDS_MAX_SUPPORTED_CLIENTS
)

func (e ClientCreateError) Error() string {
	switch e {
	case ClientCreateErrorExceedsMaxSupportedClients:
		return "client creation failed: exceeds max supported clients"
	default:
		return fmt.Sprintf("client creation failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for ClientCreateError.
func (e ClientCreateError) Is(target error) bool {
	if t, ok := target.(ClientCreateError); ok {
		return e == t
	}
	return false
}

// ServerCreateError represents errors that can occur when creating a server.
type ServerCreateError int

const (
	ServerCreateErrorExceedsMaxSupportedServers ServerCreateError = C.iox2_server_create_error_e_EXCEEDS_MAX_SUPPORTED_SERVERS
)

func (e ServerCreateError) Error() string {
	switch e {
	case ServerCreateErrorExceedsMaxSupportedServers:
		return "server creation failed: exceeds max supported servers"
	default:
		return fmt.Sprintf("server creation failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for ServerCreateError.
func (e ServerCreateError) Is(target error) bool {
	if t, ok := target.(ServerCreateError); ok {
		return e == t
	}
	return false
}

// RequestSendError represents errors that can occur when sending a request.
type RequestSendError int

const (
	RequestSendErrorConnectionBroken       RequestSendError = C.iox2_request_send_error_e_CONNECTION_BROKEN_SINCE_SENDER_NO_LONGER_EXISTS
	RequestSendErrorConnectionCorrupted    RequestSendError = C.iox2_request_send_error_e_CONNECTION_CORRUPTED
	RequestSendErrorLoanOutOfMemory        RequestSendError = C.iox2_request_send_error_e_LOAN_ERROR_OUT_OF_MEMORY
	RequestSendErrorLoanExceedsMaxLoans    RequestSendError = C.iox2_request_send_error_e_LOAN_ERROR_EXCEEDS_MAX_LOANS
	RequestSendErrorLoanExceedsMaxLoanSize RequestSendError = C.iox2_request_send_error_e_LOAN_ERROR_EXCEEDS_MAX_LOAN_SIZE
	RequestSendErrorLoanInternalFailure    RequestSendError = C.iox2_request_send_error_e_LOAN_ERROR_INTERNAL_FAILURE
	RequestSendErrorConnectionError        RequestSendError = C.iox2_request_send_error_e_CONNECTION_ERROR
	RequestSendErrorExceedsMaxActiveReqs   RequestSendError = C.iox2_request_send_error_e_EXCEEDS_MAX_ACTIVE_REQUESTS
)

func (e RequestSendError) Error() string {
	switch e {
	case RequestSendErrorConnectionBroken:
		return "request send failed: connection broken"
	case RequestSendErrorConnectionCorrupted:
		return "request send failed: connection corrupted"
	default:
		return fmt.Sprintf("request send failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for RequestSendError.
func (e RequestSendError) Is(target error) bool {
	if t, ok := target.(RequestSendError); ok {
		return e == t
	}
	return false
}

// ResponseSendError represents errors that can occur when sending a response.
type ResponseSendError int

func (e ResponseSendError) Error() string {
	return fmt.Sprintf("response send failed: error code %d", int(e))
}

// Is implements errors.Is support for ResponseSendError.
func (e ResponseSendError) Is(target error) bool {
	if t, ok := target.(ResponseSendError); ok {
		return e == t
	}
	return false
}

// ConnectionError represents connection-related errors.
type ConnectionError int

func (e ConnectionError) Error() string {
	return fmt.Sprintf("connection error: error code %d", int(e))
}

// Is implements errors.Is support for ConnectionError.
func (e ConnectionError) Is(target error) bool {
	if t, ok := target.(ConnectionError); ok {
		return e == t
	}
	return false
}

// WaitSetCreateError represents errors when creating a waitset.
type WaitSetCreateError int

const (
	WaitSetCreateErrorInternalError         WaitSetCreateError = C.iox2_waitset_create_error_e_INTERNAL_ERROR
	WaitSetCreateErrorInsufficientResources WaitSetCreateError = C.iox2_waitset_create_error_e_INSUFFICIENT_RESOURCES
)

func (e WaitSetCreateError) Error() string {
	switch e {
	case WaitSetCreateErrorInternalError:
		return "waitset creation failed: internal error"
	case WaitSetCreateErrorInsufficientResources:
		return "waitset creation failed: insufficient resources"
	default:
		return fmt.Sprintf("waitset creation failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for WaitSetCreateError.
func (e WaitSetCreateError) Is(target error) bool {
	if t, ok := target.(WaitSetCreateError); ok {
		return e == t
	}
	return false
}

// WaitSetRunError represents errors during waitset run.
type WaitSetRunError int

const (
	WaitSetRunErrorInsufficientPermissions WaitSetRunError = C.iox2_waitset_run_error_e_INSUFFICIENT_PERMISSIONS
	WaitSetRunErrorInternalError           WaitSetRunError = C.iox2_waitset_run_error_e_INTERNAL_ERROR
	WaitSetRunErrorNoAttachments           WaitSetRunError = C.iox2_waitset_run_error_e_NO_ATTACHMENTS
	WaitSetRunErrorTerminationRequest      WaitSetRunError = C.iox2_waitset_run_error_e_TERMINATION_REQUEST
	WaitSetRunErrorInterrupt               WaitSetRunError = C.iox2_waitset_run_error_e_INTERRUPT
)

func (e WaitSetRunError) Error() string {
	switch e {
	case WaitSetRunErrorInsufficientPermissions:
		return "waitset run failed: insufficient permissions"
	case WaitSetRunErrorInternalError:
		return "waitset run failed: internal error"
	case WaitSetRunErrorNoAttachments:
		return "waitset run failed: no attachments"
	case WaitSetRunErrorTerminationRequest:
		return "waitset run failed: termination request"
	case WaitSetRunErrorInterrupt:
		return "waitset run failed: interrupt"
	default:
		return fmt.Sprintf("waitset run failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for WaitSetRunError.
func (e WaitSetRunError) Is(target error) bool {
	if t, ok := target.(WaitSetRunError); ok {
		return e == t
	}
	return false
}

// WaitSetAttachmentError represents errors when attaching to a waitset.
type WaitSetAttachmentError int

const (
	WaitSetAttachmentErrorInsufficientCapacity  WaitSetAttachmentError = C.iox2_waitset_attachment_error_e_INSUFFICIENT_CAPACITY
	WaitSetAttachmentErrorAlreadyAttached       WaitSetAttachmentError = C.iox2_waitset_attachment_error_e_ALREADY_ATTACHED
	WaitSetAttachmentErrorInternalError         WaitSetAttachmentError = C.iox2_waitset_attachment_error_e_INTERNAL_ERROR
	WaitSetAttachmentErrorInsufficientResources WaitSetAttachmentError = C.iox2_waitset_attachment_error_e_INSUFFICIENT_RESOURCES
)

func (e WaitSetAttachmentError) Error() string {
	switch e {
	case WaitSetAttachmentErrorInsufficientCapacity:
		return "waitset attachment failed: insufficient capacity"
	case WaitSetAttachmentErrorAlreadyAttached:
		return "waitset attachment failed: already attached"
	case WaitSetAttachmentErrorInternalError:
		return "waitset attachment failed: internal error"
	case WaitSetAttachmentErrorInsufficientResources:
		return "waitset attachment failed: insufficient resources"
	default:
		return fmt.Sprintf("waitset attachment failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for WaitSetAttachmentError.
func (e WaitSetAttachmentError) Is(target error) bool {
	if t, ok := target.(WaitSetAttachmentError); ok {
		return e == t
	}
	return false
}

// ServiceListError represents errors when listing services.
type ServiceListError int

const (
	ServiceListErrorInsufficientPermissions ServiceListError = C.iox2_service_list_error_e_INSUFFICIENT_PERMISSIONS
	ServiceListErrorInternalError           ServiceListError = C.iox2_service_list_error_e_INTERNAL_ERROR
)

func (e ServiceListError) Error() string {
	switch e {
	case ServiceListErrorInsufficientPermissions:
		return "service list failed: insufficient permissions"
	case ServiceListErrorInternalError:
		return "service list failed: internal error"
	default:
		return fmt.Sprintf("service list failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for ServiceListError.
func (e ServiceListError) Is(target error) bool {
	if t, ok := target.(ServiceListError); ok {
		return e == t
	}
	return false
}

// ServiceDetailsError represents errors when getting service details.
type ServiceDetailsError int

const (
	ServiceDetailsErrorFailedToOpenStaticServiceInfo        ServiceDetailsError = C.iox2_service_details_error_e_FAILED_TO_OPEN_STATIC_SERVICE_INFO
	ServiceDetailsErrorFailedToReadStaticServiceInfo        ServiceDetailsError = C.iox2_service_details_error_e_FAILED_TO_READ_STATIC_SERVICE_INFO
	ServiceDetailsErrorFailedToDeserializeStaticServiceInfo ServiceDetailsError = C.iox2_service_details_error_e_FAILED_TO_DESERIALIZE_STATIC_SERVICE_INFO
	ServiceDetailsErrorServiceInInconsistentState           ServiceDetailsError = C.iox2_service_details_error_e_SERVICE_IN_INCONSISTENT_STATE
	ServiceDetailsErrorVersionMismatch                      ServiceDetailsError = C.iox2_service_details_error_e_VERSION_MISMATCH
	ServiceDetailsErrorInternalError                        ServiceDetailsError = C.iox2_service_details_error_e_INTERNAL_ERROR
	ServiceDetailsErrorFailedToAcquireNodeState             ServiceDetailsError = C.iox2_service_details_error_e_FAILED_TO_ACQUIRE_NODE_STATE
)

func (e ServiceDetailsError) Error() string {
	switch e {
	case ServiceDetailsErrorFailedToOpenStaticServiceInfo:
		return "service details failed: failed to open static service info"
	case ServiceDetailsErrorFailedToReadStaticServiceInfo:
		return "service details failed: failed to read static service info"
	case ServiceDetailsErrorFailedToDeserializeStaticServiceInfo:
		return "service details failed: failed to deserialize static service info"
	case ServiceDetailsErrorServiceInInconsistentState:
		return "service details failed: service in inconsistent state"
	case ServiceDetailsErrorVersionMismatch:
		return "service details failed: version mismatch"
	case ServiceDetailsErrorInternalError:
		return "service details failed: internal error"
	case ServiceDetailsErrorFailedToAcquireNodeState:
		return "service details failed: failed to acquire node state"
	default:
		return fmt.Sprintf("service details failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for ServiceDetailsError.
func (e ServiceDetailsError) Is(target error) bool {
	if t, ok := target.(ServiceDetailsError); ok {
		return e == t
	}
	return false
}

// ErrHandleClosed indicates a handle has been closed.
var ErrHandleClosed = errors.New("iceoryx2: handle is closed")

// AttributeDefinitionError represents errors when defining attributes.
type AttributeDefinitionError int

const (
	AttributeDefinitionErrorExceedsMaxSupportedAttributes AttributeDefinitionError = C.iox2_attribute_definition_error_e_EXCEEDS_MAX_SUPPORTED_ATTRIBUTES
)

func (e AttributeDefinitionError) Error() string {
	switch e {
	case AttributeDefinitionErrorExceedsMaxSupportedAttributes:
		return "attribute definition failed: exceeds max supported attributes"
	default:
		return fmt.Sprintf("attribute definition failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for AttributeDefinitionError.
func (e AttributeDefinitionError) Is(target error) bool {
	if t, ok := target.(AttributeDefinitionError); ok {
		return e == t
	}
	return false
}

// AttributeVerificationError represents errors when verifying attributes.
type AttributeVerificationError int

const (
	AttributeVerificationErrorNonExistingKey   AttributeVerificationError = C.iox2_attribute_verification_error_e_NON_EXISTING_KEY
	AttributeVerificationErrorIncompatibleAttr AttributeVerificationError = C.iox2_attribute_verification_error_e_INCOMPATIBLE_ATTRIBUTE
)

func (e AttributeVerificationError) Error() string {
	switch e {
	case AttributeVerificationErrorNonExistingKey:
		return "attribute verification failed: non-existing key"
	case AttributeVerificationErrorIncompatibleAttr:
		return "attribute verification failed: incompatible attribute"
	default:
		return fmt.Sprintf("attribute verification failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for AttributeVerificationError.
func (e AttributeVerificationError) Is(target error) bool {
	if t, ok := target.(AttributeVerificationError); ok {
		return e == t
	}
	return false
}

// NodeListError represents errors when listing nodes.
type NodeListError int

const (
	NodeListErrorInsufficientPermissions NodeListError = C.iox2_node_list_failure_e_INSUFFICIENT_PERMISSIONS
	NodeListErrorInterrupt               NodeListError = C.iox2_node_list_failure_e_INTERRUPT
	NodeListErrorInternalError           NodeListError = C.iox2_node_list_failure_e_INTERNAL_ERROR
)

func (e NodeListError) Error() string {
	switch e {
	case NodeListErrorInsufficientPermissions:
		return "node list failed: insufficient permissions"
	case NodeListErrorInterrupt:
		return "node list failed: interrupted"
	case NodeListErrorInternalError:
		return "node list failed: internal error"
	default:
		return fmt.Sprintf("node list failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for NodeListError.
func (e NodeListError) Is(target error) bool {
	if t, ok := target.(NodeListError); ok {
		return e == t
	}
	return false
}

// NodeCleanupError represents errors when cleaning up stale resources.
type NodeCleanupError int

const (
	NodeCleanupErrorInterrupt               NodeCleanupError = C.iox2_node_cleanup_failure_e_INTERRUPT
	NodeCleanupErrorInternalError           NodeCleanupError = C.iox2_node_cleanup_failure_e_INTERNAL_ERROR
	NodeCleanupErrorInsufficientPermissions NodeCleanupError = C.iox2_node_cleanup_failure_e_INSUFFICIENT_PERMISSIONS
	NodeCleanupErrorVersionMismatch         NodeCleanupError = C.iox2_node_cleanup_failure_e_VERSION_MISMATCH
)

func (e NodeCleanupError) Error() string {
	switch e {
	case NodeCleanupErrorInterrupt:
		return "node cleanup failed: interrupted"
	case NodeCleanupErrorInternalError:
		return "node cleanup failed: internal error"
	case NodeCleanupErrorInsufficientPermissions:
		return "node cleanup failed: insufficient permissions"
	case NodeCleanupErrorVersionMismatch:
		return "node cleanup failed: version mismatch"
	default:
		return fmt.Sprintf("node cleanup failed: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for NodeCleanupError.
func (e NodeCleanupError) Is(target error) bool {
	if t, ok := target.(NodeCleanupError); ok {
		return e == t
	}
	return false
}

// ConnectionFailure represents errors related to connection issues.
type ConnectionFailure int

const (
	ConnectionFailureFailedToEstablish      ConnectionFailure = C.iox2_connection_failure_e_FAILED_TO_ESTABLISH_CONNECTION
	ConnectionFailureUnableToMapDataSegment ConnectionFailure = C.iox2_connection_failure_e_UNABLE_TO_MAP_SENDERS_DATA_SEGMENT
)

func (e ConnectionFailure) Error() string {
	switch e {
	case ConnectionFailureFailedToEstablish:
		return "connection failure: failed to establish connection"
	case ConnectionFailureUnableToMapDataSegment:
		return "connection failure: unable to map sender's data segment"
	default:
		return fmt.Sprintf("connection failure: unknown error (%d)", int(e))
	}
}

// Is implements errors.Is support for ConnectionFailure.
func (e ConnectionFailure) Is(target error) bool {
	if t, ok := target.(ConnectionFailure); ok {
		return e == t
	}
	return false
}
