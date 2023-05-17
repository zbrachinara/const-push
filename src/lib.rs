#![no_std]
//! Provides an arrayvec-like type which can be modified at const-time.
//!
//! Depends on the feature const_ptr_read, which is stable in the nightly rust version `1.71.0`.

use core::ops::Deref;
use core::ptr::addr_of;
use core::{mem::ManuallyDrop, panic};

use tap::Tap;

/// This is a macro meant for internal use on `ConstVec`. It copies the element at the given index
/// to the stack, not modifying the original index.
///
/// # Safety
///
/// At the time of writing, there are a lot of limitations around const. In this case, the relevant
/// issue is that we are not allowed to get references to objects which may contain [`UnsafeCell`]s
/// -- both `&self` and `&self.xs` are impossible. This is used to do operations such as remove,
/// remove_swap, or pop, which would usually require a reference to `self.xs` at least in order to
/// copy its contents onto the stack. Thus if T contains an `UnsafeCell` this approach probably
/// isn't allowed, or maybe it just isn't allowed at all because of the strange things we do with a
/// zst pointer.
///
/// And of course, since this function performs a copy of a non-copy type, you need to make sure
/// that *the element at this index is never accessed as a `T` again*.
macro_rules! copy_item {
    ($self:ident<$item_type:ty>[$ix:expr]) => {{
        // we can't get a pointer to xs or self, but we can get one to a zst with the same address
        let ptr_to_xs = core::ptr::addr_of!($self.xs_addr) as *const $item_type;
        // we have a pointer to our array now, but we need a pointer to the item's location
        let ptr_to_elem = ptr_to_xs.add($ix);
        // and then use a ptr read obtain the item
        core::ptr::read(ptr_to_elem)
    }};
}

pub struct CapacityError<T, const CAP: usize> {
    pub vector: ConstVec<T, CAP>,
    pub item: T,
}

impl<T, const CAP: usize> core::fmt::Debug for CapacityError<T, CAP> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CapacityError")
            .field("capacity", &CAP)
            .finish()
    }
}

/// Essentially a [std::mem::MaybeUninit], but with externals exposed for const contexts
union MaybeUninit<T> {
    uninit: (),
    value: ManuallyDrop<T>,
}

impl<T> MaybeUninit<T> {
    const fn uninit() -> Self {
        Self { uninit: () }
    }

    /// #Safety
    /// Undefined behavior where `T` is uninhabited
    const unsafe fn assume_init(self) -> T {
        ManuallyDrop::into_inner(self.value)
    }
}

#[repr(C)]
pub struct ConstVec<T, const CAP: usize> {
    len: usize,
    xs_addr: (),
    xs: [MaybeUninit<T>; CAP],
}

impl<T, const CAP: usize> ConstVec<T, CAP> {
    pub const fn new() -> Self {
        Self {
            xs: unsafe { MaybeUninit::uninit().assume_init() },
            xs_addr: (),
            len: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(addr_of!(self.xs_addr) as *const T, self.len) }
    }

    pub const fn get(&self, ix: usize) -> Option<&T> {
        if ix < self.len {
            Some(unsafe { core::mem::transmute(&self.xs[ix].value) })
        } else {
            None
        }
    }

    pub const fn try_swap_remove(mut self, ix: usize) -> (Self, Option<T>) {
        if self.len > 0 {
            if ix == self.len - 1 {
                self.try_pop()
            } else {
                unsafe {
                    let removing = copy_item!(self<T>[ix]);
                    let swapping = copy_item!(self<ManuallyDrop<T>>[self.len - 1]);
                    self.xs[ix] = MaybeUninit { value: swapping };
                    let len = self.len - 1;
                    self = self.set_len(len);
                    (self, Some(removing))
                }
            }
        } else {
            (self, None)
        }
    }

    pub const fn pop(mut self) -> (Self, T) {
        if self.len > 0 {
            let new_len = self.len - 1;
            let item = unsafe {
                let item = copy_item!(self<T>[new_len]);
                self = self.set_len(new_len);
                item
            };
            (self, item)
        } else {
            panic!()
        }
    }

    pub const fn try_pop(mut self) -> (Self, Option<T>) {
        if self.len > 0 {
            let new_len = self.len - 1;
            let item = unsafe {
                let item = copy_item!(self<T>[new_len]);
                self = self.set_len(new_len);
                item
            };
            (self, Some(item))
        } else {
            (self, None)
        }
    }

    /// # Safety
    ///
    /// At the time of writing, there are a lot of limitations around const. In this case, the
    /// relevant issue is that we are not allowed to get references to objects which may contain
    /// [`UnsafeCell`]s -- both `&self` and `&self.xs` are impossible. There is a lot of weird code
    /// in here to get around this, but the bottom line is that I don't know if this is actually a
    /// sound thing to do, and actually, with pointer provenance I think it is completey unsound.
    /// However, if you still feel like using this approach, I guess just avoid putting in types
    /// which have [`UnsafeCell`]s in them (which shouldn't be hard anyway, given that const
    /// disallows heap allocations).
    pub const unsafe fn pop_unchecked(mut self) -> (Self, T) {
        debug_assert!(self.len > 0);
        let new_len = self.len - 1;
        let item = copy_item!(self<T>[new_len]);
        self = self.set_len(new_len);
        (self, item)
    }

    pub const fn push(self, item: T) -> Self {
        if self.len < CAP {
            unsafe { self.push_unchecked(item) }
        } else {
            panic!()
        }
    }

    pub const fn try_push(self, item: T) -> Result<Self, CapacityError<T, CAP>> {
        if self.len < CAP {
            unsafe { Ok(self.push_unchecked(item)) }
        } else {
            Err(CapacityError { vector: self, item })
        }
    }

    pub const unsafe fn push_unchecked(mut self, item: T) -> Self {
        debug_assert!(self.len < CAP);
        self.xs[self.len] = MaybeUninit {
            // TODO actually make this unchecked
            value: ManuallyDrop::new(item),
        };
        let len = self.len;
        self = self.set_len(len + 1);
        self
    }

    pub const unsafe fn set_len(mut self, length: usize) -> Self {
        debug_assert!(length <= CAP);
        self.len = length;
        self
    }
}

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

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array, const N: usize> From<ConstVec<A::Item, N>> for smallvec::SmallVec<A> {
    fn from(value: ConstVec<A::Item, N>) -> Self {
        smallvec::SmallVec::new().tap_mut(|v| v.extend(value))
    }
}
#[cfg(feature = "arrayvec")]
impl<T, const N: usize> From<ConstVec<T, N>> for arrayvec::ArrayVec<T, N> {
    fn from(value: ConstVec<T, N>) -> Self {
        arrayvec::ArrayVec::new().tap_mut(|v| v.extend(value))
    }
}
