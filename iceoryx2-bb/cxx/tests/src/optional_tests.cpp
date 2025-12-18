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

#include "iox2/bb/stl/optional.hpp"

#include "testing/observable.hpp"
#include "testing/test_utils.hpp"

#include <gtest/gtest.h>

namespace {
using namespace iox2::bb::stl;

using iox2::bb::testing::Observable;

class OptionalFixture : public iox2::bb::testing::DetectLeakedObservablesFixture { };

TEST(Optional, default_constructor_initializes_empty_optional) {
    // [optional.ctor] / 2
    Optional<int32_t> const sut;
    ASSERT_FALSE(sut.has_value());
}

TEST_F(OptionalFixture, default_constructor_does_not_initialize_an_object_of_contained_type) {
    // [optional.ctor] / 3
    Observable::reset_all_counters();
    Optional<Observable> const sut;
    ASSERT_FALSE(sut.has_value());
    ASSERT_EQ(Observable::s_counter.was_initialized, 0);
    ASSERT_EQ(Observable::s_counter.total_instances, 0);
}

TEST(Optional, nullopt_constructor_initializes_empty_optional) {
    Optional<int32_t> const sut(NULLOPT);
    ASSERT_FALSE(sut.has_value());
}

TEST_F(OptionalFixture, nullopt_constructor_does_not_initialize_an_object_of_contained_type) {
    Observable::s_counter.was_initialized = 0;
    Optional<Observable> const sut(NULLOPT);
    ASSERT_FALSE(sut.has_value());
    ASSERT_EQ(Observable::s_counter.was_initialized, 0);
    ASSERT_EQ(Observable::s_counter.total_instances, 0);
}

TEST(Optional, value_constructor_initializes_the_contained_value) {
    int32_t const contained_value = 42;
    Optional<int32_t> const sut(contained_value);
    ASSERT_TRUE(sut.has_value());
    EXPECT_EQ(*sut, contained_value);
}

TEST_F(OptionalFixture, value_constructor_move_constructs_for_rvalue) {
    Observable::s_counter.was_initialized = 0;
    Observable::s_counter.was_move_constructed = 0;
    Optional<Observable> const sut(Observable {});
    ASSERT_TRUE(sut.has_value());
    EXPECT_EQ(Observable::s_counter.was_initialized, 1);
    EXPECT_EQ(Observable::s_counter.was_move_constructed, 1);
}

TEST_F(OptionalFixture, value_constructor_copy_constructs_for_lvalue) {
    Observable::s_counter.was_initialized = 0;
    Observable::s_counter.was_copy_constructed = 0;
    int32_t const contained_value = 9999;
    Observable value;
    value.id = contained_value;
    Optional<Observable> sut(value);
    ASSERT_TRUE(sut.has_value());
    EXPECT_EQ(sut->id, value.id);
    EXPECT_EQ(Observable::s_counter.was_initialized, 1);
    EXPECT_EQ(Observable::s_counter.was_copy_constructed, 1);
}

TEST_F(OptionalFixture, destructor_does_nothing_on_empty_optiona) {
    Observable::s_counter.was_destructed = 0;
    {
        Optional<Observable> const sut(NULLOPT);
        ASSERT_TRUE(!sut.has_value());
    }
    EXPECT_EQ(Observable::s_counter.was_destructed, 0);
}

TEST_F(OptionalFixture, destructor_destructs_contained_values) {
    Observable::s_counter.was_destructed = 0;
    {
        Optional<Observable> const sut(Observable {});
        ASSERT_TRUE(sut.has_value());
        EXPECT_EQ(Observable::s_counter.was_destructed, 1);
        Observable::s_counter.was_destructed = 0;
    }
    EXPECT_EQ(Observable::s_counter.was_destructed, 1);
}

TEST(Optional, copy_constructor_constructs_empty_from_empty) {
    Optional<int32_t> const empty;
    Optional<int32_t> sut { empty };
    ASSERT_TRUE(!sut.has_value());
    iox2::bb::testing::opaque_use(sut);
}

TEST_F(OptionalFixture, copy_construction_from_empty_does_not_initialize_object) {
    {
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_destructed = 0;
        Optional<Observable> const empty;
        Optional<Observable> sut { empty };
        ASSERT_TRUE(!sut.has_value());
        ASSERT_EQ(Observable::s_counter.was_initialized, 0);
        iox2::bb::testing::opaque_use(sut);
    }
    EXPECT_EQ(Observable::s_counter.was_destructed, 0);
}

TEST(Optional, copy_construction_from_filled_object_constructs_new_object) {
    int32_t const contained_valued = 42;
    Optional<int32_t> const full(contained_valued);
    Optional<int32_t> sut { full };
    ASSERT_TRUE(sut.has_value());
    EXPECT_EQ(*sut, contained_valued);
    iox2::bb::testing::opaque_use(sut);
}


TEST_F(OptionalFixture, copy_construction_from_filled_object_invokes_copy_constructor) {
    int32_t const tracking_id = 12345;
    {
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_copy_constructed = 0;
        Observable::s_counter.was_destructed = 0;
        Optional<Observable> full(Observable {});
        ASSERT_EQ(Observable::s_counter.was_destructed, 1);
        Observable::s_counter.was_destructed = 0;
        ASSERT_EQ(Observable::s_counter.was_initialized, 1);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
        full->id = tracking_id;
        Optional<Observable> const sut { full };
        ASSERT_EQ(Observable::s_counter.was_initialized, 1);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 1);
        ASSERT_TRUE(sut.has_value());
        EXPECT_EQ(sut->id, tracking_id);
        ASSERT_TRUE(full.has_value());
        EXPECT_EQ(full->id, tracking_id);
        iox2::bb::testing::opaque_use(sut);
        ASSERT_EQ(Observable::s_counter.was_destructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 2);
}

TEST(Optional, move_constructor_constructs_empty_from_empty) {
    Optional<int32_t> empty;
    Optional<int32_t> const sut { std::move(empty) };
    ASSERT_TRUE(!sut.has_value());
}

TEST_F(OptionalFixture, move_construction_from_empty_does_not_initialize_object) {
    {
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_destructed = 0;
        Optional<Observable> empty;
        Optional<Observable> sut { std::move(empty) };
        ASSERT_TRUE(!sut.has_value());
        ASSERT_EQ(Observable::s_counter.was_initialized, 0);
        iox2::bb::testing::opaque_use(sut);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 0);
}

TEST(Optional, move_constructor_from_filled_object_constructs_new_object) {
    int32_t const contained_value = 42;
    Optional<int32_t> full(contained_value);
    Optional<int32_t> sut { std::move(full) };
    ASSERT_TRUE(sut.has_value());
    EXPECT_EQ(*sut, contained_value);
}

TEST_F(OptionalFixture, move_constructor_from_filled_object_moves_value) {
    int32_t const tracking_id = 12345;
    {
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_destructed = 0;
        Optional<Observable> full(Observable {});
        ASSERT_EQ(Observable::s_counter.was_destructed, 1);
        Observable::s_counter.was_destructed = 0;
        ASSERT_EQ(Observable::s_counter.was_initialized, 1);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 1);
        Observable::s_counter.was_move_constructed = 0;
        full->id = tracking_id;
        Optional<Observable> sut { std::move(full) };
        ASSERT_EQ(Observable::s_counter.was_initialized, 1);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 1);
        ASSERT_TRUE(sut.has_value());
        EXPECT_EQ(sut->id, tracking_id);
        iox2::bb::testing::opaque_use(sut);
        ASSERT_EQ(Observable::s_counter.was_destructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 2);
}

