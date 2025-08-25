// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

use crate::posix::*;

pub unsafe fn pthread_rwlockattr_setkind_np(_attr: *mut pthread_rwlockattr_t, _pref: int) -> int {
    // Not supported on QNX
    crate::internal::EOK as _
}

pub unsafe fn pthread_barrier_wait(barrier: *mut pthread_barrier_t) -> int {
    crate::internal::pthread_barrier_wait(barrier)
}

pub unsafe fn pthread_barrier_init(
    barrier: *mut pthread_barrier_t,
    attr: *const pthread_barrierattr_t,
    count: uint,
) -> int {
    crate::internal::pthread_barrier_init(barrier, attr, count)
}

pub unsafe fn pthread_barrier_destroy(barrier: *mut pthread_barrier_t) -> int {
    crate::internal::pthread_barrier_destroy(barrier)
}

pub unsafe fn pthread_barrierattr_destroy(attr: *mut pthread_barrierattr_t) -> int {
    crate::internal::pthread_barrierattr_destroy(attr)
}

pub unsafe fn pthread_barrierattr_init(attr: *mut pthread_barrierattr_t) -> int {
    crate::internal::pthread_barrierattr_init(attr)
}

pub unsafe fn pthread_barrierattr_setpshared(
    attr: *mut pthread_barrierattr_t,
    pshared: int,
) -> int {
    crate::internal::pthread_barrierattr_setpshared(attr, pshared)
}

pub unsafe fn pthread_attr_init(attr: *mut pthread_attr_t) -> int {
    let result = crate::internal::pthread_attr_init(&mut (*attr).native);
    if result != crate::internal::EOK as _ {
        return result;
    }
    (*attr).affinity_mask = None; // indicates no specified affinity
    result
}

pub unsafe fn pthread_attr_destroy(attr: *mut pthread_attr_t) -> int {
    crate::internal::pthread_attr_destroy(&mut (*attr).native)
}

pub unsafe fn pthread_attr_setguardsize(attr: *mut pthread_attr_t, guardsize: size_t) -> int {
    crate::internal::pthread_attr_setguardsize(&mut (*attr).native, guardsize)
}

pub unsafe fn pthread_attr_setinheritsched(attr: *mut pthread_attr_t, inheritsched: int) -> int {
    crate::internal::pthread_attr_setinheritsched(&mut (*attr).native, inheritsched)
}

pub unsafe fn pthread_attr_setschedpolicy(attr: *mut pthread_attr_t, policy: int) -> int {
    crate::internal::pthread_attr_setschedpolicy(&mut (*attr).native, policy)
}

pub unsafe fn pthread_attr_setschedparam(
    attr: *mut pthread_attr_t,
    param: *const sched_param,
) -> int {
    crate::internal::pthread_attr_setschedparam(&mut (*attr).native, param)
}

pub unsafe fn pthread_attr_setstacksize(attr: *mut pthread_attr_t, stacksize: size_t) -> int {
    crate::internal::pthread_attr_setstacksize(&mut (*attr).native, stacksize)
}

pub unsafe fn pthread_attr_setaffinity_np(
    attr: *mut pthread_attr_t,
    _cpusetsize: size_t,
    cpuset: *const cpu_set_t,
) -> int {
    (*attr).affinity_mask = Some(*cpuset);
    crate::internal::EOK as _
}

// TODO (#965): Properly use ThreadCtl to ensure the created thread has the desired affinity
pub unsafe fn pthread_create(
    thread: *mut pthread_t,
    attr: *const pthread_attr_t,
    start_routine: unsafe extern "C" fn(*mut void) -> *mut void,
    arg: *mut void,
) -> int {
    crate::internal::pthread_create(thread, &(*attr).native, Some(start_routine), arg)
}

pub unsafe fn pthread_join(thread: pthread_t, retval: *mut *mut void) -> int {
    crate::internal::pthread_join(thread, retval)
}

pub unsafe fn pthread_self() -> pthread_t {
    crate::internal::pthread_self()
}

pub unsafe fn pthread_setname_np(thread: pthread_t, name: *const c_char) -> int {
    crate::internal::pthread_setname_np(thread, name)
}

pub unsafe fn pthread_getname_np(thread: pthread_t, name: *mut c_char, len: size_t) -> int {
    crate::internal::pthread_getname_np(thread, name, len as int)
}

pub unsafe fn pthread_kill(thread: pthread_t, sig: int) -> int {
    crate::internal::pthread_kill(thread, sig)
}

