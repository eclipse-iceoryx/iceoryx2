# Copyright (c) 2026 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache Software License 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
# which is available at https://opensource.org/licenses/MIT.
#
# SPDX-License-Identifier: Apache-2.0 OR MIT

"""Each callback contract case runs in a child process with a hard timeout."""

import gc
import logging
import subprocess  # nosec B404 - fixed test entry point with no shell
import sys
import threading
import traceback
from pathlib import Path

import iceoryx2 as iox
import pytest

CONTINUE = iox.CallbackProgression.Continue
STOP = iox.CallbackProgression.Stop


def setup(kind, count=2):
    node = (
        iox.NodeBuilder.new()
        .config(iox.testing.generate_isolated_config())
        .create(kind)
    )
    service = (
        node.service_builder(iox.testing.generate_service_name())
        .event()
        .max_listeners(max(count, 1))
        .max_notifiers(4)
        .event_id_max_value(100)
        .create()
    )
    listeners = [
        service.listener_builder().name(iox.PortName.new(f"listener-{i}")).create()
        for i in range(count)
    ]
    notifier = service.notifier_builder().default_event_id(iox.EventId.new(7)).create()
    return node, service, listeners, notifier


def error(call, typ=RuntimeError, message=None):
    try:
        call()
    except typ as caught:
        if message is not None:
            assert message in str(caught), str(caught)
        return caught
    raise AssertionError(f"Expected {typ.__name__}")


def mono_calls(mono):
    return [
        mono.notify,
        lambda: mono.notify_with_custom_event_id(iox.EventId.new(8)),
        mono.listener_key,
    ]


def expired(mono):
    for call in mono_calls(mono):
        error(call, message="callback has ended")


def drain(listener, expected):
    assert [
        (event.id.as_value, event.count) for event in listener.try_wait()
    ] == expected


def delivery(kind):
    node, _service, listeners, notifier = setup(kind)
    kept = []

    def callback(mono, details):
        kept.append((mono, mono.listener_key(), details))
        if details.listener_name.as_str() == "listener-1":
            mono.notify()
            mono.notify_with_custom_event_id(iox.EventId.new(8))
        return CONTINUE

    notifier.for_each_listener(callback)
    assert len(kept) == 2
    drain(listeners[0], [])
    drain(listeners[1], [(7, 1), (8, 1)])
    for mono, key, details in kept:
        expired(mono)
        index = int(details.listener_name.as_str().split("-")[1])
        assert details.listener_id == listeners[index].id
        assert details.node_id == node.id
        for _ in range(3):
            notifier.notify_single_listener(key)
            drain(listeners[index], [(7, 1)])
            notifier.notify_single_listener_with_custom_event_id(
                key, iox.EventId.new(9)
            )
            drain(listeners[index], [(9, 1)])

    for listener in listeners:
        listener.delete()
    for mono, key, details in kept:
        assert details.listener_name.as_str().startswith("listener-")
        assert details.listener_id.value > 0
        assert details.node_id.value > 0
        error(
            lambda key=key: notifier.notify_single_listener(key),
            iox.NotifierNotifyError,
        )


def iterations(kind):
    _node, _service, _listeners, notifier = setup(kind)
    kept = []

    def callback(mono, _details):
        for old in kept:
            expired(old)
        mono.notify()
        kept.append(mono)
        return CONTINUE

    for _ in range(3):
        notifier.for_each_listener(callback)
        for mono in kept:
            expired(mono)
    stopped = []
    notifier.for_each_listener(lambda mono, details: (stopped.append(mono), STOP)[1])
    assert len(stopped) == 1
    expired(stopped[0])
    notifier.for_each_listener(callback)
    expired(stopped[0])
    notifier.delete()
    del notifier
    gc.collect()
    for mono in kept + stopped:
        expired(mono)


def restricted(kind):
    _node, service, _listeners, notifier = setup(kind, 1)
    other = service.notifier_builder().create()
    alias = notifier

    def callback(mono, _details):
        key = mono.listener_key()
        calls = [
            alias.notify,
            lambda: alias.notify_with_custom_event_id(iox.EventId.new(8)),
            lambda: alias.id,
            lambda: alias.name,
            lambda: alias.deadline,
            alias.delete,
            lambda: alias.for_each_listener(lambda m, d: CONTINUE),
            lambda: alias.notify_single_listener(key),
            lambda: alias.notify_single_listener_with_custom_event_id(
                key, iox.EventId.new(8)
            ),
        ]
        for call in calls:
            error(call, message="being traversed")
        assert other.notify() == 1
        inner = []

        def nested(m, _details):
            inner.append(m)
            # The outer callback is still active while a different notifier traverses.
            mono.notify()
            m.notify()
            return STOP

        other.for_each_listener(nested)
        expired(inner[0])
        mono.notify()
        return STOP

    notifier.for_each_listener(callback)
    assert notifier.notify() == 1
    assert notifier.id is not None
    notifier.delete()


