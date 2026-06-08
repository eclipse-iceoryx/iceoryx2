# iceoryx2-tunnel

The `iceoryx2` tunnel extends the communication beyond the boundary of a
host.

The tunnel is provided as a library so that users have the choice of embedding
it into their own application. The implementation does not spawn any threads,
giving the user complete control over its execution.

The `iox2 tunnel` CLI is provided as a convenience for spinning up tunnels in
isolated processes, e.g. with the Zenoh backend:

```bash
iox2 tunnel zenoh
```

## Tunnel Mechanisms

The tunnel is implemented against generic traits thus has no knowledge over the
specifics of the mechanism being used.

A custom tunnelling mechanism can be provided by implementing the traits in the
`iceoryx2-services-tunnel-backend` crate and passing the implementation when
initializing the tunnel.

Ready-to-use backend implementations are available in the `iceoryx2-services/tunnel-**`
crates.

## Usage

The tunnel is driven by two operations that the user is in full control of:

* `discover()` — reconciles local and remote services.
* `propagate()` — moves data bidirectionally between shared memory and the backend.

### Polled mode

The tunnel is driven manually. Here it is paced by the node's `wait`, which also
provides a clean shutdown signal:

```rust
use core::time::Duration;
use iceoryx2_services_tunnel::Tunnel;

const POLL_INTERVAL: Duration = Duration::from_millis(100);

// Create a tunnel with the selected Backend. Any configuration that is not
// provided falls back to `Default::default()`.
let mut tunnel = Tunnel::<Service, Backend>::new()
    .polled()
    .create()
    .expect("failed to create tunnel");

while tunnel.node().wait(POLL_INTERVAL).is_ok() {
    tunnel.discover().expect("discovery failed");
    tunnel.propagate().expect("propagation failed");
}
```

### Reactive mode

When the backend supports it, the tunnel can be woken only when there is data
ready to propagate, rather than polling. `create()` additionally returns a
`Listener` to wait on:

```rust
use iceoryx2_services_tunnel::Tunnel;

let (mut tunnel, listener) = Tunnel::<Service, Backend>::new()
    .reactive()
    .create()
    .expect("failed to create tunnel");

while listener.blocking_wait_all(|_| {}).is_ok() {
    tunnel.discover().expect("discovery failed");
    tunnel.propagate().expect("propagation failed");
}
```
