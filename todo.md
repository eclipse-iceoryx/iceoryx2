# TODO

## POSIX

* Windows stat.rs, allowed to fail if file no longer exists
* Cargo nextest run -p iceoryx2 --test service_tests gracefully
    * Service should not be in corrupted state when listed and currently being
    removed

## Bugs

* `posix::Mutex`
    * Implement priority ceiling
* File::read_to_string & File::read_range_to_string, check that there are no
  invalid UTF-8 characters contained
* Verify ACL_LIST_CAPACITY in POSIX_config
* Unix datagram socket, sending credentials and file descriptors no longer works
    * Was introduced with Rust 1.71.0
    * Activate unit test below again fn
    `unix_datagram_socket_sending_receiving_with_max_supported_fd_and_credentials_works`
* `MAX_NUMBER_OF_ENTRIES` in shared_memory_directory/mod.rs can lead to stack
  overflow when too large in
  `unsafe { files.write(FileReferenceSet::default()) };` We require placement
  new!
  ```rust
  ptr->write(MaybeUninit<MyThing>::new());
  (ptr as *mut MaybeUninit<MyThinkg>)->do_stuff_to_Initialize_yourself();
  ```
* Better interrupt signal handling. The interrupt signal shall be always
  propagated up to the user for better CTRL+C handling etc.
* `condition_variable` has spurious wakeups on windows since WaitOnAddress with
  a timeout does not really work. Remove ignore for windows from condition
  variable tests and fix problem.
* `CTRL+c` does not work in windows, all examples do not clean up
* `Directory`, `Path`, `FileName`, `FilePath` cannot handle UTF-8 file names
    * `Directory` panics when a directory contains non-ascii (UTF-8) characters
* Windows: `publisher::zero_copy::publisher_block_when_unable_to_deliver_blocks`
  seems to sometimes deadlock

## Incomplete

* Document
    * Service
* ?is publisher history tested?
* Test all QoS of all services, also with requirements when opening
* Write tests for `Sample` & `SampleMut`
* Fix platforms
    * macOS
        * `iceoryx2_bb_posix`
            * Add timeout to `pthread_cond_timedwait`
        * Use concepts based on shared memory directory to circumvent to 30 char
      restriction on the shm name in macOS
    * Use second shm for shm size storage of main shm in windows instead of file
* Stale services when creator goes out of scope
    * Add ability to create persistent service
        * Add persistency flag in static storage
    * Simplify Service (zero_copy/process_local)
        * `ServiceState` has new/drop to check for stale services
    * When `OpenDynamicStorageFailure::IsMarkedForDestruction` activated then wait
    for some time and retry to create the service, only if open or create was
    used
* Event Messaging Pattern
    * Introduce degradation callback for notifier, used when a connection to a
    listener could not be Established
    * Make `Publisher::update_connections` private
* Rename PortFactory<...> into Service<...> and make it easier to access by the
  user
    * ability to store it in vec etc
* Splitup `NamedConceptBuilder` into `NamedConceptOpener` and
  `NamedConceptCreator`
    * `has_ownership` only for creator
    * `has_ownership`/`release_ownership` should be part of the NamedConceptMgmt
    * Introduce `acquire_ownership` in `NamedConceptMgmt`
    * Make Windows shm persistent first
* Event concept
    * `process_local` should be based on a bitset, introduce max trigger id into
    event/notifier \* Implement atomic bitset, with compile time capacity,
    atomic set and atomic exchange consume via callback
    * ?add ability to create Listener in a different process and open it later,
    excludes every socket variant?
* `KeepAlive` of `Subscriber`/`Publisher`
    * Set `Publisher`/`Subscriber` timing contract, if violated
    `Subscriber`/`Publisher` can close connection
* `ZeroCopyConnection`
    * Remove `blocking_send`
    * The builder shall be able to either create a connection or open a sender or
    receiver
* `CommunicationChannel`
    * Remove safe_overflow support
    * ?add ability to create receiver in a different instance and open it later,
    excludes every socket variant?
* Rename `zero_copy::Service` into `ipc::Service`, -> ??`intra_process::Service`
  for `process_local`??
* Consolidate name- and config-generation functions from
  `ipc/src/port/publisher.rs` `ipc/src/service/mod.rs`
  `{dynamic|static}_config_storage_{name|config} {connection}\_{name|config}`
* Cargo.toml, add version inheritance from root workspace
* FreeBSD
    * Advanced signal handling -
    `signal_call_and_fetch_with_registered_handler_works`
* Windows
    * Back shm in windows with file
* POSIX thread wrapper uses currently heap when creating new thread with
  `pthread_create`
    * Introduce global mempool allocated on program start for this operation
* Split up `UnixDatagramSocket` into one with ancillary data and one without
    * Add to builder if credentials shall always be sent
    * Check if fd exchange with messages is supported
    * Check if multiple fds can be exchanged with one message
