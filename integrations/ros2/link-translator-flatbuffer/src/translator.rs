// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use iceoryx2::prelude::{Flatbuffer, Path};
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2_bb_flatbuffers::{
    FindSchemaFileError, TypeName as SchemaTypeName, find_best_fitting_binary_schema_file,
};
use iceoryx2_integrations_ros2_link_adapter::translator::{
    EmptyHeader, TranslationError as HeaderError,
};
use iceoryx2_integrations_ros2_link_adapter::{MessageType, MirroredHeader, TypeName};
use iceoryx2_link_adapter::{LocalTypes, SampleShape, SampleTranscoders, Translator};
use iceoryx2_link_backend::service_description::{
    InvalidSchema, SampleTypes, Schema, TypeDescription, TypeIdentifier,
};
use iceoryx2_log::{fail, origin};

use crate::pairing;
use crate::transcoder::FlatbufferTranscoder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationError {
    /// The local payload is not a flatbuffer.
    NotAFlatbuffer,
    /// The schema's root type is not a ROS 2 type name with dots.
    InvalidRootType,
    /// The topic type does not fit into a schema type name.
    TypeNameTooLong,
    /// No flatbuffer schema search path is configured.
    NoSchemaPath,
    /// The search path holds no schema for the topic type.
    SchemaNotFound,
    /// The search path could not be searched.
    SchemaLookup(FindSchemaFileError),
    /// The schema file could not be read.
    SchemaUnreadable,
    /// The schema file is not a usable schema.
    InvalidSchema(InvalidSchema),
    /// The schema's root type and the topic type are not the same type.
    Mismatch,
    /// The schema and the message do not pair member by member.
    Pairing(pairing::PairingError),
    /// The local user header is neither the RosHeader nor absent.
    Header(HeaderError),
}

impl core::fmt::Display for TranslationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TranslationError::{self:?}")
    }
}

impl core::error::Error for TranslationError {}

impl From<HeaderError> for TranslationError {
    fn from(error: HeaderError) -> Self {
        Self::Header(error)
    }
}

/// The translator for services whose payload is a flatbuffer with a
/// schema that describes a ROS 2 message.
#[derive(Debug, Default, Clone)]
pub struct FlatbufferTranslator {
    /// The user header of the services that topics are propagated to.
    pub header: MirroredHeader,
    /// The directory the schemas of propagated topics are looked up in, as
    /// `iceoryx2` is configured with.
    pub schema_path: Option<Path>,
}

impl Translator<SampleShape> for FlatbufferTranslator {
    type RemoteTypes = MessageType;
    type Transcoders = SampleTranscoders<EmptyHeader, FlatbufferTranscoder>;
    type Error = TranslationError;

    fn local(&self, remote: &MessageType) -> Result<LocalTypes<SampleShape>, Self::Error> {
        let origin = origin!("FlatbufferTranslator::local");

        let type_name = remote.type_name.as_str();
        let Some(schema_path) = &self.schema_path else {
            fail!(
                from origin,
                with TranslationError::NoSchemaPath,
                "No flatbuffer schema search path is configured to look up a schema for '{}' in", type_name
            );
        };
        let schema_type_name = schema_type_name(type_name)?;
        let file = match find_best_fitting_binary_schema_file(&schema_type_name, schema_path) {
            Ok(Some(file)) => file,
            Ok(None) => {
                fail!(
                    from origin,
                    with TranslationError::SchemaNotFound,
                    "No binary flatbuffer schema for '{}' in '{}'", type_name, schema_path.as_str()
                );
            }
            Err(error) => {
                fail!(
                    from origin,
                    with TranslationError::SchemaLookup(error),
                    "The schema search path '{}' could not be searched", schema_path.as_str()
                );
            }
        };
        let bytes = fail!(
            from origin,
            when std::fs::read(file.as_str()),
            with TranslationError::SchemaUnreadable,
            "The schema file '{}' could not be read", file.as_str()
        );
        let schema = match Schema::new(bytes) {
            Ok(schema) => schema,
            Err(error) => {
                fail!(
                    from origin,
                    with TranslationError::InvalidSchema(error),
                    "The schema file '{}' is not usable", file.as_str()
                );
            }
        };
        if ros_type_name(schema.root_type())? != remote.type_name {
            fail!(
                from origin,
                with TranslationError::Mismatch,
                "The schema file '{}' has the root type '{}', not '{}'", file.as_str(), schema.root_type(), type_name
            );
        }

        Ok(SampleTypes {
            payload: payload_description(schema),
            user_header: self.header.type_description(),
        })
    }