TEST(Optional, copy_assignment_from_empty_to_empty_leaves_optional_empty) {
    Optional<int32_t> sut;
    Optional<int32_t> const empty;
    ASSERT_TRUE(!sut.has_value());
    ASSERT_TRUE(!empty.has_value());
    sut = empty;
    ASSERT_TRUE(!sut.has_value());
    ASSERT_TRUE(!empty.has_value());
}

TEST_F(OptionalFixture, copy_assignment_from_empty_to_empty_does_not_construct_any_objects) {
    {
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_copy_constructed = 0;
        Observable::s_counter.was_copy_assigned = 0;
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        Observable::s_counter.was_destructed = 0;
        Optional<Observable> sut;
        Optional<Observable> const empty;
        ASSERT_TRUE(!sut.has_value());
        ASSERT_TRUE(!empty.has_value());
        sut = empty;
        ASSERT_TRUE(!sut.has_value());
        ASSERT_TRUE(!empty.has_value());
        ASSERT_EQ(Observable::s_counter.was_initialized, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 0);
}


TEST(Optional, copy_assignment_from_empty_to_full_empties_target) {
    int32_t const contained_value = 42;
    Optional<int32_t> sut { contained_value };
    Optional<int32_t> const empty;
    ASSERT_TRUE(sut.has_value());
    ASSERT_TRUE(!empty.has_value());
    sut = empty;
    ASSERT_TRUE(!sut.has_value());
    ASSERT_TRUE(!empty.has_value());
}

TEST_F(OptionalFixture, copy_assignment_from_empty_to_full_destructs_object_in_target) {
    {
        Optional<Observable> sut { Observable {} };
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_copy_constructed = 0;
        Observable::s_counter.was_copy_assigned = 0;
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        Observable::s_counter.was_destructed = 0;
        Optional<Observable> const empty;
        ASSERT_TRUE(sut.has_value());
        ASSERT_TRUE(!empty.has_value());
        sut = empty;
        ASSERT_TRUE(!sut.has_value());
        ASSERT_TRUE(!empty.has_value());
        ASSERT_EQ(Observable::s_counter.was_initialized, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_destructed, 1);
        Observable::s_counter.was_destructed = 0;
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 0);
}

TEST(Optional, copy_assignment_from_full_to_empty_assigns_value_to_target) {
    int32_t const contained_value = 42;
    Optional<int32_t> sut;
    Optional<int32_t> full { contained_value };
    ASSERT_TRUE(!sut.has_value());
    ASSERT_TRUE(full.has_value());
    sut = full;
    ASSERT_TRUE(sut.has_value());
    ASSERT_TRUE(full.has_value());
    ASSERT_EQ(*sut, contained_value);
    ASSERT_EQ(*full, contained_value);
}

TEST_F(OptionalFixture, copy_assignment_from_full_to_empty_constructs_object_in_target) {
    int32_t const tracking_id = 12345;
    {
        Optional<Observable> sut;
        Optional<Observable> full { Observable {} };
        ASSERT_TRUE(!sut.has_value());
        ASSERT_TRUE(full.has_value());
        full->id = tracking_id;
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_copy_constructed = 0;
        Observable::s_counter.was_copy_assigned = 0;
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        Observable::s_counter.was_destructed = 0;
        sut = full;
        ASSERT_TRUE(sut.has_value());
        ASSERT_TRUE(full.has_value());
        EXPECT_EQ(sut->id, tracking_id);
        EXPECT_EQ(full->id, tracking_id);
        ASSERT_EQ(Observable::s_counter.was_initialized, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 1);
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_destructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 2);
}

TEST(Optional, copy_assignment_from_full_to_full_overwrites_target_value) {
    int32_t const contained_value = 42;
    int32_t const original_target_value = -99;
    Optional<int32_t> sut { original_target_value };
    Optional<int32_t> const full { contained_value };
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
    int32_t const tracking_id = 12345;
    int32_t const overwritten_id = 1111111;
    {
        Optional<Observable> sut { Observable {} };
        Optional<Observable> full { Observable {} };
        ASSERT_TRUE(sut.has_value());
        ASSERT_TRUE(full.has_value());
        sut->id = overwritten_id;
        full->id = tracking_id;
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_copy_constructed = 0;
        Observable::s_counter.was_copy_assigned = 0;
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        Observable::s_counter.was_destructed = 0;
        sut = full;
        ASSERT_TRUE(sut.has_value());
        ASSERT_TRUE(full.has_value());
        ASSERT_EQ(sut->id, tracking_id);
        ASSERT_EQ(full->id, tracking_id);
        ASSERT_EQ(Observable::s_counter.was_initialized, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 1);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_destructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 2);
}

TEST(Optional, copy_assignment_returns_reference_to_this) {
    Optional<Observable> sut;
    Optional<Observable> const full(Observable {});
    ASSERT_EQ(&(sut = full), &sut);
}

TEST(Optional, move_assignment_from_empty_to_empty_leaves_optional_empty) {
    Optional<int32_t> sut;
    Optional<int32_t> empty;
    ASSERT_TRUE(!sut.has_value());
    ASSERT_TRUE(!empty.has_value());
    sut = std::move(empty);
    ASSERT_TRUE(!sut.has_value());
}

TEST_F(OptionalFixture, move_assignment_from_empty_to_empty_does_not_construct_any_objects) {
    {
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_copy_constructed = 0;
        Observable::s_counter.was_copy_assigned = 0;
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        Observable::s_counter.was_destructed = 0;
        Optional<Observable> sut;
        Optional<Observable> empty;
        ASSERT_TRUE(!sut.has_value());
        ASSERT_TRUE(!empty.has_value());
        sut = std::move(empty);
        ASSERT_TRUE(!sut.has_value());
        ASSERT_EQ(Observable::s_counter.was_initialized, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 0);
}

TEST(Optional, move_assignment_from_empty_to_full_empties_target) {
    int32_t const contained_value = 42;
    Optional<int32_t> sut { contained_value };
    Optional<int32_t> empty;
    ASSERT_TRUE(sut.has_value());
    ASSERT_TRUE(!empty.has_value());
    sut = std::move(empty);
    ASSERT_TRUE(!sut.has_value());
}

TEST_F(OptionalFixture, move_assignment_from_empty_to_full_destructs_object_in_target) {
    {
        Optional<Observable> sut { Observable {} };
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_copy_constructed = 0;
        Observable::s_counter.was_copy_assigned = 0;
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        Observable::s_counter.was_destructed = 0;
        Optional<Observable> empty;
        ASSERT_TRUE(sut.has_value());
        ASSERT_TRUE(!empty.has_value());
        sut = std::move(empty);
        ASSERT_TRUE(!sut.has_value());
        ASSERT_EQ(Observable::s_counter.was_initialized, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_destructed, 1);
        Observable::s_counter.was_destructed = 0;
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 0);
}

TEST(Optional, move_assignment_from_full_to_empty_assigns_value_to_target) {
    int32_t const contained_value = 42;
    Optional<int32_t> sut;
    Optional<int32_t> full { contained_value };
    ASSERT_TRUE(!sut.has_value());
    ASSERT_TRUE(full.has_value());
    sut = std::move(full);
    ASSERT_TRUE(sut.has_value());
    EXPECT_EQ(*sut, contained_value);
}

TEST_F(OptionalFixture, move_assignment_from_full_to_empty_move_constructs_object_in_target) {
    int32_t const tracking_id = 12345;
    {
        Optional<Observable> sut;
        Optional<Observable> full { Observable {} };
        ASSERT_TRUE(!sut.has_value());
        ASSERT_TRUE(full.has_value());
        full->id = tracking_id;
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_copy_constructed = 0;
        Observable::s_counter.was_copy_assigned = 0;
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        Observable::s_counter.was_destructed = 0;
        sut = std::move(full);
        ASSERT_TRUE(sut.has_value());
        ASSERT_EQ(sut->id, tracking_id);
        ASSERT_EQ(Observable::s_counter.was_initialized, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 1);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_destructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 2);
}

TEST(Optional, move_assignment_from_full_to_full_overwrites_target_value) {
    int32_t const contained_value = 42;
    int32_t const overwritten_value = -99;
    Optional<int32_t> sut { overwritten_value };
    Optional<int32_t> full { contained_value };
    ASSERT_TRUE(sut.has_value());
    ASSERT_TRUE(full.has_value());
    ASSERT_EQ(*sut, overwritten_value);
    sut = std::move(full);
    ASSERT_TRUE(sut.has_value());
    EXPECT_EQ(*sut, contained_value);
}

TEST_F(OptionalFixture, move_assignment_from_full_to_full_move_assigns_to_target) {
    int32_t const tracking_id = 12345;
    int32_t const overwritten_id = 111111;
    {
        Optional<Observable> sut { Observable {} };
        Optional<Observable> full { Observable {} };
        ASSERT_TRUE(sut.has_value());
        ASSERT_TRUE(full.has_value());
        sut->id = overwritten_id;
        full->id = tracking_id;
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_copy_constructed = 0;
        Observable::s_counter.was_copy_assigned = 0;
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        Observable::s_counter.was_destructed = 0;
        sut = std::move(full);
        ASSERT_TRUE(sut.has_value());
        ASSERT_EQ(sut->id, tracking_id);
        ASSERT_EQ(Observable::s_counter.was_initialized, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 1);
        ASSERT_EQ(Observable::s_counter.was_destructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 2);
}

TEST(Optional, move_assignment_returns_reference_to_this) {
    Optional<Observable> sut;
    Optional<Observable> full(Observable {});
    ASSERT_EQ(&(sut = std::move(full)), &sut);
}

TEST(Optional, assignment_from_nullopt_to_empty_leaves_optional_empty) {
    Optional<int32_t> sut;
    ASSERT_TRUE(!sut.has_value());
    sut = NULLOPT;
    ASSERT_TRUE(!sut.has_value());
}

TEST(Optional, assignment_from_nullopt_to_empty_works_with_braces_syntax) {
    Optional<int32_t> sut;
    ASSERT_TRUE(!sut.has_value());
    sut = {};
    ASSERT_TRUE(!sut.has_value());
}

TEST_F(OptionalFixture, assignment_from_nullopt_to_empty_does_not_construct_an_object) {
    {
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_copy_constructed = 0;
        Observable::s_counter.was_copy_assigned = 0;
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        Observable::s_counter.was_destructed = 0;
        Optional<Observable> sut;
        ASSERT_TRUE(!sut.has_value());
        sut = NULLOPT;
        ASSERT_TRUE(!sut.has_value());
        ASSERT_EQ(Observable::s_counter.was_initialized, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 0);
}

TEST(Optional, assignment_from_nullopt_to_full_empties_optional) {
    int32_t const overwritten_value = -99;
    Optional<int32_t> sut { overwritten_value };
    ASSERT_TRUE(sut.has_value());
    sut = NULLOPT;
    ASSERT_TRUE(!sut.has_value());
}

TEST(Optional, assignment_from_nullopt_to_full_works_with_braces_syntax) {
    int32_t const overwritten_value = -99;
    Optional<int32_t> sut { overwritten_value };
    ASSERT_TRUE(sut.has_value());
    sut = {};
    ASSERT_TRUE(!sut.has_value());
}

TEST_F(OptionalFixture, assignment_from_nullopt_to_full_destructs_contained_object) {
    {
        Optional<Observable> sut { Observable {} };
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_copy_constructed = 0;
        Observable::s_counter.was_copy_assigned = 0;
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        Observable::s_counter.was_destructed = 0;
        ASSERT_TRUE(sut.has_value());
        sut = NULLOPT;
        ASSERT_TRUE(!sut.has_value());
        ASSERT_EQ(Observable::s_counter.was_initialized, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_destructed, 1);
        Observable::s_counter.was_destructed = 0;
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 0);
}

TEST(Optional, assignment_from_nullopt_returns_reference_to_this) {
    Optional<Observable> sut { Observable {} };
    ASSERT_EQ(&(sut = NULLOPT), &sut);
}

TEST(Optional, emplace_in_empty_optional_works) {
    int32_t const contained_value = 42;
    Optional<int32_t> sut;
    sut.emplace(contained_value);
    ASSERT_TRUE(sut.has_value());
    EXPECT_EQ(*sut, contained_value);
}

TEST_F(OptionalFixture, emplace_with_value_move_constructs_for_rvalue) {
    Observable::s_counter.was_initialized = 0;
    Observable::s_counter.was_move_constructed = 0;
    Optional<Observable> sut;
    sut.emplace(Observable {});
    ASSERT_TRUE(sut.has_value());
    EXPECT_EQ(Observable::s_counter.was_initialized, 1);
    EXPECT_EQ(Observable::s_counter.was_move_constructed, 1);
}

TEST_F(OptionalFixture, emplace_with_value_copy_constructs_for_lvalue) {
    Observable::s_counter.was_initialized = 0;
    Observable::s_counter.was_copy_constructed = 0;
    int32_t const contained_value = 888;
    Observable value;
    value.id = contained_value;
    Optional<Observable> sut;
    sut.emplace(value);
    ASSERT_TRUE(sut.has_value());
    EXPECT_EQ(sut->id, value.id);
    EXPECT_EQ(Observable::s_counter.was_initialized, 1);
    EXPECT_EQ(Observable::s_counter.was_copy_constructed, 1);
}

TEST_F(OptionalFixture, emplaced_value_will_be_destructed) {
    Optional<Observable> sut;
    sut.emplace(Observable {});
    Observable::s_counter.was_destructed = 0;
    sut.reset();
    ASSERT_EQ(Observable::s_counter.was_destructed, 1);
}

TEST(Optional, emplace_in_non_empty_optional_replaces_the_old_value) {
    int32_t const contained_old_value = 777;
    int32_t const contained_new_value = 666;
    Observable old_value;
    Observable new_value;
    old_value.id = contained_old_value;
    new_value.id = contained_new_value;
    Optional<Observable> sut(old_value);
    Observable::s_counter.was_initialized = 0;
    Observable::s_counter.was_copy_constructed = 0;
    Observable::s_counter.was_copy_assigned = 0;
    Observable::s_counter.was_move_constructed = 0;
    Observable::s_counter.was_move_assigned = 0;
    Observable::s_counter.was_destructed = 0;
    sut.emplace(new_value);
    ASSERT_TRUE(sut.has_value());
    EXPECT_EQ(sut->id, new_value.id);
    ASSERT_EQ(Observable::s_counter.was_initialized, 0);
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 1);
    ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
    ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
    ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
    ASSERT_EQ(Observable::s_counter.was_destructed, 1);
}

TEST(Optional, operator_arrow_returns_nullptr_for_empty_optional) {
    Optional<int32_t> sut;
    ASSERT_EQ(sut.operator->(), nullptr);
}

TEST(Optional, operator_arrow_returns_pointer_to_contained_value_for_full_optional) {
    int32_t const contained_value = 42;
    Optional<int32_t> sut { contained_value };
    ASSERT_NE(sut.operator->(), nullptr);
    EXPECT_EQ(*(sut.operator->()), contained_value);
}

TEST(Optional, const_operator_arrow_returns_nullptr_for_empty_optional) {
    Optional<int32_t> const sut;
    ASSERT_EQ(sut.operator->(), nullptr);
}

TEST(Optional, const_operator_arrow_returns_pointer_to_contained_value_for_full_optional) {
    int32_t const contained_value = 42;
    Optional<int32_t> const sut { contained_value };
    ASSERT_NE(sut.operator->(), nullptr);
    EXPECT_EQ(*(sut.operator->()), contained_value);
}

TEST(Optional, operator_star_returns_mutable_reference_to_contained_value) {
    int32_t const contained_value = 42;
    Optional<int32_t> sut { contained_value };
    ASSERT_EQ(*sut, contained_value);
    int32_t const alternative_value = 55;
    *sut = alternative_value;
    ASSERT_EQ(*sut, alternative_value);
}

TEST(Optional, const_operator_star_dereferences_contained_value) {
    int32_t const contained_value = 42;
    Optional<int32_t> const sut1 { contained_value };
    ASSERT_EQ(*sut1, 42);
    int32_t const alternative_value = 55;
    Optional<int32_t> const sut2 { alternative_value };
    ASSERT_EQ(*sut2, alternative_value);
}

TEST_F(OptionalFixture, rvalue_operator_star_dereferences_to_rvalue) {
    int32_t const tracking_id = 12345;
    Observable value;
    value.id = tracking_id;
    {
        Optional<Observable> sut { value };
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        Observable::s_counter.was_destructed = 0;
        Observable const move_target = *std::move(sut);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 1);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_destructed, 0);
        ASSERT_EQ(move_target.id, tracking_id);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 2);
}

TEST_F(OptionalFixture, const_rvalue_operator_star_dereferences_to_const_rvalue_and_is_just_not_very_useful_overall) {
    int32_t const tracking_id = 12345;
    Observable value;
    value.id = tracking_id;
    {
        Optional<Observable> const sut { value };
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        Observable::s_counter.was_destructed = 0;
        Observable const&& ref = *std::move(sut);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_destructed, 0);
        ASSERT_EQ(ref.id, tracking_id);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 1);
}

