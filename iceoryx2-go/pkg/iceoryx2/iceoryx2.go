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
#cgo CFLAGS: -I${SRCDIR}/../../../target/debug/iceoryx2-ffi-c-cbindgen/include
#cgo LDFLAGS: -L${SRCDIR}/../../../target/debug -liceoryx2_ffi_c
#cgo LDFLAGS: -Wl,-rpath,${SRCDIR}/../../../target/debug

#include "iox2/iceoryx2.h"
#include <stdlib.h>
#include <string.h>
*/
import "C"

import (
	"fmt"
	"time"
)

// ServiceType defines the communication domain for services.
type ServiceType int

const (
	// ServiceTypeLocal restricts communication to the same process.
	ServiceTypeLocal ServiceType = C.iox2_service_type_e_LOCAL
	// ServiceTypeIpc enables inter-process communication across multiple processes.
	ServiceTypeIpc ServiceType = C.iox2_service_type_e_IPC
)

// String implements fmt.Stringer for ServiceType.
func (s ServiceType) String() string {
	switch s {
	case ServiceTypeLocal:
		return "Local"
	case ServiceTypeIpc:
		return "IPC"
	default:
		return fmt.Sprintf("ServiceType(%d)", int(s))
	}
}

// LogLevel defines the logging verbosity level.
type LogLevel int

const (
	LogLevelTrace LogLevel = C.iox2_log_level_e_TRACE
	LogLevelDebug LogLevel = C.iox2_log_level_e_DEBUG
	LogLevelInfo  LogLevel = C.iox2_log_level_e_INFO
	LogLevelWarn  LogLevel = C.iox2_log_level_e_WARN
	LogLevelError LogLevel = C.iox2_log_level_e_ERROR
	LogLevelFatal LogLevel = C.iox2_log_level_e_FATAL
)

// String implements fmt.Stringer for LogLevel.
func (l LogLevel) String() string {
	switch l {
	case LogLevelTrace:
		return "Trace"
	case LogLevelDebug:
		return "Debug"
	case LogLevelInfo:
		return "Info"
	case LogLevelWarn:
		return "Warn"
	case LogLevelError:
		return "Error"
	case LogLevelFatal:
		return "Fatal"
	default:
		return fmt.Sprintf("LogLevel(%d)", int(l))
	}
}

// CallbackProgression controls the iteration flow in callback functions.
type CallbackProgression int

const (
	// CallbackProgressionStop stops the iteration.
	CallbackProgressionStop CallbackProgression = C.iox2_callback_progression_e_STOP
	// CallbackProgressionContinue continues the iteration.
	CallbackProgressionContinue CallbackProgression = C.iox2_callback_progression_e_CONTINUE
)

// String implements fmt.Stringer for CallbackProgression.
func (c CallbackProgression) String() string {
	switch c {
	case CallbackProgressionStop:
		return "Stop"
	case CallbackProgressionContinue:
		return "Continue"
	default:
		return fmt.Sprintf("CallbackProgression(%d)", int(c))
	}
}

// TypeVariant defines how payload size is determined.
type TypeVariant int

const (
	// TypeVariantFixedSize means the payload has a fixed size.
	TypeVariantFixedSize TypeVariant = C.iox2_type_variant_e_FIXED_SIZE
	// TypeVariantDynamic means the payload has a dynamic size.
	TypeVariantDynamic TypeVariant = C.iox2_type_variant_e_DYNAMIC
)

// String implements fmt.Stringer for TypeVariant.
func (t TypeVariant) String() string {
	switch t {
	case TypeVariantFixedSize:
		return "FixedSize"
	case TypeVariantDynamic:
		return "Dynamic"
	default:
		return fmt.Sprintf("TypeVariant(%d)", int(t))
	}
}

// UnableToDeliverStrategy defines behavior when a subscriber's buffer is full.
type UnableToDeliverStrategy int

