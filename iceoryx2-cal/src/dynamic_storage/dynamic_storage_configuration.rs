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

use crate::{
    hash::{sha1::Sha1, Hash},
    named_concept::NamedConceptConfiguration,
};
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_system_types::{file_name::*, file_path::FilePath, path::Path};

pub trait DynamicStorageConfiguration: NamedConceptConfiguration {
    fn type_name(&self) -> &str;

    fn path_for_with_type(&self, value: &FileName) -> FilePath {
        let mut path = self.get_path_hint().clone();
        let type_hash = Sha1::new(self.type_name().as_bytes()).value();

        fatal_panic!(from self, when path.add_path_entry(&self.get_prefix().into()),
                    "The path \"{}\" in combination with the prefix \"{}\" exceed the maximum supported path length of {} of the operating system.",
                    path, value, Path::max_len());
        fatal_panic!(from self, when path.push_bytes(type_hash.as_base64url().as_bytes()),
                    "The path \"{}\" in combination with the type hash \"{}\" exceed the maximum supported path length of {} of the operating system.",
                    path, value, Path::max_len());
        fatal_panic!(from self, when path.push(b'_'),
                    "The path \"{}\" in combination with \"_\" exceed the maximum supported path length of {} of the operating system.",
                    path, Path::max_len());
        fatal_panic!(from self, when path.push_bytes(value.as_string()),
                    "The path \"{}\" in combination with the file name \"{}\" exceed the maximum supported path length of {} of the operating system.",
                    path, value, Path::max_len());
        fatal_panic!(from self, when path.push_bytes(self.get_suffix()),
                    "The path \"{}\" in combination with the suffix \"{}\" exceed the maximum supported path length of {} of the operating system.",
                    path, self.get_suffix(), Path::max_len());

        unsafe { FilePath::new_unchecked(path.as_bytes()) }
    }

    fn extract_name_from_file_with_type(&self, value: &FileName) -> Option<FileName> {
        let mut file = value.clone();

        if !fatal_panic!(from self, when file.strip_prefix(self.get_prefix().as_bytes()),
                    "Stripping the prefix \"{}\" from the file name \"{}\" leads to invalid content.",
                    self.get_prefix(), file)
        {
            return None;
        }

        let type_hash = Sha1::new(self.type_name().as_bytes()).value();

        if !fatal_panic!(from self, when file.strip_prefix(type_hash.as_base64url().as_bytes()),
                    "Stripping the type hash \"{:?}\" from the file name \"{}\" leads to invalid content.",
                    type_hash, file)
        {
            return None;
        }

        if !fatal_panic!(from self, when file.strip_prefix(b"_"),
                    "Stripping the type hash proceeding \"_\" from the file name \"{}\" leads to invalid content.",
                    file)
        {
            return None;
        }

        if !fatal_panic!(from self, when file.strip_suffix(self.get_suffix().as_bytes()),
                    "Stripping the suffix \"{}\" from the file name \"{}\" leads to invalid content.",
                    self.get_suffix(), file)
        {
            return None;
        }

        Some(file)
    }
}