TEST(Optional, operator_bool_checks_for_non_empty) {
    Optional<int32_t> sut;
    ASSERT_FALSE(static_cast<bool>(sut));
    int32_t const just_some_arbitrary_value = 42;
    sut = just_some_arbitrary_value;
    ASSERT_TRUE(static_cast<bool>(sut));
}

TEST(Optional, has_value_checks_for_non_empty) {
    Optional<int32_t> sut;
    ASSERT_FALSE(sut.has_value());
    int32_t const just_some_arbitrary_value = 42;
    sut = just_some_arbitrary_value;
    ASSERT_TRUE(sut.has_value());
}

TEST(Optional, value_returns_mutable_reference_to_contained_value) {
    int32_t const contained_value = 42;
    int32_t const alternative_value = 55;
    Optional<int32_t> sut { contained_value };
    ASSERT_EQ(sut.value(), contained_value);
    sut.value() = alternative_value;
    ASSERT_EQ(sut.value(), alternative_value);
}

TEST(Optional, const_value_dereferences_contained_value) {
    int32_t const contained_value = 42;
    int32_t const alternative_value = 55;
    Optional<int32_t> const sut1 { contained_value };
    ASSERT_EQ(sut1.value(), contained_value);
    Optional<int32_t> const sut2 { alternative_value };
    ASSERT_EQ(sut2.value(), alternative_value);
}

