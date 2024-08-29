# Benchmarks

## Publish-Subscribe

The benchmark quantifies the latency between a `Publisher` sending a message and
a `Subscriber` receiving it. In the setup, a bidirectional connection is
established from process `a` to `b` (service name `a2b`) and back
(service name `b2a`). `Subscriber`s employ multithreaded busy waiting and promptly
respond upon message reception. This process repeats `n` times, and the average
latency is subsequently computed.

```sh
cargo run --bin benchmark-publish-subscribe --release -- --bench-all
```

For more benchmark configuration details, see

```sh
cargo run --bin benchmark-publish-subscribe --release -- --help
```

## Event

The event quantifies the latency between a `Notifier` sending a notification and
a `Listener` waking up from and responding to it. In the setup, a bidirectional connection is
established from process `a` to `b` (service name `a2b`) and back
(service name `b2a`). The `Listener` employs a blocking wait and wakes up on signal
reception to promptly respond with a return signal notification. This process repeats `n`
times, and the average latency is subsequently computed.

```sh
cargo run --bin benchmark-event --release -- --bench-all
```

For more benchmark configuration details, see

```sh
cargo run --bin benchmark-event --release -- --help
```
