# Best Practices

## Everything Can Be Questioned

* If you find a rule/best practice that does not make sense or seems not Rust
  idiomatic then please pack your suggestion in a pull request and we discuss
  and merge it.

## When To Use What Log Level

* **information dedicated to the developer of the application**
    * `TRACE` - interesting application events, e.g. whenever a resource is
    created/destroyed
    * `DEBUG` - only when the function returns a result that contains an error
* **information dedicated to the user of the application**
    * `INFO` - some interesting stuff for the user
    * `WARN` - warnings dedicated to the user of the application, the
    functionality is not restricted but some kind of internal recoverable
    misbehavior occurred
    * `ERROR` - some severe failure occurred that is still handled with a result
    containing an error but the application can continue if the user can recover
    from it
    * `FATAL` - the application panics after the log output

### Error Handling

* Never return `Err(...)`, always use `fail!` macro.
    * iceoryx2 shall always log a message to `DEBUG` whenever an `Err(...)` is.
    * When providing for instance `self` as origin, the current state of the
    object that caused the problem is logged.
        ```rust
        // bad
        impl MyStruct {
          fn do_stuff(&self) -> Result<u64, u64> {
            Err(123)
          }
        }

        // good
        use iceoryx2_bb_log::fail;

        impl MyStruct {
          fn do_stuff(&self) -> Result<u64, u64> {
            // uses Debug to print self
            fail!(from self, with 123, "Failed to do stuff!");
          }
        }
        ```

### Fatal Error Handling

* Never call `panic!(...)` directly, always use the `fatal_panic!` macro.
    * iceoryx2 shall always log a message to `FATAL` whenever a panic occurs.
    * When providing for instance `self` as origin, the current state of the
    object that caused the problem is logged.
        ```rust
        // bad
        impl MyStruct {
          fn whatever(&self) {
            panic!("whatever");
          }
        }
        
        // good
        impl MyStruct {
          fn whatever(&self) {
            // uses Debug to print self
            fatal_panic!(from self, "whatever");
          }
        }
        ```

* If a panic error check is expensive and located on the hot path then use
  `debug_assert!`.
    * Use it only when the panic check prevents API misuse and the misbehavior can
    be easily detected with unit tests
        ```rust
         // bad
         if expensive_check() {
             fatal_panic!("oh no ...");
         }
        
         // good
         debug_assert!(expensive_check(), "oh no ...");
        ```

### Re-Exports And Preludes

* The most common functionality of a construct shall be fully usable by only
  using:
    ```rust
    use my::construct::prelude::*;
    ```

* Use `pub use ...` to re-export requirements.
* Use preludes, see
  <https://doc.rust-lang.org/beta/reference/names/preludes.html>

### Documentation And Examples

* Use `?` operator in documentation examples to reduce error handling clutter.
* Use `#` to hide boilerplate code in documentation

### Self-Referencing Structs

* Use the `ouroboros` crate for self-referencing structs, required for
  non-movable types like `Mutex` & `MutexHandle`, `Barrier` & `BarrierHandle`,
  `UnnamedSemaphore` & `SemaphoreHandle`

### Testing

* Always use the `assert_that!` macro from `iceoryx2_bb_testing`

### Timing tests

* Never test a maximum runtime in a unit or integration test.
* Test at least runtimes with
  `assert_that!(start.elapsed(), time_at_least TIMEOUT)`
* Do not wait for events (indefinitely), use
  `assert_that!(|| { some_condition }, block_until_true)`
    * It starts a `iceoryx2_bb_testing::watchdog::Watchdog` in the background that
    terminates the test when it deadlocks.
* If the test can deadlock, instantiate a
  `iceoryx2_bb_testing::watchdog::Watchdog` in the beginning of the test.
