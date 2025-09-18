# Async API Design Document

## Remarks

For this document currently we describe `Events` and `PubSub` however this
analogy shall continue for other messaging patterns later on.

## Terminology

## Overview

A high-level pitch of the feature:

* The new `async` API will provide non_blocking, but still linear, behavior in
  the code thanks to `async/await`
* The new `async` API will solve a usage of `iceoryx2` API in `async` code which
  currently need to be handmade manually either via:
    * polling and repetition logic within functions, causing additional not needed
    code on user side
    * custom WaitSet usage in separate thread and a bridge to `async` world.
    * or some other custom work
* The `async` usage in Rust is already well established technique that is
  adopted by many crates, including those with highest usage

### Introduction to async in context of iceoryx2

The Rust `async` APIs are build with the help of usage of compiler. At the end
`async` and `await` is just a syntactic sugar that is removed during `HIR`. In
practice it means:

* `async fn abc() -> bool` signature is turned into
  `fn abc() -> impl core::future::Future<Output=bool>`. This basically means
  each async fn is a anonymous type that implements `core::future::Future`
* `await` is turned be the compiler into a generated state machine that simply
  decomposes to all states (continues and returns) in a function using `Future`
  trait API. For a small example, see the code below:

```rust

async fn some_other_async_fn() {
  println!("Test");
}

async fn abc() -> bool {

  some_other_async_fn().await

  true
}
```

Will be turned into (pseudocode):

```rust

fn some_other_async_fn() -> impl core::future::Future<Output=()> {
  println!("Test");
  core::task::Poll::Ready(())
}

fn abc() -> impl core::future::Future<Output=bool> {

  let future = some_other_async_fn();
  let mut _hidden_satte = 0;

  if _hidden_satte == 0 {
    match future.poll(...) {
      core::task::Poll::Pending => return Pending
      core::task::Poll::Ready(_) => {
        ///
        _hidden_satte = 1;
      }
    }
  }

  /// And for next .awaits, continue the idea
  /// Compiler will ofc generate something totally different but thats the idea behind

  true
}

```

At the end, Rust SIG has decided to provide interface (traits) and compiler
syntactic sugar to support `async` but left the `runtime` being external needed
component that is not part of language.

#### Brief simplification how does Futures works

 Once creating own `Future`implementation using `core::future::Future` trait You
 are provided the `Context` that holds the `Waker`. The implementer
 responsibility for future is to return either `core::task::Poll::Pending` once
 Your Future is not ready yet or `core::task::Poll::Ready(_)` once Future is
 done wih work. In the first case, once You detected you are not ready, Your
 obligation is to:

* take `Waker` and store it somewhere
* return `core::task::Poll::Pending` (at this moment runtime will remove You
  from processing queue and will not process until woken)
* call `Waker::wake*` API once You know your Future can progress

After telling runtime via `Waker::wake*` that you are ready, `runtime` will
bring You back to some of its workers and again will execute `Future::poll` to
check Your progress.

### Connection to iceoryx2 messaging patterns

#### Events

Nothing to add, blocking API needs async versions

#### Publisher Subscriber

Here in async world (and even in non async really) as a user expectation would
be that I can react once a new `sample` is produced so that I don't have to poll
for it, instead simply `await` it.

#### Request Response

Here in async world (and even in non async really) as a user expectation would
be that there is one entity that is a "Server" (the one that hosts method and
produces responses) and there are clients that do requests. Due to this,
`request-response` would need `async` API to only act once there is request and
once there is reply for it.

## Requirements

* **R1: Async API look and feel** \* The new `async` API shall provide the same
  look and feel as standard one

## Use Cases

### Use-Case 1: Waiting for an event

* **As a** developer
* **I want** to wait on `event` API
* **So that** it does not block current thread and continues only once event is
  delivered

### Use-Case 2: Waiting for a new sample

* **As a** developer
* **I want** to wait on `new sample` in pub-sub
* **So that** it does not block current thread and continues only once sample is
  delivered

## Usage

### Example: Await on events

```rust
let node = NodeBuilder::new().create::<ipc_threadsafe::Service>().unwrap();

let event = node
    .service_builder(&"MyEventName".try_into().unwrap())
    .event()
    .open_or_create()
    .unwrap();

let listener = event.listener_builder().create_async().unwrap();

println!("Awaiting for Iceoryx event in batches while doing something else ...");

listener
    .wait_all(&mut |event_id| {
        print!("Received Iceoryx event: {:?}\n", event_id);
    })
    .await
    .unwrap();
```

## Implementation

### Achieving async API

Since all iceoryx2 messaging patterns (except `Event`) are poll based, we need
to pair them them with the `Event` to achieve possibility to react in `async`
API only on change of data. This means:

* `PubSub` should be paired with `Event`
* `RequestResponse` should be paired with `Event`
* and so on

Due to this, further document assumes direct usage of high level `Event`
messaging pattern to facilitate the feature.