TEST_F(OptionalFixture, rvalue_value_returns_rvalue_dereferences_to_contained_value) {
    int32_t const tracking_id = 12345;
    Observable value;
    value.id = tracking_id;
    {
        Optional<Observable> sut { value };
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        Observable::s_counter.was_destructed = 0;
        Observable const target = std::move(sut).value();
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 1);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_destructed, 0);
        ASSERT_EQ(target.id, tracking_id);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 2);
}

TEST_F(OptionalFixture, const_rvalue_value_dereferences_to_const_rvalue_and_is_just_not_very_useful_overall) {
    int32_t const tracking_id = 12345;
    Observable value;
    value.id = tracking_id;
    {
        Optional<Observable> const sut { value };
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        Observable::s_counter.was_destructed = 0;
        Observable const&& ref = std::move(sut).value();
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_destructed, 0);
        ASSERT_EQ(ref.id, tracking_id);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 1);
}

TEST(Optional, value_or_returns_contained_value_on_full_optional) {
    int32_t const contained_value = 42;
    Optional<int32_t> const sut { contained_value };
    int32_t const fallback = -1;
    ASSERT_EQ(sut.value_or(fallback), contained_value);
}

TEST_F(OptionalFixture, value_or_returns_copy_of_contained_value_on_full_optional) {
    int32_t const tracking_id = 12345;
    int32_t const fallback_id = -1;
    {
        Optional<Observable> sut { Observable {} };
        sut->id = tracking_id;
        Observable fallback;
        fallback.id = fallback_id;
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_copy_constructed = 0;
        Observable::s_counter.was_copy_assigned = 0;
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        Observable::s_counter.was_destructed = 0;
        ASSERT_EQ(sut.value_or(fallback).id, tracking_id);
        ASSERT_EQ(Observable::s_counter.was_initialized, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 1);
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_destructed, 1);
        Observable::s_counter.was_destructed = 0;
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 2);
}

