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

import (
	"context"
	"runtime/cgo"
	"time"
	"unsafe"
)

/*
#include "iox2/iceoryx2.h"
#include <stdlib.h>
#include <string.h>

// Forward declaration of Go callback - must match the //export signature exactly
extern long long goWaitSetDispatch(
    void* attachment_id,
    void* ctx
);

// C trampoline that matches iox2_waitset_run_callback signature and calls Go
static iox2_callback_progression_e waitset_callback_trampoline(
    iox2_waitset_attachment_id_h attachment_id,
    iox2_callback_context ctx
) {
    long long result = goWaitSetDispatch((void*)attachment_id, ctx);
    return (iox2_callback_progression_e)result;
}

// Wrapper function for wait_and_process_once with our trampoline
static int iox2_waitset_wait_and_process_once_with_trampoline(
    iox2_waitset_h_ref handle,
    void* callback_ctx,
    enum iox2_waitset_run_result_e* result
) {
    return iox2_waitset_wait_and_process_once(handle, waitset_callback_trampoline, callback_ctx, result);
}

// Wrapper function for wait_and_process_once_with_timeout with our trampoline
static int iox2_waitset_wait_and_process_once_with_timeout_trampoline(
    iox2_waitset_h_ref handle,
    void* callback_ctx,
    uint64_t secs,
    uint32_t nanos,
    enum iox2_waitset_run_result_e* result
) {
    return iox2_waitset_wait_and_process_once_with_timeout(handle, waitset_callback_trampoline, callback_ctx, secs, nanos, result);
}

// Wrapper function for wait_and_process (blocking) with our trampoline
static int iox2_waitset_wait_and_process_with_trampoline(
    iox2_waitset_h_ref handle,
    void* callback_ctx,
    enum iox2_waitset_run_result_e* result
) {
    return iox2_waitset_wait_and_process(handle, waitset_callback_trampoline, callback_ctx, result);
}

// Wrapper callback that invokes pass-through to allow Go callback.
// We use this as a simple C callback that just continues processing.
static enum iox2_callback_progression_e simple_waitset_callback(
    iox2_waitset_attachment_id_h attachment_id,
    void* context
) {
    // Drop the attachment id we received
    iox2_waitset_attachment_id_drop(attachment_id);
    // Always continue processing all events
    return iox2_callback_progression_e_CONTINUE;
}

// Wrapper function for wait_and_process_once with simple callback
static int iox2_waitset_wait_and_process_once_simple(
    iox2_waitset_h_ref handle,
    enum iox2_waitset_run_result_e* result
) {
    return iox2_waitset_wait_and_process_once(handle, simple_waitset_callback, NULL, result);
}

// Wrapper function for wait_and_process_once_with_timeout with simple callback
static int iox2_waitset_wait_and_process_once_with_timeout_simple(
    iox2_waitset_h_ref handle,
    uint64_t secs,
    uint32_t nanos,
    enum iox2_waitset_run_result_e* result
) {
    return iox2_waitset_wait_and_process_once_with_timeout(handle, simple_waitset_callback, NULL, secs, nanos, result);
}
*/
import "C"

// WaitSetRunResult represents the result of a WaitSet run operation.
type WaitSetRunResult int

const (
	// WaitSetRunResultTerminationRequest indicates a termination was requested.
	WaitSetRunResultTerminationRequest WaitSetRunResult = C.iox2_waitset_run_result_e_TERMINATION_REQUEST
	// WaitSetRunResultInterrupt indicates the wait was interrupted.
	WaitSetRunResultInterrupt WaitSetRunResult = C.iox2_waitset_run_result_e_INTERRUPT
	// WaitSetRunResultStopRequest indicates a stop was requested.
	WaitSetRunResultStopRequest WaitSetRunResult = C.iox2_waitset_run_result_e_STOP_REQUEST
	// WaitSetRunResultAllEventsHandled indicates all events were handled.
	WaitSetRunResultAllEventsHandled WaitSetRunResult = C.iox2_waitset_run_result_e_ALL_EVENTS_HANDLED
)

