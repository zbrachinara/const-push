
//! Provides an arrayvec-like type which can be modified at const-time.

use std::mem::MaybeUninit;

pub struct ConstVec<T, const CAP: usize> {
    xs: [MaybeUninit<T>; CAP],
    len: usize,
}