TEST_F(OptionalFixture, value_or_with_rvalue_argument_returns_copy_of_contained_value_on_full_optional) {
    int32_t const tracking_id = 12345;
    int32_t const fallback_id = -1;
    {
        Optional<Observable> sut { Observable {} };
        sut->id = tracking_id;
        Observable fallback;
        fallback.id = fallback_id;
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_copy_constructed = 0;
        Observable::s_counter.was_copy_assigned = 0;
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        Observable::s_counter.was_destructed = 0;
        ASSERT_EQ(sut.value_or(std::move(fallback)).id, tracking_id);
        ASSERT_EQ(Observable::s_counter.was_initialized, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 1);
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_destructed, 1);
        Observable::s_counter.was_destructed = 0;
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 2);
}

TEST(Optional, value_or_returns_fallback_argument_on_empty_optional) {
    int32_t const fallback_value = 225;
    Optional<int32_t> const sut;
    ASSERT_EQ(sut.value_or(fallback_value), fallback_value);
}

TEST_F(OptionalFixture, value_or_returns_copy_of_fallback_argument_on_empty_optional) {
    int32_t const fallback_tracking_id = 225;
    {
        Optional<Observable> const sut;
        Observable fallback;
        fallback.id = fallback_tracking_id;
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_copy_constructed = 0;
        Observable::s_counter.was_copy_assigned = 0;
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        Observable::s_counter.was_destructed = 0;
        ASSERT_EQ(sut.value_or(fallback).id, fallback_tracking_id);
        ASSERT_EQ(Observable::s_counter.was_initialized, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 1);
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_destructed, 1);
        Observable::s_counter.was_destructed = 0;
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 1);
}

