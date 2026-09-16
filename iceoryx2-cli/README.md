# iceoryx2-cli

CLI tooling for interacting with `iceoryx2` systems.

## Installation

Install via `cargo`:

```console
cargo install iceoryx2-cli
```

## Entrypoint

The entrypoint to the CLI is `iox2`:

```console
$ iox2 --help
The command-line interface entrypoint to iceoryx2.

Usage: iox2 [OPTIONS] [COMMAND]

Options:
  -l, --list     List all installed external commands
  -p, --paths    Display paths that will be checked for external commands
  -h, --help     Print help
  -V, --version  Print version

Commands:
  ...            See external installed commands with --list
```

Sub-commands are separate binaries (prefixed with `iox2-`) which can be
discovered by the entrypoint:

```console
$ iox2 --list
Discovered Commands:
  node
  service
```

Sub-commands can be run using their discovered name.

## Service

The `iox2 service` sub-command queries information about `iceoryx2`
services.

```console
$ iox2 service --help
Query information about iceoryx2 services

Usage: iox2 service [OPTIONS] [COMMAND]

Options:
  -f, --format <FORMAT>  [default: RON] [possible values: RON, JSON, YAML]
  -h, --help             Print help
  -V, --version          Print version

Commands:
  list     List all services
  details  Show service details
```

## Node

The `iox2 node` sub-command queries information about `iceoryx2` nodes.

```console
$ iox2 node --help
Query information about iceoryx2 nodes

Usage: iox2 node [OPTIONS] [COMMAND]

Options:
  -f, --format <FORMAT>  [default: RON] [possible values: RON, JSON, YAML]
  -h, --help             Print help
  -V, --version          Print version

Commands:
  list     List all nodes
  details  Show node details
```

## Gateway

The `iox2 gateway` sub-command bridges `iceoryx2` instances running on
different hosts or networks. `iox2-gateway` itself does not implement any
transport; it discovers and delegates to backend-specific binaries named
`iox2-gateway-<backend>`, which must be installed separately.

```console
$ iox2 gateway --help
Launch a gateway between iceoryx2 instances.

Usage: iox2 gateway [OPTIONS]

Options:
  -l, --list     List all installed gateway backends
  -p, --paths    Display paths that will be checked for gateway backends
  -h, --help     Print help
  -V, --version  Print version

Commands:
  ...            See installed gateway backends with --list
```

### Backends

Available backends:

* **Zenoh** — `cargo install iceoryx2-integrations-zenoh-gateway-cli`

Once installed, a backend is discovered automatically:

```console
$ iox2 gateway --list
Discovered Commands:
  zenoh
```

Invoke a backend by name; any additional arguments are forwarded to the
backend binary:

```console
$ iox2 gateway zenoh --help
Launch an iceoryx2 gateway using Zenoh as the transport.

Usage: iox2 gateway zenoh [OPTIONS]

Options:
  -z, --zenoh-config <PATH>          Path to a zenoh configuration file
  -d, --discovery-service <DISCOVERY_SERVICE>
                                     Name of a service providing discovery updates to connect to
      --poll <RATE>                  Poll for discovery updates and samples at the provided rate in milliseconds [default: 100]
      --reactive                     Reactively process discovery updates and samples
  -h, --help                         Print help
  -V, --version                      Print version
```

## Link

> [!IMPORTANT]
> The link is currently a prototype and requires validation in real
> deployments. Only recommended for experimentation in development
> deployments.
>
> If encountering issues, create an issue to help us converge to stability.

The `iox2 link` sub-command extends `iceoryx2` services across the boundary of
a shared memory domain. `iox2-link` itself implements no backend; it groups
the implementations by backend and delegates to binaries named
`iox2-link-<backend>-<implementation>`, which are installed separately.

```console
$ iox2 link --help
Launch a link extending iceoryx2 services across a boundary.

Usage: iox2 link <COMMAND>

Options:
  -h, --help     Print help
  -V, --version  Print version

Commands:
  tunnel   Launch a tunnel to other iceoryx2 systems over a carrier.
  gateway  Launch a gateway to another middleware over an adapter.
```

### Implementations

* **Tunnel over Zenoh**, `cargo install iceoryx2-integrations-zenoh-link-tunnel-cli`
* **Gateway to ROS 2**, built from source inside a sourced ROS 2 environment,
  see `integrations/ros2/link-gateway-cli`

Once installed, an implementation is listed under its backend:

```console
$ iox2 link tunnel --list
Discovered Commands:
  zenoh
```

Invoke an implementation by name; any additional arguments are forwarded to
its binary:

```console
$ iox2 link tunnel zenoh --help
Launch an iceoryx2 tunnel to other iceoryx2 systems over zenoh.

Usage: iox2 link tunnel zenoh [OPTIONS]

Options:
  -z, --zenoh-config <PATH>       Path to a zenoh configuration file
  -a, --allow <SERVICE>           Bridge iceoryx2 services whose names match this wildcard pattern, where '*' matches zero or more characters and '?' matches one. Repeatable. When omitted, all services are bridged.
      --poll <RATE>               Polling rate in milliseconds for discovery and sample propagation (defaults to 100ms when no other wake source is given; otherwise must be set explicitly to enable polling)
      --reactive                  Wake the tunnel when the peers have new data
      --listener <EVENT_SERVICE>  Additionally wake the tunnel when the named iceoryx2 event service fires (repeatable)
      --monitor                   Report what every bridge moved after each propagation, at trace level
  -h, --help                      Print help
  -V, --version                   Print version
```

## Extending

1. The CLI can be augmented with your own custom tool by developing binaries
   with a name prefixed by `iox2-` and placing it on the `PATH` to be discovered
   by `iox2`
2. Depend on `iceoryx2-cli` for some helpers to help with implementation:
   1. An `output` module defining the various output structures used by this
      crate
   2. A `Filter` trait for filtering data retrieved from `iceoryx2`
   3. A `Format` enum providing functionality for outputting in different
      formats
