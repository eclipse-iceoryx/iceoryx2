# iceoryx2-cli

CLI tooling for interacting with `iceoryx2` systems.

## Installation

Install via `cargo`:

```console
cargo install iceoryx2-cli
```

## Usage

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

Sub-commands can be run using their discovered name:

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
