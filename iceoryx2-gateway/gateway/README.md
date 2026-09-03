# iceoryx2-gateway

The `iceoryx2` gateway extends the communication beyond the boundaries of
shared memory. It propagates the data flowing through shared memory over a
pluggable communication mechanism.

The gateway is provided as a library so that users have the choice of embedding
it into their own application. The implementation does not spawn any threads,
giving the user complete control over its execution.

Alternatively, the `iox2 gateway` CLI is provided as a convenience for spinning
up gateways in isolated processes, e.g. with the Zenoh backend:

```bash
iox2 gateway zenoh
```

## Configuration

### Service Configuration

A service is propagated only while every host offering it uses identical
settings. Settings set explicitly on the service builder and settings
inherited from the node's [configuration](
https://github.com/eclipse-iceoryx/iceoryx2/tree/main/config) both count, so
hosts with differing config files may conflict even when the application code
is the same.

Requiring identical settings is a conservative approach to ensure remote
services are able to communicate. This may be relaxed to compatible settings in
the future.

## Usage

The gateway is driven by two operations that the user is in full control of:

* `discover()` — reconciles local and remote services.
* `propagate()` — moves data bidirectionally between shared memory and the
  communication mechanism used by the gateway.

### Polled mode

The gateway is driven manually. Here it is paced by the node's `wait`, which
also provides a clean shutdown signal:

```rust
use core::time::Duration;
use iceoryx2_gateway::Gateway;

const POLL_INTERVAL: Duration = Duration::from_millis(100);

// Create a gateway with the selected Backend. Any configuration that is not
// provided falls back to `Default::default()`.
let mut gateway = Gateway::<Service, Backend>::new()
    .polled()
    .create()
    .expect("failed to create gateway");

while gateway.node().wait(POLL_INTERVAL).is_ok() {
    gateway.discover().expect("discovery failed");
    gateway.propagate().expect("propagation failed");
}
```

### Reactive mode

When the gateway mechanism supports it, the gateway can be woken only when
there is data ready to propagate, rather than polling. `create()` additionally
returns a `Listener` to wait on:

```rust
use iceoryx2_gateway::Gateway;

let (mut gateway, listener) = Gateway::<Service, Backend>::new()
    .reactive()
    .create()
    .expect("failed to create gateway");

while listener.blocking_wait_all(|_| {}).is_ok() {
    gateway.discover().expect("discovery failed");
    gateway.propagate().expect("propagation failed");
}
```

## Additional Gateway Mechanisms

The gateway is implemented against generic traits and has no knowledge of the
specific communication mechanism being used. The traits are available in the
`iceoryx2-gateway-backend` crate.

By implementing these traits, it is possible to extend the gateway to work with
additional communication mechanisms. Ready-to-use backend implementations are
available in the `integrations/*/gateway-backend` crates.
