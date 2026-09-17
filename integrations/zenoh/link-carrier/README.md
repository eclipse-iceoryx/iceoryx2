# iceoryx2-integrations-zenoh-link-carrier

> [!IMPORTANT]
> The link is currently a prototype and requires validation in real
> deployments. Only recommended for experimentation in development
> deployments.
>
> If encountering issues, create an issue to help us converge to stability.

The zenoh carrier of bytes for a tunnel, connecting `iceoryx2` systems on
different hosts through [zenoh](https://zenoh.io) sessions.

Tunnels whose sessions reach each other see each other's services. With the
default zenoh configuration, that is every tunnel on the same network.

## Usage

The carrier runs under a tunnel:

```rust
use iceoryx2::prelude::*;
use iceoryx2_integrations_zenoh_link_carrier::ZenohCarrier;
use iceoryx2_link::Link;
use iceoryx2_link_tunnel::Tunnel;

let config = iceoryx2::config::Config::default();
let node = NodeBuilder::new().config(&config).create::<ipc::Service>()?;

let carrier = ZenohCarrier::create(zenoh::Config::default())?;
// A zenoh configuration from a file instead:
// let carrier = ZenohCarrier::create(zenoh::Config::from_file("zenoh.json5")?)?;

let tunnel = Tunnel::new(carrier, &config);

let mut link = Link::new(node, tunnel);
let listener = link.listener()?;

loop {
    listener.blocking_wait(|_| {})?;
    link.discover()?;
    link.propagate()?;
}
```

## CLI

The tunnel is also available as a command line tool, see
[`link-tunnel-cli`](../link-tunnel-cli/README.md).
