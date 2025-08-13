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

#include <gtest/gtest.h>

namespace {
namespace testing {
int g_ObservableWasInitialized = 0;
int g_ObservableWasCopied = 0;
int g_ObservableWasCopyAssigned = 0;
int g_ObservableWasMoved = 0;
int g_ObservableWasMoveAssigned = 0;
int g_ObservableWasDestructed = 0;

class Observable {
public:
    int id = 0;
public:
    Observable() {
        ++g_ObservableWasInitialized;
    }

    Observable(Observable const& rhs)
    :id(rhs.id)
    {
        ++g_ObservableWasCopied;
    }

    Observable(Observable&& rhs)
    :id(rhs.id)
    {
        ++g_ObservableWasMoved;
    }

    ~Observable() {
        ++g_ObservableWasDestructed;
    }

    Observable& operator=(Observable const& rhs) {
        ++g_ObservableWasCopyAssigned;
        id = rhs.id;
        return *this;
    }
    
    Observable& operator=(Observable&& rhs) {
        ++g_ObservableWasMoveAssigned;
        id = rhs.id;
        return *this;
    }
};
}

TEST(Optional, DefaultConstructor) {
    // [optional.ctor] / 2
    {
        iox2::container::Optional<int> o;
        ASSERT_FALSE(o.has_value());
    }
    // [optional.ctor] / 3
    {
        // No contained value is initialized
        testing::g_ObservableWasInitialized = 0;
        iox2::container::Optional<testing::Observable> o;
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
    }
}

TEST(Optional, NulloptConstructor) {
    {
        iox2::container::Optional<int> o(iox2::container::NulloptT{});
        ASSERT_FALSE(o.has_value());
    }
    {
        testing::g_ObservableWasInitialized = 0;
        iox2::container::Optional<testing::Observable> o(iox2::container::NulloptT{});
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
    }
}

TEST(Optional, ValueConstructor) {
    {
        iox2::container::Optional<int> o(42);
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(*o, 42);
    }
    {
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasMoved = 0;
        iox2::container::Optional<testing::Observable> o(testing::Observable{});
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(testing::g_ObservableWasInitialized, 1);
        ASSERT_EQ(testing::g_ObservableWasMoved, 1);
    }
    {
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::Observable value;
        value.id = 9999;
        iox2::container::Optional<testing::Observable> o(value);
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(o->id, 9999);
        ASSERT_EQ(testing::g_ObservableWasInitialized, 1);
        ASSERT_EQ(testing::g_ObservableWasCopied, 1);
    }
}

TEST(Optional, Destructor) {
    testing::g_ObservableWasDestructed = 0;
    {
        iox2::container::Optional<testing::Observable> o(iox2::container::NulloptT{});
        ASSERT_TRUE(!o.has_value());
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 0);

    testing::g_ObservableWasDestructed = 0;
    {
        iox2::container::Optional<testing::Observable> o(testing::Observable{});
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(testing::g_ObservableWasDestructed, 1);
        testing::g_ObservableWasDestructed = 0;
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 1);
}

TEST(Optional, CopyConstruction_FromEmpty) {
    {
        iox2::container::Optional<int> empty;
        iox2::container::Optional<int> o{empty};
        ASSERT_TRUE(!o.has_value());
    }
    {
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasDestructed = 0;
        iox2::container::Optional<testing::Observable> empty;
        iox2::container::Optional<testing::Observable> o{empty};
        ASSERT_TRUE(!o.has_value());
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 0);
}

TEST(Optional, CopyConstruction_FromFull) {
    {
        iox2::container::Optional<int> full(42);
        iox2::container::Optional<int> o{full};
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(*o, 42);
    }
    {
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasDestructed = 0;
        iox2::container::Optional<testing::Observable> full(testing::Observable{});
        ASSERT_EQ(testing::g_ObservableWasDestructed, 1);
        testing::g_ObservableWasDestructed = 0;
        ASSERT_EQ(testing::g_ObservableWasInitialized, 1);
        ASSERT_EQ(testing::g_ObservableWasCopied, 0);
        full->id = 12345;
        iox2::container::Optional<testing::Observable> o{full};
        ASSERT_EQ(testing::g_ObservableWasInitialized, 1);
        ASSERT_EQ(testing::g_ObservableWasCopied, 1);
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(o->id, 12345);
        ASSERT_TRUE(full.has_value());
        ASSERT_EQ(full->id, 12345);
        ASSERT_EQ(testing::g_ObservableWasDestructed, 0);
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 2);
}

TEST(Optional, MoveConstruction_FromEmpty) {
    {
        iox2::container::Optional<int> empty;
        iox2::container::Optional<int> o{std::move(empty)};
        ASSERT_TRUE(!o.has_value());
    }
    {
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasDestructed = 0;
        iox2::container::Optional<testing::Observable> empty;
        iox2::container::Optional<testing::Observable> o{std::move(empty)};
        ASSERT_TRUE(!o.has_value());
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 0);
}

TEST(Optional, MoveConstruction_FromFull) {
    {
        iox2::container::Optional<int> full(42);
        iox2::container::Optional<int> o{std::move(full)};
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(*o, 42);
    }
    {
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasDestructed = 0;
        iox2::container::Optional<testing::Observable> full(testing::Observable{});
        ASSERT_EQ(testing::g_ObservableWasDestructed, 1);
        testing::g_ObservableWasDestructed = 0;
        ASSERT_EQ(testing::g_ObservableWasInitialized, 1);
        ASSERT_EQ(testing::g_ObservableWasMoved, 1);
        testing::g_ObservableWasMoved = 0;
        full->id = 12345;
        iox2::container::Optional<testing::Observable> o{std::move(full)};
        ASSERT_EQ(testing::g_ObservableWasInitialized, 1);
        ASSERT_EQ(testing::g_ObservableWasMoved, 1);
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(o->id, 12345);
        ASSERT_TRUE(full.has_value());
        ASSERT_EQ(full->id, 12345);
        ASSERT_EQ(testing::g_ObservableWasDestructed, 0);
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 2);
}

TEST(Optional, CopyAssignment_EmptyToEmpty) {
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
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasCopyAssigned = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        testing::g_ObservableWasDestructed = 0;
        iox2::container::Optional<testing::Observable> o;
        iox2::container::Optional<testing::Observable> empty;
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(!empty.has_value());
        o = empty;
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(!empty.has_value());
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
        ASSERT_EQ(testing::g_ObservableWasCopied, 0);
        ASSERT_EQ(testing::g_ObservableWasCopyAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasMoved, 0);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 0);
}

