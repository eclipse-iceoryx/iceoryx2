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

#include "iox2/container/optional.hpp"

#include "testing/observable.hpp"

#include <gtest/gtest.h>

namespace {
using iox2::container::testing::Observable;

class Optional : public iox2::container::testing::DetectLeakedObservablesFixture { };

TEST_F(Optional, DefaultConstructor) {
    // [optional.ctor] / 2
    {
        iox2::container::Optional<int> o;
        ASSERT_FALSE(o.has_value());
    }
    // [optional.ctor] / 3
    {
        // No contained value is initialized
        Observable::s_counter.wasInitialized = 0;
        iox2::container::Optional<Observable> o;
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
    }
}

TEST_F(Optional, NulloptConstructor) {
    {
        iox2::container::Optional<int> o(iox2::container::NulloptT {});
        ASSERT_FALSE(o.has_value());
    }
    {
        Observable::s_counter.wasInitialized = 0;
        iox2::container::Optional<Observable> o(iox2::container::NulloptT {});
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
    }
}

TEST_F(Optional, ValueConstructor) {
    {
        iox2::container::Optional<int> o(42);
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(*o, 42);
    }
    {
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        iox2::container::Optional<Observable> o(Observable {});
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(Observable::s_counter.wasInitialized, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 1);
    }
    {
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable value;
        value.id = 9999;
        iox2::container::Optional<Observable> o(value);
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(o->id, 9999);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 1);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 1);
    }
}

