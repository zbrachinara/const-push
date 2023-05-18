use const_push::ConstVec;

const fn construct_from_array() -> ConstVec<u32, 10>{
    ConstVec::from_array([10, 20, 30])
}
const CONSTRUCT_FROM_ARRAY: ConstVec<u32, 10> = construct_from_array();
#[test]
fn test_construct_from_array() {
    assert_eq!(CONSTRUCT_FROM_ARRAY.as_slice(), &[10, 20, 30]);
}