TEST(Optional, CopyAssignment_EmptyToFull) {
    {
        iox2::container::Optional<int> o{42};
        iox2::container::Optional<int> empty;
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(!empty.has_value());
        o = empty;
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(!empty.has_value());
    }
    {
        iox2::container::Optional<testing::Observable> o{testing::Observable{}};
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasCopyAssigned = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        testing::g_ObservableWasDestructed = 0;
        iox2::container::Optional<testing::Observable> empty;
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(!empty.has_value());
        o = empty;
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(!empty.has_value());
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
        ASSERT_EQ(testing::g_ObservableWasCopied, 0);
        ASSERT_EQ(testing::g_ObservableWasCopyAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasMoved, 0);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasDestructed, 1);
        testing::g_ObservableWasDestructed = 0;
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 0);
}

TEST(Optional, CopyAssignment_FullToEmpty) {
    {
        iox2::container::Optional<int> o;
        iox2::container::Optional<int> full{42};
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(full.has_value());
        o = full;
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(full.has_value());
        ASSERT_EQ(*o, 42);
        ASSERT_EQ(*full, 42);
    }
    {
        iox2::container::Optional<testing::Observable> o;
        iox2::container::Optional<testing::Observable> full{testing::Observable{}};
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(full.has_value());
        full->id = 12345;
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasCopyAssigned = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        testing::g_ObservableWasDestructed = 0;
        o = full;
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(full.has_value());
        ASSERT_EQ(o->id, 12345);
        ASSERT_EQ(full->id, 12345);
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
        ASSERT_EQ(testing::g_ObservableWasCopied, 1);
        ASSERT_EQ(testing::g_ObservableWasCopyAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasMoved, 0);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasDestructed, 0);
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 2);
}

