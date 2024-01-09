# Brace Yourself Rust Is Coming - Announcing iceoryx2

I'm thrilled to unveil a significant milestone in the development of iceoryx
and to announce the release of iceoryx generation 2, crafted entirely in
Rust. As one of the main developers of iceoryx, this journey commenced over a
year ago as a personal side project.

Iceoryx2, or iceoryx generation 2, is poised to inherit all the familiar
features you've come to rely on from its predecessor. In its initial release,
expect support for publish-subscribe, service discovery, and dedicated event
messaging — all without the need for a central broker (RouDi).
Furthermore, iceoryx2 is designed to seamlessly run on Linux, FreeBSD, and Windows.

Our ongoing efforts extend beyond this initial release. We are actively
developing request-response messaging, refining the waitset, and expanding
platform support to include macOS and Android. While these exciting features
are in the pipeline, their official integration is slated for the coming year
due to time constraints.

One of the most notable enhancements with iceoryx2 lies in its performance over
iceoryx, demonstrating a significant increase across various benchmarks.
Explore the detailed [benchmarks]() to witness the tangible improvements.

However, the true gems of this upgrade manifest through the adoption of Rust.
Switching to Rust has unlocked several advantages that I find particularly
compelling:

- **Comprehensive Documentation:** Thanks to `cargo doc`, iceoryx2 boasts a
    consistent and detailed documentation, enriched with numerous examples.

- **Enhanced Safety:** The Rust language inherently provides a safer API,
    eliminating common pitfalls such as lifetime and hidden concurrency issues
    — a challenge often encountered in C++.

- **Simplified Certification:** From my perspective, certifying the code base
    becomes more straightforward with Rust. The language effectively addresses
    numerous challenges inherent in C++, contributing to a more reliable and
    secure version of iceoryx.

As we step into this new era with iceoryx generation 2, powered by Rust, we
invite you to explore the enhanced capabilities and witness firsthand the
positive impact on both performance and reliability. Your feedback is
invaluable as we continue to refine and expand iceoryx2 to meet the evolving
needs of our user community.

## Why Rust

