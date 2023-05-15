use std::assert_eq;

use const_push::ConstVec;


const BASIC_PUSH: ConstVec<u32, 10> = basic_push_value();
const fn basic_push_value() -> ConstVec<u32, 10> {
    let vec = ConstVec::new();

    vec   
}

#[test]
fn basic_push_test() {
}