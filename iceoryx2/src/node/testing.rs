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

/// # Safety
///
///  * only for internal testing purposes
///  * shall be called at most once
///
pub unsafe fn __internal_node_staged_death<S: crate::service::Service>(
    node: &mut crate::node::Node<S>,
) -> <S::Monitoring as iceoryx2_cal::monitoring::Monitoring>::Token {
    node.staged_death()
}
