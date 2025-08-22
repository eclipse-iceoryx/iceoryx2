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
#include "testing/test_utils.hpp"

#include <gtest/gtest.h>

namespace {
using iox2::container::testing::Observable;

class OptionalFixture : public iox2::container::testing::DetectLeakedObservablesFixture { };

TEST(Optional, default_constructor_initializes_empty_optional) {
    // [optional.ctor] / 2
    iox2::container::Optional<int> const sut;
    ASSERT_FALSE(sut.has_value());
}

TEST_F(OptionalFixture, default_constructor_does_not_initialize_an_object_of_contained_type) {
    // [optional.ctor] / 3
    Observable::reset_all_counters();
    iox2::container::Optional<Observable> const sut;
    ASSERT_FALSE(sut.has_value());
    ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
    ASSERT_EQ(Observable::s_counter.totalInstances, 0);
}

TEST(Optional, nullopt_constructor_initializes_empty_optional) {
    iox2::container::Optional<int> const sut(iox2::container::NulloptT {});
    ASSERT_FALSE(sut.has_value());
}

TEST_F(OptionalFixture, nullopt_constructor_does_not_initialize_an_object_of_contained_type) {
    Observable::s_counter.wasInitialized = 0;
    iox2::container::Optional<Observable> const sut(iox2::container::NulloptT {});
    ASSERT_FALSE(sut.has_value());
    ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
    ASSERT_EQ(Observable::s_counter.totalInstances, 0);
}

TEST(Optional, value_constructor_initializes_the_contained_value) {
    int const contained_value = 42;
    iox2::container::Optional<int> const sut(contained_value);
    ASSERT_TRUE(sut.has_value());
    EXPECT_EQ(*sut, contained_value);
}

TEST_F(OptionalFixture, value_constructor_move_constructs_for_rvalue) {
    Observable::s_counter.wasInitialized = 0;
    Observable::s_counter.wasMoveConstructed = 0;
    iox2::container::Optional<Observable> const sut(Observable {});
    ASSERT_TRUE(sut.has_value());
    EXPECT_EQ(Observable::s_counter.wasInitialized, 1);
    EXPECT_EQ(Observable::s_counter.wasMoveConstructed, 1);
}

TEST_F(OptionalFixture, value_constructor_copy_constructs_for_lvalue) {
    Observable::s_counter.wasInitialized = 0;
    Observable::s_counter.wasCopyConstructed = 0;
    int const contained_value = 9999;
    Observable value;
    value.id = contained_value;
    iox2::container::Optional<Observable> sut(value);
    ASSERT_TRUE(sut.has_value());
    EXPECT_EQ(sut->id, value.id);
    EXPECT_EQ(Observable::s_counter.wasInitialized, 1);
    EXPECT_EQ(Observable::s_counter.wasCopyConstructed, 1);
}

TEST_F(OptionalFixture, destructor_does_nothing_on_empty_optiona) {
    Observable::s_counter.wasDestructed = 0;
    {
        iox2::container::Optional<Observable> const sut(iox2::container::NulloptT {});
        ASSERT_TRUE(!sut.has_value());
    }
    EXPECT_EQ(Observable::s_counter.wasDestructed, 0);
}

TEST_F(OptionalFixture, destructor_destructs_contained_values) {
    Observable::s_counter.wasDestructed = 0;
    {
        iox2::container::Optional<Observable> const sut(Observable {});
        ASSERT_TRUE(sut.has_value());
        EXPECT_EQ(Observable::s_counter.wasDestructed, 1);
        Observable::s_counter.wasDestructed = 0;
    }
    EXPECT_EQ(Observable::s_counter.wasDestructed, 1);
}

TEST(Optional, copy_constructor_constructs_empty_from_empty) {
    iox2::container::Optional<int> const empty;
    iox2::container::Optional<int> sut { empty };
    ASSERT_TRUE(!sut.has_value());
    iox2::container::testing::opaque_use(sut);
}

TEST_F(OptionalFixture, copy_construction_from_empty_does_not_initialize_object) {
    {
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasDestructed = 0;
        iox2::container::Optional<Observable> const empty;
        iox2::container::Optional<Observable> sut { empty };
        ASSERT_TRUE(!sut.has_value());
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        iox2::container::testing::opaque_use(sut);
    }
    EXPECT_EQ(Observable::s_counter.wasDestructed, 0);
}

TEST(Optional, copy_construction_from_filled_object_constructs_new_object) {
    int const contained_valued = 42;
    iox2::container::Optional<int> const full(contained_valued);
    iox2::container::Optional<int> sut { full };
    ASSERT_TRUE(sut.has_value());
    EXPECT_EQ(*sut, contained_valued);
    iox2::container::testing::opaque_use(sut);
}


TEST_F(OptionalFixture, copy_construction_from_filled_object_invokes_copy_constructor) {
    int const tracking_id = 12345;
    {
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasDestructed = 0;
        iox2::container::Optional<Observable> full(Observable {});
        ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
        Observable::s_counter.wasDestructed = 0;
        ASSERT_EQ(Observable::s_counter.wasInitialized, 1);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        full->id = tracking_id;
        iox2::container::Optional<Observable> const sut { full };
        ASSERT_EQ(Observable::s_counter.wasInitialized, 1);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 1);
        ASSERT_TRUE(sut.has_value());
        EXPECT_EQ(sut->id, tracking_id);
        ASSERT_TRUE(full.has_value());
        EXPECT_EQ(full->id, tracking_id);
        iox2::container::testing::opaque_use(sut);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 2);
}