TEST_F(OptionalFixture, value_or_moves_rvalue_fallback_argument_on_empty_optional) {
    int32_t const fallback_tracking_id = 225;
    {
        Optional<Observable> const sut;
        Observable fallback;
        fallback.id = fallback_tracking_id;
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_copy_constructed = 0;
        Observable::s_counter.was_copy_assigned = 0;
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        Observable::s_counter.was_destructed = 0;
        ASSERT_EQ(sut.value_or(std::move(fallback)).id, fallback_tracking_id);
        ASSERT_EQ(Observable::s_counter.was_initialized, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 1);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_destructed, 1);
        Observable::s_counter.was_destructed = 0;
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 1);
}


TEST(Optional, rvalue_value_or_returns_contained_value_on_full_optional) {
    int32_t const contained_value = 42;
    int32_t const fallback_value = 225;
    Optional<int32_t> sut { contained_value };
    ASSERT_EQ(std::move(sut).value_or(fallback_value), contained_value);
}


TEST_F(OptionalFixture, rvalue_value_or_moves_contained_value_on_full_optional) {
    int32_t const tracking_id = 12345;
    int32_t const fallback_tracking_id = -1;
    {
        Optional<Observable> sut { Observable {} };
        sut->id = tracking_id;
        Observable fallback;
        fallback.id = fallback_tracking_id;
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_copy_constructed = 0;
        Observable::s_counter.was_copy_assigned = 0;
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        ASSERT_EQ(std::move(sut).value_or(fallback).id, tracking_id);
        ASSERT_EQ(Observable::s_counter.was_initialized, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 1);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
    }
}