def threaded(kind):
    _node, service, _listeners, notifier = setup(kind, 1)
    other = service.notifier_builder().create()
    kept = []

    def callback(mono, _details):
        kept.append(mono)
        key = mono.listener_key()
        failures = []
        completed = threading.Event()

        def worker():
            try:
                for call in mono_calls(mono):
                    error(call, message="another thread")
                for call in [
                    notifier.notify,
                    lambda: notifier.notify_with_custom_event_id(iox.EventId.new(8)),
                    lambda: notifier.id,
                    lambda: notifier.name,
                    lambda: notifier.deadline,
                    notifier.delete,
                    lambda: notifier.for_each_listener(lambda m, d: STOP),
                    lambda key=key: notifier.notify_single_listener(key),
                    lambda: notifier.notify_single_listener_with_custom_event_id(
                        key, iox.EventId.new(8)
                    ),
                ]:
                    error(call, message="being traversed")
                assert other.notify() == 1
                other.for_each_listener(lambda m, d: (m.notify(), STOP)[1])
            except BaseException as caught:
                failures.append(caught)
            finally:
                completed.set()

        thread = threading.Thread(target=worker, daemon=True)
        thread.start()
        # A child-process timeout also catches a C lock wait that holds the GIL.
        thread.join(3)
        assert completed.is_set() and not thread.is_alive(), "callback worker hung"
        assert not failures, failures
        mono.notify()
        return STOP

    notifier.for_each_listener(callback)
    expired(kept[0])
    failures = []

    def after():
        try:
            expired(kept[0])
            assert notifier.notify() == 1
        except BaseException as caught:
            failures.append(caught)

    thread = threading.Thread(target=after, daemon=True)
    thread.start()
    thread.join(3)
    assert not thread.is_alive()
    assert not failures, failures


def exception(kind):
    _node, _service, listeners, notifier = setup(kind, 1)
    kept = []

    class Sentinel(Exception):
        pass

    marker = Sentinel("original", 42)
    marker.payload = {"answer": 42}
    cause = ValueError("cause")
    context = LookupError("context")

    def callback(mono, _details):
        kept.append(mono)
        mono.notify()
        try:
            raise context
        except LookupError:
            raise marker from cause

    caught = error(lambda: notifier.for_each_listener(callback), Sentinel)
    assert caught is marker
    assert type(caught) is Sentinel  # pylint: disable=unidiomatic-typecheck
    assert caught.args == ("original", 42)
    assert caught.payload == {"answer": 42}
    assert caught.__cause__ is cause and caught.__context__ is context
    assert "callback" in [
        frame.name for frame in traceback.extract_tb(caught.__traceback__)
    ]
    expired(kept[0])
    drain(listeners[0], [(7, 1)])
    notifier.for_each_listener(lambda m, d: (m.notify(), STOP)[1])
    drain(listeners[0], [(7, 1)])
    assert notifier.notify() == 1
    drain(listeners[0], [(7, 1)])


def invalid_result(kind):
    _node, _service, listeners, notifier = setup(kind, 1)
    kept = []
    for result in [None, True, 0, "Continue", object()]:

        def callback(mono, _details, result=result):
            kept.append(mono)
            return result

        error(lambda callback=callback: notifier.for_each_listener(callback), TypeError)
        expired(kept[-1])
        assert notifier.notify() == 1
        drain(listeners[0], [(7, 1)])
        notifier.for_each_listener(lambda m, d: STOP)

    outcomes = []

    class ReturnValue:
        def __init__(self, mono):
            self.mono = mono

        def __del__(self):
            # Cleanup of the invalid result must see an already-expired handle.
            try:
                expired(self.mono)
                outcomes.append("expired")
            except BaseException as caught:
                outcomes.append(str(caught))

    error(lambda: notifier.for_each_listener(lambda m, d: ReturnValue(m)), TypeError)
    gc.collect()
    assert outcomes == ["expired"], outcomes
    assert notifier.notify() == 1


def empty(kind):
    _node, _service, _listeners, notifier = setup(kind, 0)
    visited = []
    notifier.for_each_listener(lambda m, d: visited.append(m))
    assert not visited
    assert notifier.notify() == 0