* Check all POSIX wrapper that creation/open happens in the builder to avoid
  invalid states in objects
* Ensure non-movability via separate handle + remove is_initialized and
  construct in builder
    * Condition variable
        * Remove IPC support for multi condition variable
* Refactor error handling
    * Error pyramid concept
    * Extend and test `fail!` macro
* `ZeroCopyConnection`, add builder option to enable tracking of samples from
  sender to receiver and back
* Add this to port factory publisher builder
    * TEST
        * Remove all unwrap and test them
* Refactor `SampleMut`, `TypeStatePattern` - only the initialized sample can be
  sent
    * Release underlying memory manually and sent/release it manually
* Subscriber strategies for multi-publisher
    * priority?
    * timestamp?
    * random order?
* Refactor `global_config`
* `RelocatableContainer` test which verifies that init does not write/modify
  more memory then it requires according to `memory_size()`
* Rename `CommunicationChannel` into `NamedPipe`
* Subscriber internal cache which orders samples from multiple publishers
* `iceoryx2_ipc` finalize
    * Remove all `unwrap()`
    * `global_config.rs` use correct string types
        * Introduce file_suffix_name.rs and use it there
    * Remove all string usages and replace it with semantic_string impls
* Introduce `BasePort`, contains always
    * `portId` : `UniquePortId`
    * `dynamic_config_guard`
    * `fn new(service: &Service) -> Self`
* Rename things
    * With modules, every public thing should be named like the module, it will be
    differentiate with the namespace \* All `iceoryx2_cal`s
* What if `reference_counter == 0` when creating new service ... wait for
  cleanup
* Introduce `ShmIpc` trait and derive macro to add `#[repr(C)]` to all shm
  transferable types
    * Introduce typed shared memory to avoid this problem
        * Use it in comm channel
    * Management in `communication_channel_shm_index_queue`
    * `IndexQueue`
    * `UnnamedSemaphore`
    * All the other things in POSIX/etc
* Use Rust singleton pattern for signal handler
  <https://docs.rust-embedded.org/book/peripherals/singletons.html>

## Default Setup - without configuration & Grouping Support

* Goal: multiple concepts share the same resources
    * ?introduce second name in NamedConcept, the group as FileName?
* Create dynamic shared memory abstraction (below concept layer)
    * Must be able to dynamically resize (grow and shrink)
    * Requires sophisticated allocator that can handle arbitrary sizes
    * Must be emulate some kind of file system
        * Idea is that the concept can use this construct to create and open
      specified instances via name
* Use dynamic shared memory

## Performance

* `mpmc_container`, `mpmc_unique_index_set`, try to avoid constant
  `verify_memory_initialization` call

## Important Questions

* Free list approach, what happens when publisher goes out of scope before
  subscriber does?
* `RelocatableContainer`, make this trait somehow unmovable.
* `ServiceStaticConfig`, pack individual communication pattern configs into
  `MessagingPattern`
* `Performance` and `Safety Critical` mode
    * Default clock is in performance mode Realtime
* Revisit concept abstraction trait structure?
    * Maybe: `concept_name.rs`
        * Contains trait and all impls as pubs under same namespace
* `DynamicStorage`
    * Restrict `T` to
    <https://doc.rust-lang.org/std/intrinsics/fn.needs_drop.html> non dropable

## Advanced

* Make relative_ptr/OffsetRepository lock-free
* Introduce deadline timer
* Bitset enums like iceoryx2_bb_POSIX::Permission require const construction
  with bitset operations
* Make POSIX shared memory resizable with `mremap` & `ftruncate`, see:
  <https://stackoverflow.com/questions/40398901/resize-posix-shared-memory-a-working-example>
* Use `Kani`, `loom` and `miri` for lock-free code testing

## Containers

* `FilePath`/`Path` check that every single entry is not longer than
  `FILENAME_LENGTH`
* Implement `StackBox` where the box max size is provided as generic user
  argument
    * Use it in allocator Memory cleanup function
* Introduce string
* Introduce vector
* Introduce list
* Introduce red-black-tree
* Introduce set
* Introduce hashmap

## Concurrent

* Lock-free list
    * Use unique index set as base
* Introduce `ThreadPool`
    * Introduce `ActiveObject`

## IPC

* POSH/IPC/service
    * Unique global service list file
        * New service must add entry there, on remove, remove entry

## Open Problems

* `iceoryx2_config` use lock free singleton approach from `iceoryx2`
* Remove `ENAMETOOLONG` and replace it with a path/filename string with maximum
  size
    * See `unlink`, `does_file_exist`
* `Semaphore`
    * Implement `timedwait` with GNU libc `sem_clockwait`
* `ReadWriteMutex`
    * Implement `timedwait` with GNU libc `pthread_rwlock_clockrdlock`.
    `pthread_rwlock_clockwrlock`
* Publisher delivers history only when calling send
    * New subscriber may waits a long time when send is not called
