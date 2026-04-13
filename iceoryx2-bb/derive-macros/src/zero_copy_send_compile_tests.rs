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

/// === Structs ===

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
///
/// #[derive(ZeroCopySend)]
/// struct Foo(u16);
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_when_type_is_not_annotated_with_repr_c() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// struct Foo(u16);
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// struct NamedTestStruct {
///     val1: u64,
///     val2: Foo,
/// }
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_named_struct_when_not_all_members_implement_it() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// struct Foo(u16);
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// struct UnnamedTestStruct(i32, u32, Foo);
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_unnamed_struct_when_not_all_members_implement_it() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// struct GenericNamedTestStruct<T1, T2> {
///     val1: T1,
///     val2: T2,
/// }
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_generic_named_struct_when_members_do_not_implement_it() {
}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// struct GenericNamedTestStruct<T1: ZeroCopySend, T2> {
///     val1: T1,
///     val2: T2,
/// }
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_generic_named_struct_when_not_all_members_implement_it()
{
}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// struct GenericUnnamedTestStruct<T1, T2>(T1, T2);
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_generic_unnamed_struct_when_members_do_not_implement_it()
{
}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// struct GenericUnnamedTestStruct<T1: ZeroCopySend, T2>(T1, T2);
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_generic_unnamed_struct_when_not_all_members_implement_it()
 {
}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// #[name("abc")]
/// struct TestStruct(u8, i32);
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_with_wrong_attribute() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// #[type_name("abc")]
/// #[type_name("def")]
/// struct TestStruct(u8, i32);
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_with_more_than_one_attribute() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// #[type_name]
/// struct TestStruct(u8, i32);
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_without_attribute_argument() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// #[type_name()]
/// struct TestStruct(u8, i32);
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_with_empty_attribute_argument() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// #[type_name("abc" "def")]
/// struct TestStruct(u8, i32);
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_with_more_than_one_attribute_argument() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// #[type_name("abc", "def")]
/// struct TestStruct(u8, i32);
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_with_more_than_one_attribute_argument_comma_separated() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// #[type_name(abc)]
/// struct TestStruct(u8, i32);
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_with_non_string_literal_attribute() {}

/// === Enums ===

/// ```
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// enum TestEnum {
///     Variant1,
///     Variant2(u64),
///     Variant3 { field: u32 },
/// }
///
/// fn require_zero_copy_send<T: ZeroCopySend>(_: &T) {}
///
/// let x = TestEnum::Variant1;
/// require_zero_copy_send(&x);
///
/// let y = TestEnum::Variant2(42);
/// require_zero_copy_send(&y);
///
/// let z = TestEnum::Variant3 { field: 123 };
/// require_zero_copy_send(&z);
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_works_for_enum_with_various_variant_types() {}

/// ```
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// enum TestEnum<T: ZeroCopySend, U: ZeroCopySend> {
///     Variant1,
///     Variant2(T),
///     Variant3 { field: U },
/// }
///
/// fn require_zero_copy_send<T: ZeroCopySend>(_: &T) {}
///
/// let x = TestEnum::Variant1::<u32, u64>;
/// require_zero_copy_send(&x);
///
/// let y = TestEnum::Variant2::<u32, u64>(42);
/// require_zero_copy_send(&y);
///
/// let z = TestEnum::Variant3::<u32, u64> { field: 123 };
/// require_zero_copy_send(&z);
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_works_for_generic_enum_when_all_members_implement_it() {}

/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[derive(ZeroCopySend)]
/// enum EmptyEnum {}
///
/// fn require_zero_copy_send<T: ZeroCopySend>() {}
/// require_zero_copy_send::<EmptyEnum>;
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_works_for_empty_enum() {}

