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

mod internal;
mod requires_std;
mod test;
mod tests;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    test::proc_macro(attr, item)
}

#[proc_macro_attribute]
pub fn tests(attr: TokenStream, item: TokenStream) -> TokenStream {
    tests::proc_macro(attr, item)
}

#[proc_macro_attribute]
pub fn requires_std(attr: TokenStream, item: TokenStream) -> TokenStream {
    requires_std::proc_macro(attr, item)
}