TEST(Optional, CopyAssignment_FullToFull) {
    {
        iox2::container::Optional<int> o{-99};
        iox2::container::Optional<int> full{42};
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
        iox2::container::Optional<testing::Observable> o{testing::Observable{}};
        iox2::container::Optional<testing::Observable> full{testing::Observable{}};
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(full.has_value());
        o->id = 111111;
        full->id = 12345;
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasCopyAssigned = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        testing::g_ObservableWasDestructed = 0;
        o = full;
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(full.has_value());
        ASSERT_EQ(o->id, 12345);
        ASSERT_EQ(full->id, 12345);
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
        ASSERT_EQ(testing::g_ObservableWasCopied, 0);
        ASSERT_EQ(testing::g_ObservableWasCopyAssigned, 1);
        ASSERT_EQ(testing::g_ObservableWasMoved, 0);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasDestructed, 0);
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 2);
}

TEST(Optional, MoveAssignment_EmptyToEmpty) {
    {
        iox2::container::Optional<int> o;
        iox2::container::Optional<int> empty;
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(!empty.has_value());
        o = std::move(empty);
        ASSERT_TRUE(!o.has_value());
    }
    {
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasCopyAssigned = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        testing::g_ObservableWasDestructed = 0;
        iox2::container::Optional<testing::Observable> o;
        iox2::container::Optional<testing::Observable> empty;
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(!empty.has_value());
        o = std::move(empty);
        ASSERT_TRUE(!o.has_value());
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
        ASSERT_EQ(testing::g_ObservableWasCopied, 0);
        ASSERT_EQ(testing::g_ObservableWasCopyAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasMoved, 0);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 0);
}

TEST(Optional, MoveAssignment_EmptyToFull) {
    {
        iox2::container::Optional<int> o{42};
        iox2::container::Optional<int> empty;
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(!empty.has_value());
        o = std::move(empty);
        ASSERT_TRUE(!o.has_value());
    }
    {
        iox2::container::Optional<testing::Observable> o{testing::Observable{}};
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasCopyAssigned = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        testing::g_ObservableWasDestructed = 0;
        iox2::container::Optional<testing::Observable> empty;
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(!empty.has_value());
        o = std::move(empty);
        ASSERT_TRUE(!o.has_value());
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
        ASSERT_EQ(testing::g_ObservableWasCopied, 0);
        ASSERT_EQ(testing::g_ObservableWasCopyAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasMoved, 0);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasDestructed, 1);
        testing::g_ObservableWasDestructed = 0;
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 0);
}

TEST(Optional, MoveAssignment_FullToEmpty) {
    {
        iox2::container::Optional<int> o;
        iox2::container::Optional<int> full{42};
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(full.has_value());
        o = std::move(full);
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(*o, 42);
    }
    {
        iox2::container::Optional<testing::Observable> o;
        iox2::container::Optional<testing::Observable> full{testing::Observable{}};
        ASSERT_TRUE(!o.has_value());
        ASSERT_TRUE(full.has_value());
        full->id = 12345;
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasCopyAssigned = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        testing::g_ObservableWasDestructed = 0;
        o = std::move(full);
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(o->id, 12345);
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
        ASSERT_EQ(testing::g_ObservableWasCopied, 0);
        ASSERT_EQ(testing::g_ObservableWasCopyAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasMoved, 1);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasDestructed, 0);
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 2);
}

