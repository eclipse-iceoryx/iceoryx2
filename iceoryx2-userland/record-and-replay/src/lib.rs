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

#![warn(missing_docs)]

//! # iceoryx2 userland - record and replay
//!
//! Provides building blocks to record and replay data from and into iceoryx2.
//! The library itself does not capture the data, this has to be implemented by the user.
//!
//! ## Example
//!
//! ### Record Data
//!
//! The individual types used by [`ServiceTypes`](crate::recorder::ServiceTypes) can be acquired
//! from the [`StaticConfig`](iceoryx2::service::static_config::StaticConfig) of a
//! [`Service`](iceoryx2::service::Service). For publish susbcribe one can call for instance
//! [`publish_subscribe::StaticConfig::message_type_details()`](iceoryx2::service::static_config::publish_subscribe::StaticConfig::message_type_details())
//!
//! ```
//! use iceoryx2::prelude::*;
//! use iceoryx2_userland_record_and_replay::prelude::*;
//! use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeNameString};
//! use iceoryx2::service::static_config::message_type_details::TypeVariant;
//! use core::time::Duration;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let service_types = ServiceTypes {
//!     payload: TypeDetail::new::<u64>(TypeVariant::FixedSize),
//!     user_header: TypeDetail::new::<()>(TypeVariant::FixedSize),
//!     system_header: TypeDetail::new::<u64>(TypeVariant::FixedSize),
//! };
//!
//! // create the file recorder
//! let mut recorder = RecorderBuilder::new(&service_types)
//!     .data_representation(DataRepresentation::HumanReadable)
//!     .messaging_pattern(MessagingPattern::PublishSubscribe)
//!     .create(&FilePath::new(b"recorded_data.iox2")?, &ServiceName::new("my-service")?)?;
//!
//! # iceoryx2_bb_posix::file::File::remove(&FilePath::new(b"recorded_data.iox2")?)?;
//!
//! // add some recorded data
//! recorder.write(RawRecord {
//!     timestamp: Duration::ZERO,
//!     system_header: &[0u8; 8],
//!     user_header: &[0u8; 0],
//!     payload: &[0u8; 8]
//! })?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ### Load Recorded Data Into Memory Buffer (Small Payload)
//!
//! The whole recorded file is loaded into memory. Useful, when the data is not that large.
//!
//! ```no_run
//! use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeNameString};
//! use iceoryx2::service::static_config::message_type_details::TypeVariant;
//! use iceoryx2::prelude::*;
//! use iceoryx2_userland_record_and_replay::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//!
//! let replay = ReplayerOpener::new(&FilePath::new(b"recorded_data.iox2")?)
//!     .data_representation(DataRepresentation::HumanReadable)
//!     .open()?;
//!  let record_header = replay.header().clone();
//!  let buffer = replay.read_into_buffer().unwrap();
//!
//! println!("record header of service types {record_header:?}");
//!
//! for record in buffer {
//!     println!("payload: {:?}", record.payload);
//!     println!("user_header: {:?}", record.user_header);
//!     println!("system_header: {:?}", record.system_header);
//!     println!("timestamp: {:?}", record.timestamp);
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//! ### Read Record One-By-One (Large Payload)
//!
//! The recorded file is opened and the records are read one-by-one.
//!
//! ```no_run
//! use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeNameString};
//! use iceoryx2::service::static_config::message_type_details::TypeVariant;
//! use iceoryx2::prelude::*;
//! use iceoryx2_userland_record_and_replay::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//!
//! let mut replayer = ReplayerOpener::new(&FilePath::new(b"recorded_data.iox2")?)
//!     .data_representation(DataRepresentation::HumanReadable)
//!     .open()?;
//!
//! println!("record header of service types {:?}", replayer.header());
//!
//! while let Some(record) = replayer.next_record()? {
//!     println!("payload: {:?}", record.payload);
//!     println!("user_header: {:?}", record.user_header);
//!     println!("system_header: {:?}", record.system_header);
//!     println!("timestamp: {:?}", record.timestamp);
//! }
//!
//! # Ok(())
//! # }
//! ```

/// Free functions to convert bytes to a hex string and back.
pub mod hex_conversion;

/// Loads a meaninful subset.
pub mod prelude;

/// Contains the [`Record`](crate::record::Record) and [`RawRecord`](crate::record::RawRecord)
/// which represent a single entry that is added to the file.
pub mod record;

/// The header of the record file which contains all necessary type information.
pub mod record_header;

/// Contains the [`Recorder`](crate::recorder::Recorder) to write captured payload into a file.
pub mod recorder;

/// Contains the [`Replayer`](crate::replayer::Replayer) to read captured payload from a file.
pub mod replayer;

#[doc(hidden)]
pub mod testing;
