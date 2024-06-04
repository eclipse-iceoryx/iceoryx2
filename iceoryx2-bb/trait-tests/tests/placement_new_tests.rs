#[cfg(test)]
mod placement_new {
    use std::{
        alloc::{alloc, dealloc, Layout},
        sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    };

    use iceoryx2_bb_derive_macros::PlacementDefault;
    use iceoryx2_bb_elementary::placement_new::PlacementDefault;
    use iceoryx2_bb_testing::assert_that;

    static DEFAULT_CTOR_COUNT: AtomicUsize = AtomicUsize::new(0);
    static FUU_VALUE: AtomicU64 = AtomicU64::new(0);
    static BAR_VALUE: AtomicU64 = AtomicU64::new(0);

    #[derive(Copy, Clone)]
    struct UnitStruct;

    impl PlacementDefault for UnitStruct {
        unsafe fn placement_default(_ptr: *mut Self) {
            DEFAULT_CTOR_COUNT.fetch_add(1, Ordering::Relaxed);
        }
    }

    struct Fuu(u64);

    impl PlacementDefault for Fuu {
        unsafe fn placement_default(ptr: *mut Self) {
            DEFAULT_CTOR_COUNT.fetch_add(1, Ordering::Relaxed);
            ptr.write(Self(FUU_VALUE.load(Ordering::Relaxed)))
        }
    }

    struct Bar {
        value: u64,
    }

    impl PlacementDefault for Bar {
        unsafe fn placement_default(ptr: *mut Self) {
            DEFAULT_CTOR_COUNT.fetch_add(1, Ordering::Relaxed);
            ptr.write(Self {
                value: BAR_VALUE.load(Ordering::Relaxed),
            })
        }
    }

    #[derive(PlacementDefault)]
    struct NamedTestStruct {
        value1: UnitStruct,
        value2: Fuu,
        value3: Bar,
    }

    #[derive(PlacementDefault)]
    struct UnnamedTestStruct(Fuu, Bar, Bar, UnitStruct, UnitStruct, UnitStruct);

    #[derive(PlacementDefault)]
    struct GenericStruct<T1: PlacementDefault, T2: PlacementDefault> {
        value1: T1,
        value2: T2,
    }

    #[derive(PlacementDefault)]
    struct GenericUnnamedStruct<T1: PlacementDefault, T2: PlacementDefault>(T1, T2);

    #[test]
    fn placement_default_derive_for_structs_works() {
        DEFAULT_CTOR_COUNT.store(0, Ordering::Relaxed);
        FUU_VALUE.store(123, Ordering::Relaxed);
        BAR_VALUE.store(456, Ordering::Relaxed);

        let layout = Layout::new::<NamedTestStruct>();
        let memory = unsafe { alloc(layout) } as *mut NamedTestStruct;
        unsafe { NamedTestStruct::placement_default(memory) };

        assert_that!(DEFAULT_CTOR_COUNT.load(Ordering::Relaxed), eq 3);
        assert_that!(unsafe{ &*memory }.value2.0, eq FUU_VALUE.load(Ordering::Relaxed));
        assert_that!(unsafe{ &*memory }.value3.value, eq BAR_VALUE.load(Ordering::Relaxed));

        unsafe { dealloc(memory.cast(), layout) };
    }

    #[test]
    fn placement_default_derive_for_unnamed_structs_works() {
        DEFAULT_CTOR_COUNT.store(0, Ordering::Relaxed);
        FUU_VALUE.store(789, Ordering::Relaxed);
        BAR_VALUE.store(1337, Ordering::Relaxed);

        let layout = Layout::new::<UnnamedTestStruct>();
        let memory = unsafe { alloc(layout) } as *mut UnnamedTestStruct;
        unsafe { UnnamedTestStruct::placement_default(memory) };

        assert_that!(DEFAULT_CTOR_COUNT.load(Ordering::Relaxed), eq 6);
        assert_that!(unsafe{ &*memory }.0.0, eq FUU_VALUE.load(Ordering::Relaxed));
        assert_that!(unsafe{ &*memory }.1.value, eq BAR_VALUE.load(Ordering::Relaxed));
        assert_that!(unsafe{ &*memory }.2.value, eq BAR_VALUE.load(Ordering::Relaxed));

        unsafe { dealloc(memory.cast(), layout) };
    }

    #[test]
    fn placement_default_derive_for_generic_structs_works() {
        type SutType = GenericStruct<Fuu, Bar>;
        DEFAULT_CTOR_COUNT.store(0, Ordering::Relaxed);
        FUU_VALUE.store(4711, Ordering::Relaxed);
        BAR_VALUE.store(247, Ordering::Relaxed);

        let layout = Layout::new::<SutType>();
        let memory = unsafe { alloc(layout) } as *mut SutType;
        unsafe { SutType::placement_default(memory) };

        assert_that!(DEFAULT_CTOR_COUNT.load(Ordering::Relaxed), eq 2);
        assert_that!(unsafe{ &*memory }.value1.0, eq FUU_VALUE.load(Ordering::Relaxed));
        assert_that!(unsafe{ &*memory }.value2.value, eq BAR_VALUE.load(Ordering::Relaxed));

        unsafe { dealloc(memory.cast(), layout) };
    }

    #[test]
    fn placement_default_derive_for_generic_unnamed_structs_works() {
        type SutType = GenericUnnamedStruct<Fuu, Bar>;
        DEFAULT_CTOR_COUNT.store(0, Ordering::Relaxed);
        FUU_VALUE.store(895711, Ordering::Relaxed);
        BAR_VALUE.store(89547, Ordering::Relaxed);

        let layout = Layout::new::<SutType>();
        let memory = unsafe { alloc(layout) } as *mut SutType;
        unsafe { SutType::placement_default(memory) };

        assert_that!(DEFAULT_CTOR_COUNT.load(Ordering::Relaxed), eq 2);
        assert_that!(unsafe{ &*memory }.0.0, eq FUU_VALUE.load(Ordering::Relaxed));
        assert_that!(unsafe{ &*memory }.1.value, eq BAR_VALUE.load(Ordering::Relaxed));

        unsafe { dealloc(memory.cast(), layout) };
    }
}