TEST(Optional, MoveAssignment_FullToFull) {
    {
        iox2::container::Optional<int> o{-99};
        iox2::container::Optional<int> full{42};
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(full.has_value());
        ASSERT_EQ(*o, -99);
        o = std::move(full);
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(*o, 42);
    }
    {
        iox2::container::Optional<testing::Observable> o{testing::Observable{}};
        iox2::container::Optional<testing::Observable> full{testing::Observable{}};
        ASSERT_TRUE(o.has_value());
        ASSERT_TRUE(full.has_value());
        o->id = 111111;
        full->id = 12345;
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasCopyAssigned = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        testing::g_ObservableWasDestructed = 0;
        o = std::move(full);
        ASSERT_TRUE(o.has_value());
        ASSERT_EQ(o->id, 12345);
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
        ASSERT_EQ(testing::g_ObservableWasCopied, 0);
        ASSERT_EQ(testing::g_ObservableWasCopyAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasMoved, 0);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 1);
        ASSERT_EQ(testing::g_ObservableWasDestructed, 0);
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 2);
}

TEST(Optional, Assignment_NulloptToEmpty) {
    {
        iox2::container::Optional<int> o;
        ASSERT_TRUE(!o.has_value());
        o = iox2::container::NulloptT{};
        ASSERT_TRUE(!o.has_value());
    }
    {
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasCopyAssigned = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        testing::g_ObservableWasDestructed = 0;
        iox2::container::Optional<testing::Observable> o;
        ASSERT_TRUE(!o.has_value());
        o = iox2::container::NulloptT{};
        ASSERT_TRUE(!o.has_value());
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
        ASSERT_EQ(testing::g_ObservableWasCopied, 0);
        ASSERT_EQ(testing::g_ObservableWasCopyAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasMoved, 0);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 0);
}

TEST(Optional, Assignment_NulloptToFull) {
    {
        iox2::container::Optional<int> o{42};
        ASSERT_TRUE(o.has_value());
        o = iox2::container::NulloptT{};
        ASSERT_TRUE(!o.has_value());
    }
    {
        iox2::container::Optional<testing::Observable> o{testing::Observable{}};
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasCopyAssigned = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        testing::g_ObservableWasDestructed = 0;
        ASSERT_TRUE(o.has_value());
        o = iox2::container::NulloptT{};
        ASSERT_TRUE(!o.has_value());
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
        ASSERT_EQ(testing::g_ObservableWasCopied, 0);
        ASSERT_EQ(testing::g_ObservableWasCopyAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasMoved, 0);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasDestructed, 1);
        testing::g_ObservableWasDestructed = 0;
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 0);
}

TEST(Optional, OperatorArrowPointerAccess) {
    iox2::container::Optional<int> o;
    ASSERT_EQ(o.operator->(), nullptr);
    o = 42;
    ASSERT_NE(o.operator->(), nullptr);
    ASSERT_EQ(*(o.operator->()), 42);
}

TEST(Optional, OperatorArrowConstPointerAccess) {
    iox2::container::Optional<int> const o1;
    ASSERT_EQ(o1.operator->(), nullptr);
    iox2::container::Optional<int> const o2{42};
    ASSERT_NE(o2.operator->(), nullptr);
    ASSERT_EQ(*(o2.operator->()), 42);
}

TEST(Optional, OperatorStarDereferenceAccess) {
    iox2::container::Optional<int> o{42};
    ASSERT_EQ(*o, 42);
    *o = 55;
    ASSERT_EQ(*o, 55);
}

TEST(Optional, OperatorStarConstDereferenceAccess) {
    iox2::container::Optional<int> const o1{42};
    ASSERT_EQ(*o1, 42);
    iox2::container::Optional<int> const o2{55};
    ASSERT_EQ(*o2, 55);
}

