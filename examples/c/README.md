# Instructions

## Build

```console
cmake -B target/lang-c .
cmake --build target/lang-c
```

## Run Examples

### Publish-Subscribe

Run in two separate terminals. Note, currently the examples run for 10 seconds.

```console
target/lang-c/examples/c/publish_subscribe/example_c_publisher
```

```console
target/lang-c/examples/c/publish_subscribe/example_c_subscriber
```

### Discovery

```console
target/lang-c/examples/c/discovery/example_c_discovery
```
