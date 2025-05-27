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

#[macro_export(local_inner_macros)]
macro_rules! ErrnoEnumGenerator {
    (assign $($entry:ident = $value:expr),*; map $($map_entry:ident),*) => {
        #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
        #[repr(i32)]
        pub enum Errno {
            $($entry = $value),*,
            $($map_entry = $crate::internal::$map_entry as _),*,
            NOTIMPLEMENTED = i32::MAX
        }

        // we explicitly only want to convert from enum to i32 and not the other way around
        #[allow(clippy::from_over_into)]
        impl Into<Errno> for u32 {
        #[deny(clippy::from_over_into)]
            fn into(self) -> Errno {
                match self as _ {
                    $($value => Errno::$entry),*,
                    $($crate::internal::$map_entry => Errno::$map_entry),*,
                    _ => Errno::NOTIMPLEMENTED
                }
            }
        }

        #[allow(clippy::from_over_into)]
        impl Into<Errno> for i32 {
        #[deny(clippy::from_over_into)]
            fn into(self) -> Errno {
                (self as u32).into()
            }
        }

        impl Display for Errno {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                const BUFFER_SIZE: usize = 1024;
                let mut buffer: [c_char; BUFFER_SIZE] = [0; BUFFER_SIZE];
                unsafe { strerror_r(*self as i32, buffer.as_mut_ptr(), BUFFER_SIZE) };
                let s = match unsafe { CStr::from_ptr(buffer.as_ptr()) }.to_str() {
                    Ok(v) => v.to_string(),
                    Err(_) => "".to_string(),
                };

                match self {
                    $(Errno::$entry => {
                        core::write!(f, "errno {{ name = \"{}\", value = {}, details = \"{}\" }}",
                            core::stringify!($entry), Errno::$entry as i32, s)
                    }),*,
                    $(Errno::$map_entry => {
                        core::write!(f, "errno {{ name = \"{}\", value = {}, details = \"{}\" }}",
                            core::stringify!($map_entry), Errno::$map_entry as i32, s)
                    }),*,
                    Errno::NOTIMPLEMENTED => {
                        core::write!(f, "errno {{ name = \"NOTIMPLEMENTED\", value = {}, details = \"???\" }}",
                            Errno::NOTIMPLEMENTED as i32)
                    }
                }
            }
        }
    };
}
