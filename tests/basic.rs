use const_push::ConstVec;

const fn basic_push_value() -> ConstVec<u32, 10> {
    ConstVec::new().push(10).push(20).push(30)
}
const BASIC_PUSH: ConstVec<u32, 10> = basic_push_value();
#[test]
fn basic_push_test() {
    assert!((&BASIC_PUSH).into_iter().copied().eq([10, 20, 30]));
}
