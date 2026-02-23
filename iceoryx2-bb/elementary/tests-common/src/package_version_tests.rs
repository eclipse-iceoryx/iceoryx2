// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_elementary::package_version::PackageVersion;
use iceoryx2_bb_testing::{assert_that, test_requires};

#[test]
fn package_version_works() {
    // NOTE: The test is skipped when not run with cargo but with bazel
    //       The CI which runs with cargo ensures that the constants defined
    //       in PackageVersion::get equal the package version.
    test_requires!(option_env!("CARGO").is_some());

    let major = option_env!("CARGO_PKG_VERSION_MAJOR")
        .and_then(|s| s.parse::<u16>().ok())
        .expect("Contains a valid major version number.");
    let minor = option_env!("CARGO_PKG_VERSION_MINOR")
        .and_then(|s| s.parse::<u16>().ok())
        .expect("Contains a valid minor version number.");
    let patch = option_env!("CARGO_PKG_VERSION_PATCH")
        .and_then(|s| s.parse::<u16>().ok())
        .expect("Contains a valid patch version number.");

    let sut = PackageVersion::get();

    assert_that!(sut.major(), eq major);
    assert_that!(sut.minor(), eq minor);
    assert_that!(sut.patch(), eq patch);

    assert_that!(major == 0 && minor == 0 && patch == 0, eq false);
}
