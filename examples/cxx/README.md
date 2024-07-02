# Instructions

## Build

In the repository root folder, execute this steps.

```bash
cmake -S . -B target/ffi/build -DBUILD_EXAMPLES=ON
cmake --build target/ffi/build
```

## Run Examples

### Publish-Subscribe

Run in two separate terminals. Note, currently the examples run for 10 seconds.

<!-- TODO -->
```bash
target/ffi/build/examples/cxx/publish_subscribe/example_cxx_publisher
```

<!-- TODO -->
```bash
target/ffi/build/examples/cxx/publish_subscribe/example_cxx_subscriber
```

### Discovery

<!-- TODO -->
```bash
target/ffi/build/examples/cxx/discovery/example_cxx_discovery
```
