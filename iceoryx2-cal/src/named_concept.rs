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

use core::fmt::Debug;

use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_bb_posix::directory::{Directory, DirectoryRemoveError};
pub use iceoryx2_bb_system_types::file_name::FileName;
pub use iceoryx2_bb_system_types::file_path::FilePath;
pub use iceoryx2_bb_system_types::path::Path;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum NamedConceptDoesExistError {
    InsufficientPermissions,
    UnderlyingResourcesBeingSetUp,
    UnderlyingResourcesCorrupted,
    InternalError,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum NamedConceptRemoveError {
    InsufficientPermissions,
    InternalError,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum NamedConceptListError {
    InsufficientPermissions,
    InternalError,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum NamedConceptPathHintRemoveError {
    InsufficientPermissions,
    InternalError,
}

/// Every [`NamedConcept`] must have a custom configuration that at least allows the user to define
/// a custom [`NamedConceptConfiguration::suffix()`] for all file names that are transparent during
/// usage as well as a [`NamedConceptConfiguration::path_hint()`] that can be ignored if the
/// underlying resource does not support it.
pub trait NamedConceptConfiguration: Default + Clone + Debug + Send {
    /// Defines the prefix that the concept will use.
    fn prefix(self, value: &FileName) -> Self;

    /// Returns the configurations prefix.
    fn get_prefix(&self) -> &FileName;

    /// Defines the suffix that the concept will use.
    fn suffix(self, value: &FileName) -> Self;

    /// Sets a path hint under which the underlying resources shall be stored. When the concept
    /// uses resources like [`iceoryx2_bb_posix::shared_memory::SharedMemory`] the path will be
    /// ignored.
    fn path_hint(self, value: &Path) -> Self;

    /// Returns the configurations suffix.
    fn get_suffix(&self) -> &FileName;

    /// Returns the configurations path hint.
    fn get_path_hint(&self) -> &Path;

    /// Returns the full path for a given value under the given configuration.
    fn path_for(&self, value: &FileName) -> FilePath {
        let mut path = self.get_path_hint().clone();
        fatal_panic!(from self, when path.add_path_entry(&self.get_prefix().into()),
                    "The path hint \"{}\" in combination with the prefix \"{}\" exceed the maximum supported path length of {} of the operating system.",
                    path, value, Path::max_len());
        fatal_panic!(from self, when path.push_bytes(value.as_string()),
                    "The path hint \"{}\" in combination with the file name \"{}\" exceed the maximum supported path length of {} of the operating system.",
                    path, value, Path::max_len());
        fatal_panic!(from self, when path.push_bytes(self.get_suffix()),
                    "The path hint \"{}\" in combination with the file name \"{}\" and the suffix \"{}\" exceed the maximum supported path length of {} of the operating system.",
                    path, value, self.get_suffix(), Path::max_len());

        unsafe { FilePath::new_unchecked(path.as_bytes()) }
    }

    /// Extracts the name from a full path under a given configuration.
    fn extract_name_from_path(&self, value: &FilePath) -> Option<FileName> {
        if *self.get_path_hint() != value.path() {
            return None;
        }

        self.extract_name_from_file(&value.file_name())
    }

    /// Extracts the name from a file name under a given configuration.
    fn extract_name_from_file(&self, value: &FileName) -> Option<FileName> {
        let mut file = value.clone();

        if !fatal_panic!(from self, when file.strip_prefix(self.get_prefix().as_bytes()),
                    "Stripping the prefix \"{}\" from the file name \"{}\" leads to invalid content.",
                    self.get_prefix(), file)
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

/// Builder trait to create new [`NamedConcept`]s.
pub trait NamedConceptBuilder<T: NamedConceptMgmt> {
    /// Defines the name of the newly created [`NamedConcept`].
    fn new(name: &FileName) -> Self;

    /// Sets the custom configuration of the concept.
    fn config(self, config: &T::Configuration) -> Self;
}

/// Every concept that is uniquely identified by a [`FileName`] and corresponds to some kind of
/// file in the file system is a [`NamedConcept`]. This trait provides the essential property of
/// these concepts [`NamedConcept::name()`]
pub trait NamedConcept: Debug {
    /// Returns the name of the concept
    fn name(&self) -> &FileName;
}

/// Every concept that is uniquely identified by a [`FileName`] and corresponds to some kind of
/// file in the file system is a [`NamedConcept`]. This trait provides common management methods
/// for such concepts, like
///  * [`NamedConceptMgmt::remove()`]
///  * [`NamedConceptMgmt::does_exist()`]
///  * [`NamedConceptMgmt::list()`]
pub trait NamedConceptMgmt: Debug {
    type Configuration: NamedConceptConfiguration;

    /// Removes an existing concept. Returns true if the concepts existed and was removed,
    /// if the concept did not exist it returns false.
    ///
    /// # Safety
    ///
    ///  * It must be ensured that no other process is using the concept.
    ///
    unsafe fn remove(name: &FileName) -> Result<bool, NamedConceptRemoveError> {
        Self::remove_cfg(name, &Self::Configuration::default())
    }

    /// Returns true if a concept with that name exists, otherwise false
    fn does_exist(name: &FileName) -> Result<bool, NamedConceptDoesExistError> {
        Self::does_exist_cfg(name, &Self::Configuration::default())
    }

    /// Returns a list of all available concepts with the default configuration.
    fn list() -> Result<Vec<FileName>, NamedConceptListError> {
        Self::list_cfg(&Self::Configuration::default())
    }

    /// Removes an existing concept under a custom configuration. Returns true if the concepts
    /// existed and was removed, if the concept did not exist it returns false.
    ///
    /// # Safety
    ///
    ///  * It must be ensured that no other process is using the concept.
    ///
    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, NamedConceptRemoveError>;

    /// Returns true if a concept with that name exists under a custom configuration, otherwise false
    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, NamedConceptDoesExistError>;

    /// Returns a list of all available concepts with a custom configuration.
    fn list_cfg(cfg: &Self::Configuration) -> Result<Vec<FileName>, NamedConceptListError>;

    /// The default prefix of every zero copy connection
    fn default_prefix() -> FileName {
        unsafe { FileName::new_unchecked(b"iox2_") }
    }

    /// The default path hint for every zero copy connection
    fn default_path_hint() -> Path {
        iceoryx2_bb_posix::config::temp_directory()
    }

    /// Removes the path hint directory. Will be realized only when the concept actually uses
    /// the path hint.
    fn remove_path_hint(value: &Path) -> Result<(), NamedConceptPathHintRemoveError>;
}

pub(crate) fn remove_path_hint(value: &Path) -> Result<(), NamedConceptPathHintRemoveError> {
    let origin = format!("remove_path_hint({value:?})");
    let msg = "Unable to remove path hint";
    match Directory::remove_empty(value) {
        Ok(()) | Err(DirectoryRemoveError::DirectoryDoesNotExist) => Ok(()),
        Err(DirectoryRemoveError::InsufficientPermissions) => {
            fail!(from origin, with NamedConceptPathHintRemoveError::InsufficientPermissions,
                "{} due to insufficient permissions.", msg);
        }
        Err(e) => {
            fail!(from origin, with NamedConceptPathHintRemoveError::InternalError,
                "{} due to an internal error ({:?}).", msg, e);
        }
    }
}