pub unsafe fn pthread_setaffinity_np(
    thread: pthread_t,
    _cpusetsize: size_t,
    cpuset: *const cpu_set_t,
) -> int {
    if thread != pthread_self() {
        panic!("setting affinity for other active threads is not supported on QNX");
    }
    let mut threadctl = match internal::ThreadCtl::init() {
        Ok(threadctl) => threadctl,
        Err(code) => return code,
    };
    threadctl.set_runmask(cpuset)
}

pub unsafe fn pthread_getaffinity_np(
    thread: pthread_t,
    _cpusetsize: size_t,
    cpuset: *mut cpu_set_t,
) -> int {
    if thread != pthread_self() {
        panic!("getting affinity for other active threads is not supported on QNX");
    }
    let mut threadctl = match internal::ThreadCtl::init() {
        Ok(threadctl) => threadctl,
        Err(code) => return code,
    };
    threadctl.get_runmask(cpuset)
}

pub unsafe fn pthread_rwlockattr_init(attr: *mut pthread_rwlockattr_t) -> int {
    crate::internal::pthread_rwlockattr_init(attr)
}

pub unsafe fn pthread_rwlockattr_destroy(attr: *mut pthread_rwlockattr_t) -> int {
    crate::internal::pthread_rwlockattr_destroy(attr)
}

pub unsafe fn pthread_rwlockattr_setpshared(attr: *mut pthread_rwlockattr_t, pshared: int) -> int {
    crate::internal::pthread_rwlockattr_setpshared(attr, pshared)
}

pub unsafe fn pthread_rwlock_init(
    lock: *mut pthread_rwlock_t,
    attr: *const pthread_rwlockattr_t,
) -> int {
    crate::internal::pthread_rwlock_init(lock, attr)
}

pub unsafe fn pthread_rwlock_destroy(lock: *mut pthread_rwlock_t) -> int {
    crate::internal::pthread_rwlock_destroy(lock)
}

pub unsafe fn pthread_rwlock_rdlock(lock: *mut pthread_rwlock_t) -> int {
    crate::internal::pthread_rwlock_rdlock(lock)
}

pub unsafe fn pthread_rwlock_tryrdlock(lock: *mut pthread_rwlock_t) -> int {
    crate::internal::pthread_rwlock_tryrdlock(lock)
}

pub unsafe fn pthread_rwlock_unlock(lock: *mut pthread_rwlock_t) -> int {
    crate::internal::pthread_rwlock_unlock(lock)
}

pub unsafe fn pthread_rwlock_wrlock(lock: *mut pthread_rwlock_t) -> int {
    crate::internal::pthread_rwlock_wrlock(lock)
}

pub unsafe fn pthread_rwlock_trywrlock(lock: *mut pthread_rwlock_t) -> int {
    crate::internal::pthread_rwlock_trywrlock(lock)
}

pub unsafe fn pthread_mutex_init(
    mtx: *mut pthread_mutex_t,
    attr: *const pthread_mutexattr_t,
) -> int {
    crate::internal::pthread_mutex_init(mtx, attr)
}

pub unsafe fn pthread_mutex_destroy(mtx: *mut pthread_mutex_t) -> int {
    crate::internal::pthread_mutex_destroy(mtx)
}

pub unsafe fn pthread_mutex_lock(mtx: *mut pthread_mutex_t) -> int {
    crate::internal::pthread_mutex_lock(mtx)
}

pub unsafe fn pthread_mutex_timedlock(
    mtx: *mut pthread_mutex_t,
    abs_timeout: *const timespec,
) -> int {
    crate::internal::pthread_mutex_timedlock(mtx, abs_timeout)
}

pub unsafe fn pthread_mutex_trylock(mtx: *mut pthread_mutex_t) -> int {
    crate::internal::pthread_mutex_trylock(mtx)
}

pub unsafe fn pthread_mutex_unlock(mtx: *mut pthread_mutex_t) -> int {
    crate::internal::pthread_mutex_unlock(mtx)
}

pub unsafe fn pthread_mutex_consistent(mtx: *mut pthread_mutex_t) -> int {
    crate::internal::pthread_mutex_consistent(mtx)
}

pub unsafe fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> int {
    crate::internal::pthread_mutexattr_init(attr)
}

pub unsafe fn pthread_mutexattr_destroy(attr: *mut pthread_mutexattr_t) -> int {
    crate::internal::pthread_mutexattr_destroy(attr)
}

