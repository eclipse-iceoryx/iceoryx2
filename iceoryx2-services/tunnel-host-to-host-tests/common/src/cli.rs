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

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum PayloadType {
    Primitive,
    Complex,
}

#[derive(Parser)]
#[command(about = "A application for iceoryx2 tunnel end-to-end testing")]
#[command(version)]
pub struct Args {
    #[arg(short, long, value_enum, default_value_t = PayloadType::Primitive)]
    pub payload_type: PayloadType,
}
