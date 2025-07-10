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

//! Easy enum generation to implement error signaling enums more efficiently.
//!
//! # Examples
//!
//! ## Define simple enum
//!
//! Generates a public enum which derives from [`Debug`], [`Clone`], [`Copy`], [`Eq`] and
//! [`PartialEq`].
//! Those two code snippets are equivalent.
//!
//! ```
//! use iceoryx2_bb_elementary::enum_gen;
//!
//! enum_gen! {
//!     /// Some optional documentation
//!     MyErrorEnum
//!
//!   entry:
//!     Failure1,
//!     Failure2
//! }
//! ```
//!
//! ```
//! /// Some optional documentation
//! #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
//! pub enum MyErrorEnum {
//!     Failure1,
//!     Failure2
//! }
//! ```
//!
//! ## Mapping enums for detailed forward propagation
//!
//! Generates a public enum which implements also [`From`] for all enum types listed under
//! `mapping`.
//! Those two code snippets are equivalent.
//!
//! ```
//! use iceoryx2_bb_elementary::enum_gen;
//!
//! #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
//! enum SomeEnum {}
//!
//! #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
//! enum AnotherEnum {}
//!
//! enum_gen! {
//!     /// Some optional documentation
//!     MyErrorEnum
//!
//!   entry:
//!     Failure1,
//!     Failure2
//!
//!   mapping:
//!     SomeEnum,
//!     AnotherEnum
//! }
//! ```
//!
//! ```
//! #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
//! enum SomeEnum {}
//!
//! #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
//! enum AnotherEnum {}
//!
//! /// Some optional documentation
//! #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
//! pub enum MyErrorEnum {
//!     Failure1,
//!     Failure2,
//!     SomeEnum(SomeEnum),
//!     AnotherEnum(AnotherEnum)
//! }
//!
//! impl From<SomeEnum> for MyErrorEnum {
//!     fn from(value: SomeEnum) -> Self {
//!         MyErrorEnum::SomeEnum(value)
//!     }
//! }
//!
//! impl From<AnotherEnum> for MyErrorEnum {
//!     fn from(value: AnotherEnum) -> Self {
//!         MyErrorEnum::AnotherEnum(value)
//!     }
//! }
//! ```
//!
//! ## Generalize enums to propagate errors in a coarse fashion
//!
//! Generates a public enum which implements also [`From`] for all enum types listed under
//! 'generalization' but it discards the underlying enum value.
//! Those two code snippets are equivalent.
//!
//! ```
//! use iceoryx2_bb_elementary::enum_gen;
//!
//! enum SomeEnum {}
//! enum AnotherEnum {}
//! enum Widget {}
//! enum AllMightyBlubb {}
//!
//! enum_gen! {
//!     /// Some optional documentation
//!     MyErrorEnum
//!
//!   entry:
//!     Failure1,
//!     Failure2
//!
//!   generalization:
//!     InternalFailure <= SomeEnum; Widget; AllMightyBlubb,
//!     OhWhatever <= AnotherEnum
//! }
//! ```
//!
//! ```
//! enum SomeEnum {}
//! enum AnotherEnum {}
//! enum Widget {}
//! enum AllMightyBlubb {}
//!
//! /// Some optional documentation
//! #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
//! pub enum MyErrorEnum {
//!     Failure1,
//!     Failure2,
//!     InternalFailure,
//!     OhWhatever
//! }
//!
//! impl From<SomeEnum> for MyErrorEnum {
//!     fn from(_: SomeEnum) -> Self {
//!         MyErrorEnum::InternalFailure
//!     }
//! }
//!
//! impl From<Widget> for MyErrorEnum {
//!     fn from(_: Widget) -> Self {
//!         MyErrorEnum::InternalFailure
//!     }
//! }
//!
//! impl From<AllMightyBlubb> for MyErrorEnum {
//!     fn from(_: AllMightyBlubb) -> Self {
//!         MyErrorEnum::InternalFailure
//!     }
//! }
//!
//! impl From<AnotherEnum> for MyErrorEnum {
//!     fn from(_: AnotherEnum) -> Self {
//!         MyErrorEnum::OhWhatever
//!     }
//! }
//! ```