// String implements fmt.Stringer for WaitSetRunResult.
func (r WaitSetRunResult) String() string {
	switch r {
	case WaitSetRunResultTerminationRequest:
		return "TerminationRequest"
	case WaitSetRunResultInterrupt:
		return "Interrupt"
	case WaitSetRunResultStopRequest:
		return "StopRequest"
	case WaitSetRunResultAllEventsHandled:
		return "AllEventsHandled"
	default:
		return "Unknown"
	}
}

// WaitSetBuilder is used to configure and create a WaitSet.
type WaitSetBuilder struct {
	handle C.iox2_waitset_builder_h
}

// NewWaitSetBuilder creates a new WaitSetBuilder.
func NewWaitSetBuilder() *WaitSetBuilder {
	var handle C.iox2_waitset_builder_h
	C.iox2_waitset_builder_new(nil, &handle)
	return &WaitSetBuilder{
		handle: handle,
	}
}

// SignalHandlingMode sets how signals are handled by the WaitSet.
func (b *WaitSetBuilder) SignalHandlingMode(mode SignalHandlingMode) *WaitSetBuilder {
	if b.handle != nil {
		cMode := C.enum_iox2_signal_handling_mode_e(mode)
		C.iox2_waitset_builder_set_signal_handling_mode(&b.handle, cMode)
	}
	return b
}

// Create creates a new WaitSet.
func (b *WaitSetBuilder) Create(serviceType ServiceType) (*WaitSet, error) {
	if b.handle == nil {
		return nil, ErrBuilderConsumed
	}

	var wsHandle C.iox2_waitset_h
	cServiceType := C.enum_iox2_service_type_e(serviceType)
	result := C.iox2_waitset_builder_create(b.handle, cServiceType, nil, &wsHandle)

	// Builder handle is consumed
	b.handle = nil

	if result != C.IOX2_OK {
		return nil, WaitSetCreateError(result)
	}

	return &WaitSet{
		handle: wsHandle,
	}, nil
}

// Close releases the builder resources without creating a WaitSet.
// Implements io.Closer.
func (b *WaitSetBuilder) Close() error {
	if b.handle != nil {
		C.iox2_waitset_builder_drop(b.handle)
		b.handle = nil
	}
	return nil
}

// WaitSet provides event-driven waiting for multiple sources.
type WaitSet struct {
	handle C.iox2_waitset_h
}

// AttachNotification attaches a listener to the WaitSet for notification events.
func (w *WaitSet) AttachNotification(listener *Listener) (*WaitSetGuard, error) {
	if w.handle == nil {
		return nil, ErrWaitSetClosed
	}
	if listener == nil || listener.handle == nil {
		return nil, ErrNilHandle
	}

	// Get the file descriptor from the listener
	fd := C.iox2_listener_get_file_descriptor(&listener.handle)

	var guardHandle C.iox2_waitset_guard_h
	result := C.iox2_waitset_attach_notification(&w.handle, fd, nil, &guardHandle)

	if result != C.IOX2_OK {
		return nil, WaitSetAttachmentError(result)
	}

	return &WaitSetGuard{
		handle: guardHandle,
	}, nil
}

// AttachDeadline attaches a listener with a deadline to the WaitSet.
func (w *WaitSet) AttachDeadline(listener *Listener, deadline time.Duration) (*WaitSetGuard, error) {
	if w.handle == nil {
		return nil, ErrWaitSetClosed
	}
	if listener == nil || listener.handle == nil {
		return nil, ErrNilHandle
	}

	// Get the file descriptor from the listener
	fd := C.iox2_listener_get_file_descriptor(&listener.handle)

	secs := uint64(deadline.Seconds())
	nanos := uint32(deadline.Nanoseconds() % 1e9)

	var guardHandle C.iox2_waitset_guard_h
	result := C.iox2_waitset_attach_deadline(
		&w.handle,
		fd,
		C.uint64_t(secs),
		C.uint32_t(nanos),
		nil,
		&guardHandle,
	)

	if result != C.IOX2_OK {
		return nil, WaitSetAttachmentError(result)
	}

	return &WaitSetGuard{
		handle: guardHandle,
	}, nil
}

