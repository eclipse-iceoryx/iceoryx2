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


def test_attribute_verifier_require_works() -> None:
    key = iox2.AttributeKey.new("ding dong")
    value = iox2.AttributeValue.new("rainbow")

    sut = iox2.AttributeVerifier.new().require(key, value)
    specifier = iox2.AttributeSpecifier.new().define(key, value)

    assert sut.required_attributes == specifier.attributes
    assert sut.required_attributes.number_of_attributes == 1
    attribute = sut.required_attributes.values
    assert attribute[0].key == key
    assert attribute[0].value == value


def test_attribute_verifier_require_key_works() -> None:
    key = iox2.AttributeKey.new("me and my brain")
    value = iox2.AttributeValue.new("we are not friends")

    sut = iox2.AttributeVerifier.new().require_key(key)
    specifier = iox2.AttributeSpecifier.new().define(key, value)

    keys = sut.required_keys
    assert len(keys) == 1
    assert keys[0] == key

    assert sut.verify_requirements(specifier.attributes) is None