#[macro_export(local_inner_macros)]
macro_rules! enum_gen {
    { $(#[$documentation:meta])*
      $enum_name:ident
      entry:
        $($entry:ident$(($bla:ident))?),*}
    => {
        $(#[$documentation])*
        #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
        pub enum $enum_name {
            $($entry$(($bla))?),*
        }
    };

    { $(#[$documentation:meta])*
      $enum_name:ident
      mapping:
        $($equivalent:ident),*}
    => {
        $(#[$documentation])*
        #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
        pub enum $enum_name {
            $($equivalent($equivalent)),*
        }

        $(impl From<$equivalent> for $enum_name {
            fn from(v: $equivalent) -> Self {
                $enum_name::$equivalent(v)
            }
        })*
    };

    { $(#[$documentation:meta])*
      $enum_name:ident
      mapping:
        $($equivalent:ident to $value_name:ident),*}
    => {
        $(#[$documentation])*
        #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
        pub enum $enum_name {
            $($value_name($equivalent)),*
        }

        $(impl From<$equivalent> for $enum_name {
            fn from(v: $equivalent) -> Self {
                $enum_name::$value_name(v)
            }
        })*
    };

    { $(#[$documentation:meta])*
      $enum_name:ident
      entry:
        $($entry:ident$(($bla:ident))?),*
      mapping:
        $($equivalent:ident),*}
    => {
        $(#[$documentation])*
        #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
        pub enum $enum_name {
            $($entry$(($bla))?),*,
            $($equivalent($equivalent)),*
        }

        $(impl From<$equivalent> for $enum_name {
            fn from(v: $equivalent) -> Self {
                $enum_name::$equivalent(v)
            }
        })*
    };

    { $(#[$documentation:meta])*
      $enum_name:ident
      entry:
        $($entry:ident$(($bla:ident))?),*
      mapping:
        $($equivalent:ident to $value_name:ident),*}
    => {
        $(#[$documentation])*
        #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
        pub enum $enum_name {
            $($entry$(($bla))?),*,
            $($value_name($equivalent)),*
        }

        $(impl From<$equivalent> for $enum_name {
            fn from(v: $equivalent) -> Self {
                $enum_name::$value_name(v)
            }
        })*
    };

    { $(#[$documentation:meta])*
      $enum_name:ident
      generalization:
        $($destination:ident <= $($source:ident);*),*}
    => {
        $(#[$documentation])*
        #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
        pub enum $enum_name {
            $($destination),*,
        }

        $($(impl From<$source> for $enum_name {
            fn from(_: $source) -> Self {
                $enum_name::$destination
            }
        })*)*
    };

    { $(#[$documentation:meta])*
      $enum_name:ident
      entry:
        $($entry:ident$(($bla:ident))?),*
      generalization:
        $($destination:ident <= $($source:ident);*),*}
    => {
        $(#[$documentation])*
        #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
        pub enum $enum_name {
            $($entry$(($bla))?),*,
            $($destination),*,
        }

        $($(impl From<$source> for $enum_name {
            fn from(_: $source) -> Self {
                $enum_name::$destination
            }
        })*)*
    };

    { $(#[$documentation:meta])*
      $enum_name:ident
      entry:
        $($entry:ident$(($bla:ident))?),*
      mapping:
        $($equivalent:ident),*
      generalization:
        $($destination:ident <= $($source:ident);*),*}
    => {
        $(#[$documentation])*
        #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
        pub enum $enum_name {
            $($entry$(($bla))?),*,
            $($equivalent($equivalent)),*,
            $($destination),*,
        }

        $(impl From<$equivalent> for $enum_name {
            fn from(v: $equivalent) -> Self {
                $enum_name::$equivalent(v)
            }
        })*

        $($(impl From<$source> for $enum_name {
            fn from(_: $source) -> Self {
                $enum_name::$destination
            }
        })*)*
    };

    { $(#[$documentation:meta])*
      $name:ident
      unknown_translates_to:
        $unknown_entry:ident = $unn:ident::$uvalue:ident
      entry:
        $($entry:ident = $nn:ident::$value:ident),* }
    => {
      $(#[$documentation])*
      #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
      #[repr(i32)]
      pub enum $name {
          $($entry = $nn::$value),*,
          $unknown_entry = $unn::$uvalue
      }

      impl Into<$name> for i32 {
          fn into(self) -> $name {
              match self {
                  $($nn::$value => $name::$entry),*,
                  _ => $name::$unknown_entry
              }
          }
      }

      impl Display for $name {
          fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
              core::write!(f, "{}::{:?}", core::stringify!($name), self )
          }
      }
    };
}
