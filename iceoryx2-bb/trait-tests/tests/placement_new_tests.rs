#[cfg(test)]
mod placement_new {
    use std::{
        alloc::{alloc, dealloc, Layout},
        sync::atomic::{AtomicUsize, Ordering},
    };

    use iceoryx2_bb_derive_macros::PlacementDefault;
    use iceoryx2_bb_elementary::placement_new::PlacementDefault;
    use iceoryx2_bb_testing::assert_that;

    static DEFAULT_CTOR_COUNT: AtomicUsize = AtomicUsize::new(0);

    #[derive(Copy, Clone)]
    struct UnitStruct;

    impl PlacementDefault for UnitStruct {
        unsafe fn placement_default(_ptr: *mut Self) {
            DEFAULT_CTOR_COUNT.fetch_add(1, Ordering::Relaxed);
        }
    }

    struct Fuu(i32);

    impl PlacementDefault for Fuu {
        unsafe fn placement_default(ptr: *mut Self) {
            DEFAULT_CTOR_COUNT.fetch_add(1, Ordering::Relaxed);
            ptr.write(Self(0))
        }
    }

    struct Bar {
        value: u64,
    }

    impl PlacementDefault for Bar {
        unsafe fn placement_default(ptr: *mut Self) {
            DEFAULT_CTOR_COUNT.fetch_add(1, Ordering::Relaxed);
            ptr.write(Self { value: 123 })
        }
    }

    #[derive(PlacementDefault)]
    struct TestStruct {
        value1: UnitStruct,
        value2: Fuu,
        value3: Bar,
    }

    #[test]
    fn placement_default_works() {
        DEFAULT_CTOR_COUNT.store(0, Ordering::Relaxed);

        let layout = Layout::new::<TestStruct>();
        let memory = unsafe { alloc(layout) } as *mut TestStruct;
        unsafe { TestStruct::placement_default(memory) };

        assert_that!(DEFAULT_CTOR_COUNT.load(Ordering::Relaxed), eq 3);

        unsafe { dealloc(memory.cast(), layout) };
    }
}