TEST(Optional, OperatorStarMoveAccess) {
    testing::Observable value;
    value.id = 12345;
    {
        iox2::container::Optional<testing::Observable> o{value};
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        testing::g_ObservableWasDestructed = 0;
        testing::Observable m = *std::move(o);
        ASSERT_EQ(testing::g_ObservableWasMoved, 1);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasDestructed, 0);
        ASSERT_EQ(m.id, 12345);
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 2);
}

TEST(Optional, OperatorStarConstRvalueAccess) {
    testing::Observable value;
    value.id = 12345;
    {
        iox2::container::Optional<testing::Observable> const o{value};
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        testing::g_ObservableWasDestructed = 0;
        testing::Observable const&& ref = *std::move(o);
        ASSERT_EQ(testing::g_ObservableWasMoved, 0);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasDestructed, 0);
        ASSERT_EQ(ref.id, 12345);
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 1);
}

TEST(Optional, OperatorBool) {
    iox2::container::Optional<int> o;
    ASSERT_FALSE(static_cast<bool>(o));
    o = 42;
    ASSERT_TRUE(static_cast<bool>(o));
}

TEST(Optional, HasValue) {
    iox2::container::Optional<int> o;
    ASSERT_FALSE(o.has_value());
    o = 42;
    ASSERT_TRUE(o.has_value());
}

TEST(Optional, ValueAccess) {
    iox2::container::Optional<int> o{42};
    ASSERT_EQ(o.value(), 42);
    o.value() = 55;
    ASSERT_EQ(o.value(), 55);
}

TEST(Optional, ValueConstAccess) {
    iox2::container::Optional<int> const o1{42};
    ASSERT_EQ(o1.value(), 42);
    iox2::container::Optional<int> const o2{55};
    ASSERT_EQ(o2.value(), 55);
}

TEST(Optional, ValueMoveAccess) {
    testing::Observable value;
    value.id = 12345;
    {
        iox2::container::Optional<testing::Observable> o{value};
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        testing::g_ObservableWasDestructed = 0;
        testing::Observable m = std::move(o).value();
        ASSERT_EQ(testing::g_ObservableWasMoved, 1);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasDestructed, 0);
        ASSERT_EQ(m.id, 12345);
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 2);
}

TEST(Optional, ValueConstRvalueAccess) {
    testing::Observable value;
    value.id = 12345;
    {
        iox2::container::Optional<testing::Observable> const o{value};
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        testing::g_ObservableWasDestructed = 0;
        testing::Observable const&& ref = std::move(o).value();
        ASSERT_EQ(testing::g_ObservableWasMoved, 0);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasDestructed, 0);
        ASSERT_EQ(ref.id, 12345);
    }
    ASSERT_EQ(testing::g_ObservableWasDestructed, 1);
}

TEST(Optional, ValueOr_Full) {
    {
        iox2::container::Optional<int> o{42};
        ASSERT_EQ(o.value_or(-1), 42);
    }
    {
        iox2::container::Optional<testing::Observable> o{testing::Observable{}};
        o->id = 12345;
        testing::Observable fallback;
        fallback.id = -1;
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasCopyAssigned = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        ASSERT_EQ(o.value_or(fallback).id, 12345);
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
        ASSERT_EQ(testing::g_ObservableWasCopied, 1);
        ASSERT_EQ(testing::g_ObservableWasCopyAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasMoved, 0);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
    }
    {
        iox2::container::Optional<testing::Observable> o{testing::Observable{}};
        o->id = 12345;
        testing::Observable fallback;
        fallback.id = -1;
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasCopyAssigned = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        ASSERT_EQ(o.value_or(std::move(fallback)).id, 12345);
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
        ASSERT_EQ(testing::g_ObservableWasCopied, 1);
        ASSERT_EQ(testing::g_ObservableWasCopyAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasMoved, 0);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
    }
}