// AttachInterval attaches an interval timer to the WaitSet.
func (w *WaitSet) AttachInterval(interval time.Duration) (*WaitSetGuard, error) {
	if w.handle == nil {
		return nil, ErrWaitSetClosed
	}

	secs := uint64(interval.Seconds())
	nanos := uint32(interval.Nanoseconds() % 1e9)

	var guardHandle C.iox2_waitset_guard_h
	result := C.iox2_waitset_attach_interval(
		&w.handle,
		C.uint64_t(secs),
		C.uint32_t(nanos),
		nil,
		&guardHandle,
	)

	if result != C.IOX2_OK {
		return nil, WaitSetAttachmentError(result)
	}

	return &WaitSetGuard{
		handle: guardHandle,
	}, nil
}

// WaitSetCallback is the callback function type for WaitSet processing.
// The callback receives a WaitSetAttachmentId that can be used to identify
// which attachment triggered the event. The callback must close the attachment ID
// when done, or return it to indicate it should be reused.
type WaitSetCallback func(*WaitSetAttachmentId) CallbackProgression

//export goWaitSetDispatch
func goWaitSetDispatch(
	attachmentIdPtr unsafe.Pointer,
	ctx unsafe.Pointer,
) C.longlong {
	// Retrieve the callback using cgo.Handle
	h := cgo.Handle(uintptr(ctx))
	callback, ok := h.Value().(WaitSetCallback)
	if !ok {
		// If callback is invalid, drop the attachment id and stop
		if attachmentIdPtr != nil {
			C.iox2_waitset_attachment_id_drop(C.iox2_waitset_attachment_id_h(attachmentIdPtr))
		}
		return C.longlong(C.iox2_callback_progression_e_STOP)
	}

	// Create the WaitSetAttachmentId wrapper
	attachmentId := &WaitSetAttachmentId{
		handle: C.iox2_waitset_attachment_id_h(attachmentIdPtr),
	}

	// Call the Go callback
	result := callback(attachmentId)

	// If the callback didn't close the attachment id, close it now
	if attachmentId.handle != nil {
		attachmentId.Close()
	}

	return C.longlong(result)
}

// WaitAndProcessOnce waits for events and processes them once.
// This uses a simple callback that handles all events and continues processing.
func (w *WaitSet) WaitAndProcessOnce() (WaitSetRunResult, error) {
	if w.handle == nil {
		return 0, ErrWaitSetClosed
	}

	var runResult C.enum_iox2_waitset_run_result_e

	result := C.iox2_waitset_wait_and_process_once_simple(
		&w.handle,
		&runResult,
	)

	if result != C.IOX2_OK {
		return 0, WaitSetRunError(result)
	}

	return WaitSetRunResult(runResult), nil
}

// WaitAndProcessOnceWithTimeout waits for events with a timeout and processes them.
func (w *WaitSet) WaitAndProcessOnceWithTimeout(timeout time.Duration) (WaitSetRunResult, error) {
	if w.handle == nil {
		return 0, ErrWaitSetClosed
	}

	secs := uint64(timeout.Seconds())
	nanos := uint32(timeout.Nanoseconds() % 1e9)

	var runResult C.enum_iox2_waitset_run_result_e
	result := C.iox2_waitset_wait_and_process_once_with_timeout_simple(
		&w.handle,
		C.uint64_t(secs),
		C.uint32_t(nanos),
		&runResult,
	)

	if result != C.IOX2_OK {
		return 0, WaitSetRunError(result)
	}

	return WaitSetRunResult(runResult), nil
}

