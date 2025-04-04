// Test if something doesn't compile?
//
// Tests with free functions or struct like:
//struct IsRelocatable<T: Relocatable> {
//phantom: PhantomData<T>,
//}

#[cfg(test)]
mod zero_copy_send {
    use iceoryx2_bb_derive_macros::ZeroCopySend;
    use iceoryx2_bb_elementary::relocatable::Relocatable;
    use iceoryx2_bb_elementary::zero_copy_send::ZeroCopySend;

    #[derive(ZeroCopySend)]
    //#[zero_copy_id(x < 1)]
    struct MyType {
        val: u64,
        //val: String,
    }

    #[derive(ZeroCopySend)]
    struct MyTupleStruct(i32, u64);
    //struct MyTupleStruct(i32, String);

    //#[derive(ZeroCopySend)]
    //struct MyUnitStruct;

    #[test]
    fn blub_works() {
        let x = MyType { val: 9 };
        //let x = MyType {
        //val: String::from("test"),
        //};
        println!("val = {}", x.val);

        let y = MyTupleStruct(0, 0);
        //let y = MyTupleStruct(0, String::from("hui"));
        println!("tuple = {}, {}", y.0, y.1);
    }
}
