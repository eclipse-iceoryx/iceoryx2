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

use iceoryx2::prelude::*;
use iceoryx2_bb_container::byte_string::FixedSizeByteString;

// TODO: remove
fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Trace);
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .blackboard_creator::<u32>()
        .max_readers(10)
        .add::<f32>(5, 23.4)
        .add::<FixedSizeByteString<100>>(13, "Hello World".try_into()?)
        .add::<u64>(99, 2023)
        .create()?;

    {
        let _writer1 = service.writer_builder().create()?;
        let writer2 = service.writer_builder().create();
        assert!(writer2.is_err());
    }

    let opened_service = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .blackboard_opener::<u32>()
        .max_readers(10)
        .open()?;

    let reader = opened_service.reader_builder().create()?;
    let reader_handle_u64 = reader.entry::<u64>(&99)?;
    let reader_handle_f32 = reader.entry::<f32>(&5)?;
    let reader_handle_string = reader.entry::<FixedSizeByteString<100>>(&13)?;
    println!("read u64: {}", reader_handle_u64.get());
    println!("read f32: {}", reader_handle_f32.get());
    println!("read string: {}", reader_handle_string.get());

    let writer = opened_service.writer_builder().create()?;

    let writer_handle_u64 = writer.entry::<u64>(&99)?;
    writer_handle_u64.update_with_copy(2008);
    let writer_handle_f32 = writer.entry::<f32>(&5)?;
    writer_handle_f32.update_with_copy(11.11);
    let writer_handle_string = writer.entry::<FixedSizeByteString<100>>(&13)?;
    writer_handle_string.update_with_copy("Bye world.".try_into()?);
    println!("read u64: {}", reader_handle_u64.get());
    println!("read f32: {}", reader_handle_f32.get());
    println!("read string: {}", reader_handle_string.get());

    writer_handle_u64.update_with_copy(1990);
    println!("read u64: {}", reader_handle_u64.get());

    let writer_handle = writer.entry::<u64>(&99);
    assert!(writer_handle.is_err());
    let reader_handle = reader.entry::<u64>(&99)?;
    println!("read u64: {}", reader_handle.get());

    drop(writer_handle_u64);
    let writer_handle = writer.entry::<u64>(&99)?;
    writer_handle.update_with_copy(333);
    println!("read u64: {}", reader_handle_u64.get());

    let entry_value_uninit = writer_handle.loan_uninit();
    let entry_value = entry_value_uninit.write(1956);
    let writer_handle = entry_value.update();
    println!("read u64: {}", reader_handle.get());

    let entry_value_uninit = writer_handle.loan_uninit();
    let entry_value = entry_value_uninit.write(1984);
    let _writer_handle = entry_value.update();
    println!("read u64: {}", reader_handle.get());

    Ok(())
}