TEST_F(OptionalFixture, rvalue_value_or_with_rvalue_argument_moves_contained_value_on_full_optional) {
    int32_t const tracking_id = 12345;
    int32_t const fallback_tracking_id = -1;
    {
        Optional<Observable> sut { Observable {} };
        sut->id = tracking_id;
        Observable fallback;
        fallback.id = fallback_tracking_id;
        Observable::s_counter.was_initialized = 0;
        Observable::s_counter.was_copy_constructed = 0;
        Observable::s_counter.was_copy_assigned = 0;
        Observable::s_counter.was_move_constructed = 0;
        Observable::s_counter.was_move_assigned = 0;
        ASSERT_EQ(std::move(sut).value_or(std::move(fallback)).id, tracking_id);
        ASSERT_EQ(Observable::s_counter.was_initialized, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 1);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
    }
}

TEST(Optional, rvalue_value_or_returns_fallback_on_empty_optional) {
    int32_t const fallback_value = 225;
    Optional<int32_t> sut;
    ASSERT_EQ(std::move(sut).value_or(fallback_value), fallback_value);
}
TEST_F(OptionalFixture, rvalue_value_or_returns_fallback_on_empty_optional) {
    int32_t const fallback_tracking_id = 225;
    Optional<Observable> sut;
    Observable fallback;
    fallback.id = fallback_tracking_id;
    Observable::s_counter.was_initialized = 0;
    Observable::s_counter.was_copy_constructed = 0;
    Observable::s_counter.was_copy_assigned = 0;
    Observable::s_counter.was_move_constructed = 0;
    Observable::s_counter.was_move_assigned = 0;
    ASSERT_EQ(std::move(sut).value_or(fallback).id, fallback_tracking_id);
    ASSERT_EQ(Observable::s_counter.was_initialized, 0);
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 1);
    ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
    ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
    ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
}
TEST_F(OptionalFixture, rvalue_value_or_with_rvalue_argument_moves_fallback_on_empty_optional) {
    int32_t const fallback_tracking_id = 225;
    Optional<Observable> sut;
    Observable fallback;
    fallback.id = fallback_tracking_id;
    Observable::s_counter.was_initialized = 0;
    Observable::s_counter.was_copy_constructed = 0;
    Observable::s_counter.was_copy_assigned = 0;
    Observable::s_counter.was_move_constructed = 0;
    Observable::s_counter.was_move_assigned = 0;
    ASSERT_EQ(std::move(sut).value_or(std::move(fallback)).id, fallback_tracking_id);
    ASSERT_EQ(Observable::s_counter.was_initialized, 0);
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
    ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
    ASSERT_EQ(Observable::s_counter.was_move_constructed, 1);
    ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
}

TEST(Optional, reset_leaves_empty_optional_in_empty_state) {
    Optional<int32_t> sut;
    ASSERT_TRUE(!sut.has_value());
    sut.reset();
    ASSERT_TRUE(!sut.has_value());
}

TEST(Optional, reset_puts_full_optional_to_empty_state) {
    int32_t const contained_value = 42;
    Optional<int32_t> sut { contained_value };
    ASSERT_TRUE(sut.has_value());
    sut.reset();
    ASSERT_TRUE(!sut.has_value());
}

TEST_F(OptionalFixture, reset_on_full_optional_destructs_contained_value) {
    {
        Optional<Observable> sut { Observable {} };
        ASSERT_TRUE(sut.has_value());
        Observable::s_counter.was_destructed = 0;
        sut.reset();
        ASSERT_TRUE(!sut.has_value());
        ASSERT_EQ(Observable::s_counter.was_destructed, 1);
        Observable::s_counter.was_destructed = 0;
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 0);
}

TEST(Optional, operator_arrow_should_bypass_overloaded_operator_ampersand) {
    int32_t const tracking_id = 54321;
    iox2::bb::testing::CustomAddressOperator obj;
    obj.id = tracking_id;
    Optional<iox2::bb::testing::CustomAddressOperator> sut { obj };
    iox2::bb::testing::CustomAddressOperator::s_count_address_operator = 0;
    ASSERT_EQ(sut->id, tracking_id);
    ASSERT_EQ(iox2::bb::testing::CustomAddressOperator::s_count_address_operator, 0);
}

TEST(Optional, const_operator_arrow_should_bypass_overloaded_operator_ampersand) {
    int32_t const tracking_id = 54321;
    iox2::bb::testing::CustomAddressOperator obj;
    obj.id = tracking_id;
    Optional<iox2::bb::testing::CustomAddressOperator> const sut { obj };
    iox2::bb::testing::CustomAddressOperator::s_count_address_operator = 0;
    ASSERT_EQ(sut->id, tracking_id);
    ASSERT_EQ(iox2::bb::testing::CustomAddressOperator::s_count_address_operator, 0);
}

TEST(Optional, operator_equal_with_two_empty_optionals_are_equal) {
    const Optional<uint64_t> lhs { NULLOPT };
    const Optional<uint64_t> rhs { NULLOPT };

    ASSERT_TRUE(lhs == rhs);
}

TEST(Optional, operator_equal_with_two_optionals_with_the_same_value_are_equal) {
    constexpr uint64_t EQUAL_VALUE { 42 };
    const Optional<uint64_t> lhs { EQUAL_VALUE };
    const Optional<uint64_t> rhs { EQUAL_VALUE };

    ASSERT_TRUE(lhs == rhs);
}