const (
	// UnableToDeliverStrategyBlock blocks until space is available.
	UnableToDeliverStrategyBlock UnableToDeliverStrategy = C.iox2_unable_to_deliver_strategy_e_BLOCK
	// UnableToDeliverStrategyDiscardSample discards the oldest sample.
	UnableToDeliverStrategyDiscardSample UnableToDeliverStrategy = C.iox2_unable_to_deliver_strategy_e_DISCARD_SAMPLE
)

// String implements fmt.Stringer for UnableToDeliverStrategy.
func (u UnableToDeliverStrategy) String() string {
	switch u {
	case UnableToDeliverStrategyBlock:
		return "Block"
	case UnableToDeliverStrategyDiscardSample:
		return "DiscardSample"
	default:
		return fmt.Sprintf("UnableToDeliverStrategy(%d)", int(u))
	}
}

// EventId represents an event identifier used in the event messaging pattern.
type EventId uint64

// String implements fmt.Stringer for EventId.
func (e EventId) String() string {
	return fmt.Sprintf("EventId(%d)", uint64(e))
}

// SignalHandlingMode defines how signals are handled.
type SignalHandlingMode int

const (
	// SignalHandlingModeHandleTerminationRequests registers SIGINT and SIGTERM handlers.
	SignalHandlingModeHandleTerminationRequests SignalHandlingMode = C.iox2_signal_handling_mode_e_HANDLE_TERMINATION_REQUESTS
	// SignalHandlingModeDisabled disables signal handling.
	SignalHandlingModeDisabled SignalHandlingMode = C.iox2_signal_handling_mode_e_DISABLED
)

// String implements fmt.Stringer for SignalHandlingMode.
func (s SignalHandlingMode) String() string {
	switch s {
	case SignalHandlingModeHandleTerminationRequests:
		return "HandleTerminationRequests"
	case SignalHandlingModeDisabled:
		return "Disabled"
	default:
		return fmt.Sprintf("SignalHandlingMode(%d)", int(s))
	}
}

// AllocationStrategy defines the memory allocation strategy.
type AllocationStrategy int

const (
	// AllocationStrategyPowerOfTwo allocates memory in power of two sizes.
	AllocationStrategyPowerOfTwo AllocationStrategy = C.iox2_allocation_strategy_e_POWER_OF_TWO
	// AllocationStrategyBestFit allocates the smallest fitting block.
	AllocationStrategyBestFit AllocationStrategy = C.iox2_allocation_strategy_e_BEST_FIT
)

// String implements fmt.Stringer for AllocationStrategy.
func (a AllocationStrategy) String() string {
	switch a {
	case AllocationStrategyPowerOfTwo:
		return "PowerOfTwo"
	case AllocationStrategyBestFit:
		return "BestFit"
	default:
		return fmt.Sprintf("AllocationStrategy(%d)", int(a))
	}
}

// Constants for string length limits
const (
	ServiceNameMaxLength = C.IOX2_SERVICE_NAME_LENGTH
	NodeNameMaxLength    = C.IOX2_NODE_NAME_LENGTH
)

// durationToSecsNanos converts a time.Duration to seconds and nanoseconds
// for use with C API calls that expect separate sec/nsec parameters.
func durationToSecsNanos(d time.Duration) (secs C.uint64_t, nanos C.uint32_t) {
	secs = C.uint64_t(d / time.Second)
	nanos = C.uint32_t((d % time.Second).Nanoseconds())
	return
}

// SetLogLevelFromEnvOr sets the log level from environment variable IOX2_LOG_LEVEL,
// or uses the provided default if the environment variable is not set.
func SetLogLevelFromEnvOr(defaultLevel LogLevel) {
	C.iox2_set_log_level_from_env_or(uint32(defaultLevel))
}

// SetLogLevel sets the global log level.
func SetLogLevel(level LogLevel) {
	C.iox2_set_log_level(uint32(level))
}
