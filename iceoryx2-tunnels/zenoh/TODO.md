# TODOs

1. [ ] Configuration
    1. [ ] Configure via file
    1. [ ] Configure: Peers
    1. [x] Configure: External service discovery service
    1. [x] Configure: Execution mode
    1. [ ] Configure: Static discovery
1. [x] Polling Execution
    1. [x] Discover local `iceoryx2` services
    1. [x] Make local `iceoryx2` services discoverable from `zenoh`
    1. [x] Discover remote services in `zenoh`
    1. [x] Propagate from local `iceoryx2` participants` to remote hosts
    1. [x] Propagate from remote hosts to local `iceoryx2` participants
1. [ ] Reactive Execution
    1. [ ] Implement `FileDescriptionBased` for zenoh subscribers
    1. [ ] Attach listeners for `iceoryx2` subscribers to (external) `WaitSet`
        * Assume listener has same service name as subscriber
    1. [ ] Attach zenoh subscribers to (external) `WaitSet`
1. [ ] Testing
    1. [x] Unit testing infrastructure
    1. [x] Unit Test: Discover local services
    1. [x] Unit Test: Discover remote services
    1. [x] Unit Test: Propagate `[u8]` Payload
    1. [x] Unit Test: Propagate `struct` Payload
    1. [ ] Host-to-host testing infrastructure
1. [ ] Refactoring
    1. [x] Tunnel constructor should take config by reference
    1. [ ] Synchronize entire discovery state with discovery service on startup
    1. [ ] Properly handle all error cases in implementation
