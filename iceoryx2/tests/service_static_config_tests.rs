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

#[cfg(test)]
mod service_static_config_message_type_details {
    use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
    use iceoryx2_bb_derive_macros::ZeroCopySend;
    use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn test_internal_new() {
        #[derive(ZeroCopySend)]
        #[repr(C)]
        struct Tmp;
        let sut = TypeDetail::new::<Tmp>(TypeVariant::FixedSize);
        assert_that!(*sut.type_name(), eq core::any::type_name::<Tmp>());
        assert_that!(sut.variant(), eq TypeVariant::FixedSize);
        assert_that!(sut.size(), eq core::mem::size_of::<Tmp>());
        assert_that!(sut.alignment(), eq core::mem::align_of::<Tmp>());

        let sut = TypeDetail::new::<i64>(TypeVariant::FixedSize);
        assert_that!(*sut.type_name(), eq core::any::type_name::<i64>());
        assert_that!(sut.variant(), eq TypeVariant::FixedSize);
        assert_that!(sut.size(), eq core::mem::size_of::<i64>());
        assert_that!(sut.alignment(), eq core::mem::align_of::<i64>());
    }
}
