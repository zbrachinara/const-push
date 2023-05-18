use crate::{ConstVec, MaybeUninit};

pub struct ConstVecIter<'a, T, const N: usize> {
    vec: &'a ConstVec<T, N>,
    ix: usize,
}

impl<'a, T, const N: usize> IntoIterator for &'a ConstVec<T, N> {
    type Item = &'a T;

    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<'a, T, const N: usize> Iterator for ConstVecIter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        use core::ops::Deref;
        if self.ix < self.vec.len {
            let res = unsafe { &self.vec.xs[self.ix].value }.deref();
            self.ix += 1;
            Some(res)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.vec.len - self.ix;
        (len, Some(len))
    }
}

pub struct ConstVecIntoIter<T, const CAP: usize> {
    xs: [MaybeUninit<T>; CAP],
    ix: usize,
    len: usize,
}

impl<T, const CAP: usize> IntoIterator for ConstVec<T, CAP> {
    type Item = T;

    type IntoIter = ConstVecIntoIter<T, CAP>;

    fn into_iter(self) -> Self::IntoIter {
        ConstVecIntoIter {
            xs: self.xs,
            ix: 0,
            len: self.len,
        }
    }
}

impl<T, const CAP: usize> Iterator for ConstVecIntoIter<T, CAP> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        (self.ix < self.len).then(|| {
            let item = core::mem::replace(&mut self.xs[self.ix], MaybeUninit::uninit());
            self.ix += 1;
            unsafe { item.assume_init() }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len - self.ix;
        (len, Some(len))
    }
}

impl<T, const CAP: usize> core::fmt::Debug for ConstVec<T, CAP>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self).finish()
    }
}
