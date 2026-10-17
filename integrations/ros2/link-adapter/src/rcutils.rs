// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

//! Safe wrappers around the `rcutils` bindings, covering what the gateway
//! needs from them.

use std::ffi::c_void;

use r2r_rcl::rcutils_allocator_t;

/// What became of the one request a loan allocator serves.
enum Loan<LoanFn> {
    /// The request has not come yet.
    Pending(LoanFn),
    /// The request was served with the loaned buffer.
    Loaned,
    /// The request was declined by the loan function.
    Declined,
}

/// An rcutils allocator backed by a function to loan shared memory.
pub(crate) struct LoanAllocator<LoanFn> {
    loan: Loan<LoanFn>,
}

impl<LoanFn: FnOnce(usize) -> Option<*mut u8>> LoanAllocator<LoanFn> {
    /// Wraps `loan`, which returns a buffer of the requested size or `None`
    /// if declined.
    ///
    /// It is called at most once, from C, and must not panic.
    pub(crate) fn new(loan: LoanFn) -> Self {
        Self {
            loan: Loan::Pending(loan),
        }
    }

    /// The allocator of an `rcutils_uint8_array_t`.
    ///
    /// It points at this value, which must outlive the array and not move
    /// while it is used.
    pub(crate) fn allocator(&mut self) -> rcutils_allocator_t {
        rcutils_allocator_t {
            allocate: Some(allocate::<LoanFn>),
            deallocate: Some(deallocate),
            reallocate: Some(reallocate::<LoanFn>),
            zero_allocate: Some(zero_allocate),
            state: (&mut self.loan as *mut Loan<LoanFn>).cast(),
        }
    }

    /// Whether the loan function declined the one request.
    pub(crate) fn declined(&self) -> bool {
        matches!(self.loan, Loan::Declined)
    }
}

unsafe extern "C" fn allocate<LoanFn: FnOnce(usize) -> Option<*mut u8>>(
    size: usize,
    state: *mut c_void,
) -> *mut c_void {
    // SAFETY: rcutils passes back the `state` it was given, which stays
    // alive and in place for this call.
    let state = unsafe { &mut *(state as *mut Loan<LoanFn>) };
    if !matches!(state, Loan::Pending(_)) {
        // The allocation can only be done once.
        return core::ptr::null_mut();
    }
    let Loan::Pending(loan) = core::mem::replace(state, Loan::Declined) else {
        unreachable!()
    };
    match loan(size) {
        Some(buffer) => {
            *state = Loan::Loaned;
            buffer as *mut c_void
        }
        None => core::ptr::null_mut(),
    }
}

unsafe extern "C" fn reallocate<LoanFn: FnOnce(usize) -> Option<*mut u8>>(
    pointer: *mut c_void,
    size: usize,
    state: *mut c_void,
) -> *mut c_void {
    if !pointer.is_null() {
        // Growing an already-loaned buffer is not possible.
        return core::ptr::null_mut();
    }
    unsafe { allocate::<LoanFn>(size, state) }
}

unsafe extern "C" fn deallocate(_pointer: *mut c_void, _state: *mut c_void) {
    // The deallocation of the loan is left to the caller.
}

unsafe extern "C" fn zero_allocate(
    _number_of_elements: usize,
    _size_of_element: usize,
    _state: *mut c_void,
) -> *mut c_void {
    core::ptr::null_mut()
}
