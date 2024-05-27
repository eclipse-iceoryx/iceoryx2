#[cfg(test)]
mod placement_new {
    use std::alloc::{alloc, dealloc, Layout};

    use iceoryx2_bb_derive_macros::PlacementDefault;
    use iceoryx2_bb_elementary::placement_new::PlacementDefault;

    #[derive(Default, PlacementDefault)]
    struct TestStruct {
        a: i32,
        b: u64,
    }

    #[test]
    fn whatever() {
        let layout = Layout::new::<TestStruct>();
        let memory = unsafe { alloc(layout) } as *mut TestStruct;
        let sut = TestStruct::default();
        unsafe { TestStruct::placement_default(memory) };

        unsafe { dealloc(memory.cast(), layout) };
    }
}