// WaitAndProcessOnceWithContext waits for events with context cancellation support.
// This is the idiomatic Go way to wait with cancellation support.
// The pollInterval parameter controls the internal timeout for context checking (default 100ms if 0).
func (w *WaitSet) WaitAndProcessOnceWithContext(ctx context.Context, pollInterval time.Duration) (WaitSetRunResult, error) {
	if w.handle == nil {
		return 0, ErrWaitSetClosed
	}

	if pollInterval == 0 {
		pollInterval = 100 * time.Millisecond
	}

	for {
		select {
		case <-ctx.Done():
			return 0, ctx.Err()
		default:
			result, err := w.WaitAndProcessOnceWithTimeout(pollInterval)
			if err != nil {
				// Check if it's a timeout (which we expect when polling)
				if result == WaitSetRunResultTerminationRequest {
					continue // Keep waiting
				}
				return result, err
			}
			return result, nil
		}
	}
}

// WaitAndProcessOnceWithCallback waits for events and processes them once with a custom callback.
// The callback is invoked for each attachment that triggered an event.
// The callback can return CallbackProgressionStop to stop processing remaining events,
// or CallbackProgressionContinue to continue processing all events.
func (w *WaitSet) WaitAndProcessOnceWithCallback(callback WaitSetCallback) (WaitSetRunResult, error) {
	if w.handle == nil {
		return 0, ErrWaitSetClosed
	}

	// Create a cgo.Handle for the callback.
	// cgo.Handle prevents garbage collection until Delete() is called.
	h := cgo.NewHandle(callback)
	defer h.Delete()

	// Store handle value to pass through C.
	// SAFETY: cgo.Handle is designed to be passed through C as void*.
	hVal := uintptr(h)

	var runResult C.enum_iox2_waitset_run_result_e
	result := C.iox2_waitset_wait_and_process_once_with_trampoline(
		&w.handle,
		// #nosec G103 - intentional use of unsafe.Pointer for cgo callback context
		*(*unsafe.Pointer)(unsafe.Pointer(&hVal)),
		&runResult,
	)

	if result != C.IOX2_OK {
		return 0, WaitSetRunError(result)
	}

	return WaitSetRunResult(runResult), nil
}

// WaitAndProcessOnceWithTimeoutAndCallback waits for events with a timeout and processes them with a custom callback.
// The callback is invoked for each attachment that triggered an event.
func (w *WaitSet) WaitAndProcessOnceWithTimeoutAndCallback(timeout time.Duration, callback WaitSetCallback) (WaitSetRunResult, error) {
	if w.handle == nil {
		return 0, ErrWaitSetClosed
	}

	// Create a cgo.Handle for the callback.
	// cgo.Handle prevents garbage collection until Delete() is called.
	h := cgo.NewHandle(callback)
	defer h.Delete()

	// Store handle value to pass through C.
	// SAFETY: cgo.Handle is designed to be passed through C as void*.
	hVal := uintptr(h)

	secs := uint64(timeout.Seconds())
	nanos := uint32(timeout.Nanoseconds() % 1e9)

	var runResult C.enum_iox2_waitset_run_result_e
	result := C.iox2_waitset_wait_and_process_once_with_timeout_trampoline(
		&w.handle,
		// #nosec G103 - intentional use of unsafe.Pointer for cgo callback context
		*(*unsafe.Pointer)(unsafe.Pointer(&hVal)),
		C.uint64_t(secs),
		C.uint32_t(nanos),
		&runResult,
	)

	if result != C.IOX2_OK {
		return 0, WaitSetRunError(result)
	}

	return WaitSetRunResult(runResult), nil
}

// Run blocks indefinitely, waiting for events and invoking the callback for each one.
// The callback is invoked for each attachment that triggered an event.
// Run returns when:
// - The callback returns CallbackProgressionStop (returns WaitSetRunResultStopRequest)
// - A termination signal is received (returns WaitSetRunResultTerminationRequest)
// - An interrupt occurs (returns WaitSetRunResultInterrupt)
// - An error occurs
func (w *WaitSet) Run(callback WaitSetCallback) (WaitSetRunResult, error) {
	if w.handle == nil {
		return 0, ErrWaitSetClosed
	}

	// Create a cgo.Handle for the callback.
	// cgo.Handle prevents garbage collection until Delete() is called.
	h := cgo.NewHandle(callback)
	defer h.Delete()

	// Store handle value to pass through C.
	// SAFETY: cgo.Handle is designed to be passed through C as void*.
	hVal := uintptr(h)

	var runResult C.enum_iox2_waitset_run_result_e
	result := C.iox2_waitset_wait_and_process_with_trampoline(
		&w.handle,
		// #nosec G103 - intentional use of unsafe.Pointer for cgo callback context
		*(*unsafe.Pointer)(unsafe.Pointer(&hVal)),
		&runResult,
	)

	if result != C.IOX2_OK {
		return 0, WaitSetRunError(result)
	}

	return WaitSetRunResult(runResult), nil
}