> DISCLAIMER: It may be that iceoryx2 authors will have better idea than that.
> The only issue I do see now is probably impact on zero-trust deployment during
> configuration where ie. async PubSub Producer shall also have rights to be
> Event Notifier on some connected topic.

### Split of code

The main idea is to split source code into two parts:

1. The Event implementation that is both `OS` dependent and `runtime` dependent
    since it incurs some IO call
2. All the other API that do need `Event` implementation, but the rest is
    purely `runtime` independent and can be pure `async`

To facilitate above, below class diagram is showing one of the solution

![Class Diagram](new_classes.svg)

### Building objects

The next step is to provide a way to build objects that provide `async` API.

There are two approaches that can be chosen for implementation:

#### 1. Use custom event for each pair (messaging patern, data type) based on `ServiceName`

##### Pros

* all messaging pattern in service will work as usual
* no limitations

##### Cons

* dynamic Service creation to obtain event for each messaging pattern, like for
  `ServiceNameABC` (PubSub, int) we need to create also internally service
  `ServiceNameABC/__internal_pubsub_event` to obtain event for notifications

#### 2. Use event from the service `ServiceName`

##### Pros

* no need to create dynamic event name

##### Cons

* limit a service to only single messaging pattern as using event will cause no
  way to use it again

Considering above, continuation is done based on option 1. Below shows only
small snippet where extension for creating object can be placed.

![Port factory](port_factory.svg)

### Implementation

#### AsyncListener - `messaging-patter == event`

This is purely `runtime` specific implementation but is currently doable with
`non-blocking` api of `Listener`. Working example:
[Code sample](assets/event_example.rs) During implementation it may come
beneficial to either expose some properties (of current listener) or `add` new
sync api with different signature.

#### AsyncSubscriber - `messaging-patter == pubsub`

Pure `async` implementation not using any specifics of `runtime` shall be
doable. In case some unexpected dependency will be needed it has to be exposed
over defined abstraction same as `AsyncListenerTrait`

#### Builder

Implement the `Builders` extensions to they can provide `async` versions of objects

## Certification & Safety-Critical Usage

Answer:

* Applicable standards (e.g., ASIL-D, ISO 26262)
* Support for **zero-trust deployments**: can rogue processes break it?
* Evidence for claims (e.g., subscribers cannot corrupt publisher-owned
  read-only memory)
* Real-time suitability: any blocking calls, background threads, or
  indeterminism?
* If unsuitable for zero-trust or real-time use, how do we prevent accidental
  misuse?

PR: To me idea, we will build async API on top of existing non-blocking, non
async implementation and the API will be just a thin wrapper. The connection to
specific `runtime` is outside of project and in gesture of specific `runtime` to
guarantee any of above.

## Milestones

### Milestone 1 – Provide Traits and object skeletons

* TBD later

**Results:**

* User will see that there is ongoing work on `async` API

### Milestone 2 – Implement Event in Async Runtime

* TBD later

**Results:**

* User will get first support for async API for specific runtime. This will also
  open a way for other to implement a bridge to other runtime like `tokio` as
  basic idea will be shown

### Milestone 2 – Implement PubSub

* TBD later

**Results:**

* PubSub API will have `async` API available

## Extended examples

### Pub Sub pseudo code example

> NOTE: `async main` is the extension provided by runtimes, it simply wraps
> regular main into creation of runtime and put this into execution

#### Process 1

```rust
async fn main() {
  let node = NodeBuilder::new().create::<ipc_threadsafe::Service>().unwrap();

  let event = node
      .service_builder(&"MyEventName".try_into().unwrap())
      .event()
      .open_or_create()
      .unwrap();

  let listener = event.listener_builder().create_async().unwrap();
  let listener2 = event.listener_builder().create_async().unwrap();

  let task1_handle = spawn(async move {
    println!("Awaiting for iceoryx event in batches while doing something else ...");
    loop {

        listener
              .wait_all(&mut |event_id| {
                  print!("Received iceoryx event: {:?}\n", event_id);
              })
              .await // During this not being ready, worker can do any other work
              .unwrap();
       } 
  });


  spawn(async move {
   
    // Some logic
    // ..

    // now I need sample in this place due to my logic, so I simply 'await' it and once
    // I have a sample, this code will continue executing further
    let sample_res = listener2.wait().await; // During this not being ready, worker can do any other work

    // Process sample

    // ..
    
  });



  // Optionally You may wait until task finishes
  task1_handle.await.unwrap(); // During this not being ready, worker can do any other work
}
```

#### Process 2

```rust
async fn main() {
  let node = NodeBuilder::new().create::<ipc_threadsafe::Service>().unwrap();

  let event = node
    .service_builder(&"MyEventName".try_into().unwrap())
    .event()
    .open_or_create()
    .unwrap();

  let notifier = event.notifier_builder().create_async().unwrap();
  println!("Awaiting for Iceoryx event in batches while doing something else ...");

  loop {
    notifier.notify();
    sleep(100).await; // During that sleep, worker can be doing any other work ;)
  }
}
```