/// ```
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// struct InnerStruct {
///     field: u32,
/// }
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// enum TestEnum {
///     Variant1,
///     Variant2(InnerStruct),
///     Variant3 { field: InnerStruct },
/// }
///
/// fn require_zero_copy_send<T: ZeroCopySend>(_: &T) {}
///
/// let inner = InnerStruct { field: 42 };
/// let x = TestEnum::Variant1;
/// require_zero_copy_send(&x);
///
/// let y = TestEnum::Variant2(inner);
/// require_zero_copy_send(&y);
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_works_for_enum_with_nested_zero_copy_send_types() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// struct Foo(u16);
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// enum TestEnum {
///     Variant1,
///     Variant2(u64),
///     Variant3 { field: Foo },
/// }
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_enum_when_not_all_members_implement_it() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// struct Foo(u16);
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// enum TestEnum {
///     Variant1,
///     Variant2(u64, Foo),
/// }
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_tuple_variant_when_not_all_members_implement_it() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// struct Foo(u16);
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// enum TestEnum {
///     Variant1,
///     Variant2 { field1: u64, field2: Foo },
/// }
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_struct_variant_when_not_all_members_implement_it() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// enum GenericEnum<T1, T2> {
///     Variant1,
///     Variant2(T1),
///     Variant3 { field: T2 },
/// }
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_generic_enum_when_members_do_not_implement_it() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// enum GenericEnum<T1: ZeroCopySend, T2> {
///     Variant1,
///     Variant2(T1),
///     Variant3 { field: T2 },
/// }
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_generic_enum_when_not_all_members_implement_it() {}

/// === Unions ===

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[derive(Copy, Clone)]
/// struct Foo(u16);
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// union SomeUnion {
///     val1: u64,
///     val2: Foo,
/// }
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_union_when_not_all_members_implement_it() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// union GenericUnion<T1: Copy, T2: Copy> {
///     val1: T1,
///     val2: T2,
/// }
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_generic_union_when_members_do_not_implement_it() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// union GenericUnion<T1: Copy + ZeroCopySend, T2: Copy> {
///     val1: T1,
///     val2: T2,
/// }
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_generic_union_when_not_all_members_implement_it() {}

/// ```
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// struct NamedTestStruct {
///     val1: u8,
///     val2: u64,
///     val3: u32,
/// }
/// let t = NamedTestStruct {val1: 0, val2: 0, val3: 0};
/// let v = t.__get_members();
/// assert_eq!(v[0], (0, 1));
/// assert_eq!(v[1], (8, 8));
/// assert_eq!(v[2], (16, 4));
/// assert_eq!(v[2].0, 16);
/// ```
#[cfg(doctest)]
fn blub_struct() {}

/// ```
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// struct NamedTestStruct<T: ZeroCopySend> {
///     val1: u8,
///     val2: T,
///     val3: u32,
/// }
/// let t = NamedTestStruct {val1: 0, val2: -9, val3: 0};
/// let v = t.__get_members();
/// assert_eq!(v[0], (0, 1));
/// assert_eq!(v[1], (4, 4));
/// assert_eq!(v[2], (8, 4));
/// assert_eq!(v[2].0, 8);
/// ```
#[cfg(doctest)]
fn blub_generic_struct() {}

/// ```
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// struct Foo(u16);
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// struct UnnamedTestStruct(i32, u64, Foo);
/// let t = UnnamedTestStruct(0, 0, Foo(0));
/// let v = t.__get_members();
/// assert_eq!(v[0], (0, 4));
/// assert_eq!(v[1], (8, 8));
/// assert_eq!(v[2], (16, 2));
/// assert_eq!(v[2].0, 16);
/// ```
#[cfg(doctest)]
fn blub_tuple_struct() {}

/// ```
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// struct Foo(u16);
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// struct UnnamedTestStruct<T: ZeroCopySend>(i32, T, Foo);
/// let t = UnnamedTestStruct(0, 0u64, Foo(0));
/// let v = t.__get_members();
/// assert_eq!(v[0], (0, 4));
/// assert_eq!(v[1], (8, 8));
/// assert_eq!(v[2], (16, 2));
/// assert_eq!(v[2].0, 16);
/// ```
#[cfg(doctest)]
fn blub_generic_tuple_struct() {}

