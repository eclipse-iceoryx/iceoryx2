# Go Examples

This directory contains examples demonstrating how to use the iceoryx2 Go bindings.

## Prerequisites

Before running the examples, build the iceoryx2 C FFI library:

```bash
# From the iceoryx2 repository root
cargo build -p iceoryx2-ffi-c
```

## Examples

### Publish-Subscribe

A simple publish-subscribe example with a custom data structure.

**Publisher** (`publish_subscribe/publisher.go`):
- Creates a publisher that sends `TransmissionData` structs
- Sends data every second

**Subscriber** (`publish_subscribe/subscriber.go`):
- Creates a subscriber that receives `TransmissionData` structs
- Prints received data to console

Run:
```bash
# Terminal 1
cd publish_subscribe
go run publisher.go

# Terminal 2
cd publish_subscribe
go run subscriber.go
```

### Event

A simple event notification example.

**Notifier** (`event/notifier.go`):
- Creates a notifier that sends event IDs
- Triggers events every second

**Listener** (`event/listener.go`):
- Creates a listener that waits for events
- Prints received event IDs to console

Run:
```bash
# Terminal 1
cd event
go run notifier.go

# Terminal 2
cd event
go run listener.go
```

### Request-Response

A client-server example demonstrating the request-response pattern.

**Server** (`request_response/server.go`):
- Creates a server that receives calculation requests
- Responds with the sum of two numbers

**Client** (`request_response/client.go`):
- Creates a client that sends addition requests
- Receives and prints the calculated results

Run:
```bash
# Terminal 1 - Start the server first
cd request_response
go run server.go

# Terminal 2 - Then start the client
cd request_response
go run client.go
```

### WaitSet

An example demonstrating event-driven waiting with multiple sources.

**WaitSet** (`waitset/waitset.go`):
- Creates a WaitSet that waits for events
- Attaches a listener for notifications
- Attaches an interval timer (heartbeat every 2 seconds)
- Processes events as they arrive

**Notifier** (`waitset/notifier.go`):
- Creates a notifier to send events to the waitset
- Sends events every second

Run:
```bash
# Terminal 1 - Start the waitset first
cd waitset
go run waitset.go

# Terminal 2 - Then start the notifier
cd waitset
go run notifier.go
```

## Cross-Language Communication

These Go examples are compatible with the equivalent examples in other languages:
- `examples/rust/` - Rust examples
- `examples/c/` - C examples
- `examples/cxx/` - C++ examples
- `examples/python/` - Python examples

You can run a publisher in one language and a subscriber in another language, as long as:
1. They use the same service name
2. The payload type has the same memory layout
