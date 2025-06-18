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

#[generic_tests::define]
mod service_blackboard {
    use iceoryx2::prelude::*;
    use iceoryx2::service::builder::blackboard::{BlackboardCreateError, BlackboardOpenError};
    use iceoryx2::service::static_config::message_type_details::TypeVariant;
    use iceoryx2::testing::*;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "service_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[test]
    fn open_or_create_with_attributes_succeeds_when_service_does_exist<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let attr = AttributeVerifier::new();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<i64>()
            .open_or_create_with_attributes(&attr);
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard::<i64>()
            .open_or_create_with_attributes(&attr);

        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_or_create_with_attributes_succeeds_when_attribute_is_satisfied<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let attr = AttributeVerifier::new()
            .require(&"hello".try_into().unwrap(), &"world".try_into().unwrap());
        let sut = node
            .service_builder(&service_name)
            .blackboard::<i64>()
            .open_or_create_with_attributes(&attr);
        assert_that!(sut, is_ok);

        let attr1 = AttributeVerifier::new().require_key(&"hello".try_into().unwrap());
        let sut2 = node
            .service_builder(&service_name)
            .blackboard::<i64>()
            .open_or_create_with_attributes(&attr1);

        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_or_create_with_attributes_fails_when_service_key_types_differ<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let attr = AttributeVerifier::new();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .open_or_create_with_attributes(&attr);
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard::<i64>()
            .open_or_create_with_attributes(&attr);