TEST(Optional, move_constructor_constructs_empty_from_empty) {
    iox2::container::Optional<int> empty;
    iox2::container::Optional<int> const sut { std::move(empty) };
    ASSERT_TRUE(!sut.has_value());
}

TEST_F(OptionalFixture, move_construction_from_empty_does_not_initialize_object) {
    {
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasDestructed = 0;
        iox2::container::Optional<Observable> empty;
        iox2::container::Optional<Observable> sut { std::move(empty) };
        ASSERT_TRUE(!sut.has_value());
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        iox2::container::testing::opaque_use(sut);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
}

TEST(Optional, move_constructor_from_filled_object_constructs_new_object) {
    int const contained_value = 42;
    iox2::container::Optional<int> full(contained_value);
    iox2::container::Optional<int> sut { std::move(full) };
    ASSERT_TRUE(sut.has_value());
    EXPECT_EQ(*sut, contained_value);
}

TEST_F(OptionalFixture, move_constructor_from_filled_object_moves_value) {
    int const tracking_id = 12345;
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
        full->id = tracking_id;
        iox2::container::Optional<Observable> sut { std::move(full) };
        ASSERT_EQ(Observable::s_counter.wasInitialized, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 1);
        ASSERT_TRUE(sut.has_value());
        EXPECT_EQ(sut->id, tracking_id);
        iox2::container::testing::opaque_use(sut);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 2);
}

TEST(Optional, copy_assignment_from_empty_to_empty_leaves_optional_empty) {
    iox2::container::Optional<int> sut;
    iox2::container::Optional<int> const empty;
    ASSERT_TRUE(!sut.has_value());
    ASSERT_TRUE(!empty.has_value());
    sut = empty;
    ASSERT_TRUE(!sut.has_value());
    ASSERT_TRUE(!empty.has_value());
}

