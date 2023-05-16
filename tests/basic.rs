use const_push::ConstVec;

const fn basic_push_value() -> ConstVec<u32, 10> {
    ConstVec::new().push(10).push(20).push(30)
}
const BASIC_PUSH: ConstVec<u32, 10> = basic_push_value();
#[test]
fn basic_push_test() {
    assert!((&BASIC_PUSH).into_iter().copied().eq([10, 20, 30]));
}

const fn drop_const_vec() {
    ConstVec::<u32, 10>::new();
}
#[allow(unused)]
const DROP_CONST_VEC: () = drop_const_vec();

const fn drop_const_vec_with_elems() {
    ConstVec::<u32, 10>::new().push(10).push(20).push(30);
}
#[allow(unused)]
const DROP_CONST_VEC_WITH_ELEMS: () = drop_const_vec_with_elems();