def saved_keys_and_bounds(kind):
    _node, service, listeners, notifier = setup(kind, 1)
    saved = []
    notifier.for_each_listener(lambda m, d: (saved.append(m.listener_key()), STOP)[1])
    old_key = saved[0]
    error(
        lambda: notifier.notify_single_listener_with_custom_event_id(
            old_key, iox.EventId.new(101)
        ),
        iox.NotifierNotifyError,
        "EventIdOutOfBounds",
    )
    listeners[0].delete()
    error(
        lambda: notifier.notify_single_listener(old_key),
        iox.NotifierNotifyError,
        "InvalidListenerKey",
    )
    replacement = service.listener_builder().create()
    for call in [
        lambda: notifier.notify_single_listener(old_key),
        lambda: notifier.notify_single_listener_with_custom_event_id(
            old_key, iox.EventId.new(8)
        ),
    ]:
        error(call, iox.NotifierNotifyError, "InvalidListenerKey")
    drain(replacement, [])
    fresh = []
    notifier.for_each_listener(lambda m, d: (fresh.append(m.listener_key()), STOP)[1])
    notifier.notify_single_listener(fresh[0])
    drain(replacement, [(7, 1)])

    source_node, source_service, source_listeners, source = setup(kind, 2)
    keys = []
    source.for_each_listener(lambda m, d: (keys.append(m.listener_key()), CONTINUE)[1])
    assert len(keys) == 2
    for key in keys:
        error(
            lambda key=key: notifier.notify_single_listener(key),
            iox.NotifierNotifyError,
            "InvalidListenerKey",
        )
        error(
            lambda key=key: notifier.notify_single_listener_with_custom_event_id(
                key, iox.EventId.new(8)
            ),
            iox.NotifierNotifyError,
            "InvalidListenerKey",
        )
        error(
            lambda key=key: notifier.notify_single_listener_with_custom_event_id(
                key, iox.EventId.new(101)
            ),
            iox.NotifierNotifyError,
            "EventIdOutOfBounds",
        )
    drain(replacement, [])
    assert (
        source_node is not None
        and source_service is not None
        and len(source_listeners) == 2
    )


def monofier_event_bounds(kind):
    _node, _service, listeners, notifier = setup(kind, 1)

    def callback(mono, _details):
        error(
            lambda: mono.notify_with_custom_event_id(iox.EventId.new(101)),
            iox.NotifierNotifyError,
            "EventIdOutOfBounds",
        )
        mono.notify()
        return STOP

    notifier.for_each_listener(callback)
    drain(listeners[0], [(7, 1)])


def ordinary_waiting(kind):
    # The Rust failure log enters this Python handler while an ordinary operation
    # owns admission. Event.wait releases the GIL, exposing real contention even
    # on GIL-enabled CPython. A waiter must neither fail fast nor hold the GIL.
    logging.basicConfig(level=logging.DEBUG)
    iox.set_log_level(iox.LogLevel.Debug)
    _node, _service, listeners, notifier = setup(kind, 1)
    for traverse in (False, True):
        ordinary_waiting_case(notifier, listeners[0], traverse)


def ordinary_waiting_case(notifier, listener, traverse):
    entered = threading.Event()
    release = threading.Event()
    started = threading.Event()
    finished = threading.Event()
    failures = []

    class HoldOperation(logging.Handler):
        def emit(self, record):
            if "exceeds the maximum supported EventId" in record.getMessage():
                try:
                    error(lambda: notifier.id, message="cannot re-enter itself")
                    entered.set()
                    assert release.wait(3), "ordinary operation was not released"
                except BaseException as caught:
                    failures.append(caught)

    handler = HoldOperation()
    logging.getLogger().addHandler(handler)

    def owner():
        try:
            error(
                lambda: notifier.notify_with_custom_event_id(iox.EventId.new(101)),
                iox.NotifierNotifyError,
                "EventIdOutOfBounds",
            )
        except BaseException as caught:
            failures.append(caught)

    def waiter():
        try:
            started.set()
            if traverse:
                notifier.for_each_listener(lambda m, d: (m.notify(), STOP)[1])
            else:
                assert notifier.notify() == 1
        except BaseException as caught:
            failures.append(caught)
        finally:
            finished.set()

    first = threading.Thread(target=owner, daemon=True)
    second = threading.Thread(target=waiter, daemon=True)
    try:
        first.start()
        assert entered.wait(3), "Rust failure log did not reach Python"
        second.start()
        assert started.wait(3)
        assert not finished.wait(0.05), "competing ordinary access must wait"
    finally:
        release.set()
        first.join(3)
        if second.ident is not None:
            second.join(3)
        logging.getLogger().removeHandler(handler)
    assert not first.is_alive() and not second.is_alive()
    assert finished.is_set()
    assert not failures, failures
    drain(listener, [(7, 1)])


CASES = [
    delivery,
    iterations,
    restricted,
    threaded,
    exception,
    invalid_result,
    empty,
    saved_keys_and_bounds,
    monofier_event_bounds,
    ordinary_waiting,
]


@pytest.mark.parametrize("kind", ["Local", "Ipc"])
@pytest.mark.parametrize("case", [case.__name__ for case in CASES])
def test_contract(case, kind):
    result = subprocess.run(  # nosec B603 - executable and test cases are controlled
        [sys.executable, str(Path(__file__).resolve()), case, kind],
        capture_output=True,
        text=True,
        timeout=15,
        check=False,
    )
    assert result.returncode == 0, result.stdout + result.stderr


if __name__ == "__main__":
    globals()[sys.argv[1]](getattr(iox.ServiceType, sys.argv[2]))
    print(f"PASS {sys.argv[1]} {sys.argv[2]}")