TEST_F(OptionalFixture, copy_assignment_from_empty_to_empty_does_not_construct_any_objects) {
    {
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        iox2::container::Optional<Observable> sut;
        iox2::container::Optional<Observable> const empty;
        ASSERT_TRUE(!sut.has_value());
        ASSERT_TRUE(!empty.has_value());
        sut = empty;
        ASSERT_TRUE(!sut.has_value());
        ASSERT_TRUE(!empty.has_value());
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
}


TEST(Optional, copy_assignment_from_empty_to_full_empties_target) {
    int const contained_value = 42;
    iox2::container::Optional<int> sut { contained_value };
    iox2::container::Optional<int> const empty;
    ASSERT_TRUE(sut.has_value());
    ASSERT_TRUE(!empty.has_value());
    sut = empty;
    ASSERT_TRUE(!sut.has_value());
    ASSERT_TRUE(!empty.has_value());
}

TEST_F(OptionalFixture, copy_assignment_from_empty_to_full_destructs_object_in_target) {
    {
        iox2::container::Optional<Observable> sut { Observable {} };
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        iox2::container::Optional<Observable> const empty;
        ASSERT_TRUE(sut.has_value());
        ASSERT_TRUE(!empty.has_value());
        sut = empty;
        ASSERT_TRUE(!sut.has_value());
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

TEST(Optional, copy_assignment_from_full_to_empty_assigns_value_to_target) {
    int const contained_value = 42;
    iox2::container::Optional<int> sut;
    iox2::container::Optional<int> full { contained_value };
    ASSERT_TRUE(!sut.has_value());
    ASSERT_TRUE(full.has_value());
    sut = full;
    ASSERT_TRUE(sut.has_value());
    ASSERT_TRUE(full.has_value());
    ASSERT_EQ(*sut, contained_value);
    ASSERT_EQ(*full, contained_value);
}

TEST_F(OptionalFixture, copy_assignment_from_full_to_empty_constructs_object_in_target) {
    int const tracking_id = 12345;
    {
        iox2::container::Optional<Observable> sut;
        iox2::container::Optional<Observable> full { Observable {} };
        ASSERT_TRUE(!sut.has_value());
        ASSERT_TRUE(full.has_value());
        full->id = tracking_id;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        sut = full;
        ASSERT_TRUE(sut.has_value());
        ASSERT_TRUE(full.has_value());
        EXPECT_EQ(sut->id, tracking_id);
        EXPECT_EQ(full->id, tracking_id);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 2);
}

TEST(Optional, copy_assignment_from_full_to_full_overwrites_target_value) {
    int const contained_value = 42;
    int const original_target_value = -99;
    iox2::container::Optional<int> sut { original_target_value };
    iox2::container::Optional<int> const full { contained_value };
    ASSERT_TRUE(sut.has_value());
    ASSERT_TRUE(full.has_value());
    EXPECT_EQ(*sut, original_target_value);
    sut = full;
    ASSERT_TRUE(sut.has_value());
    ASSERT_TRUE(full.has_value());
    ASSERT_EQ(*sut, contained_value);
    ASSERT_EQ(*full, contained_value);
}

TEST_F(OptionalFixture, copy_assignment_from_full_to_full_copy_assigns_to_target) {
    int const tracking_id = 12345;
    int const overwritten_id = 1111111;
    {
        iox2::container::Optional<Observable> sut { Observable {} };
        iox2::container::Optional<Observable> full { Observable {} };
        ASSERT_TRUE(sut.has_value());
        ASSERT_TRUE(full.has_value());
        sut->id = overwritten_id;
        full->id = tracking_id;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        sut = full;
        ASSERT_TRUE(sut.has_value());
        ASSERT_TRUE(full.has_value());
        ASSERT_EQ(sut->id, tracking_id);
        ASSERT_EQ(full->id, tracking_id);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 2);
}

TEST(Optional, copy_assignment_returns_reference_to_this) {
    iox2::container::Optional<Observable> sut;
    iox2::container::Optional<Observable> const full(Observable {});
    ASSERT_EQ(&(sut = full), &sut);
}

TEST(Optional, move_assignment_from_empty_to_empty_leaves_optional_empty) {
    iox2::container::Optional<int> sut;
    iox2::container::Optional<int> empty;
    ASSERT_TRUE(!sut.has_value());
    ASSERT_TRUE(!empty.has_value());
    sut = std::move(empty);
    ASSERT_TRUE(!sut.has_value());
}

TEST_F(OptionalFixture, move_assignment_from_empty_to_empty_does_not_construct_any_objects) {
    {
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        iox2::container::Optional<Observable> sut;
        iox2::container::Optional<Observable> empty;
        ASSERT_TRUE(!sut.has_value());
        ASSERT_TRUE(!empty.has_value());
        sut = std::move(empty);
        ASSERT_TRUE(!sut.has_value());
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
}

TEST(Optional, move_assignment_from_empty_to_full_empties_target) {
    int const contained_value = 42;
    iox2::container::Optional<int> sut { contained_value };
    iox2::container::Optional<int> empty;
    ASSERT_TRUE(sut.has_value());
    ASSERT_TRUE(!empty.has_value());
    sut = std::move(empty);
    ASSERT_TRUE(!sut.has_value());
}

TEST_F(OptionalFixture, move_assignment_from_empty_to_full_destructs_object_in_target) {
    {
        iox2::container::Optional<Observable> sut { Observable {} };
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        iox2::container::Optional<Observable> empty;
        ASSERT_TRUE(sut.has_value());
        ASSERT_TRUE(!empty.has_value());
        sut = std::move(empty);
        ASSERT_TRUE(!sut.has_value());
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

TEST(Optional, move_assignment_from_full_to_empty_assigns_value_to_target) {
    int const contained_value = 42;
    iox2::container::Optional<int> sut;
    iox2::container::Optional<int> full { contained_value };
    ASSERT_TRUE(!sut.has_value());
    ASSERT_TRUE(full.has_value());
    sut = std::move(full);
    ASSERT_TRUE(sut.has_value());
    EXPECT_EQ(*sut, contained_value);
}

TEST_F(OptionalFixture, move_assignment_from_full_to_empty_move_constructs_object_in_target) {
    int const tracking_id = 12345;
    {
        iox2::container::Optional<Observable> sut;
        iox2::container::Optional<Observable> full { Observable {} };
        ASSERT_TRUE(!sut.has_value());
        ASSERT_TRUE(full.has_value());
        full->id = tracking_id;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        sut = std::move(full);
        ASSERT_TRUE(sut.has_value());
        ASSERT_EQ(sut->id, tracking_id);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 2);
}

TEST(Optional, move_assignment_from_full_to_full_overwrites_target_value) {
    int const contained_value = 42;
    int const overwritten_value = -99;
    iox2::container::Optional<int> sut { overwritten_value };
    iox2::container::Optional<int> full { contained_value };
    ASSERT_TRUE(sut.has_value());
    ASSERT_TRUE(full.has_value());
    ASSERT_EQ(*sut, overwritten_value);
    sut = std::move(full);
    ASSERT_TRUE(sut.has_value());
    EXPECT_EQ(*sut, contained_value);
}

TEST_F(OptionalFixture, move_assignment_from_full_to_full_move_assigns_to_target) {
    int const tracking_id = 12345;
    int const overwritten_id = 111111;
    {
        iox2::container::Optional<Observable> sut { Observable {} };
        iox2::container::Optional<Observable> full { Observable {} };
        ASSERT_TRUE(sut.has_value());
        ASSERT_TRUE(full.has_value());
        sut->id = overwritten_id;
        full->id = tracking_id;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        sut = std::move(full);
        ASSERT_TRUE(sut.has_value());
        ASSERT_EQ(sut->id, tracking_id);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 1);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 2);
}

TEST(Optional, move_assignment_returns_reference_to_this) {
    iox2::container::Optional<Observable> sut;
    iox2::container::Optional<Observable> full(Observable {});
    ASSERT_EQ(&(sut = std::move(full)), &sut);
}

TEST(Optional, assignment_from_nullopt_to_empty_leaves_optional_empty) {
    iox2::container::Optional<int> sut;
    ASSERT_TRUE(!sut.has_value());
    sut = iox2::container::NulloptT {};
    ASSERT_TRUE(!sut.has_value());
}

TEST_F(OptionalFixture, assignment_from_nullopt_to_empty_does_not_construct_an_object) {
    {
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        iox2::container::Optional<Observable> sut;
        ASSERT_TRUE(!sut.has_value());
        sut = iox2::container::NulloptT {};
        ASSERT_TRUE(!sut.has_value());
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
}

TEST(Optional, assignment_from_nullopt_to_full_empties_optional) {
    int const overwritten_value = -99;
    iox2::container::Optional<int> sut { overwritten_value };
    ASSERT_TRUE(sut.has_value());
    sut = iox2::container::NulloptT {};
    ASSERT_TRUE(!sut.has_value());
}

TEST_F(OptionalFixture, assignment_from_nullopt_to_full_destructs_contained_object) {
    {
        iox2::container::Optional<Observable> sut { Observable {} };
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        ASSERT_TRUE(sut.has_value());
        sut = iox2::container::NulloptT {};
        ASSERT_TRUE(!sut.has_value());
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

TEST(Optional, assignment_from_nullopt_returns_reference_to_this) {
    iox2::container::Optional<Observable> sut { Observable {} };
    ASSERT_EQ(&(sut = iox2::container::NulloptT {}), &sut);
}

TEST(Optional, operator_arrow_returns_nullptr_for_empty_optional) {
    iox2::container::Optional<int> sut;
    ASSERT_EQ(sut.operator->(), nullptr);
}

TEST(Optional, operator_arrow_returns_pointer_to_contained_value_for_full_optional) {
    int const contained_value = 42;
    iox2::container::Optional<int> sut { contained_value };
    ASSERT_NE(sut.operator->(), nullptr);
    EXPECT_EQ(*(sut.operator->()), contained_value);
}

TEST(Optional, const_operator_arrow_returns_nullptr_for_empty_optional) {
    iox2::container::Optional<int> const sut;
    ASSERT_EQ(sut.operator->(), nullptr);
}

TEST(Optional, const_operator_arrow_returns_pointer_to_contained_value_for_full_optional) {
    int const contained_value = 42;
    iox2::container::Optional<int> const sut { contained_value };
    ASSERT_NE(sut.operator->(), nullptr);
    EXPECT_EQ(*(sut.operator->()), contained_value);
}

TEST(Optional, operator_star_returns_mutable_reference_to_contained_value) {
    int const contained_value = 42;
    iox2::container::Optional<int> sut { contained_value };
    ASSERT_EQ(*sut, contained_value);
    int const alternative_value = 55;
    *sut = alternative_value;
    ASSERT_EQ(*sut, alternative_value);
}

TEST(Optional, const_operator_star_dereferences_contained_value) {
    int const contained_value = 42;
    iox2::container::Optional<int> const sut1 { contained_value };
    ASSERT_EQ(*sut1, 42);
    int const alternative_value = 55;
    iox2::container::Optional<int> const sut2 { alternative_value };
    ASSERT_EQ(*sut2, alternative_value);
}

TEST_F(OptionalFixture, rvalue_operator_star_dereferences_to_rvalue) {
    int const tracking_id = 12345;
    Observable value;
    value.id = tracking_id;
    {
        iox2::container::Optional<Observable> sut { value };
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        Observable const move_target = *std::move(sut);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
        ASSERT_EQ(move_target.id, tracking_id);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 2);
}

TEST_F(OptionalFixture, const_rvalue_operator_star_dereferences_to_const_rvalue_and_is_just_not_very_useful_overall) {
    int const tracking_id = 12345;
    Observable value;
    value.id = tracking_id;
    {
        iox2::container::Optional<Observable> const sut { value };
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        Observable const&& ref = *std::move(sut);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
        ASSERT_EQ(ref.id, tracking_id);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
}

TEST(Optional, operator_bool_checks_for_non_empty) {
    iox2::container::Optional<int> sut;
    ASSERT_FALSE(static_cast<bool>(sut));
    int const just_some_arbitrary_value = 42;
    sut = just_some_arbitrary_value;
    ASSERT_TRUE(static_cast<bool>(sut));
}

TEST(Optional, has_value_checks_for_non_empty) {
    iox2::container::Optional<int> sut;
    ASSERT_FALSE(sut.has_value());
    int const just_some_arbitrary_value = 42;
    sut = just_some_arbitrary_value;
    ASSERT_TRUE(sut.has_value());
}

TEST(Optional, value_returns_mutable_reference_to_contained_value) {
    int const contained_value = 42;
    int const alternative_value = 55;
    iox2::container::Optional<int> sut { contained_value };
    ASSERT_EQ(sut.value(), contained_value);
    sut.value() = alternative_value;
    ASSERT_EQ(sut.value(), alternative_value);
}

TEST(Optional, const_value_dereferences_contained_value) {
    int const contained_value = 42;
    int const alternative_value = 55;
    iox2::container::Optional<int> const sut1 { contained_value };
    ASSERT_EQ(sut1.value(), contained_value);
    iox2::container::Optional<int> const sut2 { alternative_value };
    ASSERT_EQ(sut2.value(), alternative_value);
}

TEST_F(OptionalFixture, rvalue_value_returns_rvalue_dereferences_to_contained_value) {
    int const tracking_id = 12345;
    Observable value;
    value.id = tracking_id;
    {
        iox2::container::Optional<Observable> sut { value };
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        Observable const target = std::move(sut).value();
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
        ASSERT_EQ(target.id, tracking_id);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 2);
}

TEST_F(OptionalFixture, const_rvalue_value_dereferences_to_const_rvalue_and_is_just_not_very_useful_overall) {
    int const tracking_id = 12345;
    Observable value;
    value.id = tracking_id;
    {
        iox2::container::Optional<Observable> const sut { value };
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        Observable const&& ref = std::move(sut).value();
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
        ASSERT_EQ(ref.id, tracking_id);
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
}

TEST(Optional, value_or_returns_contained_value_on_full_optional) {
    int const contained_value = 42;
    iox2::container::Optional<int> const sut { contained_value };
    int const fallback = -1;
    ASSERT_EQ(sut.value_or(fallback), contained_value);
}

TEST_F(OptionalFixture, value_or_returns_copy_of_contained_value_on_full_optional) {
    int const tracking_id = 12345;
    int const fallback_id = -1;
    {
        iox2::container::Optional<Observable> sut { Observable {} };
        sut->id = tracking_id;
        Observable fallback;
        fallback.id = fallback_id;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        ASSERT_EQ(sut.value_or(fallback).id, tracking_id);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
        Observable::s_counter.wasDestructed = 0;
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 2);
}

TEST_F(OptionalFixture, value_or_with_rvalue_argument_returns_copy_of_contained_value_on_full_optional) {
    int const tracking_id = 12345;
    int const fallback_id = -1;
    {
        iox2::container::Optional<Observable> sut { Observable {} };
        sut->id = tracking_id;
        Observable fallback;
        fallback.id = fallback_id;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        ASSERT_EQ(sut.value_or(std::move(fallback)).id, tracking_id);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
        Observable::s_counter.wasDestructed = 0;
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 2);
}

TEST(Optional, value_or_returns_fallback_argument_on_empty_optional) {
    int const fallback_value = 225;
    iox2::container::Optional<int> const sut;
    ASSERT_EQ(sut.value_or(fallback_value), fallback_value);
}

TEST_F(OptionalFixture, value_or_returns_copy_of_fallback_argument_on_empty_optional) {
    int const fallback_tracking_id = 225;
    {
        iox2::container::Optional<Observable> const sut;
        Observable fallback;
        fallback.id = fallback_tracking_id;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        ASSERT_EQ(sut.value_or(fallback).id, fallback_tracking_id);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
        Observable::s_counter.wasDestructed = 0;
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
}

TEST_F(OptionalFixture, value_or_moves_rvalue_fallback_argument_on_empty_optional) {
    int const fallback_tracking_id = 225;
    {
        iox2::container::Optional<Observable> const sut;
        Observable fallback;
        fallback.id = fallback_tracking_id;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        Observable::s_counter.wasDestructed = 0;
        ASSERT_EQ(sut.value_or(std::move(fallback)).id, fallback_tracking_id);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
        Observable::s_counter.wasDestructed = 0;
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
}


TEST(Optional, rvalue_value_or_returns_contained_value_on_full_optional) {
    int const contained_value = 42;
    int const fallback_value = 225;
    iox2::container::Optional<int> sut { contained_value };
    ASSERT_EQ(std::move(sut).value_or(fallback_value), contained_value);
}


TEST_F(OptionalFixture, rvalue_value_or_moves_contained_value_on_full_optional) {
    int const tracking_id = 12345;
    int const fallback_tracking_id = -1;
    {
        iox2::container::Optional<Observable> sut { Observable {} };
        sut->id = tracking_id;
        Observable fallback;
        fallback.id = fallback_tracking_id;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        ASSERT_EQ(std::move(sut).value_or(fallback).id, tracking_id);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
    }
}

TEST_F(OptionalFixture, rvalue_value_or_with_rvalue_argument_moves_contained_value_on_full_optional) {
    int const tracking_id = 12345;
    int const fallback_tracking_id = -1;
    {
        iox2::container::Optional<Observable> sut { Observable {} };
        sut->id = tracking_id;
        Observable fallback;
        fallback.id = fallback_tracking_id;
        Observable::s_counter.wasInitialized = 0;
        Observable::s_counter.wasCopyConstructed = 0;
        Observable::s_counter.wasCopyAssigned = 0;
        Observable::s_counter.wasMoveConstructed = 0;
        Observable::s_counter.wasMoveAssigned = 0;
        ASSERT_EQ(std::move(sut).value_or(std::move(fallback)).id, tracking_id);
        ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
        ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
        ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 1);
        ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
    }
}

TEST(Optional, rvalue_value_or_returns_fallback_on_empty_optional) {
    int const fallback_value = 225;
    iox2::container::Optional<int> sut;
    ASSERT_EQ(std::move(sut).value_or(fallback_value), fallback_value);
}
TEST_F(OptionalFixture, rvalue_value_or_returns_fallback_on_empty_optional) {
    int const fallback_tracking_id = 225;
    iox2::container::Optional<Observable> sut;
    Observable fallback;
    fallback.id = fallback_tracking_id;
    Observable::s_counter.wasInitialized = 0;
    Observable::s_counter.wasCopyConstructed = 0;
    Observable::s_counter.wasCopyAssigned = 0;
    Observable::s_counter.wasMoveConstructed = 0;
    Observable::s_counter.wasMoveAssigned = 0;
    ASSERT_EQ(std::move(sut).value_or(fallback).id, fallback_tracking_id);
    ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
    ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 1);
    ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
    ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
    ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
}
TEST_F(OptionalFixture, rvalue_value_or_with_rvalue_argument_moves_fallback_on_empty_optional) {
    int const fallback_tracking_id = 225;
    iox2::container::Optional<Observable> sut;
    Observable fallback;
    fallback.id = fallback_tracking_id;
    Observable::s_counter.wasInitialized = 0;
    Observable::s_counter.wasCopyConstructed = 0;
    Observable::s_counter.wasCopyAssigned = 0;
    Observable::s_counter.wasMoveConstructed = 0;
    Observable::s_counter.wasMoveAssigned = 0;
    ASSERT_EQ(std::move(sut).value_or(std::move(fallback)).id, fallback_tracking_id);
    ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
    ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
    ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
    ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 1);
    ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
}

TEST(Optional, reset_leaves_empty_optional_in_empty_state) {
    iox2::container::Optional<int> sut;
    ASSERT_TRUE(!sut.has_value());
    sut.reset();
    ASSERT_TRUE(!sut.has_value());
}

TEST(Optional, reset_puts_full_optional_to_empty_state) {
    int const contained_value = 42;
    iox2::container::Optional<int> sut { contained_value };
    ASSERT_TRUE(sut.has_value());
    sut.reset();
    ASSERT_TRUE(!sut.has_value());
}

TEST_F(OptionalFixture, reset_on_full_optional_destructs_contained_value) {
    {
        iox2::container::Optional<Observable> sut { Observable {} };
        ASSERT_TRUE(sut.has_value());
        Observable::s_counter.wasDestructed = 0;
        sut.reset();
        ASSERT_TRUE(!sut.has_value());
        ASSERT_EQ(Observable::s_counter.wasDestructed, 1);
        Observable::s_counter.wasDestructed = 0;
    }
    ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
}

} // namespace