    fn remote(&self, local: &LocalTypes<SampleShape>) -> Result<MessageType, Self::Error> {
        let schema = flatbuffer_schema(local)?;
        let type_name = ros_type_name(schema.root_type())?;

        Ok(MessageType { type_name })
    }

    fn transcoders(
        &self,
        local: &LocalTypes<SampleShape>,
        remote: &MessageType,
    ) -> Result<Self::Transcoders, Self::Error> {
        let origin = origin!("FlatbufferTranslator::transcoders");

        let schema = flatbuffer_schema(local)?;
        let type_name = ros_type_name(schema.root_type())?;
        if type_name != remote.type_name {
            fail!(
                from origin,
                with TranslationError::Mismatch,
                "The schema's root type '{}' is not the topic type '{}'", schema.root_type(), remote.type_name.as_str()
            );
        }
        let table = match pairing::compile(schema.bytes(), type_name.as_str()) {
            Ok(table) => table,
            Err(error) => {
                fail!(
                    from origin,
                    with TranslationError::Pairing(error),
                    "The schema of '{}' does not match the message", type_name.as_str()
                );
            }
        };

        let header = fail!(
            from origin,
            when MirroredHeader::of(local),
            to TranslationError,
            "Header '{}' is not the RosHeader, ROS 2 cannot fill it", local.user_header.identifier.type_name()
        );

        let payload = FlatbufferTranscoder { type_name, table };
        Ok(match header {
            MirroredHeader::RosHeader => SampleTranscoders::TranscodePayload(payload),
            MirroredHeader::None => SampleTranscoders::TranscodeBoth(EmptyHeader, payload),
        })
    }
}

/// The description of a flatbuffer payload of `schema`.
fn payload_description(schema: Schema) -> TypeDescription {
    let flatbuffer = TypeDetail::new::<Flatbuffer<()>>(TypeVariant::FixedSize);
    TypeDescription {
        variant: flatbuffer.variant(),
        identifier: TypeIdentifier::Flatbuffer(schema),
        size: flatbuffer.size(),
        alignment: flatbuffer.alignment(),
    }
}

/// The schema of the flatbuffer payload of `types`.
fn flatbuffer_schema(types: &SampleTypes) -> Result<&Schema, TranslationError> {
    let origin = origin!("flatbuffer_schema");

    match &types.payload.identifier {
        TypeIdentifier::Flatbuffer(schema) => Ok(schema),
        TypeIdentifier::Name(name) => {
            fail!(
                from origin,
                with TranslationError::NotAFlatbuffer,
                "Payload '{}' is not a flatbuffer", name
            );
        }
    }
}

/// The ROS 2 type name that the root type `root_type` names,
/// `sensor_msgs.msg.JointState` for `sensor_msgs/msg/JointState`.
fn ros_type_name(root_type: &str) -> Result<TypeName, TranslationError> {
    let origin = origin!("ros_type_name");

    let type_name = fail!(
        from origin,
        when TypeName::new(&root_type.replace('.', "/")),
        with TranslationError::InvalidRootType,
        "The root type '{}' does not name a ROS 2 type", root_type
    );
    Ok(type_name)
}

/// The schema type name of the root type that names the ROS 2 type
/// `type_name`, the namespace `sensor_msgs.msg` and the name `JointState`
/// for `sensor_msgs/msg/JointState`.
fn schema_type_name(type_name: &str) -> Result<SchemaTypeName, TranslationError> {
    let origin = origin!("schema_type_name");

    let (namespace, name) = type_name
        .rsplit_once('/')
        .expect("a ROS 2 type name has a package and a kind");
    let Ok(namespace) = namespace.replace('/', ".").parse() else {
        fail!(
            from origin,
            with TranslationError::TypeNameTooLong,
            "The namespace of '{}' does not fit into a schema type name", type_name
        );
    };
    let Ok(name) = name.parse() else {
        fail!(
            from origin,
            with TranslationError::TypeNameTooLong,
            "The name of '{}' does not fit into a schema type name", type_name
        );
    };

    Ok(SchemaTypeName { namespace, name })
}
