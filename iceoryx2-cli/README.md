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

## Tunnel

The `iox2 tunnel` sub-command bridges `iceoryx2` instances running on
different hosts or networks. `iox2-tunnel` itself does not implement any
transport; it discovers and delegates to backend-specific binaries named
`iox2-tunnel-<backend>`, which must be installed separately.

```console
$ iox2 tunnel --help
Launch a tunnel between iceoryx2 instances.

Usage: iox2 tunnel [OPTIONS]

Options:
  -l, --list     List all installed tunnel backends
  -p, --paths    Display paths that will be checked for tunnel backends
  -h, --help     Print help
  -V, --version  Print version

Commands:
  ...            See installed tunnel backends with --list
```

### Backends

Available backends:

* **Zenoh** — `cargo install iceoryx2-integrations-zenoh-tunnel-cli`

Once installed, a backend is discovered automatically:

```console
$ iox2 tunnel --list
Discovered Commands:
  zenoh
```

Invoke a backend by name; any additional arguments are forwarded to the
backend binary:

```console
$ iox2 tunnel zenoh --help
Launch an iceoryx2 tunnel using Zenoh as the transport.

Usage: iox2 tunnel zenoh [OPTIONS]

Options:
  -z, --zenoh-config <PATH>          Path to a zenoh configuration file
  -d, --discovery-service <DISCOVERY_SERVICE>
                                     Name of a service providing discovery updates to connect to
      --poll <RATE>                  Poll for discovery updates and samples at the provided rate in milliseconds [default: 100]
      --reactive                     Reactively process discovery updates and samples
  -h, --help                         Print help
  -V, --version                      Print version
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
