# QNX Platform Abstraction

## Limitations

### Thread Affinity

QNX does not implement the non-standard POSIX APIs for working with thread affinity:

1. `pthread_attr_setaffinity_np`
1. `pthread_setaffinity_np`
1. `pthread_getaffinity_np`

Instead, a different mechanism via [`ThreadCtl`](https://www.qnx.com/developers/docs/7.1/index.html#com.qnx.doc.neutrino.prog/topic/multicore_processor_affinity.html
) is available, however its behaviour does not map directly to the above calls.

Some additional effort is required to integrate this syscall into the platform
in such a way that:

1. It does not require changes to code in upper layers
1. It does not leave the system in an inconsistent state

Until this is done, setting thread affinity on QNX is disabled.
