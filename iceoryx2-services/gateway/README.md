# iceoryx2-gateway

The `iceoryx2` gateway extends the communication beyond the boundary of a
host.

The gateway is provided as a library so that users have the choice of embedding
it into their own application. The implementation does not spawn any threads,
giving the user complete control over its execution.

The `iox2 gateway` CLI is provided as a convenience for spinning up gateways in
isolated processes, e.g. with the Zenoh backend:

```bash
iox2 gateway zenoh
```

## Gateway Mechanisms

The gateway is implemented against generic traits thus has no knowledge over the
specifics of the mechanism being used.

A custom bridging mechanism can be provided by implementing the traits in the
`iceoryx2-gateway-backend` crate and passing the implementation when
initializing the gateway.

Ready-to-use backend implementations are available in the `iceoryx2-services/gateway-**`
crates.

## Usage

The gateway is driven by two operations that the user is in full control of:

* `discover()` — reconciles local and remote services.
* `propagate()` — moves data bidirectionally between shared memory and the backend.

### Polled mode

The gateway is driven manually. Here it is paced by the node's `wait`, which also
provides a clean shutdown signal:

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

When the backend supports it, the gateway can be woken only when there is data
ready to propagate, rather than polling. `create()` additionally returns a
`Listener` to wait on:

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
