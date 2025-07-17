Logging
-------

.. toctree::

In this section, the log mechanisms of iceoryx2 are described in detail. We
begin by analyzing the structure of log messages and how they can be utilized
when debugging a process. Next, we describe the error-handling strategy, how to
log messages in general, and provide a table that outlines when to use each log
level. The final section explains how a user can implement a custom logger.

Understanding The Log Output
^^^^^^^^^^^^^^^^^^^^^^^^^^^^

In iceoryx2, a log message typically consists of two parts: the origin and the
actual message. The origin refers to the debug output of the object that
emitted the log message.

Including the origin is important because a backtrace, which points to a
specific line in the code, is often not enough. A typical setup might instantiate
many objects of the same type in different states, and a bug could be caused by
a subtle detail that affects a later statement.

Therefore, the full object state is added as the origin. In the following example,
we see the log output of the `FileBuilder`, the class responsible for constructing
a `File` object. By including the object's state, we gain valuable details about
the file that was opened for reading.

.. code-block::

   +--------- monotonic counter
   |  +------ log level (T = Trace, D = Debug, I = Info, W = Warn, E = Error, F = Fatal)
   |  |  +--- origin: the state (debug output) of the object that emitted the log
   |  |  |            message
   |  |  |                                the actual log message ----------------+
   |  |  |                                                                       |
   |  |  |                                                                       |
   0 [T] FileBuilder { file_path: FilePath { value: FixedSizeByteString<255> {   |
         len: 20, data: "config/iceoryx2.toml" } }, access_mode: Read,           |
         permission: Permission(448), has_ownership: false, owner: None,         |
         group: None, truncate_size: None, creation_mode: None }                 |
         | opened ---------------------------------------------------------------+

How To Handle Errors
^^^^^^^^^^^^^^^^^^^^

In this section, we describe the error handling of internal iceoryx2 Rust code.
The language bindings may diverge from this approach when it does not align
with the idioms of the respective language.

Non-Recoverable Errors
""""""""""""""""""""""

Non-recoverable errors describe scenarios where the program cannot continue and
would either encounter undefined behavior or crash in the next statement. For
example, this can occur when accessing an out-of-bounds array index or when a
corruption in the underlying OS is detected due to illegally modified managed
resources.

In these cases, the only viable option is to terminate the program and provide
a detailed error message.

Only the `fatal_panic!` macro should be used for such scenarios; **never** use
Rust's default `panic!` macro. While `fatal_panic!` utilizes `panic!` under the
hood, it also adds a final log message to the logger to inform the user of the
impending crash.

.. code-block:: Rust

    // call from within an object
    fn do_stuff(&self) {
        // self must implement Debug
        fatal_panic!(from self, "Something horrible has happened!");
    }

    // from a free function
    fatal_panic!(from "some_context_here", "Something horrible has happened!");

    // only fail when a function fails
    fn do_stuff() -> Result<(), SomeError>;
    fatal_panic!(from self, when do_stuff(), "Something horrible has happened!");

Recoverable Errors
""""""""""""""""""

Recoverable errors occur when something fails but with appropriate error
handling, the program can continue. For instance, attempting to open a file
without sufficient permissions would result in a recoverable error. In such
cases, the function would return an error wrapped inside a `Result`.

In iceoryx2, we follow the rule that every `Err(..)` must be accompanied by a
debug log message, ensuring a complete error trace for easier debugging.

To enforce the correct log level and message, iceoryx2 introduces the `fail!`
macro, which requires the origin, a log message, and optionally the error value.
**Never** use a combination of `return Err(..);` with a separate `debug!` log
message.

.. code-block:: Rust

   // call from within an object
   fn do_stuff(&self) -> Result<(), SomeError> {
        // self must implement Debug
        fail!(from self, 
              with SomeError::WhatEver, 
              "An error has occurred.");
   }

    // from a free function
    fn do_stuff() -> Result<(), SomeError> {
        fail!(from "some_context_here", 
              with SomeError::WhatEver, 
              "An error has occurred.");
    }

    // only fail when function fails, otherwise use value
    fn call_me() -> Result<i32, SomeError>;
    fn do_stuff() -> Result<(), SomeError> {
        let number = fail!(from "some_context_here", 
              when call_me(),
              "An error has occurred.");
    }

A more complex example might involve a user trying to open a service, where the
following sequence of events happens under the hood:

1. The static configuration file of the service is opened.
   - Trace log message.
2. The deserialization fails because a field is missing.
   - Debug log message informing the next layer about the issue.
3. The `ServiceBuilder` fails because the static details exist but cannot be read.
   - Debug log message informing the user that the service appears to be in a corrupted state.

This sequence allows us to trace exactly why the service is in a corrupted state,
specifically due to a deserialization failure. The following snippet demonstrates
a case where the file
`/tmp/iceoryx2/services/iox2_4eacadf2695a3f4b2eb95485759246ce1a2aa906.service`
of the service `My/Funk/ServiceName` cannot be deserialized because the field
`max_subscribers` is missing.

Had we only relied on a stack trace pointing to the lines of code where the log
messages originated, we wouldn't have known which service was affected or which
underlying file had the issue.

.. code-block::

    12 [T] FileBuilder { file_path: FilePath { value: FixedSizeByteString<255> { 
           len: 76, data: "/tmp/iceoryx2/services/iox2_4eacadf2695a3f4b2eb954857
           59246ce1a2aa906.service" } }, access_mode: Read, 
           permission: Permission(448), has_ownership: false, owner: None, 
           group: None, truncate_size: None, creation_mode: None }
           | opened
    13 [D] "Toml::deserialize"
           | Failed to deserialize object (TOML parse error at line 5, column 1
           |
         5 | [messaging_pattern]
           | ^^^^^^^^^^^^^^^^^^^
         missing field `max_subscribers`
         ).
    14 [D] BuilderWithServiceType { service_config: StaticConfig { service_id: 
           ServiceId(RestrictedFileName { value: FixedSizeByteString<64> { len: 
           40, data: "4eacadf2695a3f4b2eb95485759246ce1a2aa906" } }), 
           service_name: ServiceName { value: "My/Funk/ServiceName" }, .... 
           | Unable to deserialize the service config. Is the service corrupted?

How To Log - What LogLevel To Use When
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

======== ========= ======================================================================================================
LogLevel Recipient Description
======== ========= ======================================================================================================
Trace    Developer For interesting application events, e.g., whenever a resource like a file is created or destroyed.
Debug    Developer Only used when a function that returns a `Result` encounters an error.
Info     User      For messages that are relevant to the user, such as "ready to perform tasks."
Warn     User      When functionality is restricted due to a recoverable error that has been mitigated.
Error    User      For severe failures where parts of the application cannot operate and no mitigation is possible.
Fatal    User      The last message before the application crashes.
======== ========= ======================================================================================================

Each log level has its corresponding macro and can be used in a similar manner
to the `fail!` and `fatal_panic!` macros. Below, we demonstrate the usage with
the `trace!` log macro. The `debug!`, `info!`, `warn!`, `error!`, and `fatal_panic!`
macros work in an identical way.

.. code-block:: Rust

    // call from within an object
    fn do_stuff(&self) {
        // self must implement Debug
        trace!(from self, "Something horrible has happened!");
    }

    // from a free function
    trace!(from "some_context_here", "Something horrible has happened!");

    // only log when a function fails
    fn do_stuff() -> Result<(), SomeError>;
    trace!(from self, when do_stuff(), "Something horrible has happened!");

Custom Logger
^^^^^^^^^^^^^

iceoryx2 allows users to set up their own custom logger. All approaches have two
things in common: the logger can only be set once, and once set, it will remain
the active logger until the process exits.

Additionally, the logger must be set before the first log message is created.
If log messages have already been generated, the default logger is automatically
set and can no longer be changed.

To integrate iceoryx2 with other Rust libraries, iceoryx2 provides support for
`log <https://crates.io/crates/log>`_ and
`tracing <https://crates.io/crates/tracing>`_. This can be enabled by using
either the `logger_log` or the `logger_tracing` feature flags.

.. code-block:: Toml

    # Cargo.toml
    [dependencies]
    iceoryx2 = { version = "0.4", features = ["logger_log"] }

Language: C
"""""""""""

