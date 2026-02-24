# iceoryx2-go

Go bindings for the [iceoryx2](https://github.com/eclipse-iceoryx/iceoryx2) inter-process communication library.

## Overview

iceoryx2 is a high-performance, lock-free, and zero-copy inter-process communication (IPC) library. These Go bindings provide idiomatic Go access to iceoryx2's functionality through cgo.

## Features

- **Publish-Subscribe**: Zero-copy data transfer between publishers and subscribers
- **Event**: Lightweight event notification mechanism
- **Request-Response**: Client-server communication pattern with typed requests and responses
- **WaitSet**: Event-driven waiting for multiple sources with support for notifications, deadlines, and intervals
- **Service Discovery**: Find and inspect running services
- **Type-Safe API**: Generic helpers for working with typed payloads
- **Cross-Language**: Compatible with other iceoryx2 language bindings (Rust, C, C++, Python)

## Prerequisites

Before using the Go bindings, you need to build the iceoryx2 C FFI library:

```bash
# From the iceoryx2 repository root
cargo build -p iceoryx2-ffi-c
```

This will generate:
- The C header file at `target/debug/iceoryx2-ffi-c-cbindgen/include/iox2/iceoryx2.h`
- The shared library at `target/debug/libiceoryx2_ffi.{so,dylib,dll}`

## Installation

```bash
go get github.com/eclipse-iceoryx/iceoryx2/iceoryx2-go
```

## Quick Start

### Publish-Subscribe Pattern

**Publisher:**

```go
package main

import (
    "fmt"
    "time"
    "unsafe"

    "github.com/eclipse-iceoryx/iceoryx2/iceoryx2-go/pkg/iceoryx2"
)

type MyData struct {
    Value int32
}

func main() {
    iceoryx2.SetLogLevelFromEnvOr(iceoryx2.LogLevelInfo)

    // Create a node
    node, _ := iceoryx2.NewNodeBuilder().
        Name("publisher").
        Create(iceoryx2.ServiceTypeIpc)
    defer node.Drop()

    // Create service
    serviceName, _ := iceoryx2.NewServiceName("My/Service")
    defer serviceName.Drop()

    service, _ := node.ServiceBuilder(serviceName).
        PublishSubscribe().
        PayloadType("MyData", uint64(unsafe.Sizeof(MyData{})), uint64(unsafe.Alignof(MyData{}))).
        OpenOrCreate()
    defer service.Drop()

    // Create publisher
    publisher, _ := service.PublisherBuilder().Create()
    defer publisher.Drop()

    // Send data
    for i := int32(0); i < 10; i++ {
        sample, _ := publisher.LoanUninit()
        payload := iceoryx2.PayloadMutAs[MyData](sample)
        payload.Value = i
        sample.Send()
        fmt.Printf("Sent: %d\n", i)
        time.Sleep(time.Second)
    }
}
```

**Subscriber:**

```go
package main

import (
    "fmt"
    "time"
    "unsafe"

    "github.com/eclipse-iceoryx/iceoryx2/iceoryx2-go/pkg/iceoryx2"
)

type MyData struct {
    Value int32
}

func main() {
    iceoryx2.SetLogLevelFromEnvOr(iceoryx2.LogLevelInfo)

    node, _ := iceoryx2.NewNodeBuilder().
        Name("subscriber").
        Create(iceoryx2.ServiceTypeIpc)
    defer node.Drop()

    serviceName, _ := iceoryx2.NewServiceName("My/Service")
    defer serviceName.Drop()

    service, _ := node.ServiceBuilder(serviceName).
        PublishSubscribe().
        PayloadType("MyData", uint64(unsafe.Sizeof(MyData{})), uint64(unsafe.Alignof(MyData{}))).
        OpenOrCreate()
    defer service.Drop()

    subscriber, _ := service.SubscriberBuilder().Create()
    defer subscriber.Drop()

    for {
        sample, _ := subscriber.Receive()
        if sample != nil {
            payload := iceoryx2.PayloadAs[MyData](sample)
            fmt.Printf("Received: %d\n", payload.Value)
            sample.Drop()
        }
        time.Sleep(time.Second)
    }
}
```

### Event Pattern

**Notifier:**

```go
package main

import (
    "fmt"
    "time"

    "github.com/eclipse-iceoryx/iceoryx2/iceoryx2-go/pkg/iceoryx2"
)

func main() {
    iceoryx2.SetLogLevelFromEnvOr(iceoryx2.LogLevelInfo)

    node, _ := iceoryx2.NewNodeBuilder().Create(iceoryx2.ServiceTypeIpc)
    defer node.Drop()

    serviceName, _ := iceoryx2.NewServiceName("MyEvent")
    defer serviceName.Drop()

    service, _ := node.ServiceBuilder(serviceName).Event().OpenOrCreate()
    defer service.Drop()

    notifier, _ := service.NotifierBuilder().Create()
    defer notifier.Drop()

    for i := uint64(0); i < 10; i++ {
        notifier.NotifyWithEventId(i)
        fmt.Printf("Triggered event %d\n", i)
        time.Sleep(time.Second)
    }
}
```

**Listener:**

```go
package main

import (
    "fmt"
    "time"

    "github.com/eclipse-iceoryx/iceoryx2/iceoryx2-go/pkg/iceoryx2"
)

func main() {
    iceoryx2.SetLogLevelFromEnvOr(iceoryx2.LogLevelInfo)

    node, _ := iceoryx2.NewNodeBuilder().Create(iceoryx2.ServiceTypeIpc)
    defer node.Drop()

    serviceName, _ := iceoryx2.NewServiceName("MyEvent")
    defer serviceName.Drop()

    service, _ := node.ServiceBuilder(serviceName).Event().OpenOrCreate()
    defer service.Drop()

    listener, _ := service.ListenerBuilder().Create()
    defer listener.Drop()

    for {
        eventId, _ := listener.TimedWaitOne(time.Second)
        if eventId != nil {
            fmt.Printf("Received event %d\n", eventId.Value)
        }
    }
}
```

## API Reference

### Core Types

- `ServiceType`: `ServiceTypeLocal` (same process) or `ServiceTypeIpc` (inter-process)
- `EventId`: Event identifier for event services
- `CallbackProgression`: `CallbackProgressionStop` or `CallbackProgressionContinue`
- `MessagingPattern`: `MessagingPatternPublishSubscribe`, `MessagingPatternEvent`, `MessagingPatternRequestResponse`

### Node

The central entry point for iceoryx2:

```go
node, err := iceoryx2.NewNodeBuilder().
    Name("my-app").
    SignalHandlingMode(iceoryx2.SignalHandlingModeHandleTerminationRequests).
    Create(iceoryx2.ServiceTypeIpc)
```

### Service Name

Create a unique service identifier:

```go
serviceName, err := iceoryx2.NewServiceName("My/Service/Name")
```

### Publish-Subscribe Service

```go
service, err := node.ServiceBuilder(serviceName).
    PublishSubscribe().
    PayloadType("TypeName", size, alignment).
    MaxPublishers(10).
    MaxSubscribers(100).
    HistorySize(5).
    OpenOrCreate()
```

### Event Service

```go
service, err := node.ServiceBuilder(serviceName).
    Event().
    MaxNotifiers(10).
    MaxListeners(100).
    EventIdMaxValue(255).
    OpenOrCreate()
```

### Request-Response Service

```go
service, err := node.ServiceBuilder(serviceName).
    RequestResponse().
    RequestPayloadType("Request", size, alignment).
    ResponsePayloadType("Response", size, alignment).
    MaxClients(10).
    MaxServers(5).
    OpenOrCreate()

// Create a client
client, err := service.Client().Create()

// Send a request and wait for response
pendingResponse, err := iceoryx2.SendCopyAs(client, &request)
response, err := pendingResponse.Receive()

// Create a server
server, err := service.Server().Create()

// Receive and respond to requests
activeRequest, err := server.Receive()
err = iceoryx2.ActiveRequestSendCopyAs(activeRequest, &response)
```

### WaitSet

Event-driven waiting for multiple sources:

```go
// Create a WaitSet
waitset, err := iceoryx2.NewWaitSetBuilder().
    SignalHandlingMode(iceoryx2.SignalHandlingModeHandleTerminationRequests).
    Create(iceoryx2.ServiceTypeIpc)

// Attach a listener for notifications
guard, err := waitset.AttachNotification(listener)
defer guard.Drop()

// Attach an interval timer
intervalGuard, err := waitset.AttachInterval(2 * time.Second)
defer intervalGuard.Drop()

// Wait and process events
result, err := waitset.WaitAndProcessOnce(func(id *iceoryx2.WaitSetAttachmentId) iceoryx2.CallbackProgression {
    if id.EventOriginatedFrom(guard) {
        // Handle event notification
    }
    return iceoryx2.CallbackProgressionContinue
})
```

### Service Discovery

Find and inspect running services:

```go
// Create a service discovery instance
discovery := iceoryx2.NewServiceDiscovery(iceoryx2.ServiceTypeIpc)

// Check if a service exists
exists, err := discovery.Exists(serviceName, iceoryx2.MessagingPatternPublishSubscribe)

// Get service details
info, err := discovery.FindPubSubService("My/Service")
if info != nil {
    fmt.Printf("Service: %s, Pattern: %s\n", info.Name, info.MessagingPattern.String())
}
```

## Building and Running Examples

```bash
# Build the iceoryx2 FFI library first
cargo build -p iceoryx2-ffi-c

# Run publish-subscribe example
cd examples/go/publish_subscribe
go run publisher.go
# In another terminal:
go run subscriber.go

# Run event examples
cd examples/go/event
go run notifier.go
# In another terminal:
go run listener.go

# Run request-response examples
cd examples/go/request_response
go run server.go
# In another terminal:
go run client.go

# Run waitset example
cd examples/go/waitset
go run waitset.go
# In another terminal:
go run notifier.go
```

## Cross-Language Communication

The Go bindings are fully compatible with other iceoryx2 language bindings. You can have:
- A Rust publisher with a Go subscriber
- A C++ notifier with a Go listener
- Any combination of languages using the same services

Just ensure:
1. The service name matches across applications
2. The payload type has the same memory layout (size and alignment)
3. All applications use the same service configuration

## License

Licensed under either of Apache 2.0 or MIT at your option.
