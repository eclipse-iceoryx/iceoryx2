Hi Rustaceans,

I'm excited to announce the release of iceoryx2, a new inter-process zero-copy
framework and middleware written entirely in Rust.

This Rust implementation succeeds the C++ project iceoryx, offering a
significant speed increase and improved latency. Rest assured, iceoryx, the
framework you've come to rely on, remains an active player in the field. It
continues to receive consistent enhancements and is regularly updated with new
features. However, the future is looking mighty Rust-y with iceoryx2.

In its initial version, iceoryx2 supports publish-subscribe messaging and
efficient event transmission between processes. Notably, there's no longer a
reliance on a central broker, simplifying the setup process.

For those interested, our GitHub repository contains the roadmap. I welcome
your feedback, whether it's about missing features or your top two or three
desired functionalities.

Our current areas of focus include Mac OS platform support, a C binding, the
waitset, a reactor event-multiplexing abstraction, and the expansion of
messaging patterns with features like request-response and pipelines.

Additionally, for those acquainted with our demonstrator robot Larry,
anticipate a Rusty transformation as he eagerly ventures into the realm of
Rust.

The iceoryx developers invite you to explore the GitHub repository, engage in
discussions, and contribute to the development of iceoryx2.

Elfenpiff

Links:

* repo: https://github.com/eclipse-iceoryx/iceoryx2
* roadmap: https://github.com/eclipse-iceoryx/iceoryx2/blob/main/ROADMAP.md
* crates.io: https://crates.io/crates/iceoryx2
* docs.rs: https://docs.rs/iceoryx2/0.1.0/iceoryx2/
