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

#[generic_tests::define]
mod monitoring {
    use iceoryx2_bb_system_types::file_name::*;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::monitoring::*;
    use iceoryx2_cal::named_concept::*;
    use iceoryx2_cal::testing::*;

    #[test]
    fn create_works<Sut: Monitoring>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_token = Sut::Builder::new(&name).config(&config).token();
        assert_that!(sut_token, is_ok);
        assert_that!(*sut_token.as_ref().unwrap().name(), eq name);
    }

    #[test]
    fn create_same_token_twice_fails<Sut: Monitoring>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_token_1 = Sut::Builder::new(&name).config(&config).token();
        assert_that!(sut_token_1, is_ok);

        let sut_token_2 = Sut::Builder::new(&name).config(&config).token();
        assert_that!(sut_token_2, is_err);
        assert_that!(
            sut_token_2.err().unwrap(), eq
            MonitoringCreateTokenError::AlreadyExists
        );
    }

    #[test]
    fn token_removes_resources_when_going_out_of_scope<Sut: Monitoring>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_token_1 = Sut::Builder::new(&name).config(&config).token();
        assert_that!(sut_token_1, is_ok);
        drop(sut_token_1);

        let sut_token_2 = Sut::Builder::new(&name).config(&config).token();
        assert_that!(sut_token_2, is_ok);
    }

    #[test]
    fn create_cleaner_fails_when_token_does_not_exist<Sut: Monitoring>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_cleaner = Sut::Builder::new(&name).config(&config).cleaner();
        assert_that!(sut_cleaner, is_err);
        assert_that!(sut_cleaner.err().unwrap(), eq MonitoringCreateCleanerError::DoesNotExist);
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "macos",
        target_os = "nto"
    )))]
    #[test]
    fn monitor_works<Sut: Monitoring>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_monitor = Sut::Builder::new(&name).config(&config).monitor().unwrap();
        assert_that!(*sut_monitor.name(), eq name);
        assert_that!(sut_monitor.state().unwrap(), eq State::DoesNotExist);

        let sut_token = Sut::Builder::new(&name).config(&config).token();
        assert_that!(sut_monitor.state().unwrap(), eq State::Alive);

        drop(sut_token);
        assert_that!(sut_monitor.state().unwrap(), eq State::DoesNotExist);
    }

    #[test]
    fn list_monitoring_token_works<Sut: Monitoring>() {
        let mut sut_names = vec![];
        const LIMIT: usize = 10;
        let config = generate_isolated_config::<Sut>();

        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config).unwrap(), len 0);
        for i in 0..LIMIT {
            sut_names.push(generate_name());
            assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_names[i], &config), eq Ok(false));
            core::mem::forget(Sut::Builder::new(&sut_names[i]).config(&config).token());
            assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_names[i], &config), eq Ok(true));

            let list = <Sut as NamedConceptMgmt>::list_cfg(&config).unwrap();
            assert_that!(list, len i + 1);
            let does_exist_in_list = |value| {
                for e in &list {
                    if e == value {
                        return true;
                    }
                }
                false
            };

            for name in &sut_names {
                assert_that!(does_exist_in_list(name), eq true);
            }
        }

        for i in 0..LIMIT {
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove_cfg(&sut_names[i], &config)}, eq Ok(true));
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove_cfg(&sut_names[i], &config)}, eq Ok(false));
        }

        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config).unwrap(), len 0);
    }

    #[test]
    fn custom_suffix_keeps_monitoring_token_separated<Sut: Monitoring>() {
        let config = generate_isolated_config::<Sut>();
        let config_1 = config
            .clone()
            .suffix(unsafe { &FileName::new_unchecked(b".suffix_1") });
        let config_2 = config.suffix(unsafe { &FileName::new_unchecked(b".suffix_2") });

        let sut_name = generate_name();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 0);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let sut_1 = Sut::Builder::new(&sut_name)
            .config(&config_1)
            .token()
            .unwrap();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let sut_2 = Sut::Builder::new(&sut_name)
            .config(&config_2)
            .token()
            .unwrap();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 1);

        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap()[0], eq sut_name);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap()[0], eq sut_name);

        assert_that!(*sut_1.name(), eq sut_name);
        assert_that!(*sut_2.name(), eq sut_name);

        core::mem::forget(sut_1);
        core::mem::forget(sut_2);

        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_1)}, eq Ok(true));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_1)}, eq Ok(false));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_2)}, eq Ok(true));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_2)}, eq Ok(false));
    }

    #[test]
    fn defaults_for_configuration_are_set_correctly<Sut: Monitoring>() {
        let config = <Sut as NamedConceptMgmt>::Configuration::default();
        assert_that!(*config.get_suffix(), eq Sut::default_suffix());
        assert_that!(*config.get_path_hint(), eq Sut::default_path_hint());
        assert_that!(*config.get_prefix(), eq Sut::default_prefix());
    }

    #[instantiate_tests(<iceoryx2_cal::monitoring::file_lock::FileLockMonitoring>)]
    mod file_lock {}

    #[instantiate_tests(<iceoryx2_cal::monitoring::process_local::ProcessLocalMonitoring>)]
    mod process_local {}
}
