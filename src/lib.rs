#![feature(const_ptr_read)]
//! Provides an arrayvec-like type which can be modified at const-time.

use core::{mem::ManuallyDrop, panic};
use std::{ops::Deref, ptr::addr_of};

pub struct CapacityError<T, const CAP: usize> {
    pub vector: ConstVec<T, CAP>,
    pub item: T,
}

impl<T, const CAP: usize> std::fmt::Debug for CapacityError<T, CAP> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    const T_SIZE: usize = std::mem::size_of::<T>();

    pub const fn new() -> Self {
        Self {
            xs: unsafe { MaybeUninit::uninit().assume_init() },
            xs_addr: (),
            len: 0,
        }
    }

    pub const fn get(&self, ix: usize) -> Option<&T> {
        if ix < self.len {
            Some(unsafe { core::mem::transmute(&self.xs[ix].value) })
        } else {
            None
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
        // we can't get a pointer to xs or self, but we can get a one to a zst with the same address 
        let ptr_to_xs = addr_of!(self.xs_addr) as *const u8;
        // we have a pointer to our array now, but we need a pointer to the item's location
        let ptr_to_elem = ptr_to_xs.add(Self::T_SIZE * (self.len - 1));
        // now we have enough information to get a slice containing the item
        let item_as_u8_slice = core::slice::from_raw_parts(ptr_to_elem as *const u8, Self::T_SIZE);
        // and then use an improvised transmute_copy to obtain the item
        let item = std::ptr::read_unaligned(item_as_u8_slice as *const _ as *const T);
        // now that we aren't using the pointers anymore, reduce the length
        let len = self.len;
        self = self.set_len(len - 1);

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

    type IntoIter = ConstVecIter<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        ConstVecIter { vec: self, ix: 0 }
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
}
