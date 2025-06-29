# Copyright (c) 2025 Contributors to the Eclipse Foundation
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

import iceoryx2 as iox2
import pytest


def test_duration_from_secs_works() -> None:
    sut = iox2.Duration.from_secs(2)
    assert sut.as_secs() == 2


def test_duration_from_millis_works() -> None:
    sut = iox2.Duration.from_millis(25)
    assert sut.as_millis() == 25


def test_duration_from_micros_works() -> None:
    sut = iox2.Duration.from_micros(981)
    assert sut.as_micros() == 981


def test_duration_from_nanos_works() -> None:
    sut = iox2.Duration.from_nanos(421)
    assert sut.as_nanos() == 421


def test_duration_from_secs_f64_works() -> None:
    sut = iox2.Duration.from_secs_f64(12.34)
    assert sut.as_secs() == 12
    assert sut.as_millis() == 12340
    assert sut.as_secs_f64() == 12.34


def test_duration_subsec_micros_works() -> None:
    sut = iox2.Duration.from_secs_f64(2.34567891011)
    assert sut.subsec_micros() == 345678


def test_duration_subsec_millis_works() -> None:
    sut = iox2.Duration.from_secs_f64(3.4567890101112)
    assert sut.subsec_millis() == 456


def test_duration_subsec_nanos_works() -> None:
    sut = iox2.Duration.from_secs_f64(4.567891011)
    assert sut.subsec_nanos() == 567891011