// RunWithContext runs the WaitSet with context cancellation support.
// This combines the blocking behavior of Run with the ability to cancel via context.
// The pollInterval parameter controls the internal timeout for context checking (default 100ms if 0).
func (w *WaitSet) RunWithContext(ctx context.Context, callback WaitSetCallback, pollInterval time.Duration) (WaitSetRunResult, error) {
	if w.handle == nil {
		return 0, ErrWaitSetClosed
	}

	if pollInterval == 0 {
		pollInterval = 100 * time.Millisecond
	}

	for {
		select {
		case <-ctx.Done():
			return 0, ctx.Err()
		default:
			result, err := w.WaitAndProcessOnceWithTimeoutAndCallback(pollInterval, callback)
			if err != nil {
				return result, err
			}
			// If user callback returned stop, honor it
			if result == WaitSetRunResultStopRequest {
				return result, nil
			}
			// Otherwise continue looping
		}
	}
}

// Close releases the WaitSet resources.
// Implements io.Closer.
func (w *WaitSet) Close() error {
	if w.handle != nil {
		C.iox2_waitset_drop(w.handle)
		w.handle = nil
	}
	return nil
}

// NumberOfAttachments returns the number of attachments in the WaitSet.
func (w *WaitSet) NumberOfAttachments() uint64 {
	if w.handle == nil {
		return 0
	}
	return uint64(C.iox2_waitset_len(&w.handle))
}

// Capacity returns the maximum number of attachments the WaitSet can hold.
func (w *WaitSet) Capacity() uint64 {
	if w.handle == nil {
		return 0
	}
	return uint64(C.iox2_waitset_capacity(&w.handle))
}

// IsEmpty returns true if no attachments are present.
func (w *WaitSet) IsEmpty() bool {
	if w.handle == nil {
		return true
	}
	return bool(C.iox2_waitset_is_empty(&w.handle))
}

// WaitSetGuard represents an attachment guard in the WaitSet.
type WaitSetGuard struct {
	handle C.iox2_waitset_guard_h
}

// Close releases the guard and detaches from the WaitSet.
// Implements io.Closer.
func (g *WaitSetGuard) Close() error {
	if g.handle != nil {
		C.iox2_waitset_guard_drop(g.handle)
		g.handle = nil
	}
	return nil
}

// WaitSetAttachmentId identifies which attachment triggered an event.
type WaitSetAttachmentId struct {
	handle C.iox2_waitset_attachment_id_h
}

// HasEventFrom checks if the attachment id corresponds to the given guard.
func (a *WaitSetAttachmentId) HasEventFrom(guard *WaitSetGuard) bool {
	if a.handle == nil || guard == nil || guard.handle == nil {
		return false
	}
	return bool(C.iox2_waitset_attachment_id_has_event_from(&a.handle, &guard.handle))
}

// HasMissedDeadline checks if a deadline was missed.
func (a *WaitSetAttachmentId) HasMissedDeadline(guard *WaitSetGuard) bool {
	if a.handle == nil || guard == nil || guard.handle == nil {
		return false
	}
	return bool(C.iox2_waitset_attachment_id_has_missed_deadline(&a.handle, &guard.handle))
}

// Close releases the attachment id.
// Implements io.Closer interface.
func (a *WaitSetAttachmentId) Close() error {
	if a.handle != nil {
		C.iox2_waitset_attachment_id_drop(a.handle)
		a.handle = nil
	}
	return nil
}