pub unsafe fn pthread_mutexattr_setprotocol(attr: *mut pthread_mutexattr_t, protocol: int) -> int {
    crate::internal::pthread_mutexattr_setprotocol(attr, protocol)
}

pub unsafe fn pthread_mutexattr_setpshared(attr: *mut pthread_mutexattr_t, pshared: int) -> int {
    crate::internal::pthread_mutexattr_setpshared(attr, pshared)
}

pub unsafe fn pthread_mutexattr_setrobust(attr: *mut pthread_mutexattr_t, robustness: int) -> int {
    crate::internal::pthread_mutexattr_setrobust(attr, robustness)
}

pub unsafe fn pthread_mutexattr_settype(attr: *mut pthread_mutexattr_t, mtype: int) -> int {
    crate::internal::pthread_mutexattr_settype(attr, mtype)
}

mod internal {

    use super::*;

    // Adapted from <sys/neutrino.h>:
    // #define RMSK_SIZE(num_cpu)	(((num_cpu) - 1) / __INT_BITS__ + 1)
    pub(crate) unsafe fn rmsk_size(num_cpu: u16) -> usize {
        const INT_BITS: crate::posix::uint =
            core::mem::size_of::<crate::posix::uint>() as crate::posix::uint * 8;
        (((num_cpu as u32 - 1) / INT_BITS) + 1) as usize
    }

    // Adapted from <sys/neutrino.h>:
    // #define rmsk_set(cpu, p)	((p)[(cpu) / __int_bits__] |=	\
    // 							 (1 << ((cpu) % __int_bits__)))
    pub(crate) unsafe fn rmsk_set(cpu: crate::posix::uint, mask: *mut crate::posix::uint) {
        const INT_BITS: crate::posix::uint =
            core::mem::size_of::<crate::posix::uint>() as crate::posix::uint * 8;
        let idx = (cpu / INT_BITS) as isize;
        let bit = cpu % INT_BITS;

        *mask.offset(idx) |= 1 << bit;
    }

    // Adapted from <sys/neutrino.h>:
    // #define rmsk_isset(cpu, p)	((p)[(cpu) / __int_bits__] &	\
    // 							 (1 << ((cpu) % __int_bits__)))
    #[allow(dead_code)]
    pub(crate) unsafe fn rmsk_isset(
        cpu: crate::posix::uint,
        mask: *const crate::posix::uint,
    ) -> bool {
        const INT_BITS: crate::posix::uint =
            core::mem::size_of::<crate::posix::uint>() as crate::posix::uint * 8;
        let idx = (cpu / INT_BITS) as isize;
        let bit = cpu % INT_BITS;
        (*mask.offset(idx) & (1 << bit)) != 0
    }

    /// RAII wrapper for ThreadCtl
    pub(crate) struct ThreadCtl {
        data: *mut u8,
        rmaskp: *mut crate::posix::uint,
        imaskp: *mut crate::posix::uint,
        num_elements: usize,
        layout: alloc::alloc::Layout,
    }

    impl ThreadCtl {
        /// Initialize ThreadCtl data structures
        /// Adapted from:
        /// https://www.qnx.com/developers/docs/7.1/index.html#com.qnx.doc.neutrino.prog/topic/multicore_processor_affinity.html
        pub(crate) unsafe fn init() -> Result<Self, int> {
            use alloc::alloc::{alloc, Layout};
            use core::ptr;

            let num_cpu = (*crate::internal::_syspage_ptr).num_cpu;

            // Determine the number of array elements required to hold the masks,
            // based on the number of processors in the system.
            let num_elements = internal::rmsk_size(num_cpu);

            // Determine the size of the data, in bytes.
            let masksize_bytes = num_elements * core::mem::size_of::<crate::posix::uint>();

            // Allocate memory for the data structure that we'll pass to ThreadCtl().
            // We need space for an integer (the number of elements in each mask array)
            // and the two masks (run mask and inherit mask).
            let size = core::mem::size_of::<crate::posix::int>() + 2 * masksize_bytes;

            let layout =
                Layout::from_size_align_unchecked(size, core::mem::align_of::<crate::posix::int>());
            let data = alloc(layout);
            if data.is_null() {
                return Err(crate::internal::ENOMEM as _);
            }
            ptr::write_bytes(data, 0, size);

            // Set up pointers
            let rsizep = data as *mut crate::posix::int;
            let rmaskp = rsizep.add(1) as *mut crate::posix::uint;
            let imaskp = rmaskp.add(num_elements) as *mut crate::posix::uint;

            // Set the size
            *rsizep = num_elements as crate::posix::int;

            Ok(Self {
                data,
                rmaskp,
                imaskp,
                num_elements,
                layout,
            })
        }

