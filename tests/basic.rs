use std::assert_eq;

use const_push::ConstVec;

const BASIC_PUSH: ConstVec<u32, 10> = basic_push_value();
const fn basic_push_value() -> ConstVec<u32, 10> {
    ConstVec::new().push(10).push(20).push(30)
}

#[test]
fn basic_push_test() {}