        assert_that!(sut2, is_err);
    }

    #[test]
    fn open_or_create_with_attributes_failed_when_attribute_isnt_satisfied<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let attr = AttributeVerifier::new()
            .require(&"hello".try_into().unwrap(), &"world".try_into().unwrap());
        let sut = node
            .service_builder(&service_name)
            .blackboard::<i64>()
            .open_or_create_with_attributes(&attr);
        assert_that!(sut, is_ok);

        let attr1 = AttributeVerifier::new().require_key(&"non-exist".try_into().unwrap());
        let sut2 = node
            .service_builder(&service_name)
            .blackboard::<i64>()
            .open_or_create_with_attributes(&attr1);

        assert_that!(sut2, is_err);
    }

    #[test]
    fn creating_non_existing_service_works<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .create();

        assert_that!(sut, is_ok);
        let sut = sut.unwrap();
        assert_that!(*sut.name(), eq service_name);
    }

    #[test]
    fn creating_same_service_twice_fails<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .create();
        assert_that!(sut2, is_err);
        assert_that!(sut2.err().unwrap(), eq BlackboardCreateError::AlreadyExists);
    }

    #[test]
    fn recreate_after_drop_works<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .create();
        assert_that!(sut, is_ok);

        drop(sut);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .create();
        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_does_not_exist<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .open();
        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq BlackboardOpenError::DoesNotExist);
    }

    #[test]
    fn open_succeeds_when_service_does_exist<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .open();
        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_has_wrong_key_type<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard::<i64>()
            .open();
        assert_that!(sut2, is_err);
        assert_that!(sut2.err().unwrap(), eq BlackboardOpenError::IncompatibleKeys);
    }

    #[test]
    fn open_fails_when_service_does_not_satisfy_max_nodes_requirement<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .max_nodes(2)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .max_nodes(3)
            .open();

        assert_that!(sut2, is_err);
        assert_that!(sut2.err().unwrap(), eq BlackboardOpenError::DoesNotSupportRequestedAmountOfNodes);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .max_nodes(1)
            .open();

        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_does_not_satisfy_max_readers_requirement<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .max_readers(2)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .max_readers(3)
            .open();

        assert_that!(sut2, is_err);
        assert_that!(
        sut2.err().unwrap(), eq BlackboardOpenError::DoesNotSupportRequestedAmountOfReaders);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .max_readers(1)
            .open();

        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_does_not_fail_when_service_owner_is_dropped<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .open();
        assert_that!(sut2, is_ok);

        drop(sut);

        let sut3 = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .open();
        assert_that!(sut3, is_ok);
    }

    #[test]
    fn open_fails_when_all_previous_owners_have_been_dropped<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .open();
        assert_that!(sut2, is_ok);

        drop(sut);
        drop(sut2);

        let sut3 = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .open();
        assert_that!(sut3, is_err);
        assert_that!(sut3.err().unwrap(), eq BlackboardOpenError::DoesNotExist);
    }

    #[test]
    fn open_or_create_creates_service_if_it_does_not_exist<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .open_or_create();

        assert_that!(sut, is_ok);
    }

    #[test]
    fn open_or_create_opens_service_if_it_does_exist<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let _sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .create()
            .unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .open_or_create();

        assert_that!(sut, is_ok);
    }

    #[test]
    fn max_readers_is_set_to_config_default<Sut: Service>() {
        let service_name = generate_name();
        let node = NodeBuilder::new().create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .create()
            .unwrap();

        let defaults = &Config::global_config().defaults;

        assert_that!(sut.static_config().max_readers(), eq defaults.blackboard.max_readers);
    }

    #[test]
    fn open_uses_predefined_settings_when_nothing_is_specified<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .max_nodes(89)
            .max_readers(4)
            .create()
            .unwrap();

        assert_that!(sut.static_config().max_nodes(), eq 89);
        assert_that!(sut.static_config().max_readers(), eq 4);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .open()
            .unwrap();

        assert_that!(sut2.static_config().max_nodes(), eq 89);
        assert_that!(sut2.static_config().max_readers(), eq 4);
    }

    #[test]
    fn settings_can_be_modified_via_custom_config<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let mut custom_config = config.clone();
        custom_config.defaults.blackboard.max_nodes = 2;
        custom_config.defaults.blackboard.max_readers = 9;
        // TODO: remove max_writers so that it can't be modified for now?
        custom_config.defaults.blackboard.max_writers = 10;
        let node_1 = NodeBuilder::new()
            .config(&custom_config)
            .create::<Sut>()
            .unwrap();
        let node_2 = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node_1
            .service_builder(&service_name)
            .blackboard::<u64>()
            .create()
            .unwrap();

        assert_that!(sut.static_config().max_nodes(), eq 2);
        assert_that!(sut.static_config().max_readers(), eq 9);

        let sut2 = node_2
            .service_builder(&service_name)
            .blackboard::<u64>()
            .open()
            .unwrap();

        // NOTE: although node_2 did specify a config with default values, since
        // node_1 was created first, the values of that node have to be preset
        assert_that!(sut2.static_config().max_nodes(), eq 2);
        assert_that!(sut2.static_config().max_readers(), eq 9);
    }

    #[test]
    fn type_informations_are_correct<Sut: Service>() {
        type KeyType = u64;
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let service_name = generate_name();

        let sut = node
            .service_builder(&service_name)
            .blackboard::<KeyType>()
            .create()
            .unwrap();

        let d = sut.static_config().type_details();
        assert_that!(d.variant, eq TypeVariant::FixedSize);
        assert_that!(d.type_name, eq core::any::type_name::<KeyType>());
        assert_that!(d.size, eq core::mem::size_of::<KeyType>());
        assert_that!(d.alignment, eq core::mem::align_of::<KeyType>());
    }

    #[test]
    fn max_number_of_nodes_works<Sut: Service>() {
        let service_name = generate_name();
        const MAX_NODES: usize = 8;

        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .max_nodes(MAX_NODES)
            .create();
        assert_that!(sut, is_ok);

        let mut nodes = vec![];
        let mut services = vec![];

        nodes.push(node);
        services.push(sut.unwrap());

        for _ in 1..MAX_NODES {
            let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
            let sut = node
                .service_builder(&service_name)
                .blackboard::<u64>()
                .open();
            assert_that!(sut, is_ok);

            nodes.push(node);
            services.push(sut.unwrap());
        }

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .open();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq BlackboardOpenError::ExceedsMaxNumberOfNodes);

        nodes.pop();
        services.pop();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .open();

        assert_that!(sut, is_ok);
    }

    #[test]
    fn set_max_nodes_to_zero_adjusts_it_to_one<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .max_nodes(0)
            .create()
            .unwrap();

        assert_that!(sut.static_config().max_nodes(), eq 1);
    }

    #[test]
    fn set_max_readers_to_zero_adjusts_it_to_one<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .max_readers(0)
            .create()
            .unwrap();

        assert_that!(sut.static_config().max_readers(), eq 1);
    }

    #[test]
    fn does_exist_works_single<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        assert_that!(Sut::does_exist(&service_name, &config, MessagingPattern::Blackboard).unwrap(), eq false);

        let _sut = node
            .service_builder(&service_name)
            .blackboard::<u64>()
            .create()
            .unwrap();

        assert_that!(Sut::does_exist(&service_name, &config, MessagingPattern::Blackboard).unwrap(), eq true);
        assert_that!(Sut::does_exist(&service_name, &config, MessagingPattern::Blackboard).unwrap(), eq true);

        drop(_sut);

        assert_that!(Sut::does_exist(&service_name, &config, MessagingPattern::Blackboard).unwrap(), eq false);
    }

    #[test]
    fn does_exist_works_many<Sut: Service>() {
        const NUMBER_OF_SERVICES: usize = 8;

        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let mut services = vec![];
        let mut service_names = vec![];

        for i in 0..NUMBER_OF_SERVICES {
            let service_name = generate_name();
            assert_that!(Sut::does_exist(&service_name, &config, MessagingPattern::Blackboard).unwrap(), eq false);

            services.push(
                node.service_builder(&service_name)
                    .blackboard::<u64>()
                    .create()
                    .unwrap(),
            );
            service_names.push(service_name);

            for s in service_names.iter().take(i + 1) {
                assert_that!(Sut::does_exist(s, &config, MessagingPattern::Blackboard).unwrap(), eq true);
            }
        }

        for i in 0..NUMBER_OF_SERVICES {
            for s in service_names.iter().take(NUMBER_OF_SERVICES - i) {
                assert_that!(Sut::does_exist(s, &config, MessagingPattern::Blackboard).unwrap(), eq true);
            }

            for s in service_names
                .iter()
                .take(NUMBER_OF_SERVICES)
                .skip(NUMBER_OF_SERVICES - i)
            {
                assert_that!(Sut::does_exist(s, &config, MessagingPattern::Blackboard).unwrap(), eq false);
            }

            services.pop();
        }
    }

    #[test]
    fn list_works<Sut: Service>() {
        const NUMBER_OF_SERVICES: usize = 8;

        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let mut services = vec![];
        let mut service_names = vec![];

        let contains_service_names = |names, state: Vec<ServiceDetails<Sut>>| {
            for n in names {
                let mut name_found = false;
                for s in &state {
                    if *s.static_details.name() == n {
                        name_found = true;
                        break;
                    }
                }

                if !name_found {
                    return false;
                }
            }

            true
        };

        for i in 0..NUMBER_OF_SERVICES {
            let service_name = generate_name();

            services.push(
                node.service_builder(&service_name)
                    .blackboard::<u64>()
                    .create()
                    .unwrap(),
            );
            service_names.push(service_name);

            let mut service_list = vec![];
            Sut::list(&config, |s| {
                service_list.push(s);
                CallbackProgression::Continue
            })
            .unwrap();
            assert_that!(service_list, len i + 1);

            assert_that!(contains_service_names(service_names.clone(), service_list), eq true);
        }

        for i in 0..NUMBER_OF_SERVICES {
            services.pop();
            service_names.pop();

            let mut service_list = vec![];
            Sut::list(&config, |s| {
                service_list.push(s);
                CallbackProgression::Continue
            })
            .unwrap();
            assert_that!(service_list, len NUMBER_OF_SERVICES - i - 1);
            assert_that!(contains_service_names(service_names.clone(), service_list), eq true);
        }
    }

    #[test]
    fn open_error_display_works<Sut: Service>() {
        assert_that!(format!("{}", BlackboardOpenError::DoesNotExist), eq
    "BlackboardOpenError::DoesNotExist");
        assert_that!(format!("{}", BlackboardOpenError::InternalFailure), eq
    "BlackboardOpenError::InternalFailure");
        assert_that!(format!("{}", BlackboardOpenError::IncompatibleKeys), eq
    "BlackboardOpenError::IncompatibleKeys");
        assert_that!(format!("{}", BlackboardOpenError::IncompatibleMessagingPattern), eq
    "BlackboardOpenError::IncompatibleMessagingPattern");
        assert_that!(format!("{}", BlackboardOpenError::IncompatibleAttributes), eq
    "BlackboardOpenError::IncompatibleAttributes");
        assert_that!(format!("{}", BlackboardOpenError::DoesNotSupportRequestedAmountOfReaders), eq
    "BlackboardOpenError::DoesNotSupportRequestedAmountOfReaders");
        assert_that!(format!("{}", BlackboardOpenError::DoesNotSupportRequestedAmountOfNodes), eq
    "BlackboardOpenError::DoesNotSupportRequestedAmountOfNodes");
        assert_that!(format!("{}", BlackboardOpenError::InsufficientPermissions), eq
    "BlackboardOpenError::InsufficientPermissions");
        assert_that!(format!("{}", BlackboardOpenError::ServiceInCorruptedState), eq
    "BlackboardOpenError::ServiceInCorruptedState");
        assert_that!(format!("{}", BlackboardOpenError::HangsInCreation), eq
    "BlackboardOpenError::HangsInCreation");
        assert_that!(format!("{}", BlackboardOpenError::ExceedsMaxNumberOfNodes), eq
    "BlackboardOpenError::ExceedsMaxNumberOfNodes");
        assert_that!(format!("{}", BlackboardOpenError::IsMarkedForDestruction), eq
    "BlackboardOpenError::IsMarkedForDestruction");
    }

    #[test]
    fn create_error_display_works<Sut: Service>() {
        assert_that!(format!("{}", BlackboardCreateError::ServiceInCorruptedState), eq
    "BlackboardCreateError::ServiceInCorruptedState");
        assert_that!(format!("{}", BlackboardCreateError::HangsInCreation), eq
    "BlackboardCreateError::HangsInCreation");
        assert_that!(format!("{}", BlackboardCreateError::AlreadyExists), eq
    "BlackboardCreateError::AlreadyExists");
        assert_that!(format!("{}", BlackboardCreateError::InsufficientPermissions), eq
    "BlackboardCreateError::InsufficientPermissions");
        assert_that!(format!("{}", BlackboardCreateError::InternalFailure), eq
    "BlackboardCreateError::InternalFailure");
        assert_that!(format!("{}", BlackboardCreateError::IsBeingCreatedByAnotherInstance), eq
    "BlackboardCreateError::IsBeingCreatedByAnotherInstance");
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