        /// Execute ThreadCtl with prepared structures
        /// Adapted from:
        /// https://www.qnx.com/developers/docs/7.1/index.html#com.qnx.doc.neutrino.prog/topic/multicore_processor_affinity.html
        unsafe fn execute(&self) -> int {
            let result = crate::internal::ThreadCtl(
                crate::internal::_NTO_TCTL_RUNMASK_GET_AND_SET_INHERIT as crate::posix::int,
                self.data as *mut crate::posix::void,
            );

            if result == -1 {
                crate::internal::errno as _
            } else {
                crate::internal::EOK as _
            }
        }

        /// Clear both run and inherit masks to ensure clean state
        unsafe fn reset(&mut self) {
            core::ptr::write_bytes(
                self.rmaskp,
                0,
                self.num_elements * core::mem::size_of::<crate::posix::uint>(),
            );
            core::ptr::write_bytes(
                self.imaskp,
                0,
                self.num_elements * core::mem::size_of::<crate::posix::uint>(),
            );
        }

        /// Set the run mask from a cpu_set_t
        /// Adapted from:
        /// https://www.qnx.com/developers/docs/7.1/index.html#com.qnx.doc.neutrino.prog/topic/multicore_processor_affinity.html
        pub(crate) unsafe fn set_runmask(&mut self, cpuset: *const cpu_set_t) -> int {
            self.reset();

            // Set the data according to cpu_set_t
            for cpu in 0..(*crate::internal::_syspage_ptr).num_cpu as crate::posix::uint {
                if (*cpuset).has(cpu as usize) {
                    internal::rmsk_set(cpu, self.rmaskp);
                }
            }

            self.execute()
        }

        /// Set the inherit mask from a cpu_set_t
        /// Adapted from:
        /// https://www.qnx.com/developers/docs/7.1/index.html#com.qnx.doc.neutrino.prog/topic/multicore_processor_affinity.html
        #[allow(dead_code)]
        pub(crate) unsafe fn set_inheritmask(&mut self, cpuset: *const cpu_set_t) -> int {
            self.reset();

            // Set the data according to cpu_set_t
            for cpu in 0..(*crate::internal::_syspage_ptr).num_cpu as crate::posix::uint {
                if (*cpuset).has(cpu as usize) {
                    internal::rmsk_set(cpu, self.imaskp);
                }
            }

            self.execute()
        }

        /// Get the run mask into a cpu_set_t
        /// Adapted from:
        /// https://www.qnx.com/developers/docs/7.1/index.html#com.qnx.doc.neutrino.prog/topic/multicore_processor_affinity.html
        pub(crate) unsafe fn get_runmask(&mut self, cpuset: *mut cpu_set_t) -> int {
            self.reset();

            // Clear the cpu_set_t first
            (*cpuset).__bits.fill(0);

            let result = self.execute();
            if result != crate::internal::EOK as _ {
                return result;
            }

            // Populate cpu_set_t from the run mask
            for cpu in 0..self.num_elements {
                if internal::rmsk_isset(cpu.try_into().unwrap(), self.rmaskp) {
                    (*cpuset).set(cpu as usize);
                }
            }

            crate::internal::EOK as _
        }

        /// Get the inherit mask into a cpu_set_t
        /// Adapted from:
        /// https://www.qnx.com/developers/docs/7.1/index.html#com.qnx.doc.neutrino.prog/topic/multicore_processor_affinity.html
        #[allow(dead_code)]
        pub(crate) unsafe fn get_inheritmask(&mut self, cpuset: *mut cpu_set_t) -> int {
            self.reset();

            // Clear the cpu_set_t first
            (*cpuset).__bits.fill(0);

            let result = self.execute();
            if result != crate::internal::EOK as _ {
                return result;
            }

            // Populate cpu_set_t from the inherit mask
            for cpu in 0..self.num_elements {
                if internal::rmsk_isset(cpu.try_into().unwrap(), self.imaskp) {
                    (*cpuset).set(cpu as usize);
                }
            }

            crate::internal::EOK as _
        }
    }

    impl Drop for ThreadCtl {
        fn drop(&mut self) {
            use alloc::alloc::dealloc;
            unsafe {
                dealloc(self.data, self.layout);
            }
        }
    }
}
