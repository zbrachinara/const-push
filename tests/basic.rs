use std::{assert_eq, matches};

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
    let c = ConstVec::<u32, 10>::new().push(10).push(20).push(30);
    assert!(matches!(c.get(0), Some(&x) if x == 10));
    assert!(matches!(c.get(1), Some(&x) if x == 20));
    assert!(matches!(c.get(2), Some(&x) if x == 30));
}
#[allow(unused)]
const DROP_CONST_VEC_WITH_ELEMS: () = drop_const_vec_with_elems();

const fn pop_elems() -> u32 {
    let c = ConstVec::<u32, 10>::new().push(10).push(20);

    c.pop().1
}
const POPPED_ELEM: u32 = pop_elems();
#[test]
fn test_popped_elem() {
    assert_eq!(POPPED_ELEM, 20)
}

const fn try_swap_remove() -> (ConstVec<u32, 10>, Option<u32>) {
    ConstVec::new()
        .push(10)
        .push(20)
        .push(30)
        .push(40)
        .try_swap_remove(1)
}
const TRY_SWAP_REMOVE_TEST: (ConstVec<u32, 10>, Option<u32>) = try_swap_remove();
#[test]
fn test_try_swap_remove() {
    (&TRY_SWAP_REMOVE_TEST.0)
        .into_iter()
        .copied()
        .zip([10, 40, 30])
        .enumerate()
        .for_each(|(ix, (const_constructed, test))| {
            assert_eq!(
                const_constructed, test,
                "Elements at index {ix} do not match (list is {:?})", TRY_SWAP_REMOVE_TEST.0
            )
        });

    assert_eq!(TRY_SWAP_REMOVE_TEST.1, Some(20))
}
