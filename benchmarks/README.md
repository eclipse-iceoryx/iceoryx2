# Benchmarks

## Publish-Subscribe

The benchmark quantifies the latency between a publisher sending a message and
a subscriber receiving it. In the setup, a bidirectional connection is
established from process `a` to `b` (service name `a2b`) and back
(service name `b2a`). Subscribers employ multithreaded busy waiting and promptly
respond upon message reception. This process repeats `n` times, and the average
latency is subsequently computed.

```sh
cargo run --release benchmark_publish_subscribe
```