TEST(Optional, operator_equal_with_two_optionals_with_different_values_are_not_equal) {
    constexpr uint64_t LHS_VALUE { 37 };
    constexpr uint64_t RHS_VALUE { 73 };
    const Optional<uint64_t> lhs { LHS_VALUE };
    const Optional<uint64_t> rhs { RHS_VALUE };

    ASSERT_FALSE(lhs == rhs);
}

TEST(Optional, operator_equal_with_lhs_value_and_rhs_empty_are_not_equal) {
    constexpr uint64_t LHS_VALUE { 123 };
    const Optional<uint64_t> lhs { LHS_VALUE };
    const Optional<uint64_t> rhs { NULLOPT };

    ASSERT_FALSE(lhs == rhs);
}

TEST(Optional, operator_equal_with_lhs_empty_and_rhs_value_are_not_equal) {
    constexpr uint64_t RHS_VALUE { 13 };
    const Optional<uint64_t> lhs { NULLOPT };
    const Optional<uint64_t> rhs { RHS_VALUE };

    ASSERT_FALSE(lhs == rhs);
}

TEST(Optional, operator_not_equal_with_two_empty_optionals_are_equal) {
    const Optional<uint64_t> lhs { NULLOPT };
    const Optional<uint64_t> rhs { NULLOPT };

    ASSERT_FALSE(lhs != rhs);
}

TEST(Optional, operator_not_equal_with_two_optionals_with_the_same_value_are_equal) {
    constexpr uint64_t EQUAL_VALUE { 42 };
    const Optional<uint64_t> lhs { EQUAL_VALUE };
    const Optional<uint64_t> rhs { EQUAL_VALUE };

    ASSERT_FALSE(lhs != rhs);
}

TEST(Optional, operator_not_equal_with_two_optionals_with_different_values_are_not_equal) {
    constexpr uint64_t LHS_VALUE { 37 };
    constexpr uint64_t RHS_VALUE { 73 };
    const Optional<uint64_t> lhs { LHS_VALUE };
    const Optional<uint64_t> rhs { RHS_VALUE };

    ASSERT_TRUE(lhs != rhs);
}

TEST(Optional, operator_not_equal_with_lhs_value_and_rhs_empty_are_not_equal) {
    constexpr uint64_t LHS_VALUE { 123 };
    const Optional<uint64_t> lhs { LHS_VALUE };
    const Optional<uint64_t> rhs { NULLOPT };

    ASSERT_TRUE(lhs != rhs);
}

TEST(Optional, operator_not_equal_with_lhs_empty_and_rhs_value_are_not_equal) {
    constexpr uint64_t RHS_VALUE { 13 };
    const Optional<uint64_t> lhs { NULLOPT };
    const Optional<uint64_t> rhs { RHS_VALUE };

    ASSERT_TRUE(lhs != rhs);
}

TEST(Optional, operator_equal_with_lhs_value_and_rhs_nullopt_is_not_equal) {
    constexpr uint64_t LHS_VALUE { 666 };
    const Optional<uint64_t> lhs { LHS_VALUE };
    const NulloptT rhs { NULLOPT };

    ASSERT_FALSE(lhs == rhs);
}

TEST(Optional, operator_equal_with_lhs_empty_and_rhs_nullopt_is_equal) {
    const Optional<uint64_t> lhs { NULLOPT };
    const NulloptT rhs { NULLOPT };

    ASSERT_TRUE(lhs == rhs);
}

TEST(Optional, operator_equal_with_lhs_nullopt_and_rhs_value_is_not_equal) {
    constexpr uint64_t RHS_VALUE { 666 };
    const NulloptT lhs { NULLOPT };
    const Optional<uint64_t> rhs { RHS_VALUE };

    ASSERT_FALSE(lhs == rhs);
}

TEST(Optional, operator_equal_with_lhs_nullopt_and_rhs_empty_is_equal) {
    const NulloptT lhs { NULLOPT };
    const Optional<uint64_t> rhs { NULLOPT };

    ASSERT_TRUE(lhs == rhs);
}

TEST(Optional, operator_not_equal_with_lhs_value_and_rhs_nullopt_is_not_equal) {
    constexpr uint64_t LHS_VALUE { 666 };
    const Optional<uint64_t> lhs { LHS_VALUE };
    const NulloptT rhs { NULLOPT };

    ASSERT_TRUE(lhs != rhs);
}

TEST(Optional, operator_not_equal_with_lhs_empty_and_rhs_nullopt_is_equal) {
    const Optional<uint64_t> lhs { NULLOPT };
    const NulloptT rhs { NULLOPT };

    ASSERT_FALSE(lhs != rhs);
}

TEST(Optional, operator_not_equal_with_lhs_nullopt_and_rhs_value_is_not_equal) {
    constexpr uint64_t RHS_VALUE { 666 };
    const NulloptT lhs { NULLOPT };
    const Optional<uint64_t> rhs { RHS_VALUE };

    ASSERT_TRUE(lhs != rhs);
}

TEST(Optional, operator_not_equal_with_lhs_nullopt_and_rhs_empty_is_equal) {
    const NulloptT lhs { NULLOPT };
    const Optional<uint64_t> rhs { NULLOPT };

    ASSERT_FALSE(lhs != rhs);
}

} // namespace