/// ```
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(Copy, Clone, ZeroCopySend)]
/// struct Foo(u16);
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// union SomeUnion {
///     val1: u64,
///     val2: Foo,
/// }
///
/// let u1 = SomeUnion {val1: 0};
/// let v = u1.__get_members();
/// assert_eq!(v.len(), 1);
/// assert_eq!(v[0], (0, 8));
/// let u2 = SomeUnion {val2: Foo(0)};
/// let v2 = u2.__get_members();
/// assert_eq!(v2.len(), 1);
/// assert_eq!(v2[0], (0, 8));
/// ```
#[cfg(doctest)]
fn blub_union() {}

/// ```
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// union GenericUnion<T1: Copy + ZeroCopySend, T2: Copy + ZeroCopySend> {
///     val1: T1,
///     val2: T2,
/// }
///
/// let u1: GenericUnion<u32, u64> = GenericUnion {val1: 0};
/// let v1 = u1.__get_members();
/// assert_eq!(v1.len(), 1);
/// assert_eq!(v1[0], (0, 8));
/// let u2: GenericUnion<u32, u64> = GenericUnion {val2: 8};
/// let v2 = u2.__get_members();
/// assert_eq!(v2.len(), 1);
/// assert_eq!(v2[0], (0, 8));
/// ```
#[cfg(doctest)]
fn blub_generic_union() {}

/// ```
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[repr(align(16))]
/// #[derive(ZeroCopySend)]
/// struct AlignedU32(u32);
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// struct FieldAlignedStruct {
///     a: u8,
///     b: AlignedU32,
/// }
///
/// let t = FieldAlignedStruct { a: 3, b: AlignedU32(9) };
/// let v = t.__get_members();
/// assert_eq!(v.len(), 2);
/// assert_eq!(v[0], (0, 1));
/// assert_eq!(v[1], (16, 4));
/// ```
#[cfg(doctest)]
fn blub_with_alignment_that_changes_inner_padding() {}

/// ```
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// struct TupleStruct(u64);
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// struct FieldStruct {
///     a: u8,
///     b: i16,
/// }
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// struct NestedStruct {
///     val0: TupleStruct,
///     val1: u32,
///     val2: FieldStruct,
/// }
///
/// let t = NestedStruct { val0: TupleStruct(32687), val1: 7, val2: FieldStruct { a: 1, b: -4 } };
/// let v = t.__get_members();
/// assert_eq!(v.len(), 4);
/// assert_eq!(v[0], (0, 8));
/// assert_eq!(v[1], (8, 4));
/// assert_eq!(v[2], (12, 1));
/// assert_eq!(v[3], (14, 2));
/// ```
#[cfg(doctest)]
fn blub_nested_struct() {}

/// ```
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(Copy, Clone)]
/// #[derive(ZeroCopySend)]
/// struct FieldStruct {
///     a: u8,
///     b: u64,
/// }
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// union NestedUnion {
///     val0: u8,
///     val1: u16,
///     val2: FieldStruct,
/// }
///
/// let u1 = NestedUnion { val0: 1 };
/// let v1 = u1.__get_members();
/// assert_eq!(v1.len(), 1);
/// assert_eq!(v1[0], (0, 16));
/// let u2 = NestedUnion { val1: 2 };
/// let v2 = u2.__get_members();
/// assert_eq!(v2.len(), 1);
/// assert_eq!(v2[0], (0, 16));
/// let u3 = NestedUnion { val2: FieldStruct { a: 3, b: 4, } };
/// let v3 = u3.__get_members();
/// assert_eq!(v3.len(), 2);
/// assert_eq!(v3[0], (0, 8));
/// assert_eq!(v3[1], (8, 8));
/// ```
#[cfg(doctest)]
fn blub_union_with_struct() {}
// TODO: test union that contains a struct
