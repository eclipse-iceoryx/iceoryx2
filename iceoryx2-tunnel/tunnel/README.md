# iceoryx2-tunnel

The `iceoryx2` tunnel extends the communication beyond the boundary of a
host.

The tunnel is provided as a library so that users have the choice of embedding
it into their own application. The implementation does not spawn any threads,
giving the user complete control over its execution.

The `iox2 tunnel` CLI is provided as a convenience for spinning up tunnels in
isolated processes.

## Tunnel Mechanisms

The tunnel is implemented against generic traits thus has no knowledge over the
specifics of the mechanism being used.

A custom tunnelling mechanism can be provided by implementing the traits in the
`iceoryx2-tunnel-backend` crate and passing the implementation when
initializing the tunnel.

Ready-to-use backend implementations are available in the `iceoryx2-tunnel-**`
crates.

## Usage

```rust
// Create a tunnel with the selected Backend
let tunnel_config = iceoryx2_tunnel::Config::default();
let backend_config = Backend::Config::default();
let iceoryx_config = iceoryx2::config::Config::default();

let mut tunnel =
    Tunnel::<Service, Backend>::create(&tunnel_config, &iceoryx_config, &backend_config).unwrap();

// Have full control over tunnelling operations
let tunnel.discover().unwrap();
let tunnel.propagate().unwrap()
```
