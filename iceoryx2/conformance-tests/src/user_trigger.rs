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

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod user_trigger {
    use alloc::format;

    use iceoryx2::prelude::*;
    use iceoryx2::user_trigger::{
        UserTriggerCreateError, UserTriggerNotifyError, UserTriggerResetError,
    };
    use iceoryx2::waitset::WaitSetRunResult;
    use iceoryx2_bb_posix::thread::thread_scope;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_testing::*;

    const NUMBER_OF_NOTIFICATIONS: usize = 10000;

    fn is_send_and_sync<T: Send + Sync>(_value: &T) -> bool {
        true
    }

    #[conformance_test]
    pub fn user_trigger_is_send_and_sync<S: Service>() {
        let test = Test::<S>::new();
        let node = test.create_node();
        let sut: UserTrigger<S> = node.create_user_trigger().unwrap();

        assert_that!(is_send_and_sync(&sut), eq true);
    }

    #[conformance_test]
    pub fn reset_without_notification_reports_no_notification<S: Service>() {
        let test = Test::<S>::new();
        let node = test.create_node();
        let sut = node.create_user_trigger().unwrap();

        assert_that!(sut.reset(), eq Ok(false));
    }

    #[conformance_test]
    pub fn reset_after_notification_reports_the_notification_once<S: Service>() {
        let test = Test::<S>::new();
        let node = test.create_node();
        let sut = node.create_user_trigger().unwrap();

        assert_that!(sut.notify(), is_ok);

        assert_that!(sut.reset(), eq Ok(true));
        assert_that!(sut.reset(), eq Ok(false));
    }

    #[conformance_test]
    pub fn many_notifications_before_a_reset_are_reported_as_one<S: Service>() {
        let test = Test::<S>::new();
        let node = test.create_node();
        let sut = node.create_user_trigger().unwrap();

        for _ in 0..NUMBER_OF_NOTIFICATIONS {
            assert_that!(sut.notify(), is_ok);
        }

        assert_that!(sut.reset(), eq Ok(true));
        assert_that!(sut.reset(), eq Ok(false));
    }

    #[conformance_test]
    pub fn user_triggers_do_not_notify_each_other<S: Service>() {
        let test = Test::<S>::new();
        let node = test.create_node();
        let sut_1 = node.create_user_trigger().unwrap();
        let sut_2 = node.create_user_trigger().unwrap();

        assert_that!(sut_1.notify(), is_ok);

        assert_that!(sut_2.reset(), eq Ok(false));
        assert_that!(sut_1.reset(), eq Ok(true));
    }

    #[conformance_test]
    pub fn notified_user_trigger_wakes_up_waitset<S: Service>() {
        let test = Test::<S>::new();
        let node = test.create_node();
        let sut = node.create_user_trigger().unwrap();
        let waitset = node.waitset_builder().create().unwrap();
        let guard = waitset.attach_notification(&sut).unwrap();

        assert_that!(sut.notify(), is_ok);

        let mut woken_up_by_sut = false;
        let result = waitset.wait_and_process_once(|attachment_id| {
            woken_up_by_sut = attachment_id.has_event_from(&guard);
            CallbackProgression::Continue
        });

        assert_that!(result, eq Ok(WaitSetRunResult::AllEventsHandled));
        assert_that!(woken_up_by_sut, eq true);
        assert_that!(sut.reset(), eq Ok(true));
    }

    #[conformance_test]
    pub fn user_trigger_without_notification_does_not_wake_up_waitset<S: Service>() {
        let test = Test::<S>::new();
        let node = test.create_node();
        let sut = node.create_user_trigger().unwrap();
        let other = node.create_user_trigger().unwrap();
        let waitset = node.waitset_builder().create().unwrap();
        let guard = waitset.attach_notification(&sut).unwrap();
        let _other_guard = waitset.attach_notification(&other).unwrap();

        assert_that!(other.notify(), is_ok);

        let mut woken_up_by_sut = false;
        let result = waitset.wait_and_process_once(|attachment_id| {
            woken_up_by_sut |= attachment_id.has_event_from(&guard);
            CallbackProgression::Continue
        });

        assert_that!(result, eq Ok(WaitSetRunResult::AllEventsHandled));
        assert_that!(woken_up_by_sut, eq false);
    }

    #[conformance_test]
    pub fn reset_user_trigger_does_not_wake_up_waitset_again<S: Service>() {
        let test = Test::<S>::new();
        let node = test.create_node();
        let sut = node.create_user_trigger().unwrap();
        let other = node.create_user_trigger().unwrap();
        let waitset = node.waitset_builder().create().unwrap();
        let guard = waitset.attach_notification(&sut).unwrap();
        let _other_guard = waitset.attach_notification(&other).unwrap();

        for _ in 0..NUMBER_OF_NOTIFICATIONS {
            assert_that!(sut.notify(), is_ok);
        }
        let result = waitset.wait_and_process_once(|_| {
            assert_that!(sut.reset(), eq Ok(true));
            CallbackProgression::Continue
        });
        assert_that!(result, is_ok);

        assert_that!(other.notify(), is_ok);

        let mut woken_up_by_sut = false;
        let result = waitset.wait_and_process_once(|attachment_id| {
            woken_up_by_sut |= attachment_id.has_event_from(&guard);
            CallbackProgression::Continue
        });

        assert_that!(result, eq Ok(WaitSetRunResult::AllEventsHandled));
        assert_that!(woken_up_by_sut, eq false);
    }

    #[conformance_test]
    pub fn notification_from_another_thread_wakes_up_blocked_waitset<S: Service>() {
        let test = Test::<S>::new();
        let node = test.create_node();
        let sut = node.create_user_trigger().unwrap();
        let waitset = node.waitset_builder().create().unwrap();
        let guard = waitset.attach_notification(&sut).unwrap();

        thread_scope(|s| {
            s.thread_builder().spawn(|| {
                assert_that!(sut.notify(), is_ok);
            })?;

            let mut woken_up_by_sut = false;
            let result = waitset.wait_and_process_once(|attachment_id| {
                woken_up_by_sut = attachment_id.has_event_from(&guard);
                CallbackProgression::Continue
            });

            assert_that!(result, eq Ok(WaitSetRunResult::AllEventsHandled));
            assert_that!(woken_up_by_sut, eq true);
            assert_that!(sut.reset(), eq Ok(true));

            Ok(())
        })
        .unwrap();
    }

    #[conformance_test]
    pub fn error_display_works<S: Service>() {
        assert_that!(
            format!("{}", UserTriggerCreateError::InsufficientResources), eq "UserTriggerCreateError::InsufficientResources");
        assert_that!(
            format!("{}", UserTriggerCreateError::InternalError), eq "UserTriggerCreateError::InternalError");
        assert_that!(
            format!("{}", UserTriggerNotifyError::InternalError), eq "UserTriggerNotifyError::InternalError");
        assert_that!(
            format!("{}", UserTriggerResetError::InternalError), eq "UserTriggerResetError::InternalError");
    }
}