Over a year ago, I embarked on a journey to learn Rust — a language heralded for its
emphasis on memory safety, concurrency, and reliability. The catalyst for this
exploration was the growing industry consensus favoring memory-safe languages, as
highlighted by the NSA's endorsement of
[Software Memory Safety](https://media.defense.gov/2022/Nov/10/2003112742/-1/-1/0/CSI_SOFTWARE_MEMORY_SAFETY.PDF)
and major companies such as Amazon advocating for
[Sustainability with Rust](https://aws.amazon.com/blogs/opensource/sustainability-with-rust/).

In the ever-evolving landscape where Microsoft embraces Rust for
[writing drivers](https://www.golem.de/news/entwicklung-microsoft-legt-rust-framework-fuer-windows-treiber-offen-2309-177932.html)
and the industry witnesses a paradigm shift away from C/C++ to Rust, it is worth
noting that Rust's commitment to safety is not merely a claim; its safety features can
be [rigorously proven](https://research.ralfj.de/phd/thesis-screen.pdf).
This substantiates Rust as a reliable and secure choice and raises the
question:

Why does the automotive domain lag behind?

Enter iceoryx2 — the fruition of my efforts to reimagine iceoryx from the ground up,
exclusively in Rust. This venture aimed not only to comprehend the language but also
to evaluate whether Rust truly lived up to its promises. The resounding answer is a
definitive "yes," and more.

My day job involves certifying iceoryx written in C++ for use in safety-critical
ASIL-D environments. While grappling with algorithmic challenges, the burden of C++
introduces a myriad of additional complications, consuming considerable time and
causing frustration.

- **Documentation:** Managing doxygen code examples in harmony with the codebase.
- **Lifetimes:** Navigating lifetime dependencies when working with RAII and factories.
- **Concurrency:** Preventing inadvertent multi-threaded usage and certifying
    concurrent and lock-free code.
- **Build-System:** Handling dependencies and third-party packages seamlessly.
- **Templates:** Certifying generic C++ code for various but valid types.
- **Static Code Analysis:** Choosing the right tools and adhering to Misra,
    Autosar, C++ Core Guidelines, or a combination.

Why dwell in the abstract when we can scrutinize a real-world example, comparing
iceoryx (C++) to iceoryx2 (Rust)? Brace yourself for a journey into a future where
Rust takes the lead in inter-process zero-copy communication within our specialized
domain. Iceoryx2 is not just an upgrade — it's a leap forward into a safer, more
efficient era.

## Example: Sending Data

In both iterations of iceoryx, the communication revolves around ports as endpoints.
In this context, the `Publisher` assumes the role of the sender in a publish-subscribe
messaging pattern. When users intend to transmit data, they typically follow a set
of common steps across both versions.

1. Invoke `Publisher::loan()` to obtain a `Sample` for storing the data to be sent.
2. Utilize `Publisher::send(sample)` to dispatch the data to all receiving endpoints (`Subscriber`).

Represented here is a simplified interface for this `Publisher`:

```cpp
// C++
class Publisher {
  public:
    Sample loan();
    void send(Sample &&sample);
};
```

```rust
// Rust
struct Publisher {}

impl Publisher {
    fn loan(&'publisher self) -> Sample<'publisher> {}
    fn send(&self, sample: Sample) {}
}
```

## Documentation Best Practices: C++ vs. Rust

When it comes to documenting code and ensuring the accuracy of code examples, both C++
and Rust offer distinct approaches. Let's delve into the practices and tools used in
each language:

### C++

In the C++ realm, internal documentation is commonly managed using Doxygen, a tool
providing `@code`/`@endcode` tags for embedding code examples. However, there's a
crucial limitation — Doxygen does not verify whether the code is running or compiling.
This introduces a potential risk, as refactoring in one part of the codebase may
inadvertently break examples located elsewhere.

```cpp
/// @code
///   std::cout << "hello world, without include iostream";
/// @endcode
class Publisher {};
```

One might consider compiling examples during the build process and then
embedding code snippets from them into the Doxygen `@code` section. However, this
workaround can be cumbersome, particularly when striving to provide examples for every
method.

### Rust

In the Rust ecosystem, documentation is ingrained in the language specification,
adopting markdown syntax. Code examples are not only part of the documentation but are
actively built and tested through Rust's build system, `cargo`.

```rust
/// # Examples
/// ```
/// println!("hello world");
/// ```
struct Publisher {}
```

What sets Rust apart is the execution of documentation examples during testing
(`cargo test --doc`). This means that not only is the code compiled, but its
functionality is verified. This proactive approach ensures that any internal changes
or refactoring issues are promptly exposed.

Moreover, Rust allows for the incorporation of contracts directly into code examples
using the `assert!` macro, adding an extra layer of visibility and validation.

```rust
/// ```
/// // the doc test fails if the assertion does not hold
/// assert!(2 + 2 == 4);
/// ```
struct Publisher {}
```

In summary, Rust's documentation practices, combined with built-in testing, provide
out-of-the-box robust and reliable means of ensuring code example correctness compared
to C++'s Doxygen-based approach. This not only enhances the clarity of documentation
but also strengthens the overall integrity of the codebase.

## Lifetimes And Accidental Concurrency

Examining our earlier example, we find that the `Sample` returned by the
`Publisher` represents a memory resource susceptible to leaks if mishandled.
To address this, both C++ and Rust leverage the
[RAII](https://en.wikipedia.org/wiki/Resource_acquisition_is_initialization)
idiom, where the class/struct serves as the resource owner, ensuring that the
memory is released when the object goes out of scope.

However, subtle issues arise:

1. What happens when the `Publisher` goes out of scope before the `Sample`?
2. What if the `Sample` is moved into another thread and goes out of scope,
   leading to accidental concurrent access of the `Publisher`?

### C++

In C++, the lifetime issue can be tackled by using a `std::shared_ptr`,
where the `Publisher` owns an inner construct, and the `Sample` holds a copy of
that shared pointer. This way, the inner construct is removed when there are no
more owners. In our domain, constraints on using the heap or most STL constructs
add complexity, but solutions involving reference counting and external memory
locations are viable, albeit with drawbacks.

Moving a `Sample` to another thread, causing concurrent use of the `Publisher`,
demands more careful consideration. Options include making the `Publisher`
universally thread-safe (with potential performance impact) or a runtime
detection in the `Sample` destructor using a thread-local variable.
Unfortunately, both solutions introduce performance overhead and necessitate
thorough concurrent stress testing.

The least robust option, which I would advise against in a safety-critical
library, is to rely solely on documenting the restrictions. However, this
approach raises pertinent questions:

- Will every developer diligently read and adhere to the documentation before
  utilizing a function?
- Can we trust reviewers to thoroughly examine the documentation of all
  functions they review?
- In the dynamic environment of code evolution, will developers be cognizant
  of all affected code when contracts change?

In my view, the answer to all three questions tends to be "no." Unless
addressed by automated tools, relying solely on documentation risks inevitable
mistakes.

Consequently, all available solutions present trade-offs: they may introduce a
performance hit, entail implementation overhead, or result in an API that is
susceptible to misuse when documentation is not meticulously reviewed.

### Rust

Rust, with its strong focus on safety, resolves these challenges at the compiler
level. By explicitly adding the `'publisher` lifetime in the function declaration,

```rust
    fn loan(&'publisher self) -> Sample<'publisher> {}
```

we inform the compiler that the `Sample` can exist at most as long as the
`Publisher`. If a user violates this contract, attempting to destroy the
`Publisher` while holding a `Sample`, the compiler prevents the program from
compiling — providing an optimal solution without added overhead.

Accidental concurrency issues, such as moving a `Sample` to another thread, are
also impossible in Rust. Objects intended for thread movement must implement
the `Send` trait, which is not implemented by default. Any attempt to violate
this contract is again caught by the compiler during compilation, demonstrating
Rust's ability to handle such problems at compile time without introducing
performance or implementation overhead.

## Future

This leads me back to the initial question: Why does the automotive domain lag
behind?

A mere year ago, when I embarked on the development of iceoryx2, the absence of
a certified Rust compiler posed a critical barrier. Without this fundamental
component, certifying code in the automotive domain seemed nearly insurmountable.
Additionally, projects within the automotive sector often span years and
frequently involve interactions with legacy C and C++ code bases—essential
factors that cannot be overlooked.

Today, however, the landscape has evolved significantly. We now have access to
a [certified Rust compiler](https://ferrous-systems.com/ferrocene/). Rust
facilitates seamless invocation of functions from C code bases, thanks to
[bindgen](https://github.com/rust-lang/rust-bindgen), and although incorporating
C++ code poses some challenges, it is achievable. Moreover, the reverse is also
true — integrating a Rust library into an existing C/C++ code base and invoking
[Rust functions within C](https://doc.rust-lang.org/nomicon/ffi.html) is feasible.

In my estimation, the window of opportunity has arrived to take the next stride and
emerge as pioneers in successfully certifying code for ISO 26262 (ASIL D), with
iceoryx2 as the ideal project for this endeavor. While this undertaking will
undoubtedly require time, particularly given the intricacies of inter-process
zero-copy communication, I approach it with confidence. Drawing on my experience
certifying C++ code and the insights gained from rewriting iceoryx in Rust, I
believe we can navigate these challenges more efficiently.

Embracing Rust not only accelerates the development of new software components
but also reduces costs, courtesy of the modern safety and developer features it
offers. This, in turn, empowers our users to write superior code swiftly,
leading to substantial cost savings.

I invite you to follow our journey, staying tuned until the day I pen another
blog article proudly announcing the safety certification of iceoryx2.