TEST(Optional, ValueOr_Empty) {
    {
        iox2::container::Optional<int> o;
        ASSERT_EQ(o.value_or(-1), -1);
    }
    {
        iox2::container::Optional<testing::Observable> o;
        testing::Observable fallback;
        fallback.id = -1;
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasCopyAssigned = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        ASSERT_EQ(o.value_or(fallback).id, -1);
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
        ASSERT_EQ(testing::g_ObservableWasCopied, 1);
        ASSERT_EQ(testing::g_ObservableWasCopyAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasMoved, 0);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
    }
    {
        iox2::container::Optional<testing::Observable> o;
        testing::Observable fallback;
        fallback.id = -1;
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasCopyAssigned = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        ASSERT_EQ(o.value_or(std::move(fallback)).id, -1);
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
        ASSERT_EQ(testing::g_ObservableWasCopied, 0);
        ASSERT_EQ(testing::g_ObservableWasCopyAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasMoved, 1);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
    }
}


TEST(Optional, ValueOrRvalue_Full) {
    {
        iox2::container::Optional<int> o{42};
        ASSERT_EQ(std::move(o).value_or(-1), 42);
    }
    {
        iox2::container::Optional<testing::Observable> o{testing::Observable{}};
        o->id = 12345;
        testing::Observable fallback;
        fallback.id = -1;
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasCopyAssigned = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        ASSERT_EQ(std::move(o).value_or(fallback).id, 12345);
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
        ASSERT_EQ(testing::g_ObservableWasCopied, 0);
        ASSERT_EQ(testing::g_ObservableWasCopyAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasMoved, 1);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
    }
    {
        iox2::container::Optional<testing::Observable> o{testing::Observable{}};
        o->id = 12345;
        testing::Observable fallback;
        fallback.id = -1;
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasCopyAssigned = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        ASSERT_EQ(std::move(o).value_or(std::move(fallback)).id, 12345);
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
        ASSERT_EQ(testing::g_ObservableWasCopied, 0);
        ASSERT_EQ(testing::g_ObservableWasCopyAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasMoved, 1);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
    }
}

TEST(Optional, ValueOrRvalue_Empty) {
    {
        iox2::container::Optional<int> o;
        ASSERT_EQ(std::move(o).value_or(-1), -1);
    }
    {
        iox2::container::Optional<testing::Observable> o;
        testing::Observable fallback;
        fallback.id = -1;
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasCopyAssigned = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        ASSERT_EQ(std::move(o).value_or(fallback).id, -1);
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
        ASSERT_EQ(testing::g_ObservableWasCopied, 1);
        ASSERT_EQ(testing::g_ObservableWasCopyAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasMoved, 0);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
    }
    {
        iox2::container::Optional<testing::Observable> o;
        testing::Observable fallback;
        fallback.id = -1;
        testing::g_ObservableWasInitialized = 0;
        testing::g_ObservableWasCopied = 0;
        testing::g_ObservableWasCopyAssigned = 0;
        testing::g_ObservableWasMoved = 0;
        testing::g_ObservableWasMoveAssigned = 0;
        ASSERT_EQ(std::move(o).value_or(std::move(fallback)).id, -1);
        ASSERT_EQ(testing::g_ObservableWasInitialized, 0);
        ASSERT_EQ(testing::g_ObservableWasCopied, 0);
        ASSERT_EQ(testing::g_ObservableWasCopyAssigned, 0);
        ASSERT_EQ(testing::g_ObservableWasMoved, 1);
        ASSERT_EQ(testing::g_ObservableWasMoveAssigned, 0);
    }
}

TEST(Optional, Reset) {
    iox2::container::Optional<int> o;
    ASSERT_TRUE(!o.has_value());
    o.reset();
    ASSERT_TRUE(!o.has_value());
    o = 42;
    ASSERT_TRUE(o.has_value());
    o.reset();
    ASSERT_TRUE(!o.has_value());
}

}
