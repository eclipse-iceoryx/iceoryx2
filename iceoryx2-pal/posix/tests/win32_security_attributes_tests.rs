// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

#[cfg(target_os = "windows")]
mod win32_security_attributes {
    use iceoryx2_pal_posix::posix;
    use iceoryx2_pal_posix::posix::{
        win32_security_attributes::{
            from_mode_to_security_attributes, from_security_attributes_to_mode,
        },
        *,
    };
    use iceoryx2_pal_testing::assert_that;
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;

    fn roundtrip(orig_mode: posix::mode_t) {
        let attr = from_mode_to_security_attributes(INVALID_HANDLE_VALUE, orig_mode);
        let mode = from_security_attributes_to_mode(&attr);

        assert_that!(orig_mode, eq mode);
    }

    #[test]
    fn mode_to_security_attributes_and_back_works() {
        roundtrip(S_IRWXU | S_IRWXG | S_IRWXO);

        roundtrip(S_IRUSR);
        roundtrip(S_IRGRP);
        roundtrip(S_IROTH);

        roundtrip(S_IWUSR);
        roundtrip(S_IWGRP);
        roundtrip(S_IWOTH);

        roundtrip(S_IXUSR);
        roundtrip(S_IXGRP);
        roundtrip(S_IXOTH);

        roundtrip(S_IRWXU);
        roundtrip(S_IRWXG);
        roundtrip(S_IRWXO);

        roundtrip(S_IRUSR | S_IWUSR);
        roundtrip(S_IRGRP | S_IWGRP);
        roundtrip(S_IROTH | S_IWOTH);

        roundtrip(S_IXUSR | S_IWUSR);
        roundtrip(S_IXGRP | S_IWGRP);
        roundtrip(S_IXOTH | S_IWOTH);

        roundtrip(S_IXUSR | S_IRUSR);
        roundtrip(S_IXGRP | S_IRGRP);
        roundtrip(S_IXOTH | S_IROTH);

        roundtrip(S_IRUSR | S_IRGRP | S_IROTH);
        roundtrip(S_IWUSR | S_IWGRP | S_IWOTH);
        roundtrip(S_IXUSR | S_IXGRP | S_IXOTH);
    }
}