TEST_F(Optional, Destructor) {
    Observable::s_counter.wasDestructed = 0;
    {
        iox2::container::Optional<Observable> o(iox2::container::NulloptT {});
        ASSERT_TRUE(!o.has_value());
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 0);

    Observable::s_counter.wasDestructed = 0;
    {
        iox2::container::Optional<Observable> o(Observable {});
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
        Observable::s_counter.wasDestructed = 0;
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
}

TEST_F(Optional, CopyConstruction_FromEmpty) {
    {
        iox2::container::Optional<int> empty;
        iox2::container::Optional<int> o { empty };
        ASSERT_TRUE(!o.has_value());
    }
    {
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasDestructed = 0;
        iox2::container::Optional<Observable> empty;
        iox2::container::Optional<Observable> o { empty };
        ASSERT_TRUE(!o.has_value());
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
}

TEST_F(Optional, CopyConstruction_FromFull) {
    {
        iox2::container::Optional<int> full(42);
        iox2::container::Optional<int> o { full };
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(*o, 42);
    }
    {
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasDestructed = 0;
        iox2::container::Optional<Observable> full(Observable {});
        ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
        Observable::s_counter.wasDestructed = 0;
        ASSERT_EQ(Observable::s_counter.wasInitialized, 1);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        full->id = 12345;
        iox2::container::Optional<Observable> o { full };
        ASSERT_EQ(Observable::s_counter.wasInitialized, 1);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 1);
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(o->id, 12345);
        ASSERT_TRUE(full.has_value());
        ASSERT_EQ(full->id, 12345);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 2);
}

TEST_F(Optional, MoveConstruction_FromEmpty) {
    {
        iox2::container::Optional<int> empty;
        iox2::container::Optional<int> o { std::move(empty) };
        ASSERT_TRUE(!o.has_value());
    }
    {
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasDestructed = 0;
        iox2::container::Optional<Observable> empty;
        iox2::container::Optional<Observable> o { std::move(empty) };
        ASSERT_TRUE(!o.has_value());
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
}

TEST_F(Optional, MoveConstruction_FromFull) {
    {
        iox2::container::Optional<int> full(42);
        iox2::container::Optional<int> o { std::move(full) };
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(*o, 42);
    }
    {
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasDestructed = 0;
        iox2::container::Optional<Observable> full(Observable {});
        ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
        Observable::s_counter.wasDestructed = 0;
        ASSERT_EQ(Observable::s_counter.wasInitialized, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 1);
        Observable::s_counter.wasMoveConstructed = 0;
        full->id = 12345;
        iox2::container::Optional<Observable> o { std::move(full) };
        ASSERT_EQ(Observable::s_counter.wasInitialized, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 1);
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(o->id, 12345);
        ASSERT_TRUE(full.has_value());
        ASSERT_EQ(full->id, 12345);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 2);
}

TEST_F(Optional, CopyAssignment_EmptyToEmpty) {
    {
        iox2::container::Optional<int> o;
        iox2::container::Optional<int> empty;
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(!empty.has_value());
        o = empty;
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(!empty.has_value());
    }
    {
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        iox2::container::Optional<Observable> o;
        iox2::container::Optional<Observable> empty;
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(!empty.has_value());
        o = empty;
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(!empty.has_value());
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
}

TEST_F(Optional, CopyAssignment_EmptyToFull) {
    {
        iox2::container::Optional<int> o { 42 };
        iox2::container::Optional<int> empty;
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(!empty.has_value());
        o = empty;
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(!empty.has_value());
    }
    {
        iox2::container::Optional<Observable> o { Observable {} };
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        iox2::container::Optional<Observable> empty;
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(!empty.has_value());
        o = empty;
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(!empty.has_value());
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
        Observable::s_counter.wasDestructed = 0;
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
}

TEST_F(Optional, CopyAssignment_FullToEmpty) {
    {
        iox2::container::Optional<int> o;
        iox2::container::Optional<int> full { 42 };
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(full.has_value());
        o = full;
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(full.has_value());
        ASSERT_EQ(*o, 42);
        ASSERT_EQ(*full, 42);
    }
    {
        iox2::container::Optional<Observable> o;
        iox2::container::Optional<Observable> full { Observable {} };
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(full.has_value());
        full->id = 12345;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        o = full;
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(full.has_value());
        ASSERT_EQ(o->id, 12345);
        ASSERT_EQ(full->id, 12345);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 2);
}

TEST_F(Optional, CopyAssignment_FullToFull) {
    {
        iox2::container::Optional<int> o { -99 };
        iox2::container::Optional<int> full { 42 };
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(full.has_value());
        ASSERT_EQ(*o, -99);
        o = full;
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(full.has_value());
        ASSERT_EQ(*o, 42);
        ASSERT_EQ(*full, 42);
    }
    {
        iox2::container::Optional<Observable> o { Observable {} };
        iox2::container::Optional<Observable> full { Observable {} };
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(full.has_value());
        o->id = 111111;
        full->id = 12345;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        o = full;
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(full.has_value());
        ASSERT_EQ(o->id, 12345);
        ASSERT_EQ(full->id, 12345);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 2);
}

TEST_F(Optional, MoveAssignment_EmptyToEmpty) {
    {
        iox2::container::Optional<int> o;
        iox2::container::Optional<int> empty;
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(!empty.has_value());
        o = std::move(empty);
        ASSERT_TRUE(!o.has_value());
    }
    {
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        iox2::container::Optional<Observable> o;
        iox2::container::Optional<Observable> empty;
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(!empty.has_value());
        o = std::move(empty);
        ASSERT_TRUE(!o.has_value());
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
}

TEST_F(Optional, MoveAssignment_EmptyToFull) {
    {
        iox2::container::Optional<int> o { 42 };
        iox2::container::Optional<int> empty;
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(!empty.has_value());
        o = std::move(empty);
        ASSERT_TRUE(!o.has_value());
    }
    {
        iox2::container::Optional<Observable> o { Observable {} };
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        iox2::container::Optional<Observable> empty;
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(!empty.has_value());
        o = std::move(empty);
        ASSERT_TRUE(!o.has_value());
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
        Observable::s_counter.wasDestructed = 0;
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
}

TEST_F(Optional, MoveAssignment_FullToEmpty) {
    {
        iox2::container::Optional<int> o;
        iox2::container::Optional<int> full { 42 };
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(full.has_value());
        o = std::move(full);
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(*o, 42);
    }
    {
        iox2::container::Optional<Observable> o;
        iox2::container::Optional<Observable> full { Observable {} };
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(full.has_value());
        full->id = 12345;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        o = std::move(full);
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(o->id, 12345);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 2);
}

TEST_F(Optional, MoveAssignment_FullToFull) {
    {
        iox2::container::Optional<int> o { -99 };
        iox2::container::Optional<int> full { 42 };
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(full.has_value());
        ASSERT_EQ(*o, -99);
        o = std::move(full);
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(*o, 42);
    }
    {
        iox2::container::Optional<Observable> o { Observable {} };
        iox2::container::Optional<Observable> full { Observable {} };
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(full.has_value());
        o->id = 111111;
        full->id = 12345;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        o = std::move(full);
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(o->id, 12345);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 1);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 2);
}

TEST_F(Optional, Assignment_NulloptToEmpty) {
    {
        iox2::container::Optional<int> o;
        ASSERT_TRUE(!o.has_value());
        o = iox2::container::NulloptT {};
        ASSERT_TRUE(!o.has_value());
    }
    {
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        iox2::container::Optional<Observable> o;
        ASSERT_TRUE(!o.has_value());
        o = iox2::container::NulloptT {};
        ASSERT_TRUE(!o.has_value());
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
}

TEST_F(Optional, Assignment_NulloptToFull) {
    {
        iox2::container::Optional<int> o { 42 };
        ASSERT_TRUE(o.has_value());
        o = iox2::container::NulloptT {};
        ASSERT_TRUE(!o.has_value());
    }
    {
        iox2::container::Optional<Observable> o { Observable {} };
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        ASSERT_TRUE(o.has_value());
        o = iox2::container::NulloptT {};
        ASSERT_TRUE(!o.has_value());
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
        Observable::s_counter.wasDestructed = 0;
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
}

TEST_F(Optional, OperatorArrowPointerAccess) {
    iox2::container::Optional<int> o;
    ASSERT_EQ(o.operator->(), nullptr);
    o = 42;
    ASSERT_NE(o.operator->(), nullptr);
    ASSERT_EQ(*(o.operator->()), 42);
}

TEST_F(Optional, OperatorArrowConstPointerAccess) {
    iox2::container::Optional<int> const o1;
    ASSERT_EQ(o1.operator->(), nullptr);
    iox2::container::Optional<int> const o2 { 42 };
    ASSERT_NE(o2.operator->(), nullptr);
    ASSERT_EQ(*(o2.operator->()), 42);
}

TEST_F(Optional, OperatorStarDereferenceAccess) {
    iox2::container::Optional<int> o { 42 };
    ASSERT_EQ(*o, 42);
    *o = 55;
    ASSERT_EQ(*o, 55);
}

TEST_F(Optional, OperatorStarConstDereferenceAccess) {
    iox2::container::Optional<int> const o1 { 42 };
    ASSERT_EQ(*o1, 42);
    iox2::container::Optional<int> const o2 { 55 };
    ASSERT_EQ(*o2, 55);
}

TEST_F(Optional, OperatorStarMoveAccess) {
    Observable value;
    value.id = 12345;
    {
        iox2::container::Optional<Observable> o { value };
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        Observable m = *std::move(o);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
        ASSERT_EQ(m.id, 12345);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 2);
}

TEST_F(Optional, OperatorStarConstRvalueAccess) {
    Observable value;
    value.id = 12345;
    {
        iox2::container::Optional<Observable> const o { value };
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        Observable const&& ref = *std::move(o);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
        ASSERT_EQ(ref.id, 12345);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
}

TEST_F(Optional, OperatorBool) {
    iox2::container::Optional<int> o;
    ASSERT_FALSE(static_cast<bool>(o));
    o = 42;
    ASSERT_TRUE(static_cast<bool>(o));
}

TEST_F(Optional, HasValue) {
    iox2::container::Optional<int> o;
    ASSERT_FALSE(o.has_value());
    o = 42;
    ASSERT_TRUE(o.has_value());
}

TEST_F(Optional, ValueAccess) {
    iox2::container::Optional<int> o { 42 };
    ASSERT_EQ(o.value(), 42);
    o.value() = 55;
    ASSERT_EQ(o.value(), 55);
}

TEST_F(Optional, ValueConstAccess) {
    iox2::container::Optional<int> const o1 { 42 };
    ASSERT_EQ(o1.value(), 42);
    iox2::container::Optional<int> const o2 { 55 };
    ASSERT_EQ(o2.value(), 55);
}

TEST_F(Optional, ValueMoveAccess) {
    Observable value;
    value.id = 12345;
    {
        iox2::container::Optional<Observable> o { value };
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        Observable m = std::move(o).value();
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
        ASSERT_EQ(m.id, 12345);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 2);
}

TEST_F(Optional, ValueConstRvalueAccess) {
    Observable value;
    value.id = 12345;
    {
        iox2::container::Optional<Observable> const o { value };
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        Observable const&& ref = std::move(o).value();
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
        ASSERT_EQ(ref.id, 12345);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
}

TEST_F(Optional, ValueOr_Full) {
    {
        iox2::container::Optional<int> o { 42 };
        ASSERT_EQ(o.value_or(-1), 42);
    }
    {
        iox2::container::Optional<Observable> o { Observable {} };
        o->id = 12345;
        Observable fallback;
        fallback.id = -1;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        ASSERT_EQ(o.value_or(fallback).id, 12345);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
    }
    {
        iox2::container::Optional<Observable> o { Observable {} };
        o->id = 12345;
        Observable fallback;
        fallback.id = -1;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        ASSERT_EQ(o.value_or(std::move(fallback)).id, 12345);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
    }
}

TEST_F(Optional, ValueOr_Empty) {
    {
        iox2::container::Optional<int> o;
        ASSERT_EQ(o.value_or(-1), -1);
    }
    {
        iox2::container::Optional<Observable> o;
        Observable fallback;
        fallback.id = -1;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        ASSERT_EQ(o.value_or(fallback).id, -1);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
    }
    {
        iox2::container::Optional<Observable> o;
        Observable fallback;
        fallback.id = -1;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        ASSERT_EQ(o.value_or(std::move(fallback)).id, -1);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
    }
}


TEST_F(Optional, ValueOrRvalue_Full) {
    {
        iox2::container::Optional<int> o { 42 };
        ASSERT_EQ(std::move(o).value_or(-1), 42);
    }
    {
        iox2::container::Optional<Observable> o { Observable {} };
        o->id = 12345;
        Observable fallback;
        fallback.id = -1;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        ASSERT_EQ(std::move(o).value_or(fallback).id, 12345);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
    }
    {
        iox2::container::Optional<Observable> o { Observable {} };
        o->id = 12345;
        Observable fallback;
        fallback.id = -1;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        ASSERT_EQ(std::move(o).value_or(std::move(fallback)).id, 12345);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
    }
}

TEST_F(Optional, ValueOrRvalue_Empty) {
    {
        iox2::container::Optional<int> o;
        ASSERT_EQ(std::move(o).value_or(-1), -1);
    }
    {
        iox2::container::Optional<Observable> o;
        Observable fallback;
        fallback.id = -1;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        ASSERT_EQ(std::move(o).value_or(fallback).id, -1);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
    }
    {
        iox2::container::Optional<Observable> o;
        Observable fallback;
        fallback.id = -1;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        ASSERT_EQ(std::move(o).value_or(std::move(fallback)).id, -1);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
    }
}

TEST_F(Optional, Reset) {
    iox2::container::Optional<int> o;
    ASSERT_TRUE(!o.has_value());
    o.reset();
    ASSERT_TRUE(!o.has_value());
    o = 42;
    ASSERT_TRUE(o.has_value());
    o.reset();
    ASSERT_TRUE(!o.has_value());
}

} // namespace
