use iceoryx2_pal_concurrency_sync::iox_atomic::*;

pub trait PlacementDefault {
    /// # Safety
    ///
    ///  * ptr must have at least the alignment of Self
    ///  * ptr must point to a memory location with at least the size of Size
    ///  * ptr must point to a valid memory location
    ///  * shall not be called on already initialized memory
    unsafe fn placement_default(ptr: *mut Self);
}

macro_rules! Impl {
    ($type:ty) => {
        impl PlacementDefault for $type {
            unsafe fn placement_default(ptr: *mut Self) {
                ptr.write(<$type>::default())
            }
        }
    };
}

Impl!(f32);
Impl!(f64);
Impl!(u8);
Impl!(u16);
Impl!(u32);
Impl!(u64);
Impl!(u128);
Impl!(i8);
Impl!(i16);
Impl!(i32);
Impl!(i64);
Impl!(i128);
Impl!(isize);
Impl!(usize);
Impl!(char);
Impl!(bool);
Impl!(IoxAtomicBool);
Impl!(IoxAtomicU8);
Impl!(IoxAtomicU16);
Impl!(IoxAtomicU32);
Impl!(IoxAtomicU64);
Impl!(IoxAtomicI8);
Impl!(IoxAtomicI16);
Impl!(IoxAtomicI32);
Impl!(IoxAtomicI64);
Impl!(IoxAtomicIsize);
Impl!(IoxAtomicUsize);

impl<T: PlacementDefault + Default, const CAPACITY: usize> PlacementDefault for [T; CAPACITY] {
    unsafe fn placement_default(ptr: *mut Self) {
        for i in 0..CAPACITY {
            (ptr as *mut T).add(i).write(T::default())
        }
    }
}

impl<T: PlacementDefault + Default> PlacementDefault for (T,) {
    unsafe fn placement_default(ptr: *mut Self) {
        ptr.write((T::default(),))
    }
}

impl<T> PlacementDefault for Option<T> {
    unsafe fn placement_default(ptr: *mut Self) {
        ptr.write(None)
    }
}