The C API provides the function `iox2_set_logger(iox2_log_callback logger)`, where
`iox2_log_callback` is a function pointer with the signature 
`void (*iox2_log_callback)(enum iox2_log_level_e, const char* origin, const char* message)`.

The following code snippet demonstrates how to implement a simple `printf` logger:

.. code-block:: C

    #include "iceoryx2.h"

    void custom_logger(enum iox2_log_level_e, const char* origin, const char* message) {
        printf("origin: %s, message: %s\n", origin, message);
    }

    int main() {
        if ( !iox2_set_logger(custom_logger) ) {
            printf("Failed to set logger\n");
        }
    }

Language: C++
"""""""""""""

The C++ API provides the function `auto set_logger(Log& logger) -> bool`. Users
must provide a custom logger that implements the `Log` interface. This logger can
be attached, but it requires a static lifetime to ensure logging during the
application's shutdown phase.

The following code snippet demonstrates how to implement a simple `std::cout` logger:

.. code-block:: C++

    #include "iox2/log.hpp"

    class CoutLogger : public iox2::Log {
      public:
        void log(LogLevel log_level, const char* origin, const char* message) override {
            std::cout << "origin: " << origin << ", message: " << message << std::endl;
        }
    };

    int main() {
        static CoutLogger LOGGER;

        if ( !iox2::set_logger(&logger) ) {
            std::cerr << "Failed to set logger" << std::endl;
        }
    }

Language: Rust
""""""""""""""

The Rust API provides the function 
`pub fn set_logger<T: Log + 'static>(value: &'static T) -> bool`. 
A custom logger must implement the `Log` trait and requires a static lifetime to 
ensure logging during the application's shutdown.

The following code snippet demonstrates how to implement a simple `println!` logger:

.. code-block:: Rust

    use iceoryx2_bb_log::{set_logger, Log, LogLevel};
    use std::sync::LazyLock;

    #[derive(default)]
    struct PrintLogger {}

    impl Log for PrintLogger {
        fn log(&self, log_level: LogLevel, origin: core::fmt::Arguments, message: core::fmt::Arguments ) {
            println!("log level: {:?}, origin: {}, message: {}",
                    log_level, origin.to_string(), message.to_string());
        }
    }

    static LOGGER: LazyLock<PrintLogger> = LazyLock::new(|| PrintLogger::default());

    fn main() {
        if !set_logger(&*LOGGER) {
            println!("Failed to set logger");
        }
    }

